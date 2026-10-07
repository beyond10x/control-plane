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

/// Serialized bytes of planner evidence one progress event may record.
const EVIDENCE_BYTES: usize = 4 * 1024;

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
            let rejected_goal = goal.clone();
            let acceptance = self
                .store
                .lock()
                .await
                .activity_history(text(&goal, "goal_id")?)?["acceptance"]
                .clone();
            let rejected_repositories = repositories.clone();
            let rejected_assignments = assignments.clone();
            let runner = self.runner();
            if tokio::task::spawn_blocking(move || {
                crate::fleet::acceptance_is_unchanged(
                    &rejected_goal,
                    &acceptance,
                    &rejected_repositories,
                    &rejected_assignments,
                    &runner,
                )
            })
            .await?
            {
                continue;
            }
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
    /// Record one runtime progress event of an assignment through the fleet's progress path.
    /// The assignment's goal must still be Running at the assignment's goal revision.
    pub async fn record_progress(
        &self,
        assignment: &Value,
        action: &str,
        role: &str,
        detail: Value,
    ) -> Result<()> {
        let mut store = self.store.lock().await;
        check_goal(
            &store,
            &json!({"goal_id":assignment["goal_id"],"revision":assignment["goal_revision"]}),
        )?;
        crate::fleet::record_progress(&mut store, assignment, action, role, detail).await
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
        // Recovery of this planning attempt must reopen its Loom session and spent budget.
        // A crash-left Active session is refused by Loom; a fresh random identity would hide it.
        let session = format!("planner-{worktree_id}");
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
            // Each hook event becomes the goal's newest planner evidence; it is recorded once,
            // so it stays bounded. Full file contents and request bodies are not progress.
            let receipt = crate::context::bounded(receipt, EVIDENCE_BYTES);
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
        let command = self.config.commit_for(&p, &r)?;
        let trusted = self.config.credentialed(&r);
        let commit =
            tokio::task::spawn_blocking(move || commit_plan(&p, &r, &trusted, &command)).await??;
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
    let parse = |row: &Value| -> Result<Value> {
        let text = row["planning_receipt"].as_str().unwrap_or_default();
        if text.is_empty() {
            return Ok(json!({}));
        }
        let receipt: Value = serde_json::from_str(text).context("invalid planning receipt")?;
        ensure!(receipt.is_object(), "planning receipt must be an object");
        Ok(receipt)
    };
    let saved = parse(current)?;
    let incoming = parse(progress)?;
    let mut combined = saved.clone();
    // Persisted envelopes are snapshots, never new tool events. Flatten envelopes
    // written by older versions, retaining the authoritative outer fleet fields.
    let saved_evidence = if activity_envelope(&saved) {
        planner_evidence(&saved)
    } else if saved["kind"].is_string() {
        Some(saved.clone())
    } else {
        None
    };
    let fields = combined.as_object_mut().context("receipt not object")?;
    if fields.get("kind").is_some_and(Value::is_string) {
        for key in ["kind", "receipt", "activity_id"] {
            fields.remove(key);
        }
    }
    fields.remove("planner");
    if let Some(evidence) = saved_evidence.as_ref() {
        fields.insert("planner".into(), evidence.clone());
    }
    let fresh = !activity_envelope(&incoming)
        && incoming != saved
        && saved_evidence.as_ref() != Some(&incoming);
    let kind = if fresh {
        incoming["kind"].as_str().unwrap_or_default()
    } else {
        ""
    };
    let detail = &incoming["receipt"];
    let mut event = if kind == "activity" {
        Some(detail.clone())
    } else if !kind.is_empty() {
        let label = match kind {
            "intent" => intent_label(detail),
            "observation" => "Repository operation completed".into(),
            "plan-approved" => "Independent plan review passed".into(),
            "accept-story" => format!("Accepting {}", detail["story"].as_str().unwrap_or("story")),
            "adopt" => "Initializing the repository planning store".into(),
            _ => kind.to_owned(),
        };
        // These two hooks follow successful effects. All other generic hooks
        // announce work or observations without proof of successful completion.
        let status = match kind {
            "observation" | "plan-approved" => "completed",
            _ => "running",
        };
        Some(
            json!({"action":format!("planner.{kind}"),"role":"planner","detail":label,"status":status}),
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
        let id = (!kind.is_empty())
            .then(|| incoming["activity_id"].as_str())
            .flatten()
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
    if fresh && incoming.as_object().is_some_and(|v| !v.is_empty()) {
        combined["planner"] = incoming;
    }
    Ok(combined.to_string())
}

fn activity_envelope(receipt: &Value) -> bool {
    receipt["activity"].is_array()
        || receipt["last_activity"].is_object()
        || receipt["planner"].is_object()
}

fn planner_evidence(mut receipt: &Value) -> Option<Value> {
    while activity_envelope(receipt) {
        receipt = receipt.get("planner")?;
    }
    receipt
        .as_object()
        .filter(|fields| !fields.is_empty())
        .map(|_| receipt.clone())
}

fn intent_label(detail: &Value) -> String {
    let single_line = |value: &str| value.lines().next().unwrap_or_default().to_owned();
    let label = match detail["action"].as_str() {
        Some("read") => {
            let paths = detail["paths"]
                .as_array()
                .map(|paths| {
                    paths
                        .iter()
                        .take(3)
                        .filter_map(Value::as_str)
                        .map(single_line)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            format!("Reading {paths}")
        }
        Some("read_range" | "read_bytes") => format!(
            "Reading page of {}",
            single_line(detail["path"].as_str().unwrap_or_default())
        ),
        Some("write_specification") => format!(
            "Writing specification {}",
            single_line(detail["path"].as_str().unwrap_or_default())
        ),
        Some("aep") => {
            // Only command words; flags, values and request bodies stay in the
            // full receipt, never in the concise activity label.
            let command = detail["args"]
                .as_array()
                .map(|args| {
                    args.iter()
                        .take(3)
                        .filter_map(Value::as_str)
                        .take_while(|arg| !arg.starts_with('-'))
                        .map(single_line)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            format!("Executing aep {command}")
        }
        Some("finish") => "Validating the engineering plan".into(),
        _ => "Executing planner tool".into(),
    };
    label.chars().take(240).collect()
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
    // The planner reads no assignment (`engine::EngineInput`); their states and evidence decide
    // whether it runs. A Blocked assignment's reason is replaced whenever its cause changes, so
    // the reason is left out: a new cause alone gives the planner nothing to plan from.
    let assignments = assignments
        .iter()
        .map(|assignment| {
            let mut assignment = assignment.clone();
            if let Some(fields) = assignment.as_object_mut() {
                fields.remove("reason");
            }
            assignment
        })
        .collect::<Vec<_>>();
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
/// `trusted` runs only the commit command; it carries the commit credentials.
fn commit_plan(
    path: &Path,
    runner: &ProcessRunner,
    trusted: &ProcessRunner,
    command: &[String],
) -> Result<String> {
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
        trusted.run(path, program, &args, None)?;
    }
    Ok(runner
        .command(path, "git", &["rev-parse", "HEAD"])?
        .trim()
        .to_owned())
}

#[cfg(test)]
mod activity_tests {
    use super::*;

    fn row(phase: &str, receipt: Value) -> Value {
        json!({"revision":2,"planning_phase":phase,"planning_reason":"tool refused",
            "planning_worktree_path":"worktree","planning_receipt":receipt.to_string()})
    }

    fn normalize(current: &Value, progress: &Value) -> Value {
        serde_json::from_str(&activity_receipt(current, progress).unwrap()).unwrap()
    }

    #[test]
    fn pre_effect_events_never_claim_completion_after_refusal() {
        for kind in ["intent", "prepare-branch", "adopt", "accept-story"] {
            let current = row("Provisioning", json!({}));
            let started = normalize(
                &current,
                &row(
                    "Provisioning",
                    json!({
                        "kind":kind,"receipt":{"action":"test","story":"story:test"},"activity_id":"start"
                    }),
                ),
            );
            assert_eq!(started["last_activity"]["status"], "running", "{kind}");
            let blocked = normalize(
                &row("Provisioning", started.clone()),
                &row("Blocked", started),
            );
            assert_eq!(blocked["last_activity"]["action"], "planning.Blocked");
            assert_eq!(blocked["last_activity"]["status"], "failed");
            assert!(
                blocked["activity"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|event| event["status"] != "completed")
            );
        }
    }

    #[test]
    fn copied_envelope_stays_flat_and_preserves_opaque_fleet_evidence() {
        let evidence = json!({"namespace":"plan:2","steps":["validated"],"revision":"abc"});
        let fleet = json!({"assignment":"a","receipt":{"candidate":"def"}});
        let mut receipt = json!({"activity":[],"planner":evidence,"fleet":fleet});
        for _ in 0..180 {
            let current = row("Blocked", receipt.clone());
            receipt = normalize(&current, &current);
            assert!(receipt["planner"].get("activity").is_none());
            assert_eq!(receipt["planner"], evidence);
            assert_eq!(receipt["fleet"], fleet);
            assert!(receipt.to_string().len() < 1024);
        }
    }

    #[test]
    fn legacy_receipt_is_evidence_not_a_replayed_event_on_phase_change() {
        let legacy = json!({"kind":"observation","receipt":"old successful operation"});
        let receipt = normalize(
            &row("Provisioning", legacy.clone()),
            &row("Blocked", legacy.clone()),
        );
        assert_eq!(receipt["last_activity"]["action"], "planning.Blocked");
        assert_eq!(receipt["last_activity"]["status"], "failed");
        assert!(receipt.get("kind").is_none());
        assert_eq!(receipt["planner"], legacy);
    }

    #[test]
    fn old_nested_envelope_flattens_without_replacing_current_fleet_state() {
        let evidence = json!({"namespace":"plan:2","revision":"abc","steps":["validated"]});
        let old = json!({"activity":[],"planner":evidence,"fleet":{"a":"older"}});
        let current = row(
            "Planning",
            json!({"kind":"observation","receipt":"stale",
            "activity":[],"planner":old,"fleet":{"a":"current"},"acceptance":{"status":"waiting"}}),
        );
        let result = normalize(&current, &row("Blocked", old));
        assert_eq!(result["planner"], evidence);
        assert_eq!(result["fleet"]["a"], "current");
        assert_eq!(result["acceptance"]["status"], "waiting");
        assert!(result.get("kind").is_none());
        assert_eq!(result["last_activity"]["action"], "planning.Blocked");
    }

    #[test]
    fn corrupt_durable_receipt_is_not_silently_replaced() {
        let mut current = row("Planning", json!({}));
        current["planning_receipt"] = json!("{broken");
        assert!(activity_receipt(&current, &row("Blocked", json!({}))).is_err());
    }

    #[test]
    fn copied_leaf_activity_id_does_not_hide_final_phase() {
        let incoming = json!({"kind":"intent","receipt":{"action":"read"},"activity_id":"read"});
        let receipt = normalize(
            &row("Planning", json!({})),
            &row("Planning", incoming.clone()),
        );
        let result = normalize(&row("Planning", receipt), &row("Blocked", incoming));
        assert_eq!(result["last_activity"]["action"], "planning.Blocked");
        assert_eq!(result["activity"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn intent_labels_include_bounded_targets_but_never_body_or_contents() {
        for (detail, expected) in [
            (
                json!({"action":"read","paths":["README.md","ess/system.yaml"]}),
                "README.md, ess/system.yaml",
            ),
            (
                json!({"action":"write_specification","path":"ess/system.yaml","contents":"private body"}),
                "ess/system.yaml",
            ),
            (
                json!({"action":"aep","args":["plan","artifact","create","--body","private body"],"body":"private body"}),
                "aep plan artifact create",
            ),
        ] {
            let receipt = normalize(
                &row("Planning", json!({})),
                &row("Planning", json!({"kind":"intent","receipt":detail})),
            );
            let label = receipt["last_activity"]["detail"].as_str().unwrap();
            assert!(label.contains(expected), "{label}");
            assert!(!label.contains("private body"));
        }
        let receipt = normalize(
            &row("Planning", json!({})),
            &row(
                "Planning",
                json!({"kind":"intent","receipt":{"action":"read","paths":["長".repeat(2048)]}}),
            ),
        );
        assert!(
            receipt["last_activity"]["detail"]
                .as_str()
                .unwrap()
                .chars()
                .count()
                <= 240
        );
    }

    #[test]
    fn true_observations_and_model_results_remain_completed_and_history_is_bounded() {
        let mut receipt = json!({"fleet_checkpoint":{"assignment":"a"}});
        for index in 0..90 {
            let incoming = match index % 3 {
                0 => json!({"kind":"observation","receipt":"command exited successfully"}),
                1 => json!({"kind":"plan-approved","receipt":{"reason":"verified"}}),
                _ => {
                    json!({"kind":"activity","receipt":{"action":"model.completed","status":"completed"}})
                }
            };
            let mut incoming = incoming;
            incoming["activity_id"] = json!(index.to_string());
            receipt = normalize(&row("Planning", receipt), &row("Planning", incoming));
            assert_eq!(receipt["last_activity"]["status"], "completed");
        }
        assert_eq!(receipt["activity"].as_array().unwrap().len(), 64);
        assert_eq!(receipt["activity"][0]["id"], "26");
        assert_eq!(receipt["last_activity"]["id"], "89");
        assert_eq!(receipt["fleet_checkpoint"]["assignment"], "a");
    }
}
