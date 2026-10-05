//! Trusted fleet effects; model proposals never construct validation or publication receipts.
use crate::{
    AgentModel, ModelRequest, RuntimeConfig, SharedStore, TickReport, engine,
    process::ProcessRunner,
};
use anyhow::{Context, Result, bail, ensure};
use control_plane_core::Actor;
use control_plane_protocol::{
    AttestedEvidence, EvaluationContext, EvidenceOrigin, Protocol, ProtocolKind, canon,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct Host {
    store: SharedStore,
    handle: Handle,
    config: RuntimeConfig,
    model: Arc<dyn AgentModel>,
    runner: ProcessRunner,
    deadline: Option<Instant>,
}
impl Host {
    fn remaining(&self) -> Result<Duration> {
        match self.deadline {
            Some(deadline) => deadline
                .checked_duration_since(Instant::now())
                .context("attempt exceeded max_minutes"),
            None => Ok(self.runner.timeout),
        }
    }
    fn progress(&self, assignment: &Value, action: &str, role: &str, detail: Value) -> Result<()> {
        self.handle.block_on(async {
            let mut store=self.store.lock().await;
            let goals=store.query("GoalList")?;
            let goal=goals.as_array().context("goals not array")?.iter().find(|g|g["goal_id"]==assignment["goal_id"]).context("progress goal missing")?;
            let mut receipt=serde_json::from_str::<Value>(goal["planning_receipt"].as_str().unwrap_or("{}")).unwrap_or_else(|_|json!({}));
            if !receipt.is_object(){receipt=json!({"planning":receipt});}
            let status=if action=="blocked" {"failed"}else if action.ends_with(".completed") {"completed"}else{"running"};
            let activity=json!({"assignment_id":assignment["assignment_id"],"goal_revision":assignment["goal_revision"],"action":action,"role":role,"at":now(),"worktree":assignment["worktree_id"],"status":status,"detail":detail});
            if !receipt["fleet"].is_object(){receipt["fleet"]=json!({});}
            receipt["fleet"][text(assignment,"assignment_id")?]=activity.clone();receipt["last_activity"]=activity;
            if !receipt["activity"].is_array(){receipt["activity"]=json!([]);}
            let activity=receipt["last_activity"].clone();
            let history=receipt["activity"].as_array_mut().unwrap();history.push(activity);if history.len()>64 {history.drain(..history.len()-64);}
            let mut body=goal.as_object().context("goal not object")?.clone();body.retain(|key,_|key.starts_with("planning_")||key=="goal_id");body.insert("planning_receipt".into(),json!(receipt.to_string()));
            let outcome=store.execute("RecordPlanningProgress",Value::Object(body),Actor::Supervisor).await?;
            ensure!(outcome["outcome"]=="applied","progress append refused: {outcome}");Ok(())
        })
    }
    fn rows(&self, view: &str) -> Result<Vec<Value>> {
        self.handle.block_on(async {
            self.store
                .lock()
                .await
                .query(view)?
                .as_array()
                .cloned()
                .context("view not an array")
        })
    }
    fn acceptance_progress(
        &self,
        goal: &Value,
        fingerprint: &str,
        status: &str,
        reason: &str,
    ) -> Result<()> {
        self.handle.block_on(async {
            let mut store = self.store.lock().await;
            let goals = store.query("GoalList")?;
            let current = goals.as_array().context("goals view")?.iter().find(|g|g["goal_id"]==goal["goal_id"]).context("goal missing")?;
            ensure!(current["revision"]==goal["revision"], "goal changed during acceptance");
            let mut receipt: Value = serde_json::from_str(current["planning_receipt"].as_str().unwrap_or("{}"))?;
            receipt["acceptance"] = json!({"fingerprint":fingerprint,"status":status,"reason":reason,"at":now(),"goal_revision":goal["revision"]});
            let mut body = current.as_object().context("goal object")?.clone();
            body.retain(|key,_|key.starts_with("planning_")||key=="goal_id");
            body.insert("planning_receipt".into(),json!(receipt.to_string()));
            let outcome=store.execute("RecordPlanningProgress",Value::Object(body),Actor::Supervisor).await?;
            ensure!(outcome["outcome"]=="applied","acceptance progress refused: {outcome}");
            Ok(())
        })
    }
    fn row(&self, view: &str, key: &str, id: &Value) -> Result<Value> {
        self.rows(view)?
            .into_iter()
            .find(|r| &r[key] == id)
            .context("fleet entity no longer exists")
    }
    fn execute(&self, command: &str, body: Value) -> Result<Value> {
        self.handle.block_on(async {
            let result = self
                .store
                .lock()
                .await
                .execute(command, body, Actor::Supervisor)
                .await?;
            ensure!(
                matches!(result["outcome"].as_str(), Some("applied" | "created")),
                "{command} refused: {result}"
            );
            Ok(result)
        })
    }
    fn guard(&self, assignment: &Value, goal: &Value, repo: &Value, merge: bool) -> Result<Value> {
        ensure!(
            !self.runner.cancel.is_cancelled(),
            "fleet execution cancelled"
        );
        self.remaining()?;
        let current = self.row("GoalList", "goal_id", &goal["goal_id"])?;
        ensure!(
            current["state"] == "Running"
                && current["revision"] == goal["revision"]
                && assignment["goal_revision"] == current["revision"],
            "goal paused, cancelled or revision changed"
        );
        let registered = self.row(
            "RepositoryRegistrationList",
            "repository_id",
            &repo["repository_id"],
        )?;
        ensure!(
            &registered == repo && registered["state"] == "Registered",
            "repository configuration changed or disabled"
        );
        if goal["directories"].is_array() {
            let directories = self
                .rows("WorkspaceDirectoryList")?
                .into_iter()
                .filter(|d| d["workspace_id"] == goal["workspace_id"] && d["state"] == "Registered")
                .collect::<Vec<_>>();
            ensure!(
                json!(directories) == goal["directories"],
                "workspace directory membership changed during execution"
            );
        }
        let current_assignment = self.row(
            "AssignmentList",
            "assignment_id",
            &assignment["assignment_id"],
        )?;
        ensure!(
            !matches!(
                current_assignment["state"].as_str(),
                Some("Cancelled" | "Merged")
            ),
            "assignment is terminal"
        );
        if merge {
            ensure!(
                current["merge_authority"] == true,
                "standing merge authority is revoked"
            );
        }
        Ok(current_assignment)
    }
    fn block(&self, id: &Value, error: &str) -> Result<()> {
        let row = self.row("AssignmentList", "assignment_id", id)?;
        self.progress(&row, "blocked", "host", json!({"reason":error}))?;
        if !matches!(
            row["state"].as_str(),
            Some("Blocked" | "Merged" | "Cancelled")
        ) {
            self.execute(
                "BlockAssignment",
                json!({"assignment_id":id,"reason":error}),
            )?;
        }
        Ok(())
    }
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    engine::text(value, key)
}
fn now() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn git(host: &Host, path: &Path, args: &[&str]) -> Result<String> {
    let mut runner = host.runner.clone();
    runner.timeout = runner.timeout.min(host.remaining()?);
    runner
        .command(path, "git", args)
        .map(|out| out.trim().to_owned())
}
fn command(
    host: &Host,
    path: &Path,
    configured: &str,
    bindings: &BTreeMap<&str, String>,
) -> Result<String> {
    let mut argv = shell_words::split(configured).context("invalid configured command quoting")?;
    ensure!(!argv.is_empty(), "repository command is not configured");
    for arg in &mut argv {
        for (key, value) in bindings {
            *arg = arg.replace(&format!("{{{key}}}"), value);
        }
    }
    let mut runner = host.runner.clone();
    runner.timeout = runner.timeout.min(host.remaining()?);
    for (key, value) in bindings {
        runner.environment.push((
            format!("CONTROL_PLANE_{}", key.to_ascii_uppercase()),
            value.clone(),
        ));
    }
    runner.run(path, &argv[0], &argv[1..], None)
}
fn commit(host: &Host, path: &Path, message: &str) -> Result<String> {
    if !git(host, path, &["status", "--porcelain"])?.is_empty() {
        git(host, path, &["add", "--all"])?;
        let (program, prefix) = host
            .config
            .commit_command
            .split_first()
            .context("commit command empty")?;
        let args = [prefix.to_vec(), vec![message.to_owned()]].concat();
        host.runner.run(path, program, &args, None)?;
    }
    ensure!(
        git(host, path, &["status", "--porcelain"])?.is_empty(),
        "candidate worktree remains dirty after commit"
    );
    git(host, path, &["rev-parse", "HEAD"])
}

pub async fn run(
    store: SharedStore,
    config: RuntimeConfig,
    model: Arc<dyn AgentModel>,
    runner: ProcessRunner,
) -> Result<TickReport> {
    let host = Host {
        store,
        config,
        model,
        runner,
        handle: Handle::current(),
        deadline: None,
    };
    let discovery = host.clone();
    let (goals, repositories, assignments, publications) =
        tokio::task::spawn_blocking(move || -> Result<_> {
            reconcile_publications(&discovery)?;
            retire_stale(&discovery)?;
            let mut goals = discovery.rows("GoalList")?;
            let directories = discovery.rows("WorkspaceDirectoryList")?;
            for goal in &mut goals {
                goal["directories"] = json!(
                    directories
                        .iter()
                        .filter(|d| d["workspace_id"] == goal["workspace_id"]
                            && d["state"] == "Registered")
                        .collect::<Vec<_>>()
                );
            }
            Ok((
                goals,
                discovery.rows("RepositoryRegistrationList")?,
                discovery.rows("AssignmentList")?,
                discovery.rows("PublicationIntentList")?,
            ))
        })
        .await??;
    let mut chosen = Vec::new();
    let mut common = BTreeSet::new();
    let mut workers = BTreeMap::<String, usize>::new();
    // Recover in-flight work first. A queued alias cannot steal its common-directory slot.
    let mut ordered = assignments.clone();
    ordered.sort_by_key(|a| a["state"] == "Queued");
    for assignment in ordered {
        if !matches!(
            assignment["state"].as_str(),
            Some("Queued" | "Implementing" | "Reviewing" | "ReadyToMerge" | "Blocked")
        ) {
            continue;
        }
        let Some(goal) = goals.iter().find(|g| {
            g["goal_id"] == assignment["goal_id"]
                && g["state"] == "Running"
                && g["revision"] == assignment["goal_revision"]
        }) else {
            continue;
        };
        let Some(repo) = repositories.iter().find(|r| {
            r["repository_id"] == assignment["repository_id"] && r["state"] == "Registered"
        }) else {
            continue;
        };
        let id = text(goal, "goal_id")?.to_owned();
        let count = workers.entry(id).or_default();
        if *count
            >= goal["max_workers"]
                .as_u64()
                .context("invalid worker limit")? as usize
        {
            continue;
        }
        if assignment["attempt"].as_i64().unwrap_or(0) >= goal["max_attempts"].as_i64().unwrap_or(0)
            && assignment["state"] != "ReadyToMerge"
        {
            continue;
        }
        if common.contains(text(repo, "common_dir")?) {
            continue;
        }
        // A blocked publication still owns its repository until its outcome is observed.
        if assignments.iter().any(|other| {
            other["assignment_id"] != assignment["assignment_id"]
                && (matches!(
                    other["state"].as_str(),
                    Some("Implementing" | "Reviewing" | "ReadyToMerge" | "Merging")
                ) || (other["state"] != "Merged"
                    && publications
                        .iter()
                        .any(|p| p["assignment_id"] == other["assignment_id"])))
                && repositories.iter().any(|r| {
                    r["repository_id"] == other["repository_id"]
                        && r["common_dir"] == repo["common_dir"]
                })
        }) {
            continue;
        }
        common.insert(text(repo, "common_dir")?.to_owned());
        *count += 1;
        chosen.push((assignment, goal.clone(), repo.clone()));
    }
    let mut tasks: tokio::task::JoinSet<Result<Option<String>>> = tokio::task::JoinSet::new();
    for (assignment, goal, repo) in chosen {
        let worker = host.clone();
        tasks.spawn_blocking(move || {
            let result = deliver(&worker, &assignment, &goal, &repo);
            match result {
                Ok(()) => Ok(None),
                Err(error) => {
                    let reason = format!("{error:#}");
                    worker.block(&assignment["assignment_id"], &reason)?;
                    Ok(Some(reason))
                }
            }
        });
    }
    let mut report = TickReport::default();
    while let Some(result) = tasks.join_next().await {
        match result.context("fleet worker panicked")?? {
            Some(reason) => report.blockers.push(reason),
            None => report.merged += 1,
        }
    }
    let completion = host.clone();
    let results = tokio::task::spawn_blocking(move || satisfy_goals(&completion)).await??;
    report.satisfied = results.0;
    report.blockers.extend(results.1);
    Ok(report)
}

fn retire_stale(host: &Host) -> Result<()> {
    let goals = host.rows("GoalList")?;
    let publications = host.rows("PublicationIntentList")?;
    for a in host.rows("AssignmentList")? {
        if matches!(
            a["state"].as_str(),
            Some("Merged" | "Cancelled" | "Merging")
        ) || publications
            .iter()
            .any(|p| p["assignment_id"] == a["assignment_id"])
        {
            continue;
        }
        let goal = goals
            .iter()
            .find(|g| g["goal_id"] == a["goal_id"])
            .context("assignment goal missing")?;
        if goal["revision"] != a["goal_revision"] || goal["state"] == "Cancelled" {
            host.block(
                &a["assignment_id"],
                "Assignment superseded by changed or cancelled goal",
            )?;
            host.execute(
                "CancelAssignment",
                json!({"assignment_id":a["assignment_id"]}),
            )?;
        }
    }
    Ok(())
}

struct Lease {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<Result<()>>>,
    host: Host,
    path: PathBuf,
    session: String,
}
impl Lease {
    fn acquire(host: &Host, path: &Path, session: &str) -> Result<Self> {
        host.runner.command(
            path,
            "worktree",
            &[
                "hook",
                "session-start",
                "--path",
                path.to_str().context("worktree path")?,
                "--session",
                session,
            ],
        )?;
        let (stop, rx) = mpsc::channel();
        let worker = host.clone();
        let lease_path = path.to_path_buf();
        let lease_session = session.to_owned();
        let thread = thread::spawn(move || {
            while rx.recv_timeout(Duration::from_secs(30)) == Err(mpsc::RecvTimeoutError::Timeout) {
                if let Err(error) = worker.runner.command(
                    &lease_path,
                    "worktree",
                    &[
                        "hook",
                        "heartbeat",
                        "--path",
                        lease_path.to_str().unwrap(),
                        "--session",
                        &lease_session,
                    ],
                ) {
                    worker.runner.cancel.cancel();
                    return Err(error.context("worktree lease heartbeat failed"));
                }
            }
            Ok(())
        });
        Ok(Self {
            stop,
            thread: Some(thread),
            host: host.clone(),
            path: path.to_owned(),
            session: session.to_owned(),
        })
    }
    fn release(mut self) -> Result<()> {
        let _ = self.stop.send(());
        let heartbeat = self
            .thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| anyhow::anyhow!("lease heartbeat panicked"))?;
        let mut runner = self.host.runner.clone();
        runner.cancel = CancellationToken::new();
        let release = runner.command(
            &self.path,
            "worktree",
            &[
                "hook",
                "session-end",
                "--path",
                self.path.to_str().unwrap(),
                "--session",
                &self.session,
            ],
        );
        heartbeat?;
        release?;
        Ok(())
    }
}
fn tree(host: &Host, repo: &Value, id: &str, base: &str) -> Result<PathBuf> {
    let primary = Path::new(text(repo, "path")?);
    let inspection: Value = serde_json::from_str(&host.runner.command(
        primary,
        "worktree",
        &[
            "inspect",
            "--json",
            "--repo",
            primary.to_str().unwrap(),
            "--max-entries",
            "1",
        ],
    )?)?;
    if let Some(found) = inspection["inspections"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["record"]["id"] == id))
    {
        let path = PathBuf::from(text(&found["record"], "path")?);
        ensure!(
            path.is_dir(),
            "recorded implementation worktree missing; reconcile managed tree"
        );
        return Ok(path);
    }
    let created: Value = serde_json::from_str(&host.runner.command(
        primary,
        "worktree",
        &[
            "create",
            "--json",
            "--repo",
            primary.to_str().unwrap(),
            "--base",
            base,
            "--id",
            id,
            "--purpose",
            "control-plane implementation",
        ],
    )?)?;
    Ok(PathBuf::from(text(&created["evidence"], "path")?))
}
fn remote_head(host: &Host, path: &Path, target: &str) -> Result<String> {
    git(
        host,
        path,
        &["check-ref-format", &format!("refs/heads/{target}")],
    )?;
    let output = git(
        host,
        path,
        &[
            "ls-remote",
            "--refs",
            "origin",
            &format!("refs/heads/{target}"),
        ],
    )?;
    let mut entries = output.lines();
    let line = entries.next().context("origin target does not exist")?;
    ensure!(entries.next().is_none(), "ambiguous origin target");
    let (sha, reference) = line
        .split_once('\t')
        .context("invalid remote ref response")?;
    ensure!(
        reference == format!("refs/heads/{target}")
            && (sha.len() == 40 || sha.len() == 64)
            && sha.bytes().all(|c| c.is_ascii_hexdigit()),
        "invalid origin target observation"
    );
    Ok(sha.into())
}
fn fetch(host: &Host, path: &Path, target: &str) -> Result<String> {
    let head = remote_head(host, path, target)?;
    git(host, path, &["fetch", "--no-tags", "origin", &head])?;
    Ok(head)
}
fn deliver(host: &Host, initial: &Value, goal: &Value, repo: &Value) -> Result<()> {
    if host
        .rows("PublicationIntentList")?
        .iter()
        .any(|p| p["assignment_id"] == initial["assignment_id"])
    {
        bail!("publication intent exists; waiting for exact remote reconciliation");
    }
    let mut worker = host.clone();
    worker.runner.cancel = host.runner.cancel.child_token();
    let budget = attempt_budget(goal)?;
    worker.runner.timeout = worker.runner.timeout.min(budget);
    worker.deadline = Some(
        Instant::now()
            .checked_add(budget)
            .context("attempt deadline exceeds supported range")?,
    );
    let _deadline = AttemptDeadline::start(worker.runner.cancel.clone(), budget);
    let host = &worker;
    host.guard(initial, goal, repo, false)?;
    let primary = Path::new(text(repo, "path")?);
    let target = text(repo, "base_branch")?;
    let base = fetch(host, primary, target)?;
    let run = format!("implementor-{}", uuid::Uuid::new_v4());
    let tree_id = if initial["state"] == "Queued" {
        format!("cp-impl-{}", uuid::Uuid::new_v4())
    } else {
        text(initial, "worktree_id")?.into()
    };
    if initial["state"] == "Queued" {
        host.execute("ClaimAssignment",json!({"assignment_id":initial["assignment_id"],"worktree_id":tree_id,"implementor_run":run,"base_revision":base}))?;
    } else {
        host.block(
            &initial["assignment_id"],
            "Recover interrupted implementation with fresh tests and independent review",
        )?;
        host.execute("RepairAssignment",json!({"assignment_id":initial["assignment_id"],"reason":"Fresh execution after interrupted or blocked attempt","implementor_run":run}))?;
        ensure!(
            initial["base_revision"] == base,
            "target base changed; queued plan must be reconciled before another attempt"
        );
    }
    let assignment = host.row("AssignmentList", "assignment_id", &initial["assignment_id"])?;
    host.progress(
        &assignment,
        "worktree.prepare",
        "host",
        json!({"base":base,"tree_id":tree_id}),
    )?;
    let path = tree(host, repo, &tree_id, &base)?;
    let lease = Lease::acquire(host, &path, &run)?;
    let result = (|| -> Result<()> {
        if git(host, &path, &["branch", "--show-current"])?.is_empty() {
            git(
                host,
                &path,
                &["switch", "-c", &format!("control-plane/{run}")],
            )?;
        }
        let planned = text(initial, "candidate")?;
        if initial["state"] == "Queued"
            && git(host, &path, &["merge-base", planned, &base])? != planned
        {
            host.guard(&assignment, goal, repo, false)?;
            git(host, &path, &["merge", "--no-commit", "--no-ff", planned])?;
            commit(host, &path, "Integrate validated engineering plan")?;
        }
        let story_id = text(&assignment, "story_id")?
            .split_once("::")
            .context("assignment requires repository-qualified AEP reference")?
            .1;
        let story: Value = serde_json::from_str(&host.runner.command(
            &path,
            "aep",
            &["plan", "artifact", "show", story_id, "--format", "json"],
        )?)?;
        ensure!(
            story["status"] == "active"
                || (initial["state"] != "Queued" && story["status"] == "implemented"),
            "AEP story is no longer ready for implementation"
        );
        let scope = story["scope"]
            .as_array()
            .context("story scope missing")?
            .iter()
            .map(|s| text(s, "path").map(str::to_owned))
            .collect::<Result<Vec<_>>>()?;
        ensure!(!scope.is_empty(), "story has no machine-readable scope");
        let execution = Execution {
            host,
            assignment: &assignment,
            goal,
            repo,
            path: &path,
            run: &run,
        };
        implementation(&execution, &story, &scope)?;
        host.guard(&assignment, goal, repo, false)?;
        let candidate = commit(host, &path, "Implement accepted engineering story")?;
        host.progress(
            &assignment,
            "checks.run",
            "host",
            json!({"candidate":candidate,"command":repo["test_command"]}),
        )?;
        let checks = command(host, &path, text(repo, "test_command")?, &BTreeMap::new())?;
        ensure!(
            git(host, &path, &["rev-parse", "HEAD"])? == candidate
                && git(host, &path, &["status", "--porcelain"])?.is_empty(),
            "required checks changed the candidate"
        );
        host.guard(&assignment, goal, repo, false)?;
        if story["status"] == "active" {
            let source = format!(
                "control-plane:{}:{}:git:{}:checks:{}",
                text(&assignment, "assignment_id")?,
                run,
                candidate,
                engine::digest(&json!({"command":repo["test_command"],"output":checks}))
            );
            host.runner.command(
                &path,
                "aep",
                &[
                    "plan",
                    "artifact",
                    "evidence",
                    story_id,
                    "--kind",
                    "test_result",
                    "--source",
                    &source,
                ],
            )?;
            host.runner.command(
                &path,
                "aep",
                &["plan", "artifact", "move", story_id, "--to", "implemented"],
            )?;
        }
        let candidate = commit(host, &path, "Record observed engineering checks")?;
        host.progress(
            &assignment,
            "checks.run",
            "host",
            json!({"candidate":candidate,"command":repo["test_command"]}),
        )?;
        let checks = command(host, &path, text(repo, "test_command")?, &BTreeMap::new())?;
        ensure!(
            git(host, &path, &["rev-parse", "HEAD"])? == candidate
                && git(host, &path, &["status", "--porcelain"])?.is_empty(),
            "checks modified the final candidate"
        );
        host.execute("ReviewAssignment",json!({"assignment_id":assignment["assignment_id"],"candidate":candidate,"test_revision":candidate}))?;
        host.guard(&assignment, goal, repo, false)?;
        let diff = git(host, &path, &["diff", &base, &candidate, "--"])?;
        let reviewer = format!("reviewer-{}", uuid::Uuid::new_v4());
        host.progress(
            &assignment,
            "review.request",
            "reviewer",
            json!({"candidate":candidate,"execution_context":reviewer}),
        )?;
        let review=host.model.respond(&ModelRequest {role:"reviewer".into(),execution_context:reviewer.clone(),model:text(goal,"reviewer_model")?.into(),instructions:"Independently review the exact candidate and real host check output against this accepted AEP story and standing goal. You cannot grant publication authority or fabricate check evidence.".into(),prompt:json!({"goal":goal,"story":story,"candidate":candidate,"base":base,"diff":diff,"checks":checks}).to_string(),schema:crate::model::critique_schema(),timeout:host.remaining()?})?;
        ensure!(
            review["approved"] == true
                && review["reason"]
                    .as_str()
                    .is_some_and(|s| !s.trim().is_empty()),
            "independent review rejected candidate: {review}"
        );
        host.guard(&assignment, goal, repo, false)?;
        ensure!(
            git(host, &path, &["rev-parse", "HEAD"])? == candidate
                && git(host, &path, &["status", "--porcelain"])?.is_empty(),
            "candidate changed during independent review"
        );
        host.execute("ReadyAssignment",json!({"assignment_id":assignment["assignment_id"],"reviewer_run":reviewer,"review_revision":candidate}))?;
        let evidence = verification_evidence(&candidate, &run, &reviewer, &checks, &review)?;
        publish(&execution, &candidate, &base, &evidence)?;
        Ok(())
    })();
    let release = lease.release();
    result?;
    release?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum ImplementationAction {
    Read { paths: Vec<String> },
    Write { path: String, contents: String },
    Delete { path: String },
    Run { program: String, args: Vec<String> },
    Finish { summary: String },
}
fn scoped(path: &str, scope: &[String]) -> Result<()> {
    let relative = Path::new(path);
    ensure!(
        !relative.starts_with(".engineering") && !relative.starts_with(".git"),
        "planning and Git metadata are host-owned"
    );
    ensure!(
        scope.iter().any(|root| relative == Path::new(root)
            || relative.starts_with(Path::new(root.trim_end_matches('/')))),
        "write is outside accepted AEP scope: {path}"
    );
    ensure!(
        !matches!(
            relative.extension().and_then(|e| e.to_str()),
            Some("py" | "sh" | "bash" | "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "rb" | "go")
        ),
        "runnable source must be Rust"
    );
    Ok(())
}
#[derive(Clone, Copy)]
struct Execution<'a> {
    host: &'a Host,
    assignment: &'a Value,
    goal: &'a Value,
    repo: &'a Value,
    path: &'a Path,
    run: &'a str,
}
fn implementation(execution: &Execution<'_>, story: &Value, scope: &[String]) -> Result<()> {
    let Execution {
        host,
        assignment,
        goal,
        repo,
        path,
        run,
    } = *execution;
    let started = Instant::now();
    let budget = attempt_budget(goal)?;
    let mut transcript = vec![format!(
        "Repository files:\n{}",
        git(host, path, &["ls-files"])?
    )];
    for name in ["AGENTS.md", "README.md"] {
        if path.join(name).is_file() {
            let file = engine::confined(path, name, false)?;
            ensure!(
                std::fs::metadata(&file)?.len() <= 256 * 1024,
                "instruction file too large"
            );
            transcript.push(std::fs::read_to_string(file)?);
        }
    }
    for _ in 0..host.config.max_steps {
        host.guard(assignment, goal, repo, false)?;
        let remaining = budget
            .checked_sub(started.elapsed())
            .context("implementation attempt exceeded max_minutes")?;
        let remaining = remaining.min(host.remaining()?);
        host.progress(
            assignment,
            "model.request",
            "implementor",
            json!({"execution_context":run,"worktree":path,"step_observations":transcript.len()}),
        )?;
        let prompt =
            json!({"goal":goal,"story":story,"scope":scope,"observations":transcript}).to_string();
        ensure!(
            prompt.len() <= 1024 * 1024,
            "implementation context exceeded 1 MiB"
        );
        let response=host.model.respond(&ModelRequest {role:"implementor".into(),execution_context:run.into(),model:text(goal,"implementor_model")?.into(),instructions:"Implement the accepted AEP story in this managed worktree. Follow AGENTS.md. All runnable source is Rust, CLIs use clap derive. Only write within accepted scope. Preserve tests. You may inspect files, write/delete scoped files, and run bounded inspection/build commands. Tests, review, AEP lifecycle and publication are host-owned. Finish is a proposal, never a receipt.".into(),prompt,schema:implementation_schema(),timeout:remaining})?;
        let action: ImplementationAction = serde_json::from_value(response)?;
        host.guard(assignment, goal, repo, false)?;
        match action {
            ImplementationAction::Read { paths } => {
                ensure!(paths.len() <= 32, "too many file reads");
                for name in paths {
                    let file = engine::context_file(path, &name, goal)?;
                    ensure!(
                        std::fs::metadata(&file)?.len() <= 256 * 1024,
                        "read exceeds budget"
                    );
                    transcript.push(format!("{name}:\n{}", std::fs::read_to_string(file)?));
                }
            }
            ImplementationAction::Write {
                path: name,
                contents,
            } => {
                scoped(&name, scope)?;
                ensure!(contents.len() <= 256 * 1024, "write exceeds budget");
                let file = engine::confined(path, &name, true)?;
                std::fs::create_dir_all(file.parent().context("write parent")?)?;
                std::fs::write(file, contents)?;
                transcript.push(format!("wrote {name}"));
                host.progress(
                    assignment,
                    "file.write.completed",
                    "implementor",
                    json!({"path":name}),
                )?;
            }
            ImplementationAction::Delete { path: name } => {
                scoped(&name, scope)?;
                std::fs::remove_file(engine::confined(path, &name, false)?)?;
                transcript.push(format!("deleted {name}"));
                host.progress(
                    assignment,
                    "file.delete.completed",
                    "implementor",
                    json!({"path":name}),
                )?;
            }
            ImplementationAction::Run { program, args } => {
                ensure!(args.len() <= 128, "command argument limit");
                ensure!(
                    matches!(program.as_str(), "cargo" | "rg" | "git" | "ess"),
                    "model command is not admitted"
                );
                if program == "git" {
                    ensure!(
                        args.first().is_some_and(|a| matches!(
                            a.as_str(),
                            "status" | "diff" | "log" | "show" | "ls-files"
                        )),
                        "model cannot mutate Git or publish"
                    );
                }
                if program == "cargo" {
                    ensure!(
                        args.first().is_some_and(|a| matches!(
                            a.as_str(),
                            "test" | "check" | "clippy" | "fmt" | "build"
                        )),
                        "model cargo operation is not admitted"
                    );
                }
                if program == "ess" {
                    ensure!(
                        args.starts_with(&["specify".into(), "validate".into()]),
                        "model ESS operation is not admitted"
                    );
                }
                ensure!(
                    !args.iter().any(|arg| arg.starts_with('/')
                        || arg.contains("=/")
                        || arg.contains("../")
                        || arg.starts_with("--pre")
                        || arg.starts_with("--ext-diff")
                        || arg.starts_with("--textconv")
                        || arg.starts_with("--git-dir")
                        || arg.starts_with("--work-tree")
                        || arg.starts_with("--config")),
                    "command escapes managed worktree"
                );
                let mut runner = host.runner.clone();
                runner.timeout = runner.timeout.min(remaining);
                host.progress(
                    assignment,
                    "tool.run",
                    "implementor",
                    json!({"program":program,"args":args}),
                )?;
                transcript.push(runner.run(path, &program, &args, None)?);
                host.progress(
                    assignment,
                    "tool.run.completed",
                    "implementor",
                    json!({"program":program}),
                )?;
            }
            ImplementationAction::Finish { summary } => {
                ensure!(!summary.trim().is_empty(), "implementation summary missing");
                for name in git(host, path, &["diff", "--name-only", "HEAD"])?.lines() {
                    scoped(name, scope)?;
                }
                for name in
                    git(host, path, &["ls-files", "--others", "--exclude-standard"])?.lines()
                {
                    scoped(name, scope)?;
                }
                return Ok(());
            }
        }
    }
    bail!("implementation exhausted the action limit")
}
fn implementation_schema() -> Value {
    json!({"oneOf":[
        {"type":"object","properties":{"action":{"const":"read"},"paths":{"type":"array","items":{"type":"string"}}},"required":["action","paths"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"write"},"path":{"type":"string"},"contents":{"type":"string"}},"required":["action","path","contents"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"delete"},"path":{"type":"string"}},"required":["action","path"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"run"},"program":{"type":"string"},"args":{"type":"array","items":{"type":"string"}}},"required":["action","program","args"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"finish"},"summary":{"type":"string"}},"required":["action","summary"],"additionalProperties":false}
    ]})
}
fn attestation(
    kind: &str,
    result: &str,
    candidate: &str,
    origin: EvidenceOrigin,
) -> Result<AttestedEvidence> {
    Ok(AttestedEvidence {
        record: serde_json::from_value(
            json!({"format":"canon-evidence/1","id":uuid::Uuid::new_v4().to_string(),"kind":kind,"result":result,"subject":"implementation","subject_revision":candidate}),
        )?,
        origin,
    })
}
fn verification_evidence(
    candidate: &str,
    implementor: &str,
    reviewer: &str,
    checks: &str,
    review: &Value,
) -> Result<Vec<AttestedEvidence>> {
    ensure!(
        implementor != reviewer,
        "independent review context equals implementation context"
    );
    Ok(vec![
        attestation(
            "test_result",
            "pass",
            candidate,
            EvidenceOrigin::TestRunner {
                producer: "control-plane-host-process".into(),
                observation: engine::digest(&json!({"output":checks})),
                revision: candidate.into(),
                exit_code: 0,
            },
        )?,
        attestation(
            "independent_code_review",
            "approved",
            candidate,
            EvidenceOrigin::IndependentReviewer {
                producer: "control-plane-review-adapter".into(),
                observation: engine::digest(review),
                revision: candidate.into(),
                execution_context: reviewer.into(),
            },
        )?,
    ])
}
fn case(candidate: &str, id: &str) -> Result<canon::model::Case> {
    Ok(serde_json::from_value(
        json!({"format":"canon-case/1","id":id,"protocol":"software.change.merge","artifacts":{"implementation":{"revision":candidate}}}),
    )?)
}
fn publish(
    execution: &Execution<'_>,
    candidate: &str,
    base: &str,
    evidence: &[AttestedEvidence],
) -> Result<()> {
    let Execution {
        host,
        assignment,
        goal,
        repo,
        path,
        run: implementor,
    } = *execution;
    host.guard(assignment, goal, repo, true)?;
    ensure!(
        git(host, path, &["rev-parse", "HEAD"])? == candidate
            && git(host, path, &["status", "--porcelain"])?.is_empty(),
        "publication candidate changed"
    );
    let target = text(repo, "base_branch")?;
    ensure!(
        remote_head(host, path, target)? == base,
        "remote target changed; candidate evidence is stale"
    );
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).map_err(anyhow::Error::msg)?;
    let decision = protocol
        .evaluate_effect(
            &case(candidate, text(assignment, "case_id")?)?,
            evidence,
            EvaluationContext {
                at: &now(),
                implementor_context: Some(implementor),
            },
            "repository.merge",
            |_| {
                host.guard(assignment, goal, repo, true)
                    .map(|_| true)
                    .map_err(|e| e.to_string())
            },
        )
        .map_err(anyhow::Error::msg)?;
    ensure!(
        decision
            .actions
            .as_ref()
            .is_some_and(|a| a["repository.merge"]["status"] == "admissible"),
        "Canon refused verified merge: {decision:?}"
    );
    let created=host.execute("PreparePublication",json!({"assignment_id":assignment["assignment_id"],"candidate":candidate,"target":target,"expected_base":base}))?;
    let publication_id = created["published"][0]["payload"]["publication_id"].clone();
    ensure!(publication_id.is_string(), "publication id missing");
    host.execute(
        "MergeAssignment",
        json!({"assignment_id":assignment["assignment_id"]}),
    )?;
    host.guard(assignment, goal, repo, true)?;
    ensure!(
        remote_head(host, path, target)? == base,
        "remote base changed before publication effect"
    );
    let bindings = BTreeMap::from([
        ("candidate", candidate.into()),
        ("expected_base", base.into()),
        ("target", target.into()),
        ("operation_id", publication_id.as_str().unwrap().into()),
    ]);
    host.progress(assignment,"publication.invoke","host",json!({"operation_id":publication_id,"candidate":candidate,"expected_base":base,"target":target}))?;
    host.guard(assignment, goal, repo, true)?;
    let outcome = command(host, path, text(repo, "publish_command")?, &bindings);
    let intent = host.row("PublicationIntentList", "publication_id", &publication_id)?;
    match observe_merge(host, repo, &intent) {
        Ok(Some(receipt)) => {
            let mut completed = evidence.to_vec();
            completed.push(attestation(
                "merge_observation",
                "merged",
                candidate,
                EvidenceOrigin::RemoteObserver {
                    producer: "control-plane-git-observer".into(),
                    observation: receipt.clone(),
                    candidate: candidate.into(),
                },
            )?);
            let decision = protocol
                .evaluate(
                    &case(candidate, text(assignment, "case_id")?)?,
                    &completed,
                    EvaluationContext {
                        at: &now(),
                        implementor_context: Some(implementor),
                    },
                )
                .map_err(anyhow::Error::msg)?;
            ensure!(
                decision
                    .outcomes
                    .as_ref()
                    .is_some_and(|o| o["accepted"]["status"] == "legitimate"),
                "Canon has not accepted observed merge"
            );
            host.execute(
                "ConfirmPublication",
                json!({"publication_id":publication_id,"receipt":receipt}),
            )?;
            host.execute(
                "CompleteAssignment",
                json!({"assignment_id":assignment["assignment_id"],"merge_receipt":receipt}),
            )?;
            host.progress(
                assignment,
                "merge.completed",
                "host",
                json!({"receipt":receipt}),
            )?;
            Ok(())
        }
        observation => {
            host.execute(
                "MarkPublicationUncertain",
                json!({"publication_id":publication_id}),
            )?;
            match outcome {
                Err(error) => Err(error
                    .context("publication failed; exact intent requires remote reconciliation")),
                Ok(_) => bail!(
                    "publisher returned without an observed merge: {}",
                    match observation {
                        Err(error) => error.to_string(),
                        _ => "candidate not on target".into(),
                    }
                ),
            }
        }
    }
}
fn observe_merge(host: &Host, repo: &Value, intent: &Value) -> Result<Option<String>> {
    let path = Path::new(text(repo, "path")?);
    let head = fetch(host, path, text(intent, "target")?)?;
    let candidate = text(intent, "candidate")?;
    let base = text(intent, "expected_base")?;
    ensure!(
        git(host, path, &["merge-base", candidate, base])? == base,
        "candidate is not based on expected publication base"
    );
    if git(host, path, &["merge-base", candidate, &head])? != candidate {
        return Ok(None);
    }
    Ok(Some(json!({"kind":"git_merge_observation","operation_id":intent["publication_id"],"candidate":candidate,"expected_base":base,"target":intent["target"],"observed_head":head,"origin":git(host,path,&["remote","get-url","origin"])?,"observed_at":now()}).to_string()))
}
fn reconcile_publications(host: &Host) -> Result<()> {
    let repositories = host.rows("RepositoryRegistrationList")?;
    for intent in host.rows("PublicationIntentList")? {
        let assignment = host.row("AssignmentList", "assignment_id", &intent["assignment_id"])?;
        if assignment["state"] == "Merged" {
            continue;
        }
        let repo = repositories
            .iter()
            .find(|r| r["repository_id"] == assignment["repository_id"])
            .context("publication repository missing")?;
        match observe_merge(host, repo, &intent) {
            Ok(Some(receipt)) => {
                let receipt = if intent["state"] == "Confirmed" {
                    text(&intent, "receipt")?.to_owned()
                } else {
                    host.execute(
                        "ConfirmPublication",
                        json!({"publication_id":intent["publication_id"],"receipt":receipt}),
                    )?;
                    receipt
                };
                if assignment["state"] != "Merging" && assignment["state"] != "Blocked" {
                    host.block(
                        &assignment["assignment_id"],
                        "Recover publication observed on its exact target",
                    )?;
                }
                host.execute(
                    "ReconcileAssignment",
                    json!({"assignment_id":assignment["assignment_id"],"merge_receipt":receipt}),
                )?;
            }
            Ok(None) => {
                if intent["state"] == "Prepared" {
                    host.execute(
                        "MarkPublicationUncertain",
                        json!({"publication_id":intent["publication_id"]}),
                    )?;
                }
                host.block(
                    &assignment["assignment_id"],
                    "Publication outcome remains unresolved; no duplicate effect will be invoked",
                )?;
            }
            Err(error) => host.block(
                &assignment["assignment_id"],
                &format!("Publication observation unavailable: {error:#}"),
            )?,
        }
    }
    Ok(())
}

fn satisfy_goals(host: &Host) -> Result<(usize, Vec<String>)> {
    let assignments = host.rows("AssignmentList")?;
    let repositories = host.rows("RepositoryRegistrationList")?;
    let mut satisfied = 0;
    let mut blockers = Vec::new();
    for goal in host
        .rows("GoalList")?
        .into_iter()
        .filter(|g| g["state"] == "Running")
    {
        let assigned = assignments
            .iter()
            .filter(|a| a["goal_id"] == goal["goal_id"])
            .collect::<Vec<_>>();
        if assigned.is_empty()
            || assigned
                .iter()
                .any(|a| !matches!(a["state"].as_str(), Some("Merged" | "Cancelled")))
        {
            continue;
        }
        let current = assigned
            .iter()
            .filter(|a| a["state"] == "Merged" && a["goal_revision"] == goal["revision"])
            .copied()
            .collect::<Vec<_>>();
        let repos = repositories
            .iter()
            .filter(|r| r["workspace_id"] == goal["workspace_id"] && r["state"] == "Registered")
            .collect::<Vec<_>>();
        if repos.is_empty()
            || repos.iter().any(|repo| {
                !current
                    .iter()
                    .any(|a| a["repository_id"] == repo["repository_id"])
            })
        {
            continue;
        }
        let directories = host
            .rows("WorkspaceDirectoryList")?
            .into_iter()
            .filter(|d| d["workspace_id"] == goal["workspace_id"] && d["state"] == "Registered")
            .collect::<Vec<_>>();
        let heads = repos
            .iter()
            .map(|repo| {
                let observation = (|| {
                    remote_head(
                        host,
                        Path::new(text(repo, "path")?),
                        text(repo, "base_branch")?,
                    )
                })();
                match observation {
                    Ok(head) => json!({"repository":repo,"head":head}),
                    Err(error) => json!({"repository":repo,"error":error.to_string()}),
                }
            })
            .collect::<Vec<_>>();
        let fingerprint = engine::digest(
            &json!({"revision":goal["revision"],"assignments":current,"targets":heads,"directories":directories}),
        );
        let prior: Value = serde_json::from_str(goal["planning_receipt"].as_str().unwrap_or("{}"))
            .unwrap_or_else(|_| json!({}));
        if prior["acceptance"]["fingerprint"] == fingerprint
            && prior["acceptance"]["status"] == "failed"
        {
            continue;
        }
        host.acceptance_progress(
            &goal,
            &fingerprint,
            "running",
            "Checking observed merged targets",
        )?;
        let result = (|| -> Result<()> {
            let budget = attempt_budget(&goal)?;
            let mut worker = host.clone();
            worker.runner.cancel = host.runner.cancel.child_token();
            worker.deadline = Some(
                Instant::now()
                    .checked_add(budget)
                    .context("goal acceptance deadline overflow")?,
            );
            let _deadline = AttemptDeadline::start(worker.runner.cancel.clone(), budget);
            let host = &worker;
            let mut observations = Vec::new();
            for repo in &repos {
                let current_goal = host.row("GoalList", "goal_id", &goal["goal_id"])?;
                ensure!(
                    current_goal["state"] == "Running"
                        && current_goal["revision"] == goal["revision"],
                    "goal changed during acceptance checks"
                );
                ensure!(
                    host.row(
                        "RepositoryRegistrationList",
                        "repository_id",
                        &repo["repository_id"]
                    )? == **repo,
                    "repository configuration changed during acceptance"
                );
                let assignment = current
                    .iter()
                    .find(|a| a["repository_id"] == repo["repository_id"])
                    .context("merged assignment missing")?;
                let path = Path::new(text(repo, "path")?);
                let head = fetch(host, path, text(repo, "base_branch")?)?;
                for a in current
                    .iter()
                    .filter(|a| a["repository_id"] == repo["repository_id"])
                {
                    ensure!(
                        !text(a, "merge_receipt")?.is_empty()
                            && git(host, path, &["merge-base", text(a, "candidate")?, &head])?
                                == text(a, "candidate")?,
                        "merged assignment no longer appears on the observed target"
                    );
                }
                let id = format!("cp-accept-{}", uuid::Uuid::new_v4());
                let checkout = tree(host, repo, &id, &head)?;
                let context = format!("acceptance-{}", uuid::Uuid::new_v4());
                let lease = Lease::acquire(host, &checkout, &context)?;
                let observed = (|| -> Result<Value> {
                    host.progress(assignment,"goal.checks","host",json!({"observed_head":head,"worktree":checkout,"command":repo["test_command"]}))?;
                    let checks = command(
                        host,
                        &checkout,
                        text(repo, "test_command")?,
                        &BTreeMap::new(),
                    )?;
                    ensure!(
                        git(host, &checkout, &["status", "--porcelain"])?.is_empty(),
                        "goal acceptance checks modified observed target"
                    );
                    let diff = git(
                        host,
                        &checkout,
                        &["diff", text(assignment, "base_revision")?, &head, "--"],
                    )?;
                    Ok(
                        json!({"repository":repo["repository_id"],"head":head,"checks":checks,"diff":diff,"merge_receipts":current.iter().filter(|a|a["repository_id"]==repo["repository_id"]).map(|a|a["merge_receipt"].clone()).collect::<Vec<_>>()}),
                    )
                })();
                let released = lease.release();
                observations.push(observed?);
                released?;
            }
            let reviewer = format!("goal-reviewer-{}", uuid::Uuid::new_v4());
            host.progress(
                current[0],
                "goal.review",
                "goal_reviewer",
                json!({"execution_context":reviewer}),
            )?;
            let review=host.model.respond(&ModelRequest {role:"goal_reviewer".into(),execution_context:reviewer.clone(),model:text(&goal,"reviewer_model")?.into(),instructions:"Independently decide whether the exact standing objective and acceptance are satisfied by all observed merged targets, real check outputs and diffs. Do not infer completion from an empty queue or story statuses. Reject any acceptance obligation unsupported by evidence.".into(),prompt:json!({"goal":goal,"observations":observations}).to_string(),schema:crate::model::critique_schema(),timeout:host.remaining()?})?;
            ensure!(
                review["approved"] == true
                    && review["reason"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty()),
                "final goal review rejected: {review}"
            );
            let current_goal = host.row("GoalList", "goal_id", &goal["goal_id"])?;
            host.remaining()?;
            ensure!(
                !host.runner.cancel.is_cancelled(),
                "goal acceptance cancelled"
            );
            ensure!(
                current_goal["state"] == "Running" && current_goal["revision"] == goal["revision"],
                "goal changed during final review"
            );
            let current_repos = host
                .rows("RepositoryRegistrationList")?
                .into_iter()
                .filter(|r| r["workspace_id"] == goal["workspace_id"] && r["state"] == "Registered")
                .collect::<Vec<_>>();
            let current_directories = host
                .rows("WorkspaceDirectoryList")?
                .into_iter()
                .filter(|d| d["workspace_id"] == goal["workspace_id"] && d["state"] == "Registered")
                .collect::<Vec<_>>();
            ensure!(
                json!(current_repos) == json!(repos) && current_directories == directories,
                "workspace membership or repository configuration changed during acceptance"
            );
            for repo in &repos {
                let observed = observations
                    .iter()
                    .find(|o| o["repository"] == repo["repository_id"])
                    .unwrap();
                ensure!(
                    remote_head(
                        host,
                        Path::new(text(repo, "path")?),
                        text(repo, "base_branch")?
                    )? == text(observed, "head")?,
                    "target changed during goal acceptance review"
                );
            }
            let receipt=json!({"kind":"goal_acceptance","goal_revision":goal["revision"],"observations":observations,"review_context":reviewer,"review":review}).to_string();
            host.execute(
                "SatisfyGoal",
                json!({"goal_id":goal["goal_id"],"satisfaction_receipt":receipt}),
            )?;
            host.progress(
                current[0],
                "goal.acceptance.completed",
                "host",
                json!({"goal_revision":goal["revision"]}),
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                host.acceptance_progress(
                    &goal,
                    &fingerprint,
                    "completed",
                    "Observed targets satisfy the objective",
                )?;
                satisfied += 1;
            }
            Err(error) => {
                let reason = format!("Goal acceptance blocked: {error:#}");
                // A concurrent operator revision owns its own progress; never overwrite it.
                if host.row("GoalList", "goal_id", &goal["goal_id"])?["revision"]
                    == goal["revision"]
                {
                    host.acceptance_progress(&goal, &fingerprint, "failed", &reason)?;
                    host.progress(
                        current[0],
                        "blocked",
                        "goal_reviewer",
                        json!({"reason":reason}),
                    )?;
                }
                blockers.push(reason);
            }
        }
    }
    Ok((satisfied, blockers))
}

struct AttemptDeadline {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}
fn attempt_budget(goal: &Value) -> Result<Duration> {
    let seconds = goal["max_minutes"]
        .as_u64()
        .context("attempt deadline")?
        .checked_mul(60)
        .context("attempt deadline overflow")?;
    ensure!(seconds > 0, "attempt deadline must be positive");
    let duration = Duration::from_secs(seconds);
    Instant::now()
        .checked_add(duration)
        .context("attempt deadline exceeds supported range")?;
    Ok(duration)
}
impl AttemptDeadline {
    fn start(cancel: CancellationToken, budget: Duration) -> Self {
        let (stop, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            if rx.recv_timeout(budget) == Err(mpsc::RecvTimeoutError::Timeout) {
                cancel.cancel();
            }
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for AttemptDeadline {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_attempt_deadline_cancels_even_without_a_process() {
        let cancel = CancellationToken::new();
        let _deadline = AttemptDeadline::start(cancel.clone(), Duration::from_millis(10));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), cancel.cancelled())
                .await
                .unwrap();
        });
    }

    #[test]
    fn impossible_deadline_is_a_blocker_instead_of_a_panic() {
        assert!(attempt_budget(&json!({"max_minutes":u64::MAX})).is_err());
    }
}
