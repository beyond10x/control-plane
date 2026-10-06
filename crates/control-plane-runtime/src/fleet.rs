//! Trusted fleet effects; model proposals never construct validation or publication receipts.
use crate::{
    AgentModel, ModelRequest, RuntimeConfig, SharedStore, TickReport, engine,
    process::ProcessRunner, refusal::InputRefusal,
};
use anyhow::{Context, Result, bail, ensure};
use control_plane_core::Actor;
use control_plane_protocol::{AttestedEvidence, EvidenceOrigin};
use loom_sdk::commission::ports::governor::Governor;
use loom_sdk::commission::{
    model::{behaviour::Generated, json as wire, responsibility::*},
    outcome::RunStore,
    ports::{
        effect::{AdmittedRequest, EffectError, EffectPort},
        executor::AgentExecutor,
    },
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

/// Bytes kept of a goal review's reason. The acceptance record is recorded once per review,
/// in the decision's body and again in its outcome, which together stay within 16 KiB;
/// later progress leaves it to the goal's journal.
const ACCEPTANCE_REASON_BYTES: usize = 5 * 1024;

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
    fn respond(&self, request: &ModelRequest, path: &Path, assignment: &Value) -> Result<Value> {
        self.respond_continuing(request, path, assignment, None)
    }
    fn respond_continuing(
        &self,
        request: &ModelRequest,
        path: &Path,
        assignment: &Value,
        continuation: Option<String>,
    ) -> Result<Value> {
        let host = self.clone();
        let assignment = assignment.clone();
        self.model.respond_in(
            request,
            &crate::ModelEnvironment {
                workspace: path.into(),
                max_turns: self.config.max_steps as u64,
                cancel: self.runner.cancel.clone(),
                progress: Arc::new(move |event| {
                    let goal = host.row("GoalList", "goal_id", &assignment["goal_id"])?;
                    ensure!(
                        goal["state"] == "Running"
                            && goal["revision"] == assignment["goal_revision"],
                        "goal paused, cancelled or revision changed during model turn"
                    );
                    host.progress(&assignment, "loom.event", "runtime", event)
                }),
                continuation,
            },
        )
    }
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
            record_progress(
                &mut *self.store.lock().await,
                assignment,
                action,
                role,
                detail,
            )
            .await
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
            receipt["acceptance"] = json!({"fingerprint":fingerprint,"status":status,"reason":control_plane_core::bounded_text(reason, ACCEPTANCE_REASON_BYTES),"at":now(),"goal_revision":goal["revision"]});
            let mut body = current.as_object().context("goal object")?.clone();
            body.retain(|key,_|key.starts_with("planning_")||key=="goal_id");
            body.insert("planning_receipt".into(),json!(receipt.to_string()));
            let outcome=store.execute("RecordPlanningProgress",Value::Object(body),Actor::Supervisor).await?;
            ensure!(outcome["outcome"]=="applied","acceptance progress refused: {outcome}");
            Ok(())
        })
    }
    /// The goal's newest acceptance record, which its progress journal keeps.
    fn acceptance(&self, goal: &Value) -> Result<Value> {
        let goal = text(goal, "goal_id")?;
        self.handle.block_on(async {
            Ok(self.store.lock().await.activity_history(goal)?["acceptance"].clone())
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
/// Record one runtime progress event of an assignment on its goal's planning receipt.
/// The decision carries this activity alone; the host journal keeps the goal's history and
/// each assignment's newest activity.
pub(crate) async fn record_progress(
    store: &mut control_plane_core::Store,
    assignment: &Value,
    action: &str,
    role: &str,
    detail: Value,
) -> Result<()> {
    text(assignment, "assignment_id")?;
    let status = if action == "blocked" {
        "failed"
    } else if action.ends_with(".completed") {
        "completed"
    } else {
        "running"
    };
    let activity = json!({"id":uuid::Uuid::new_v4().to_string(),"assignment_id":assignment["assignment_id"],"goal_revision":assignment["goal_revision"],"action":action,"role":role,"at":now(),"worktree":assignment["worktree_id"],"status":status,"detail":detail});
    let goal = text(assignment, "goal_id")?;
    let outcome = store.record_activity(goal, activity).await?;
    ensure!(
        outcome["outcome"] == "applied",
        "progress append refused: {outcome}"
    );
    Ok(())
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
/// Run a repository-configured command. Only the publish command is `trusted`: it alone
/// receives `RuntimeConfig::credentials`; checks run model-written code and never do.
fn command(
    host: &Host,
    path: &Path,
    configured: &str,
    bindings: &BTreeMap<&str, String>,
    trusted: bool,
) -> Result<String> {
    let mut argv = shell_words::split(configured).context("invalid configured command quoting")?;
    ensure!(!argv.is_empty(), "repository command is not configured");
    for arg in &mut argv {
        for (key, value) in bindings {
            *arg = arg.replace(&format!("{{{key}}}"), value);
        }
    }
    let mut runner = if trusted {
        host.config.credentialed(&host.runner)
    } else {
        host.runner.clone()
    };
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
        let command = host.config.commit_for(path, &host.runner)?;
        let (program, prefix) = command.split_first().context("commit command empty")?;
        let args = [prefix.to_vec(), vec![message.to_owned()]].concat();
        host.config
            .credentialed(&host.runner)
            .run(path, program, &args, None)?;
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
            let repositories = discovery.rows("RepositoryRegistrationList")?;
            let assignments = discovery.rows("AssignmentList")?;
            let mut open = Vec::new();
            for goal in goals {
                let acceptance = discovery.acceptance(&goal)?;
                if !acceptance_is_unchanged(
                    &goal,
                    &acceptance,
                    &repositories,
                    &assignments,
                    &discovery.runner,
                ) {
                    open.push(goal);
                }
            }
            let goals = open;
            Ok((
                goals,
                repositories,
                assignments,
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
        } else if matches!(
            a["state"].as_str(),
            Some("Implementing" | "Reviewing" | "ReadyToMerge")
        ) && a["attempt"].as_i64().context("invalid attempt count")?
            >= goal["max_attempts"]
                .as_i64()
                .context("invalid attempt limit")?
        {
            host.block(&a["assignment_id"],"Interrupted attempt exhausted max_attempts; no further model or implementation effects are permitted")?;
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
    target_head(&host.runner, path, target)
}
fn target_head(runner: &ProcessRunner, path: &Path, target: &str) -> Result<String> {
    runner.command(
        path,
        "git",
        &["check-ref-format", &format!("refs/heads/{target}")],
    )?;
    let output = runner.command(
        path,
        "git",
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

fn acceptance_fingerprint(
    goal: &Value,
    repositories: &[Value],
    assignments: &[Value],
    runner: &ProcessRunner,
) -> (String, bool) {
    let current = assignments
        .iter()
        .filter(|a| {
            a["goal_id"] == goal["goal_id"]
                && a["goal_revision"] == goal["revision"]
                && a["state"] == "Merged"
        })
        .collect::<Vec<_>>();
    let heads = repositories
        .iter()
        .filter(|r| r["workspace_id"] == goal["workspace_id"] && r["state"] == "Registered")
        .map(|repo| {
            let observation = (|| {
                target_head(
                    runner,
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
    let observed = heads.iter().all(|head| head.get("error").is_none());
    (
        engine::digest(
            &json!({"revision":goal["revision"],"assignments":current,"targets":heads,"directories":goal["directories"]}),
        ),
        observed,
    )
}

/// One durable rejection latch shared by both scheduling paths. Progress and
/// timestamps cannot unlock it; goal revision, repository configuration, directory
/// membership or an observed published target must change.
/// `acceptance` is the goal's newest acceptance record (`Store::activity_history`).
pub(crate) fn acceptance_is_unchanged(
    goal: &Value,
    acceptance: &Value,
    repositories: &[Value],
    assignments: &[Value],
    runner: &ProcessRunner,
) -> bool {
    if acceptance["status"] != "failed" || acceptance["goal_revision"] != goal["revision"] {
        return false;
    }
    let (fingerprint, observed) = acceptance_fingerprint(goal, repositories, assignments, runner);
    // A failed observation is not evidence of a changed repository target.
    !observed || acceptance["fingerprint"] == fingerprint
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
        let checks = command(
            host,
            &path,
            text(repo, "test_command")?,
            &BTreeMap::new(),
            false,
        )?;
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
        let checks = command(
            host,
            &path,
            text(repo, "test_command")?,
            &BTreeMap::new(),
            false,
        )?;
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
        let review=host.respond(&ModelRequest {role:"reviewer".into(),execution_context:reviewer.clone(),model:text(goal,"reviewer_model")?.into(),instructions:"Independently review the exact candidate and real host check output against this accepted AEP story and standing goal. You cannot grant publication authority or fabricate check evidence.".into(),prompt:json!({"goal":crate::context::goal_brief(goal),"story":story,"candidate":candidate,"base":base,"diff":diff,"checks":checks}).to_string(),schema:crate::model::critique_schema(),timeout:host.remaining()?}, &path, &assignment)?;
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
/// Admit a model write or delete target. `.git` stays a fatal confinement failure; every
/// other rule here is a refusal the model can correct.
fn scoped(path: &str, scope: &[String], allow_go: bool) -> Result<()> {
    let relative = Path::new(path);
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
    {
        return Err(InputRefusal::new(
            "write_path_syntax",
            format!("{path} is not a normalized worktree-relative path"),
            "Write and delete paths are worktree-relative, without absolute paths or parent traversal.",
        )
        .into());
    }
    ensure!(
        !relative.starts_with(".git"),
        "Git administrative paths are host-owned"
    );
    if relative.starts_with(".engineering") {
        return Err(InputRefusal::new(
            "planning_store_host_owned",
            format!("{path} is in the AEP store"),
            "The host records tests, evidence and lifecycle in .engineering; change source files only.",
        )
        .into());
    }
    if !scope.iter().any(|root| {
        relative == Path::new(root) || relative.starts_with(Path::new(root.trim_end_matches('/')))
    }) {
        return Err(InputRefusal::new(
            "write_outside_scope",
            format!(
                "{path} is outside the accepted story scope ({})",
                scope.join(", ")
            ),
            "Write and delete only beneath the accepted story scope paths.",
        )
        .into());
    }
    let frontend = (relative.starts_with("frontend") || relative.starts_with("web"))
        && !relative.components().skip(1).any(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some("backend" | "server" | "tools" | "scripts")
            )
        })
        && matches!(
            relative.extension().and_then(|e| e.to_str()),
            Some("js" | "jsx" | "ts" | "tsx" | "vue")
        );
    let admitted_language = frontend
        || (allow_go
            && relative
                .extension()
                .is_some_and(|extension| extension == "go"))
        || !matches!(
            relative.extension().and_then(|e| e.to_str()),
            Some(
                "py" | "sh"
                    | "bash"
                    | "js"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "vue"
                    | "rb"
                    | "go"
            )
        );
    if !admitted_language {
        return Err(InputRefusal::new(
            "write_language_policy",
            format!("{path} is not admitted source in this repository"),
            "Backend and tooling source must be Rust; JS/TS/Vue frontend assets require accepted frontend/ or web/ scope.",
        )
        .into());
    }
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
    let host = execution.host;
    let path = execution.path;
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
    let case_id = CaseId(execution.run.into());
    let progress_host = host.clone();
    let assignment = execution.assignment.clone();
    let governor = crate::governance::open(
        control_plane_protocol::VERIFIED_MERGE_YAML,
        "source-delivery@1",
        &case_id,
        BTreeMap::from([("implementation".into(), execution.run.into())]),
        Arc::new(move |kind, value| {
            progress_host.progress(&assignment, kind, "runtime", value.clone())
        }),
    )?;
    let phase = ImplementationPhase {
        execution: *execution,
        story,
        scope,
        governor,
        case_id,
        transcript: std::sync::Mutex::new(transcript),
        pending: std::sync::Mutex::new(None),
        finished: std::sync::atomic::AtomicBool::new(false),
        failure: std::sync::Mutex::new(None),
        refusals: Default::default(),
        allow_go: host.config.local_eval(path, &host.runner)?,
    };
    let executor = ImplementationExecutor {
        phase: &phase,
        loom: loom_sdk::Loom::new(&phase, &phase, text(execution.goal, "objective")?),
    };
    let commission = Commission::new(CommissionData {
        commission_id: CommissionId(runtime_id()),
        agent_revision_id: AgentRevisionId(runtime_id()),
        case_id: phase.case_id.clone(),
        principal: PrincipalId("control-plane-supervisor".into()),
        authority_context: AuthorityContext(wire::Value::Null),
    });
    let mut runs = Generated::new(RunStore::new(|| RunId(runtime_id())));
    let end = loom_sdk::run_until_blocked(
        &phase.governor,
        &executor,
        &engine::NoAuthority,
        &phase,
        &commission,
        &mut runs,
        &mut crate::governance::ContextClock {
            max_steps: host.config.max_steps + 1,
        },
    )?;
    if let Some(error) = phase
        .failure
        .lock()
        .map_err(|_| anyhow::anyhow!("implementation failure state poisoned"))?
        .as_ref()
    {
        bail!("{error}");
    }
    ensure!(
        phase.finished.load(std::sync::atomic::Ordering::SeqCst),
        "implementation ended before a candidate proposal: {:?}",
        end.outcome
    );
    ensure!(
        matches!(
            end.outcome,
            RunOutcome::Suspended(RunOutcomeSuspended {
                reason: SuspensionReason::Evidence(_)
            })
        ),
        "unexpected implementation stop: {:?}",
        end.outcome
    );
    // This is only the model's candidate proposal. Tests, independent review and observed
    // publication still have to produce trusted evidence before the case can complete.
    Ok(())
}
fn runtime_id() -> loom_sdk::commission::model::primitives::Uuid {
    loom_sdk::commission::model::primitives::Uuid(uuid::Uuid::new_v4().to_string())
}
struct ImplementationPhase<'a> {
    execution: Execution<'a>,
    story: &'a Value,
    scope: &'a [String],
    governor: crate::governance::HostGovernor,
    case_id: CaseId,
    transcript: std::sync::Mutex<Vec<String>>,
    pending: std::sync::Mutex<Option<ImplementationAction>>,
    finished: std::sync::atomic::AtomicBool,
    failure: std::sync::Mutex<Option<String>>,
    refusals: crate::refusal::RefusalBudget,
    allow_go: bool,
}
struct ImplementationExecutor<'a> {
    phase: &'a ImplementationPhase<'a>,
    loom: loom_sdk::Loom<&'a ImplementationPhase<'a>, &'a ImplementationPhase<'a>>,
}
impl AgentExecutor for ImplementationExecutor<'_> {
    fn run(
        &self,
        commission: &Commission<commission_state::Assigned>,
        frontier: &Frontier<frontier_state::Issued>,
    ) -> ExecutorOutcome {
        if self
            .phase
            .finished
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            ExecutorOutcome::Suspended(ExecutorOutcomeSuspended {
                reason: SuspensionReason::Evidence(vec![
                    "test_result".into(),
                    "independent_code_review".into(),
                    "merge_observation".into(),
                ]),
            })
        } else {
            self.loom.run(commission, frontier)
        }
    }
}
impl loom_sdk::ActionSelector for &ImplementationPhase<'_> {
    fn select(
        &self,
        _: &loom_sdk::loom::selection::SelectionContext,
        candidates: &[loom_sdk::loom::model::run::CatalogueEntry],
    ) -> Result<loom_sdk::loom::selection::Choice, loom_sdk::loom::selection::SelectorError> {
        let proposed=(||->Result<String>{
            let Execution {host,assignment,goal,repo,path,run}=self.execution;
            host.guard(assignment,goal,repo,false)?;
            let language=if self.allow_go {"This isolated eval repository permits Go with the standard library and HTML/CSS frontend. Go commands are version or build/test/vet/list ./...."}else{"Backend and tooling source is Rust; CLIs use clap derive. Frontend JS/TS/JSX/TSX/Vue assets are allowed under accepted frontend/ or web/ scope, excluding backend/server/tools/scripts subdirectories. Frontend permission does not authorize JavaScript backend or tooling, or additional process commands."};
            let observations=self.transcript.lock().map_err(|_|anyhow::anyhow!("implementation context poisoned"))?.clone();
            let frontier=candidates.iter().map(|c|&c.action).collect::<Vec<_>>();
            let continuation=json!({"observations":observations,"frontier":frontier}).to_string();
            let prompt=json!({"goal":crate::context::goal_brief(goal),"story":self.story,"scope":self.scope,"observations":observations,"frontier":frontier}).to_string();
            host.progress(assignment,"model.request","implementor",json!({"execution_context":run}))?;
            let response=host.respond_continuing(&ModelRequest {role:"implementor".into(),execution_context:run.into(),model:text(goal,"implementor_model")?.into(),instructions:format!("Implement the accepted AEP story in this managed worktree. Follow AGENTS.md. {language} {} Only write within accepted scope. Preserve tests. You may inspect files, write/delete scoped files, and run bounded inspection/build commands. {COMMAND_GRAMMAR} A refused action changes nothing; correct it from the refusal. Tests, review, AEP lifecycle and publication are host-owned. Finish is a proposal, never a receipt.",crate::read_request::PATH_HELP),prompt,schema:implementation_schema(),timeout:host.remaining()?},path,assignment,Some(continuation))?;
            self.transcript.lock().map_err(|_|anyhow::anyhow!("implementation context poisoned"))?.clear();
            let action:ImplementationAction=serde_json::from_value(response)?;
            let name=implementation_protocol_action(&action).into();
            *self.pending.lock().map_err(|_|anyhow::anyhow!("implementation selection poisoned"))?=Some(action);
            Ok(name)
        })().map_err(|e|loom_sdk::loom::selection::SelectorError::Unavailable(e.to_string()))?;
        Ok(loom_sdk::loom::selection::Choice {
            action: proposed,
            confidence: None,
        })
    }
    fn strategy(&self) -> loom_sdk::loom::model::run::SelectionStrategy {
        loom_sdk::loom::model::run::SelectionStrategy::ReasoningModel
    }
}
fn implementation_protocol_action(action: &ImplementationAction) -> &'static str {
    match action {
        ImplementationAction::Read { .. } => "repository.inspect",
        ImplementationAction::Write { .. } | ImplementationAction::Delete { .. } => {
            "repository.edit"
        }
        ImplementationAction::Run { .. } => "tests.run",
        ImplementationAction::Finish { .. } => "review.request",
    }
}
impl loom_sdk::ArgumentGenerator for &ImplementationPhase<'_> {
    fn generate(
        &self,
        _: &loom_sdk::loom::arguments::ArgumentContext,
        _: &loom_sdk::loom::model::run::CatalogueEntry,
    ) -> Result<wire::Value, String> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| "implementation selection poisoned")?;
        wire::parse(
            &serde_json::to_string(pending.as_ref().ok_or("missing implementation selection")?)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("{e:?}"))
    }
}
impl EffectPort for ImplementationPhase<'_> {
    fn performs(&self, action: &str) -> bool {
        matches!(
            action,
            "repository.inspect" | "repository.edit" | "tests.run" | "review.request"
        )
    }
    fn invoke(
        &self,
        _: &Commission<commission_state::Assigned>,
        request: &AdmittedRequest,
    ) -> Result<EffectOutcome, EffectError> {
        let result = (|| -> Result<String> {
            let action = self
                .pending
                .lock()
                .map_err(|_| anyhow::anyhow!("implementation selection poisoned"))?
                .take()
                .context("missing implementation selection")?;
            ensure!(
                implementation_protocol_action(&action) == request.data().action,
                "implementation action changed"
            );
            self.perform(action)
        })();
        // Commission distinguishes a refused request from an unavailable effect port.
        // Only typed refusals of model input are recoverable here.
        let result = match result {
            Err(error) => match crate::refusal::refusal_of(&error) {
                Some(refusal) => match self.refusal_feedback(&refusal) {
                    Ok(reason) => {
                        return Ok(EffectOutcome::Refused(EffectOutcomeRefused { reason }));
                    }
                    Err(error) => Err(error),
                },
                None => Err(error),
            },
            result => result,
        };
        match result {
            Ok(report) => {
                self.refusals.admitted();
                Ok(EffectOutcome::Performed(EffectOutcomePerformed {
                    report: wire::Value::Text(report),
                }))
            }
            Err(error) => {
                let message = format!("{error:#}");
                if let Ok(mut failure) = self.failure.lock() {
                    *failure = Some(message.clone());
                }
                Err(EffectError::new(message))
            }
        }
    }
}
impl ImplementationPhase<'_> {
    fn refusal_feedback(&self, refusal: &InputRefusal) -> Result<String> {
        let Execution {
            host,
            assignment,
            goal,
            repo,
            ..
        } = self.execution;
        host.guard(assignment, goal, repo, false)?;
        let count = self.refusals.refused(refusal)?;
        let observation = refusal.observation();
        self.transcript
            .lock()
            .map_err(|_| anyhow::anyhow!("implementation context poisoned"))?
            .push(observation.clone());
        host.progress(
            assignment,
            "input.refused",
            "implementor",
            json!({"refusal":serde_json::from_str::<Value>(&observation)?,"count":count}),
        )?;
        Ok(observation)
    }
    fn perform(&self, action: ImplementationAction) -> Result<String> {
        let Execution {
            host,
            assignment,
            goal,
            repo,
            path,
            ..
        } = self.execution;
        host.guard(assignment, goal, repo, false)?;
        let mut transcript = self
            .transcript
            .lock()
            .map_err(|_| anyhow::anyhow!("implementation context poisoned"))?;
        let mutation = matches!(
            action,
            ImplementationAction::Write { .. }
                | ImplementationAction::Delete { .. }
                | ImplementationAction::Run { .. }
        );
        match action {
            ImplementationAction::Read { paths } => {
                if paths.len() > 32 {
                    return Err(InputRefusal::new(
                        "too_many_reads",
                        format!("{} paths requested", paths.len()),
                        "Read at most 32 files per request.",
                    )
                    .into());
                }
                for name in &paths {
                    crate::read_request::parse(name)?;
                }
                // Every file is checked before any content enters the transcript, so a
                // refusal reports a request that read nothing.
                let mut observations = Vec::new();
                for name in paths {
                    let file = engine::context_path(path, &name, goal, true)?;
                    let metadata = match std::fs::metadata(&file) {
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            observations
                                .push(format!("File not found: {name}. Create it before reading."));
                            continue;
                        }
                        metadata => metadata?,
                    };
                    if metadata.len() > 256 * 1024 {
                        return Err(InputRefusal::new(
                            "read_too_large",
                            format!("{name} has {} bytes", metadata.len()),
                            "Files over 256 KiB cannot be read whole; search them with rg PATTERN path.",
                        )
                        .into());
                    }
                    let contents = String::from_utf8(std::fs::read(file)?).map_err(|_| {
                        InputRefusal::new(
                            "read_not_utf8",
                            format!("{name} is not UTF-8 text"),
                            "Binary and non-UTF-8 files cannot be read.",
                        )
                    })?;
                    observations.push(format!("{name}:\n{contents}"));
                }
                transcript.extend(observations);
            }
            ImplementationAction::Write {
                path: name,
                contents,
            } => {
                scoped(&name, self.scope, self.allow_go)?;
                if contents.len() > 256 * 1024 {
                    return Err(InputRefusal::new(
                        "write_too_large",
                        format!("{name} would have {} bytes", contents.len()),
                        "Write at most 256 KiB per file.",
                    )
                    .into());
                }
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
                scoped(&name, self.scope, self.allow_go)?;
                let file = engine::confined(path, &name, true)?;
                if !file.is_file() {
                    return Err(InputRefusal::new(
                        "delete_missing",
                        format!("{name} is not an existing file"),
                        "Delete only existing files within scope.",
                    )
                    .into());
                }
                std::fs::remove_file(file)?;
                transcript.push(format!("deleted {name}"));
                host.progress(
                    assignment,
                    "file.delete.completed",
                    "implementor",
                    json!({"path":name}),
                )?;
            }
            ImplementationAction::Run { program, args } => {
                let args = if program == "go" && self.allow_go {
                    if !(args == ["version"]
                        || ["build", "test", "vet", "list"]
                            .iter()
                            .any(|verb| args == [*verb, "./..."]))
                    {
                        return Err(InputRefusal::new(
                            "command_not_admitted",
                            format!("go {}", args.join(" ")),
                            COMMAND_GRAMMAR,
                        )
                        .into());
                    }
                    args
                } else {
                    inspection_arguments(path, &program, &args)?
                };
                let mut runner = host.runner.clone();
                runner.timeout = runner.timeout.min(host.remaining()?);
                host.progress(
                    assignment,
                    "tool.run",
                    "implementor",
                    json!({"program":program,"args":args}),
                )?;
                match runner.run(path, &program, &args, None) {
                    Ok(output) => transcript.push(output),
                    Err(error)
                        if error
                            .downcast_ref::<crate::process::ProcessExit>()
                            .is_some() =>
                    {
                        transcript.push(format!(
                            "Command failed (no validation evidence): {error:#}"
                        ))
                    }
                    Err(error) => return Err(error),
                }
                host.progress(
                    assignment,
                    "tool.run.completed",
                    "implementor",
                    json!({"program":program}),
                )?;
            }
            ImplementationAction::Finish { summary } => {
                if summary.trim().is_empty() {
                    return Err(InputRefusal::new(
                        "finish_summary_missing",
                        "the finish summary is empty",
                        "Finish with a one-sentence summary of the change.",
                    )
                    .into());
                }
                for name in git(host, path, &["diff", "--name-only", "HEAD"])?
                    .lines()
                    .chain(
                        git(host, path, &["ls-files", "--others", "--exclude-standard"])?.lines(),
                    )
                {
                    if let Err(error) = scoped(name, self.scope, self.allow_go) {
                        let Some(refusal) = error.downcast_ref::<InputRefusal>() else {
                            return Err(error);
                        };
                        return Err(InputRefusal::new(
                            "finish_outside_scope",
                            format!("the candidate changes {name}: {}", refusal.reason),
                            "Every changed or untracked file must be admitted source within scope before Finish.",
                        )
                        .into());
                    }
                }
                self.finished
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                return Ok(
                    "candidate proposed; trusted checks and independent review are still required"
                        .into(),
                );
            }
        }
        if mutation {
            self.governor
                .update_revision(
                    &self.case_id,
                    "implementation",
                    &uuid::Uuid::new_v4().to_string(),
                )
                .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        }
        Ok(transcript
            .last()
            .cloned()
            .unwrap_or_else(|| "operation completed".into()))
    }
}
fn implementation_schema() -> Value {
    json!({"oneOf":[
        {"type":"object","properties":{"action":{"const":"read"},"paths":{"type":"array","maxItems":32,"items":{"type":"string","description":crate::read_request::PATH_HELP}}},"required":["action","paths"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"write"},"path":{"type":"string"},"contents":{"type":"string"}},"required":["action","path","contents"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"delete"},"path":{"type":"string"}},"required":["action","path"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"run"},"program":{"type":"string"},"args":{"type":"array","items":{"type":"string"}}},"required":["action","program","args"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"finish"},"summary":{"type":"string"}},"required":["action","summary"],"additionalProperties":false}
    ]})
}

// Model-facing process arguments are a closed grammar. In particular, naming a
// read-oriented executable is not proof that its options cannot write or execute.
/// What the implementor may run, sent with every command refusal. The examples in
/// `COMMAND_EXAMPLES` are checked against the grammar in a unit test.
const COMMAND_GRAMMAR: &str = "Admitted commands: git status|diff|log|show|ls-files with read-only flags (--short, --porcelain, --branch, --stat, --name-only, --name-status, --cached, --staged, --oneline, --others, --exclude-standard, --stage) and revisions; cargo fmt --check; cargo test|check|clippy|build with --quiet, --locked, --offline, --workspace, --all-targets, --lib, --tests and selectors such as -p NAME or --test NAME; rg PATTERN [worktree-relative paths] or rg --files; ess specify validate; in eval repositories only, go version or go build|test|vet|list ./... . Commands that write files, such as gofmt -w or cargo fmt without --check, are not admitted: write the formatted file instead.";
#[cfg(test)]
const COMMAND_EXAMPLES: &[&[&str]] = &[
    &["git", "status", "--short"],
    &["git", "diff", "--stat"],
    &["git", "log", "--oneline"],
    &["git", "show", "--stat", "HEAD"],
    &["git", "ls-files", "--others", "--exclude-standard"],
    &["cargo", "fmt", "--check"],
    &["cargo", "test", "--quiet", "--locked"],
    &["cargo", "check", "-p", "demo"],
    &["rg", "answer", "src"],
    &["rg", "--files"],
    &["ess", "specify", "validate"],
];

/// Admit a model command: a grammar violation is a refusal; a search path that fails
/// confinement (symlink, `.git`, changed root) stays fatal.
fn inspection_arguments(root: &Path, program: &str, args: &[String]) -> Result<Vec<String>> {
    let mut search_paths = Vec::new();
    let normalized = inspection_grammar(program, args, &mut search_paths).map_err(|error| {
        InputRefusal::new(
            "command_not_admitted",
            format!("{program} {}: {error}", args.join(" ")),
            COMMAND_GRAMMAR,
        )
    })?;
    for relative in search_paths {
        engine::confined(root, &relative, true)?;
    }
    Ok(normalized)
}

fn inspection_grammar(
    program: &str,
    args: &[String],
    search_paths: &mut Vec<String>,
) -> Result<Vec<String>> {
    ensure!(args.len() <= 128, "command argument limit");
    let verb = args
        .first()
        .context("inspection command missing arguments")?
        .as_str();
    let atom = |value: &str| {
        !value.is_empty()
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_-.,/".contains(&c))
            && !value.contains("..")
            && !value.starts_with('/')
    };
    match program {
        "git" => {
            let flags: &[&str] = match verb {
                "status" => &["--short", "--porcelain", "--branch"],
                "diff" => &[
                    "--stat",
                    "--name-only",
                    "--name-status",
                    "--cached",
                    "--staged",
                ],
                "log" => &["--oneline", "--stat", "--name-only", "--name-status"],
                "show" => &["--stat", "--name-only", "--name-status"],
                "ls-files" => &["--cached", "--others", "--exclude-standard", "--stage"],
                _ => bail!("model Git operation is not admitted"),
            };
            for arg in &args[1..] {
                let revision = matches!(verb, "diff" | "log" | "show")
                    && !arg.starts_with('-')
                    && !arg.starts_with('/')
                    && !arg.is_empty()
                    && arg
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"_/.~^:-".contains(&c));
                ensure!(
                    flags.contains(&arg.as_str()) || revision,
                    "Git inspection option is not admitted: {arg}"
                );
            }
            let mut normalized = vec![
                "--no-pager".into(),
                "-c".into(),
                "core.fsmonitor=false".into(),
                verb.into(),
            ];
            if matches!(verb, "diff" | "log" | "show") {
                normalized.extend(["--no-ext-diff".into(), "--no-textconv".into()]);
            }
            if verb == "log" {
                normalized.push("--max-count=30".into());
            }
            normalized.extend_from_slice(&args[1..]);
            Ok(normalized)
        }
        "cargo" => {
            if verb == "fmt" {
                ensure!(
                    args == ["fmt", "--check"],
                    "model formatting must be read-only: cargo fmt --check"
                );
                return Ok(args.to_vec());
            }
            ensure!(
                matches!(verb, "test" | "check" | "clippy" | "build"),
                "model Cargo operation is not admitted"
            );
            let mut index = 1;
            while index < args.len() {
                match args[index].as_str() {
                    "--quiet"
                    | "-q"
                    | "--release"
                    | "--offline"
                    | "--locked"
                    | "--frozen"
                    | "--workspace"
                    | "--all-targets"
                    | "--all-features"
                    | "--no-default-features"
                    | "--lib"
                    | "--bins"
                    | "--tests"
                    | "--examples"
                    | "--benches" => {}
                    "--package" | "-p" | "--features" | "--bin" | "--test" | "--example"
                    | "--bench" => {
                        index += 1;
                        ensure!(
                            args.get(index).is_some_and(|value| atom(value)),
                            "Cargo selector is missing or invalid"
                        );
                    }
                    other => bail!("Cargo inspection option is not admitted: {other}"),
                }
                index += 1;
            }
            Ok(args.to_vec())
        }
        "rg" => {
            if args == ["--files"] {
                return Ok(vec!["--no-config".into(), "--files".into()]);
            }
            ensure!(
                !verb.starts_with('-'),
                "search grammar is rg PATTERN [relative paths] or rg --files"
            );
            let mut normalized = vec![
                "--no-config".into(),
                "--line-number".into(),
                "--fixed-strings".into(),
                "--".into(),
                verb.into(),
            ];
            for relative in &args[1..] {
                ensure!(
                    !relative.starts_with('-'),
                    "search options are not admitted"
                );
                ensure!(
                    crate::read_request::parse(relative)
                        .is_ok_and(|(directory, _)| directory.is_none()),
                    "search paths are worktree-relative"
                );
                search_paths.push(relative.clone());
                normalized.push(relative.clone());
            }
            Ok(normalized)
        }
        "ess" => {
            ensure!(
                args == ["specify", "validate"],
                "model ESS operation is limited to specify validate"
            );
            Ok(args.to_vec())
        }
        _ => bail!("model command is not admitted"),
    }
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
    let case_id = CaseId(text(assignment, "case_id")?.into());
    let progress_host = host.clone();
    let progress_assignment = assignment.clone();
    let governor = crate::governance::open(
        control_plane_protocol::VERIFIED_MERGE_YAML,
        "source-delivery@1",
        &case_id,
        BTreeMap::from([("implementation".into(), candidate.into())]),
        Arc::new(move |kind, value| {
            progress_host.progress(&progress_assignment, kind, "runtime", value.clone())
        }),
    )?;
    for item in evidence {
        crate::governance::submit(&governor, &case_id, item, Some(implementor))?;
    }
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
    let outcome = admitted_publication(execution, &governor, &case_id, &bindings);
    let intent = host.row("PublicationIntentList", "publication_id", &publication_id)?;
    match observe_merge(host, repo, &intent) {
        Ok(Some(receipt)) => {
            let observed = attestation(
                "merge_observation",
                "merged",
                candidate,
                EvidenceOrigin::RemoteObserver {
                    producer: "control-plane-git-observer".into(),
                    observation: receipt.clone(),
                    candidate: candidate.into(),
                },
            )?;
            crate::governance::submit(&governor, &case_id, &observed, Some(implementor))?;
            ensure!(
                matches!(
                    governor
                        .completion(&case_id)
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?,
                    CompletionDetermination::Complete(_)
                ),
                "Loom governor has not accepted observed merge"
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
    for mut goal in host
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
        goal["directories"] = json!(directories);
        let prior = host.acceptance(&goal)?;
        if acceptance_is_unchanged(&goal, &prior, &repositories, &assignments, &host.runner) {
            continue;
        }
        let (fingerprint, _) =
            acceptance_fingerprint(&goal, &repositories, &assignments, &host.runner);
        if prior["fingerprint"] == fingerprint && prior["status"] == "failed" {
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
                        false,
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
            let review=host.respond(&ModelRequest {role:"goal_reviewer".into(),execution_context:reviewer.clone(),model:text(&goal,"reviewer_model")?.into(),instructions:"Independently decide whether the exact standing objective and acceptance are satisfied by all observed merged targets, real check outputs and diffs. Do not infer completion from an empty queue or story statuses. Reject any acceptance obligation unsupported by evidence.".into(),prompt:json!({"goal":crate::context::goal_brief(&goal),"observations":observations}).to_string(),schema:crate::model::critique_schema(),timeout:host.remaining()?}, Path::new(text(repos[0], "path")?), current[0])?;
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
            // The store is released between the check above and this command. Admission refuses
            // a receipt whose `goal_revision` is no longer the goal's; an edit in between then
            // leaves the goal Running, and the error arm below leaves it to the edit's planning.
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

/// Publication is one Commission-admitted effect. The host supplies current authority and the
/// exact durable intent; command success remains separate from the subsequent remote observation.
fn admitted_publication(
    execution: &Execution<'_>,
    governor: &crate::governance::HostGovernor,
    case: &CaseId,
    bindings: &BTreeMap<&str, String>,
) -> Result<()> {
    let effect = Publication {
        execution: *execution,
        bindings,
        invoked: std::sync::atomic::AtomicBool::new(false),
        result: std::sync::Mutex::new(None),
    };
    let commission = Commission::new(CommissionData {
        commission_id: CommissionId(runtime_id()),
        agent_revision_id: AgentRevisionId(runtime_id()),
        case_id: case.clone(),
        principal: PrincipalId("control-plane-supervisor".into()),
        authority_context: AuthorityContext(wire::Value::Null),
    });
    let mut runs = Generated::new(RunStore::new(|| RunId(runtime_id())));
    let end = loom_sdk::run_until_blocked(
        governor,
        &effect,
        &effect,
        &effect,
        &commission,
        &mut runs,
        &mut crate::governance::ContextClock { max_steps: 2 },
    );
    let result = effect
        .result
        .lock()
        .map_err(|_| anyhow::anyhow!("publisher state poisoned"))?
        .take();
    match result {
        Some(result) => result,
        None => {
            let end = end?;
            bail!(
                "Commission refused publication before effect: {:?}",
                end.outcome
            )
        }
    }
}
struct Publication<'a> {
    execution: Execution<'a>,
    bindings: &'a BTreeMap<&'a str, String>,
    invoked: std::sync::atomic::AtomicBool,
    result: std::sync::Mutex<Option<Result<()>>>,
}
impl AgentExecutor for Publication<'_> {
    fn run(
        &self,
        _: &Commission<commission_state::Assigned>,
        _: &Frontier<frontier_state::Issued>,
    ) -> ExecutorOutcome {
        if self.invoked.load(std::sync::atomic::Ordering::SeqCst) {
            ExecutorOutcome::Suspended(ExecutorOutcomeSuspended {
                reason: SuspensionReason::Evidence(vec!["merge_observation".into()]),
            })
        } else {
            ExecutorOutcome::ProposedAction(ExecutorOutcomeProposedAction {
                action: "repository.merge".into(),
                arguments: ProposedActionArguments(wire::Value::Null),
            })
        }
    }
}
impl loom_sdk::commission::ports::authority::AuthorityProvider for Publication<'_> {
    fn decide(
        &self,
        _: &CommissionData,
        capability: &str,
    ) -> Result<AuthorityVerdict, loom_sdk::commission::ports::authority::AuthorityProviderError>
    {
        let Execution {
            host,
            assignment,
            goal,
            repo,
            ..
        } = self.execution;
        if capability != "repository.merge" {
            return Ok(AuthorityVerdict::Deny(AuthorityVerdictDeny {
                reason: "capability not configured".into(),
            }));
        }
        host.guard(assignment, goal, repo, true).map_err(|e| {
            loom_sdk::commission::ports::authority::AuthorityProviderError::new(e.to_string())
        })?;
        Ok(AuthorityVerdict::Allow(Unit(true)))
    }
}
impl EffectPort for Publication<'_> {
    fn performs(&self, action: &str) -> bool {
        action == "repository.merge"
    }
    fn invoke(
        &self,
        _: &Commission<commission_state::Assigned>,
        request: &AdmittedRequest,
    ) -> Result<EffectOutcome, EffectError> {
        let result = (|| -> Result<()> {
            ensure!(
                request.data().action == "repository.merge"
                    && !self.invoked.swap(true, std::sync::atomic::Ordering::SeqCst),
                "publication request replayed"
            );
            let Execution {
                host,
                assignment,
                goal,
                repo,
                path,
                ..
            } = self.execution;
            host.guard(assignment, goal, repo, true)?;
            ensure!(
                git(host, path, &["rev-parse", "HEAD"])? == self.bindings["candidate"]
                    && git(host, path, &["status", "--porcelain"])?.is_empty(),
                "publication candidate changed before effect"
            );
            ensure!(
                remote_head(host, path, &self.bindings["target"])?
                    == self.bindings["expected_base"],
                "publication base changed before effect"
            );
            command(
                host,
                path,
                text(repo, "publish_command")?,
                self.bindings,
                true,
            )?;
            Ok(())
        })();
        let outcome = match &result {
            Ok(()) => Ok(EffectOutcome::Performed(EffectOutcomePerformed {
                report: wire::Value::Text(
                    "publisher returned; remote observation still required".into(),
                ),
            })),
            Err(e) => Err(EffectError::new(e.to_string())),
        };
        *self
            .result
            .lock()
            .map_err(|_| EffectError::new("publisher state poisoned"))? = Some(result);
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_grammar_examples_are_admitted() {
        for example in COMMAND_EXAMPLES {
            let (program, args) = example.split_first().unwrap();
            let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
            assert!(
                inspection_grammar(program, &args, &mut Vec::new()).is_ok(),
                "COMMAND_GRAMMAR advertises a refused command: {example:?}"
            );
        }
        let refused =
            inspection_arguments(Path::new("/"), "gofmt", &["-w".into(), "main.go".into()])
                .unwrap_err();
        assert_eq!(
            refused.downcast_ref::<InputRefusal>().unwrap().code,
            "command_not_admitted"
        );
    }

    #[test]
    fn scope_rules_are_refusals_and_git_paths_stay_fatal() {
        let scope = vec!["src/".to_owned()];
        for (path, code) in [
            ("../escape.rs", "write_path_syntax"),
            (
                ".engineering/planning/story/x.md",
                "planning_store_host_owned",
            ),
            ("tests/acceptance.rs", "write_outside_scope"),
            ("src/tool.py", "write_language_policy"),
        ] {
            let error = scoped(path, &scope, false).unwrap_err();
            assert_eq!(
                error.downcast_ref::<InputRefusal>().map(|r| r.code),
                Some(code),
                "{path}"
            );
        }
        let git = scoped(".git/config", &scope, false).unwrap_err();
        assert!(git.downcast_ref::<InputRefusal>().is_none());
    }

    #[test]
    fn scoped_frontend_assets_allow_vue_without_allowing_javascript_backends() {
        let scope = vec![
            "frontend/".into(),
            "web/".into(),
            "backend/".into(),
            "tools/".into(),
            "scripts/".into(),
        ];
        for path in [
            "frontend/src/main.js",
            "frontend/src/App.vue",
            "web/src/client.ts",
            "web/src/view.tsx",
            "frontend/src/view.jsx",
            "web/vite.config.ts",
        ] {
            assert!(
                scoped(path, &scope, false).is_ok(),
                "frontend asset refused: {path}"
            );
        }
        for path in [
            "backend/server.js",
            "tools/check.ts",
            "scripts/build.js",
            "backend/App.vue",
            "frontend/server/main.js",
            "web/backend/main.ts",
            "frontend/tools/check.js",
            "web/scripts/build.ts",
            "frontend/../backend/main.js",
            "/frontend/src/main.js",
            "frontend/src/probe.py",
            "web/src/build.sh",
        ] {
            assert!(
                scoped(path, &scope, false).is_err(),
                "non-frontend executable admitted: {path}"
            );
        }
        assert!(
            scoped("frontend/src/main.js", &["web/".into()], false).is_err(),
            "frontend exception must not bypass accepted scope"
        );
        assert!(scoped("backend/main.rs", &scope, false).is_ok());
        assert!(scoped("backend/main.go", &scope, false).is_err());
        assert!(
            scoped("backend/main.go", &scope, true).is_ok(),
            "explicit Go eval exception remains intact"
        );
    }

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

    #[test]
    fn process_grammar_refuses_output_execution_and_path_override_options() {
        for (program, args) in [
            ("git", vec!["diff", "--output=tests/sentinel"]),
            ("git", vec!["show", "--output", "tests/sentinel"]),
            ("git", vec!["diff", "--ext-diff"]),
            ("git", vec!["log", "--textconv"]),
            ("git", vec!["-c", "core.pager=command", "show"]),
            ("rg", vec!["--pre=command", "needle"]),
            ("rg", vec!["needle", "--follow"]),
            ("cargo", vec!["fmt"]),
            ("cargo", vec!["fmt", "--", "--emit=files"]),
            ("cargo", vec!["test", "--target-dir=outside"]),
            (
                "cargo",
                vec!["check", "--manifest-path", "outside/Cargo.toml"],
            ),
            ("cargo", vec!["test", "--config", "runner=command"]),
            ("ess", vec!["specify", "validate", "--output", "outside"]),
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(
                inspection_arguments(Path::new("."), program, &args).is_err(),
                "admitted {program} {args:?}"
            );
        }
    }

    #[test]
    fn normalized_git_inspection_executes_without_output_or_external_diff_hooks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for args in [
            vec!["status", "--short"],
            vec!["diff", "--stat"],
            vec!["log", "--oneline"],
            vec!["show", "HEAD", "--stat"],
            vec!["ls-files"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            let normalized = inspection_arguments(root, "git", &args).unwrap();
            let output = std::process::Command::new("git")
                .current_dir(root)
                .args(normalized)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[cfg(test)]
mod progress_tests {
    use super::*;
    use control_plane_core::Store;

    const DECISION_LIMIT: u64 = 16 * 1024;

    /// A 1 KiB progress detail, unique per event, with characters JSON must escape.
    fn detail(index: usize) -> String {
        let mut text = format!("event {index:05}: \"loom\" streamed tokens ");
        while text.len() < 1024 {
            text.push_str("abcdefghij");
        }
        text.truncate(1024);
        text
    }

    /// A model whose turn only reports Loom progress through the host's progress callback.
    /// It measures the event data each progress event appends and stops at the first one
    /// that exceeds the bound, so an unbounded build cannot fill the disk.
    struct Emitter {
        events: usize,
        store: SharedStore,
        appended: std::sync::Mutex<Vec<u64>>,
    }
    impl AgentModel for Emitter {
        fn respond(&self, _: &ModelRequest) -> Result<Value> {
            bail!("this model only reports progress")
        }
        fn respond_in(
            &self,
            _: &ModelRequest,
            environment: &crate::ModelEnvironment,
        ) -> Result<Value> {
            let bytes = || self.store.blocking_lock().appended_event_bytes();
            for index in 0..self.events {
                let before = bytes();
                (environment.progress)(json!(detail(index)))?;
                let added = bytes() - before;
                ensure!(
                    added <= DECISION_LIMIT,
                    "progress event {index} appended {added} bytes of event data"
                );
                self.appended.lock().unwrap().push(added);
            }
            Ok(json!({}))
        }
    }

    struct Fixture {
        _temp: tempfile::TempDir,
        runtime: tokio::runtime::Runtime,
        database: PathBuf,
        workspace: PathBuf,
        store: SharedStore,
        assignment: Value,
    }

    fn fixture() -> Fixture {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let workspace = temp.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let database = temp.path().join("state.sqlite");
        let (store, assignment) = runtime.block_on(async {
            let mut store = Store::open(&database).await.unwrap();
            let id = |outcome: Value, field: &str| outcome["published"][0]["payload"][field].clone();
            let ws = id(
                store
                    .execute(
                        "RegisterWorkspace",
                        json!({"path":workspace,"name":"progress"}),
                        Actor::Operator,
                    )
                    .await
                    .unwrap(),
                "workspace_id",
            );
            let goal = id(store.execute("CreateGoal",json!({"workspace_id":ws,"objective":"Report progress","acceptance":"Progress stays bounded","max_workers":1,"max_attempts":1,"max_minutes":60,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false}),Actor::Operator).await.unwrap(),"goal_id");
            store
                .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
                .await
                .unwrap();
            let assignment = json!({"goal_id":goal,"assignment_id":uuid::Uuid::new_v4().to_string(),"goal_revision":1,"worktree_id":"cp-impl-progress"});
            (Arc::new(tokio::sync::Mutex::new(store)), assignment)
        });
        Fixture {
            _temp: temp,
            runtime,
            database,
            workspace,
            store,
            assignment,
        }
    }

    /// Send `events` Loom progress events through one implementor turn, the way the fleet
    /// receives them: the host callback rechecks the goal, then records the progress.
    fn emit(fixture: &Fixture, events: usize) -> Vec<u64> {
        let model = Arc::new(Emitter {
            events,
            store: fixture.store.clone(),
            appended: Default::default(),
        });
        let host = Host {
            store: fixture.store.clone(),
            handle: fixture.runtime.handle().clone(),
            config: RuntimeConfig::default(),
            model: model.clone(),
            runner: ProcessRunner {
                environment: vec![],
                timeout: Duration::from_secs(600),
                cancel: CancellationToken::new(),
            },
            deadline: None,
        };
        let request = ModelRequest {
            role: "implementor".into(),
            execution_context: "implementor-progress".into(),
            model: "scripted".into(),
            instructions: String::new(),
            prompt: String::new(),
            schema: json!({}),
            timeout: Duration::from_secs(600),
        };
        host.respond(&request, &fixture.workspace, &fixture.assignment)
            .unwrap();
        std::mem::take(&mut *model.appended.lock().unwrap())
    }

    #[test]
    fn progress_decision_stays_under_16_kib() {
        let fixture = fixture();
        let appended = emit(&fixture, 1_000);
        assert_eq!(appended.len(), 1_000);
        assert!(
            appended.iter().all(|bytes| *bytes > 0),
            "every progress event is one durable decision"
        );
        let largest = appended.iter().max().copied().unwrap();
        eprintln!(
            "1,000 progress events: largest decision {largest} bytes, total {} bytes",
            appended.iter().sum::<u64>()
        );
        assert!(largest <= DECISION_LIMIT, "{largest} bytes");
    }

    #[test]
    fn restart_open_is_bounded() {
        let fixture = fixture();
        let appended = emit(&fixture, 10_000);
        assert_eq!(appended.len(), 10_000);
        let Fixture {
            _temp,
            runtime,
            database,
            store,
            ..
        } = fixture;
        drop(store);
        let started = Instant::now();
        let reopened = runtime.block_on(Store::open(&database)).unwrap();
        let elapsed = started.elapsed();
        eprintln!(
            "10,000 progress events: {} bytes of event data, Store::open {elapsed:?}",
            appended.iter().sum::<u64>()
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "Store::open took {elapsed:?}"
        );
        let goal = reopened.query("GoalList").unwrap()[0].clone();
        let receipt: Value =
            serde_json::from_str(goal["planning_receipt"].as_str().unwrap()).unwrap();
        let last = receipt["last_activity"]["detail"].as_str().unwrap();
        assert!(last.starts_with("event 09999:"), "{last}");
        assert!(detail(9_999).starts_with(last));
        let history = reopened
            .activity_history(goal["goal_id"].as_str().unwrap())
            .unwrap();
        let activity = history["activity"].as_array().unwrap();
        assert_eq!(activity.len(), 64);
        assert_eq!(activity.last(), Some(&receipt["last_activity"]));
    }
}
