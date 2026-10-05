use crate::{
    AgentModel, RuntimeConfig, SharedStore, TickReport,
    engine::{self, EngineInput, ProgressHook, text},
    process::ProcessRunner,
};
use anyhow::{Context, Result, ensure};
use control_plane_core::Actor;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

pub struct Supervisor {
    store: SharedStore,
    wake: Arc<Notify>,
    config: RuntimeConfig,
    model: Arc<dyn AgentModel>,
    tick_lock: Mutex<()>,
    cancel: CancellationToken,
}
impl Supervisor {
    pub fn new(
        store: SharedStore,
        wake: Arc<Notify>,
        config: RuntimeConfig,
        model: Arc<dyn AgentModel>,
    ) -> Self {
        Self {
            store,
            wake,
            config,
            model,
            tick_lock: Mutex::new(()),
            cancel: CancellationToken::new(),
        }
    }
    pub async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        loop {
            let tick = async {
                self.tick().await?;
                self.fleet_tick().await
            };
            tokio::pin!(tick);
            tokio::select! {
                result=&mut tick=>{ result?; }
                _=shutdown.cancelled()=>{ self.cancel.cancel(); tick.await?; return Ok(()); }
            }
            tokio::select! { _=shutdown.cancelled()=>return Ok(()), _=self.wake.notified()=>{}, _=tokio::time::sleep(self.config.poll_interval)=>{} }
        }
    }
    fn runner(&self) -> ProcessRunner {
        ProcessRunner {
            environment: self.config.environment.clone(),
            timeout: self.config.process_timeout,
            cancel: self.cancel.child_token(),
        }
    }
    pub async fn tick(&self) -> Result<TickReport> {
        let _exclusive = self.tick_lock.lock().await;
        let goals = rows(&self.store, "GoalList").await?;
        let mut report = TickReport::default();
        for mut goal in goals.into_iter().filter(|g| g["state"] == "Running") {
            goal["directories"] = json!(
                rows(&self.store, "WorkspaceDirectoryList")
                    .await?
                    .into_iter()
                    .filter(
                        |d| d["workspace_id"] == goal["workspace_id"] && d["state"] == "Registered"
                    )
                    .collect::<Vec<_>>()
            );
            let all_repositories = rows(&self.store, "RepositoryRegistrationList").await?;
            let repositories = all_repositories
                .iter()
                .filter(|r| r["workspace_id"] == goal["workspace_id"] && r["state"] == "Registered")
                .cloned()
                .collect::<Vec<_>>();
            self.retire_superseded_queue(&goal).await?;
            let assignments = related_assignments(
                &goal,
                &repositories,
                &all_repositories,
                rows(&self.store, "AssignmentList").await?,
            );
            let input_fingerprint =
                fingerprint(&goal, &repositories, &assignments, self.runner()).await?;
            if goal["planning_fingerprint"] == input_fingerprint
                && matches!(goal["planning_phase"].as_str(), Some("Queued" | "Blocked"))
            {
                continue;
            }
            if assignments.iter().any(|a| {
                a["goal_id"] == goal["goal_id"]
                    && matches!(
                        a["state"].as_str(),
                        Some("Implementing" | "Reviewing" | "ReadyToMerge" | "Merging")
                    )
            }) {
                continue;
            }
            let mut last = goal.clone();
            let mut failure = None;
            for repository in &repositories {
                if assignments.iter().any(|a| {
                    a["goal_id"] == goal["goal_id"]
                        && a["repository_id"] == repository["repository_id"]
                        && a["state"] == "Queued"
                        && a["goal_revision"] == goal["revision"]
                }) {
                    continue;
                }
                if assignments.iter().any(|a| {
                    assignment_in_repository(a, repository, &all_repositories)
                        && matches!(
                            a["state"].as_str(),
                            Some("Implementing" | "Reviewing" | "ReadyToMerge" | "Merging")
                        )
                }) {
                    failure = Some("repository has active implementation work".to_owned());
                    break;
                }
                match self
                    .plan_repository(&goal, repository, &input_fingerprint, &mut last)
                    .await
                {
                    Ok(queued) => {
                        report.planned += 1;
                        report.queued += queued;
                    }
                    Err(error) => {
                        failure = Some(format!("{error:#}"));
                        break;
                    }
                }
            }
            let current = related_assignments(
                &goal,
                &repositories,
                &all_repositories,
                rows(&self.store, "AssignmentList").await?,
            );
            let final_fingerprint =
                fingerprint(&goal, &repositories, &current, self.runner()).await?;
            let phase = if failure.is_some() {
                "Blocked"
            } else {
                "Queued"
            };
            let reason = failure.unwrap_or_else(|| {
                if current.iter().any(|a|a["goal_id"]==goal["goal_id"]&&a["state"]=="Queued") {String::new()}
                else {"No ready story selected; goal acceptance and verified merge evidence remain outstanding".into()}
            });
            if !reason.is_empty() {
                report.blockers.push(reason.clone());
            }
            last["planning_fingerprint"] = json!(final_fingerprint);
            last["planning_reason"] = json!(reason);
            last["planning_phase"] = json!(phase);
            // A changed or paused goal owns the next reconciliation; never overwrite its progress.
            if current_goal(&self.store, &goal).await.is_ok() {
                record(&self.store, &goal, &last).await?;
            }
        }
        Ok(report)
    }
    pub async fn fleet_tick(&self) -> Result<TickReport> {
        let _exclusive = self.tick_lock.lock().await;
        crate::fleet::run(
            self.store.clone(),
            self.config.clone(),
            self.model.clone(),
            self.runner(),
        )
        .await
    }
    async fn retire_superseded_queue(&self, goal: &Value) -> Result<()> {
        let mut store = self.store.lock().await;
        check_goal(&store, goal)?;
        let rows = store.query("AssignmentList")?;
        for assignment in rows
            .as_array()
            .context("assignments not an array")?
            .iter()
            .filter(|a| {
                a["goal_id"] == goal["goal_id"]
                    && a["goal_revision"] != goal["revision"]
                    && (a["state"] == "Queued"
                        || (a["state"] == "Blocked"
                            && a["reason"].as_str().is_some_and(|reason| {
                                reason.starts_with("Superseded queued goal revision")
                            })))
            })
        {
            if assignment["state"] == "Queued" {
                let outcome=store.execute("BlockAssignment",json!({"assignment_id":assignment["assignment_id"],"reason":format!("Superseded queued goal revision {} by revision {}",assignment["goal_revision"],goal["revision"])}),Actor::Supervisor).await?;
                applied(&outcome)?;
            }
            let outcome = store
                .execute(
                    "CancelAssignment",
                    json!({"assignment_id":assignment["assignment_id"]}),
                    Actor::Supervisor,
                )
                .await?;
            applied(&outcome)?;
        }
        Ok(())
    }
    async fn plan_repository(
        &self,
        goal: &Value,
        repository: &Value,
        fingerprint: &str,
        last: &mut Value,
    ) -> Result<usize> {
        current_goal(&self.store, goal).await?;
        let repository_id = text(repository, "repository_id")?;
        let primary = PathBuf::from(text(repository, "path")?);
        let runner = self.runner();
        let source = primary.clone();
        let check = runner.clone();
        let base = text(repository, "base_branch")?.to_owned();
        let base = tokio::task::spawn_blocking(move || -> Result<String> {
            ensure!(
                check
                    .command(&source, "git", &["status", "--porcelain"])?
                    .trim()
                    .is_empty(),
                "registered repository has uncommitted changes"
            );
            Ok(check
                .command(&source, "git", &["rev-parse", &base])?
                .trim()
                .into())
        })
        .await??;
        let reuse = goal["planning_revision"] == goal["revision"]
            && goal["planning_repository"] == repository_id
            && matches!(
                goal["planning_phase"].as_str(),
                Some("Provisioning" | "Planning" | "Validated")
            );
        let worktree_id = if reuse {
            text(goal, "planning_worktree_id")?.to_owned()
        } else {
            format!("cp-plan-{}", uuid::Uuid::new_v4())
        };
        *last = json!({"planning_revision":goal["revision"],"planning_fingerprint":fingerprint,"planning_repository":repository_id,"planning_worktree_id":worktree_id,"planning_worktree_path":if reuse {goal["planning_worktree_path"].clone()}else{json!("")},"planning_phase":"Provisioning","planning_reason":"","planning_receipt":""});
        record(&self.store, goal, last).await?;
        let id = worktree_id.clone();
        let source = primary.clone();
        let run = runner.clone();
        let path = tokio::task::spawn_blocking(move || -> Result<PathBuf> {
            let inspection: Value = serde_json::from_str(&run.command(
                &source,
                "worktree",
                &[
                    "inspect",
                    "--json",
                    "--repo",
                    source.to_str().context("repository path")?,
                    "--max-entries",
                    "1",
                ],
            )?)?;
            if let Some(items) = inspection["inspections"].as_array()
                && let Some(item) = items.iter().find(|item| item["record"]["id"] == id)
            {
                let path = item
                    .get("path")
                    .or_else(|| item["record"].get("path"))
                    .and_then(Value::as_str)
                    .context("existing worktree path missing")?;
                ensure!(
                    Path::new(path).is_dir(),
                    "recorded planner worktree is missing; reconcile it before retrying"
                );
                return Ok(PathBuf::from(path));
            }
            let created: Value = serde_json::from_str(&run.command(
                &source,
                "worktree",
                &[
                    "create",
                    "--json",
                    "--repo",
                    source.to_str().context("repository path")?,
                    "--base",
                    &base,
                    "--id",
                    &id,
                    "--purpose",
                    "control-plane governed planning",
                ],
            )?)?;
            Ok(PathBuf::from(
                created["evidence"]["path"]
                    .as_str()
                    .context("worktree create did not return a path")?,
            ))
        })
        .await??;
        last["planning_worktree_path"] = json!(path);
        last["planning_phase"] = json!("Planning");
        record(&self.store, goal, last).await?;
        let session = format!("planner-{}", uuid::Uuid::new_v4());
        let session_path = path.clone();
        let session_id = session.clone();
        let run = runner.clone();
        tokio::task::spawn_blocking(move || {
            run.command(
                &session_path,
                "worktree",
                &[
                    "hook",
                    "session-start",
                    "--path",
                    session_path.to_str().unwrap(),
                    "--session",
                    &session_id,
                ],
            )
        })
        .await??;
        let handle = tokio::runtime::Handle::current();
        let store = self.store.clone();
        let goal_copy = goal.clone();
        let metadata = last.clone();
        let expected_repository = repository.clone();
        let attempt_cancel = runner.cancel.clone();
        let hook: ProgressHook = Arc::new(move |kind, receipt| {
            ensure!(!attempt_cancel.is_cancelled(), "planning attempt cancelled");
            let mut progress = metadata.clone();
            progress["planning_receipt"] =
                json!(json!({"kind":kind,"receipt":receipt,"activity_id":uuid::Uuid::new_v4().to_string()}).to_string());
            handle.block_on(async {
                current_goal(&store, &goal_copy).await?;
                let repositories = rows(&store, "RepositoryRegistrationList").await?;
                ensure!(
                    repositories.iter().any(|r| r == &expected_repository),
                    "repository configuration changed during planning"
                );
                let directories = rows(&store, "WorkspaceDirectoryList")
                    .await?
                    .into_iter()
                    .filter(|d| {
                        d["workspace_id"] == goal_copy["workspace_id"] && d["state"] == "Registered"
                    })
                    .collect::<Vec<_>>();
                ensure!(
                    json!(directories) == goal_copy["directories"],
                    "workspace directory membership changed during planning"
                );
                record(&store, &goal_copy, &progress).await
            })
        });
        let input = EngineInput {
            path: path.clone(),
            goal: goal.clone(),
            namespace: session.clone(),
            config: self.config.clone(),
            runner: runner.clone(),
            progress: hook,
        };
        let model = self.model.clone();
        let job = tokio::task::spawn_blocking(move || engine::run(input, model));
        tokio::pin!(job);
        let result = loop {
            tokio::select! {
                result=&mut job=>break result.context("planner worker panicked").and_then(|r|r),
                _=tokio::time::sleep(std::time::Duration::from_secs(30))=>{
                    let p=path.clone();let s=session.clone();let r=runner.clone();
                    let heartbeat=tokio::task::spawn_blocking(move||r.command(&p,"worktree",&["hook","heartbeat","--path",p.to_str().unwrap(),"--session",&s])).await.context("lease heartbeat worker panicked").and_then(|r|r);
                    if let Err(error)=heartbeat {
                        runner.cancel.cancel();
                        let _=(&mut job).await;
                        break Err(error.context("planner lease heartbeat failed"));
                    }
                }
            }
        };
        let finalized:Result<usize>=async {
        let output = result?;
        last["planning_phase"] = json!("Validated");
        last["planning_receipt"] = json!(output.receipt.to_string());
        record(&self.store, goal, last).await?;
        current_goal(&self.store, goal).await?;
        let p = path.clone();
        let r = runner.clone();
        let command = self.config.commit_command.clone();
        let commit = tokio::task::spawn_blocking(move || commit_plan(&p, &r, &command)).await??;
        let mut queued = 0;
        for story in output.stories {
            let mut store = self.store.lock().await;
            check_goal(&store, goal)?;
            let reference = format!("{repository_id}::{story}");
            if store
                .query("AssignmentList")?
                .as_array()
                .context("assignments not an array")?
                .iter()
                .any(|a| {
                    a["goal_id"] == goal["goal_id"]
                        && a["story_id"] == reference
                        && a["state"] != "Cancelled"
                })
            {
                continue;
            }
            let outcome=store.execute("QueueAssignment",json!({"goal_id":goal["goal_id"],"repository_id":repository_id,"story_id":reference,"case_id":uuid::Uuid::new_v4().to_string(),"worktree_id":worktree_id,"candidate":commit,"attempt":0,"reason":"Validated and independently critiqued AEP story","implementor_run":"","reviewer_run":"","goal_revision":goal["revision"]}),Actor::Supervisor).await?;
            applied(&outcome)?;
            queued += 1;
        }
        Ok(queued)
        }.await;
        let p = path.clone();
        let s = session.clone();
        let mut r = runner.clone();
        r.cancel = CancellationToken::new();
        let release = tokio::task::spawn_blocking(move || {
            r.command(
                &p,
                "worktree",
                &[
                    "hook",
                    "session-end",
                    "--path",
                    p.to_str().unwrap(),
                    "--session",
                    &s,
                ],
            )
        })
        .await?;
        release?;
        finalized
    }
}
async fn rows(store: &SharedStore, view: &str) -> Result<Vec<Value>> {
    store
        .lock()
        .await
        .query(view)?
        .as_array()
        .cloned()
        .context("view is not an array")
}
fn assignment_in_repository(
    assignment: &Value,
    repository: &Value,
    all_repositories: &[Value],
) -> bool {
    all_repositories.iter().any(|registered| {
        registered["repository_id"] == assignment["repository_id"]
            && registered["common_dir"] == repository["common_dir"]
    })
}
fn related_assignments(
    goal: &Value,
    repositories: &[Value],
    all_repositories: &[Value],
    assignments: Vec<Value>,
) -> Vec<Value> {
    assignments
        .into_iter()
        .filter(|a| {
            a["goal_id"] == goal["goal_id"]
                || repositories
                    .iter()
                    .any(|r| assignment_in_repository(a, r, all_repositories))
        })
        .collect()
}
fn check_goal(store: &control_plane_core::Store, expected: &Value) -> Result<()> {
    let goals = store.query("GoalList")?;
    let current = goals
        .as_array()
        .context("goals not array")?
        .iter()
        .find(|g| g["goal_id"] == expected["goal_id"])
        .context("goal no longer exists")?;
    ensure!(
        current["state"] == "Running" && current["revision"] == expected["revision"],
        "goal paused or revision changed during planning"
    );
    Ok(())
}
async fn current_goal(store: &SharedStore, expected: &Value) -> Result<()> {
    check_goal(&*store.lock().await, expected)
}
fn applied(outcome: &Value) -> Result<()> {
    ensure!(
        matches!(outcome["outcome"].as_str(), Some("applied" | "created")),
        "host command refused: {outcome}"
    );
    Ok(())
}
async fn record(store: &SharedStore, goal: &Value, progress: &Value) -> Result<()> {
    let mut store = store.lock().await;
    check_goal(&store, goal)?;
    let mut payload = progress.as_object().context("progress not object")?.clone();
    payload.retain(|key, _| key.starts_with("planning_"));
    let current = store
        .query("GoalList")?
        .as_array()
        .context("goals view")?
        .iter()
        .find(|g| g["goal_id"] == goal["goal_id"])
        .context("goal missing")?
        .clone();
    payload.insert(
        "planning_receipt".into(),
        json!(activity_receipt(&current, progress)?),
    );
    payload.insert("goal_id".into(), goal["goal_id"].clone());
    let result = store
        .execute(
            "RecordPlanningProgress",
            Value::Object(payload),
            Actor::Supervisor,
        )
        .await?;
    applied(&result)
}

fn activity_receipt(current: &Value, progress: &Value) -> Result<String> {
    let parse = |row: &Value| -> Value {
        row["planning_receipt"]
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}))
    };
    let mut combined = parse(current);
    let incoming = parse(progress);
    let kind = incoming["kind"].as_str().unwrap_or_default();
    let detail = &incoming["receipt"];
    let mut event = if kind == "activity" {
        Some(detail.clone())
    } else if !kind.is_empty() {
        let label = match kind {
            "intent" => format!(
                "Executing {}",
                detail["action"].as_str().unwrap_or("planner tool")
            ),
            "observation" => "Repository operation completed".into(),
            "plan-approved" => "Independent plan review passed".into(),
            "accept-story" => format!("Accepting {}", detail["story"].as_str().unwrap_or("story")),
            "adopt" => "Initializing the repository planning store".into(),
            _ => kind.to_owned(),
        };
        Some(
            json!({"action":format!("planner.{kind}"),"role":"planner","detail":label,"status":if kind=="intent" {"running"} else {"completed"}}),
        )
    } else if current["planning_phase"] != progress["planning_phase"] {
        let phase = progress["planning_phase"].as_str().unwrap_or("Idle");
        Some(
            json!({"action":format!("planning.{phase}"),"role":"planner","detail":if phase=="Blocked" {progress["planning_reason"].as_str().unwrap_or("Planning blocked").to_owned()} else {format!("Planning phase: {phase}")},"status":match phase {"Blocked"=>"failed","Queued"=>"waiting","Validated"=>"completed",_=>"running"}}),
        )
    } else {
        None
    };
    if let Some(mut activity) = event.take() {
        let id = incoming["activity_id"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        if combined["last_activity"]["id"] != id {
            activity["id"] = json!(id);
            activity["at"] = json!(
                time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)?
            );
            activity["worktree"] = progress["planning_worktree_path"].clone();
            activity["goal_revision"] = current["revision"].clone();
            let mut history = combined["activity"].as_array().cloned().unwrap_or_default();
            history.push(activity.clone());
            if history.len() > 64 {
                history.drain(..history.len() - 64);
            }
            combined["activity"] = json!(history);
            combined["last_activity"] = activity;
        }
    }
    if incoming.as_object().is_some_and(|v| !v.is_empty()) {
        combined["planner"] = incoming;
    }
    Ok(combined.to_string())
}
async fn fingerprint(
    goal: &Value,
    repositories: &[Value],
    assignments: &[Value],
    runner: ProcessRunner,
) -> Result<String> {
    let mut goal = goal.clone();
    goal.as_object_mut()
        .context("goal not object")?
        .retain(|key, _| !key.starts_with("planning_"));
    let repositories = repositories.to_vec();
    let assignments = assignments.to_vec();
    tokio::task::spawn_blocking(move||->Result<String>{
        let mut inputs=Vec::new();
        for repo in &repositories {
            let path=Path::new(text(repo,"path")?);
            let observation=(||->Result<Value>{Ok(json!({"head":runner.command(path,"git",&["rev-parse",text(repo,"base_branch")?])?,"status":runner.command(path,"git",&["status","--porcelain"])?,"diff":runner.command(path,"git",&["diff","HEAD"])?}))})();
            inputs.push(json!({"repository":repo,"observation":match observation {Ok(value)=>value,Err(error)=>json!({"unavailable":format!("{error:#}")})}}));
        }
        Ok(engine::digest(&json!({"goal":goal,"repositories":inputs,"assignments":assignments})))
    }).await?
}
fn commit_plan(path: &Path, runner: &ProcessRunner, command: &[String]) -> Result<String> {
    let spec = engine::spec_root(path)?;
    let changed = runner.command(
        path,
        "git",
        &["ls-files", "--modified", "--others", "--exclude-standard"],
    )?;
    let mut files = Vec::new();
    for name in changed.lines() {
        let file = engine::confined(path, name, true)?;
        ensure!(
            name.starts_with(".engineering/")
                || (engine::specification_path(path, &spec, &file)
                    && matches!(
                        file.extension().and_then(|s| s.to_str()),
                        Some("yaml" | "yml")
                    )),
            "planner modified an unauthorized file: {name}"
        );
        files.push(name.to_owned());
    }
    if !files.is_empty() {
        let args = [vec!["add".into(), "--".into()], files].concat();
        runner.run(path, "git", &args, None)?;
        let (program, prefix) = command.split_first().context("commit command is empty")?;
        let args = [
            prefix.to_vec(),
            vec!["Record validated control-plane engineering plan".into()],
        ]
        .concat();
        runner.run(path, program, &args, None)?;
    }
    Ok(runner
        .command(path, "git", &["rev-parse", "HEAD"])?
        .trim()
        .to_owned())
}
