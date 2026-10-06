//! Deliberate regeneration of the recorded host history that `Store::open` must keep replaying.
//!
//! A scripted operator and supervisor drive real repositories through the public
//! `control_plane_core::Store::execute`, so the fixture holds exactly what the Store writes. The
//! script applies every generated command at least once and records one declared refusal;
//! `crates/control-plane-core/tests/recorded_history.rs` checks that coverage against the
//! generated routes. No gate runs this command: the committed fixture is evidence about stored
//! history, and refreshing it after a replay failure would discard that evidence.
use anyhow::{Context, Result, bail, ensure};
use control_plane_core::{Actor, Store};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const HISTORY: &str = "crates/control-plane-core/tests/fixtures/recorded-history.db";
pub const VIEWS: &str = "crates/control-plane-core/tests/fixtures/recorded-history.views.json";
const VIEW_NAMES: [&str; 6] = [
    "AssignmentList",
    "GoalList",
    "PublicationIntentList",
    "RepositoryRegistrationList",
    "WorkspaceDirectoryList",
    "WorkspaceList",
];

pub fn run(root: &Path, work_dir: &Path) -> Result<()> {
    let work = prepare(work_dir)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let history = root.join(HISTORY);
    if history.exists() {
        let previous = work.join("previous");
        fs::create_dir(&previous)?;
        let copy = previous.join("state.sqlite");
        fs::copy(&history, &copy)?;
        runtime
            .block_on(Store::open(&copy))
            .with_context(|| {
                format!(
                    "the committed {HISTORY} no longer replays; regenerating would discard that evidence. \
                     Migrate stored history instead, or delete the fixture deliberately before recording a new one"
                )
            })?;
    }
    for repository in ["workspace/alpha", "workspace/beta", "manual"] {
        git_init(&work.join(repository))?;
    }
    fs::create_dir_all(work.join("context"))?;
    let database = work.join("state.sqlite");
    let (steps, views) = runtime.block_on(async {
        let mut store = Store::open(&database).await?;
        let steps = script(&mut store, &work).await?;
        let views = query_views(&store)?;
        drop(store);
        anyhow::Ok((steps, views))
    })?;
    let journal = work.join("state.sqlite-wal");
    ensure!(
        !journal.exists() || fs::metadata(&journal)?.len() == 0,
        "the closed store left an unmerged write-ahead log; its database file is incomplete"
    );
    let replayed = runtime.block_on(async { query_views(&Store::open(&database).await?) })?;
    ensure!(
        replayed == views,
        "the recorded history does not replay to the views it produced"
    );
    let bytes = fs::read(&database)?;
    let mut rendered = serde_json::to_vec_pretty(&views)?;
    rendered.push(b'\n');
    refuse_personal_paths(&bytes)?;
    refuse_personal_paths(&rendered)?;
    fs::create_dir_all(root.join(HISTORY).parent().context("fixture directory")?)?;
    fs::write(root.join(HISTORY), bytes)?;
    fs::write(root.join(VIEWS), rendered)?;
    println!("recorded {steps} host commands into {HISTORY} and {VIEWS}");
    Ok(())
}

/// A fresh directory outside every home directory and Git work tree: recorded paths are committed,
/// and repository discovery must not attach the scripted workspace to an enclosing repository.
fn prepare(work_dir: &Path) -> Result<PathBuf> {
    ensure!(
        !work_dir.exists(),
        "{} already exists; choose a new directory",
        work_dir.display()
    );
    fs::create_dir_all(work_dir)?;
    let work = work_dir.canonicalize()?;
    refuse_personal_paths(work.as_os_str().as_encoded_bytes())?;
    let enclosing = Command::new("git")
        .arg("-C")
        .arg(&work)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    ensure!(
        !enclosing.status.success(),
        "{} is inside a Git work tree; choose a directory outside every repository",
        work.display()
    );
    Ok(work)
}

/// The Security gate rejects personal paths in any committed byte.
fn refuse_personal_paths(bytes: &[u8]) -> Result<()> {
    let mut personal = vec![b"/home/".to_vec(), b"/Users/".to_vec()];
    if let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        personal.push(home.as_encoded_bytes().to_vec());
    }
    for needle in personal {
        ensure!(
            !bytes.windows(needle.len()).any(|window| window == needle),
            "a recorded path contains {}; choose a work directory outside every home directory",
            String::from_utf8_lossy(&needle)
        );
    }
    Ok(())
}

fn git_init(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    let output = Command::new("git")
        .args(["init", "--quiet", "--initial-branch=main"])
        .arg(path)
        .output()?;
    ensure!(
        output.status.success(),
        "git init {} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn query_views(store: &Store) -> Result<Value> {
    let mut views = Map::new();
    for view in VIEW_NAMES {
        views.insert(view.to_owned(), store.query(view)?);
    }
    Ok(Value::Object(views))
}

fn identity(outcome: &Value, field: &str) -> Result<String> {
    outcome["published"][0]["payload"][field]
        .as_str()
        .map(str::to_owned)
        .with_context(|| format!("outcome has no {field}: {outcome}"))
}

fn repository(rows: &Value, name: &str) -> Result<String> {
    rows.as_array()
        .and_then(|rows| rows.iter().find(|row| row["name"] == name))
        .and_then(|row| row["repository_id"].as_str())
        .map(str::to_owned)
        .with_context(|| format!("repository {name} was not discovered"))
}

struct Script<'a> {
    store: &'a mut Store,
    steps: usize,
}

impl Script<'_> {
    /// Execute one command and require the declared outcome the script expects.
    async fn expect(
        &mut self,
        command: &str,
        body: Value,
        actor: Actor,
        outcome: &str,
    ) -> Result<Value> {
        let answer = self
            .store
            .execute(command, body, actor)
            .await
            .with_context(|| format!("{command} was refused"))?;
        if answer["outcome"] != outcome {
            bail!("{command} answered {answer}, not {outcome}");
        }
        self.steps += 1;
        Ok(answer)
    }
}

async fn script(store: &mut Store, work: &Path) -> Result<usize> {
    use Actor::{Operator, Supervisor};
    let mut run = Script { store, steps: 0 };
    let members = work.join("workspace");
    let workspace = identity(
        &run.expect(
            "RegisterWorkspace",
            json!({"path": members, "name": "recorded"}),
            Operator,
            "created",
        )
        .await?,
        "workspace_id",
    )?;
    let directory = identity(
        &run.expect(
            "AddWorkspaceDirectory",
            json!({"workspace_id": workspace, "path": members}),
            Operator,
            "created",
        )
        .await?,
        "directory_id",
    )?;
    run.expect(
        "AddWorkspaceDirectory",
        json!({"workspace_id": workspace, "path": work.join("context")}),
        Operator,
        "created",
    )
    .await?;
    let manual = identity(
        &run.expect(
            "RegisterRepository",
            json!({"workspace_id": workspace, "name": "manual", "path": work.join("manual"),
                "common_dir": "", "base_branch": "main", "test_command": "cargo test",
                "publish_command": "publish-helper"}),
            Operator,
            "created",
        )
        .await?,
        "repository_id",
    )?;
    let rows = run.store.query("RepositoryRegistrationList")?;
    let (alpha, beta) = (repository(&rows, "alpha")?, repository(&rows, "beta")?);
    run.expect(
        "ConfigureRepository",
        json!({"repository_id": alpha, "base_branch": "main", "test_command": "task check",
            "publish_command": "publish-helper"}),
        Operator,
        "applied",
    )
    .await?;
    for command in [
        "DisableRepositoryRegistration",
        "EnableRepositoryRegistration",
    ] {
        run.expect(
            command,
            json!({"repository_id": manual}),
            Operator,
            "applied",
        )
        .await?;
    }

    let mut goal_body = json!({"workspace_id": workspace, "objective": "Deliver the recorded change",
        "acceptance": "Tests and an independent review pass", "max_workers": 2, "max_attempts": 3,
        "max_minutes": 60, "planner_model": "scripted-planner",
        "implementor_model": "scripted-implementor", "reviewer_model": "scripted-reviewer",
        "merge_authority": true});
    let goal = identity(
        &run.expect("CreateGoal", goal_body.clone(), Operator, "created")
            .await?,
        "goal_id",
    )?;
    let subject = json!({"goal_id": goal});
    run.expect("StartGoal", subject.clone(), Operator, "applied")
        .await?;
    run.expect("PauseGoal", subject.clone(), Operator, "applied")
        .await?;
    // A declared refusal is recorded history too, and replay must reproduce it.
    run.expect("PauseGoal", subject.clone(), Operator, "wrong-state")
        .await?;
    let update = goal_body.as_object_mut().context("goal body")?;
    update.remove("workspace_id");
    update.insert("goal_id".into(), json!(goal));
    update.insert("max_minutes".into(), json!(90));
    run.expect("UpdateGoal", goal_body, Operator, "applied")
        .await?;
    run.expect("StartGoal", subject.clone(), Operator, "applied")
        .await?;
    let planning = |phase: &str| {
        json!({"goal_id": goal, "planning_revision": 2, "planning_fingerprint": "plan-fingerprint",
            "planning_repository": alpha, "planning_worktree_id": "planning-tree",
            "planning_worktree_path": work.join("planning"), "planning_reason": "",
            "planning_receipt": "plan-receipt", "planning_phase": phase})
    };
    run.expect(
        "RecordPlanningProgress",
        planning("Planning"),
        Supervisor,
        "applied",
    )
    .await?;

    let queue = |repository: &str, story: &str| {
        json!({"goal_id": goal, "repository_id": repository, "story_id": story,
            "case_id": format!("{story}/case"), "worktree_id": "", "candidate": "", "attempt": 0,
            "reason": "", "implementor_run": "", "reviewer_run": "", "goal_revision": 2})
    };
    // Reviewed, repaired from Reviewing, reviewed again, published and completed.
    let first = identity(
        &run.expect(
            "QueueAssignment",
            queue(&alpha, "story:first"),
            Supervisor,
            "created",
        )
        .await?,
        "assignment_id",
    )?;
    for (command, body) in [
        (
            "ClaimAssignment",
            json!({"worktree_id": "tree-first", "implementor_run": "implementor-first-1",
                "base_revision": "base-first"}),
        ),
        (
            "ReviewAssignment",
            json!({"candidate": "candidate-first-1", "test_revision": "candidate-first-1"}),
        ),
        (
            "RepairAssignment",
            json!({"reason": "review requested changes", "implementor_run": "implementor-first-2"}),
        ),
        (
            "ReviewAssignment",
            json!({"candidate": "candidate-first-2", "test_revision": "candidate-first-2"}),
        ),
        (
            "ReadyAssignment",
            json!({"reviewer_run": "reviewer-first", "review_revision": "candidate-first-2"}),
        ),
    ] {
        run.expect(command, with(&first, body), Supervisor, "applied")
            .await?;
    }
    let published = identity(
        &run.expect(
            "PreparePublication",
            json!({"assignment_id": first, "candidate": "candidate-first-2", "target": "main",
                "expected_base": "base-first"}),
            Supervisor,
            "created",
        )
        .await?,
        "publication_id",
    )?;
    run.expect(
        "MergeAssignment",
        json!({"assignment_id": first}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "ConfirmPublication",
        json!({"publication_id": published, "receipt": "merge-receipt-first"}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "CompleteAssignment",
        json!({"assignment_id": first, "merge_receipt": "merge-receipt-first"}),
        Supervisor,
        "applied",
    )
    .await?;

    // Blocked, repaired from Blocked, published with an uncertain outcome and reconciled.
    let second = identity(
        &run.expect(
            "QueueAssignment",
            queue(&beta, "story:second"),
            Supervisor,
            "created",
        )
        .await?,
        "assignment_id",
    )?;
    for (command, body) in [
        (
            "ClaimAssignment",
            json!({"worktree_id": "tree-second", "implementor_run": "implementor-second-1",
                "base_revision": "base-second"}),
        ),
        (
            "BlockAssignment",
            json!({"reason": "test environment unavailable"}),
        ),
        (
            "RepairAssignment",
            json!({"reason": "test environment restored", "implementor_run": "implementor-second-2"}),
        ),
        (
            "ReviewAssignment",
            json!({"candidate": "candidate-second", "test_revision": "candidate-second"}),
        ),
        (
            "ReadyAssignment",
            json!({"reviewer_run": "reviewer-second", "review_revision": "candidate-second"}),
        ),
    ] {
        run.expect(command, with(&second, body), Supervisor, "applied")
            .await?;
    }
    let uncertain = identity(
        &run.expect(
            "PreparePublication",
            json!({"assignment_id": second, "candidate": "candidate-second", "target": "main",
                "expected_base": "base-second"}),
            Supervisor,
            "created",
        )
        .await?,
        "publication_id",
    )?;
    run.expect(
        "MergeAssignment",
        json!({"assignment_id": second}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "MarkPublicationUncertain",
        json!({"publication_id": uncertain}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "BlockAssignment",
        json!({"assignment_id": second, "reason": "publication outcome uncertain"}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "ConfirmPublication",
        json!({"publication_id": uncertain, "receipt": "merge-receipt-second"}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "ReconcileAssignment",
        json!({"assignment_id": second, "merge_receipt": "merge-receipt-second"}),
        Supervisor,
        "applied",
    )
    .await?;

    let third = identity(
        &run.expect(
            "QueueAssignment",
            queue(&manual, "story:third"),
            Supervisor,
            "created",
        )
        .await?,
        "assignment_id",
    )?;
    run.expect(
        "CancelAssignment",
        json!({"assignment_id": third}),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "RecordPlanningProgress",
        planning("Queued"),
        Supervisor,
        "applied",
    )
    .await?;
    run.expect(
        "SatisfyGoal",
        json!({"goal_id": goal, "satisfaction_receipt": "acceptance-verified"}),
        Supervisor,
        "applied",
    )
    .await?;

    let abandoned = identity(
        &run.expect(
            "CreateGoal",
            json!({"workspace_id": workspace, "objective": "Abandoned objective",
                "acceptance": "Never started", "max_workers": 1, "max_attempts": 1,
                "max_minutes": 10, "planner_model": "scripted-planner",
                "implementor_model": "scripted-implementor", "reviewer_model": "scripted-reviewer",
                "merge_authority": false}),
            Operator,
            "created",
        )
        .await?,
        "goal_id",
    )?;
    for command in ["CancelGoal", "DeleteGoal"] {
        run.expect(command, json!({"goal_id": abandoned}), Operator, "applied")
            .await?;
    }
    run.expect(
        "RemoveWorkspaceDirectory",
        json!({"directory_id": directory}),
        Operator,
        "applied",
    )
    .await?;
    run.expect(
        "ArchiveWorkspace",
        json!({"workspace_id": workspace}),
        Operator,
        "applied",
    )
    .await?;
    Ok(run.steps)
}

fn with(assignment: &str, mut body: Value) -> Value {
    body["assignment_id"] = json!(assignment);
    body
}
