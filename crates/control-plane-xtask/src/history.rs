//! Deliberate regeneration of the recorded host history that `Store::open` must keep replaying.
//!
//! A scripted operator and supervisor drive real repositories through the public
//! `control_plane_core::Store::execute`, so the fixture holds exactly what the Store writes. The
//! script applies every generated command at least once and answers every declared refusal
//! outcome of every command: `not-found` for each, `wrong-state` where declared, and `DeleteGoal`'s
//! `paused`, `running` and `satisfied`. `crates/control-plane-core/tests/recorded_history.rs`
//! checks that coverage, outcome by outcome, against the generated contract. No gate runs this command: the committed fixture is evidence
//! about stored history, and refreshing it after a replay failure would discard that evidence.
use anyhow::{Context, Result, bail, ensure};
use control_plane_core::{Actor, Store};
use serde_json::{Map, Value, json};
use std::{fs, path::Path, process::Command};

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
    ensure!(
        !work_dir.exists(),
        "{} already exists; choose a new directory",
        work_dir.display()
    );
    fs::create_dir_all(work_dir)?;
    let work = work_dir.canonicalize()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    // The committed evidence is checked before anything about the new recording, so a fixture
    // that no longer replays is refused whatever else is wrong with this invocation.
    runtime.block_on(still_replays(root, &work))?;
    refuse_unsuitable(&work)?;
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

/// A committed history may be replaced only while it still replays exactly: `Store::open` accepts
/// every recorded outcome and the replayed views equal the committed views. Anything else is
/// evidence of a specification change that needs a migration, not a new recording.
async fn still_replays(root: &Path, work: &Path) -> Result<()> {
    let (history, views) = (root.join(HISTORY), root.join(VIEWS));
    match (history.exists(), views.exists()) {
        (false, false) => return Ok(()),
        (true, true) => {}
        _ => bail!("{HISTORY} and {VIEWS} are committed together; restore the missing one"),
    }
    let refuse = |why: &str| {
        format!(
            "the committed {HISTORY} {why}; re-recording would discard that evidence. \
             Migrate stored history instead, or delete both fixture files deliberately before recording"
        )
    };
    let previous = work.join("previous");
    fs::create_dir(&previous)?;
    let copy = previous.join("state.sqlite");
    fs::copy(&history, &copy)?;
    let store = Store::open(&copy)
        .await
        .with_context(|| refuse("no longer opens"))?;
    let committed: Value = serde_json::from_slice(&fs::read(&views)?)
        .with_context(|| format!("{VIEWS} is not JSON"))?;
    ensure!(
        same_state(&query_views(&store)?, &committed),
        refuse("no longer replays to the committed views")
    );
    Ok(())
}

/// Whether replay reconstructed the committed state: every committed view with the same rows and
/// values. A view the committed views lack is a new projection of that state and is tolerated. A
/// row field they lack is tolerated only while it replays as `null`, which is how a gate-admitted
/// `Optional<…>` field shows up when no recorded call ever set it.
fn same_state(replayed: &Value, committed: &Value) -> bool {
    let (Some(replayed), Some(committed)) = (replayed.as_object(), committed.as_object()) else {
        return false;
    };
    committed
        .iter()
        .all(|(view, rows)| replayed.get(view).is_some_and(|r| same_rows(r, rows)))
}

fn same_rows(replayed: &Value, committed: &Value) -> bool {
    match (replayed, committed) {
        (Value::Object(replayed), Value::Object(committed)) => {
            committed
                .iter()
                .all(|(key, value)| replayed.get(key).is_some_and(|r| same_rows(r, value)))
                && replayed
                    .iter()
                    .all(|(key, value)| committed.contains_key(key) || value.is_null())
        }
        (Value::Array(replayed), Value::Array(committed)) => {
            replayed.len() == committed.len()
                && replayed
                    .iter()
                    .zip(committed)
                    .all(|(replayed, committed)| same_rows(replayed, committed))
        }
        _ => replayed == committed,
    }
}

/// Outside every home directory and Git work tree: recorded paths are committed, and repository
/// discovery must not attach the scripted workspace to an enclosing repository.
fn refuse_unsuitable(work: &Path) -> Result<()> {
    refuse_personal_paths(work.as_os_str().as_encoded_bytes())?;
    let enclosing = Command::new("git")
        .arg("-C")
        .arg(work)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    ensure!(
        !enclosing.status.success(),
        "{} is inside a Git work tree; choose a directory outside every repository",
        work.display()
    );
    Ok(())
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
    unknown_instances(&mut run).await?;
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
    run.expect(
        "EnableRepositoryRegistration",
        json!({"repository_id": manual}),
        Operator,
        "wrong-state",
    )
    .await?;

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
    run.expect("StartGoal", subject.clone(), Operator, "wrong-state")
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
    // Ready to merge: every earlier step of the lifecycle now refuses with `wrong-state`.
    for (command, body) in [
        (
            "ClaimAssignment",
            json!({"worktree_id": "tree-first", "implementor_run": "implementor-first-3",
                "base_revision": "base-first"}),
        ),
        (
            "ReviewAssignment",
            json!({"candidate": "candidate-first-2", "test_revision": "candidate-first-2"}),
        ),
        (
            "RepairAssignment",
            json!({"reason": "late review comment", "implementor_run": "implementor-first-3"}),
        ),
        (
            "ReadyAssignment",
            json!({"reviewer_run": "reviewer-first", "review_revision": "candidate-first-2"}),
        ),
    ] {
        run.expect(command, with(&first, body), Supervisor, "wrong-state")
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
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "MergeAssignment",
            json!({"assignment_id": first}),
            Supervisor,
            outcome,
        )
        .await?;
    }
    run.expect(
        "ConfirmPublication",
        json!({"publication_id": published, "receipt": "merge-receipt-first"}),
        Supervisor,
        "applied",
    )
    .await?;
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "CompleteAssignment",
            json!({"assignment_id": first, "merge_receipt": "merge-receipt-first"}),
            Supervisor,
            outcome,
        )
        .await?;
    }
    for (command, body) in [
        (
            "BlockAssignment",
            json!({"assignment_id": first, "reason": "too late to block"}),
        ),
        (
            "MarkPublicationUncertain",
            json!({"publication_id": published}),
        ),
        (
            "ConfirmPublication",
            json!({"publication_id": published, "receipt": "merge-receipt-first"}),
        ),
    ] {
        run.expect(command, body, Supervisor, "wrong-state").await?;
    }

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
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "ReconcileAssignment",
            json!({"assignment_id": second, "merge_receipt": "merge-receipt-second"}),
            Supervisor,
            outcome,
        )
        .await?;
    }

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
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "CancelAssignment",
            json!({"assignment_id": third}),
            Supervisor,
            outcome,
        )
        .await?;
    }
    run.expect(
        "RecordPlanningProgress",
        planning("Queued"),
        Supervisor,
        "applied",
    )
    .await?;
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "SatisfyGoal",
            json!({"goal_id": goal, "satisfaction_receipt": "acceptance-verified"}),
            Supervisor,
            outcome,
        )
        .await?;
    }
    run.expect("CancelGoal", subject, Operator, "wrong-state")
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
    for (command, outcome) in [
        ("DeleteGoal", "paused"),
        ("CancelGoal", "applied"),
        ("DeleteGoal", "applied"),
    ] {
        run.expect(command, json!({"goal_id": abandoned}), Operator, outcome)
            .await?;
    }
    // A goal without assignments refuses deletion while running and once satisfied.
    let probe = identity(
        &run.expect(
            "CreateGoal",
            json!({"workspace_id": workspace, "objective": "Probe objective",
                "acceptance": "Satisfied without assignments", "max_workers": 1,
                "max_attempts": 1, "max_minutes": 10, "planner_model": "scripted-planner",
                "implementor_model": "scripted-implementor", "reviewer_model": "scripted-reviewer",
                "merge_authority": false}),
            Operator,
            "created",
        )
        .await?,
        "goal_id",
    )?;
    let probe = json!({"goal_id": probe});
    run.expect("StartGoal", probe.clone(), Operator, "applied")
        .await?;
    run.expect("DeleteGoal", probe.clone(), Operator, "running")
        .await?;
    let mut satisfied = probe.clone();
    satisfied["satisfaction_receipt"] = json!("probe-accepted");
    run.expect("SatisfyGoal", satisfied, Supervisor, "applied")
        .await?;
    run.expect("DeleteGoal", probe, Operator, "satisfied")
        .await?;
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "RemoveWorkspaceDirectory",
            json!({"directory_id": directory}),
            Operator,
            outcome,
        )
        .await?;
    }
    // Removing the directory disabled the repositories it introduced.
    run.expect(
        "DisableRepositoryRegistration",
        json!({"repository_id": alpha}),
        Operator,
        "wrong-state",
    )
    .await?;
    for outcome in ["applied", "wrong-state"] {
        run.expect(
            "ArchiveWorkspace",
            json!({"workspace_id": workspace}),
            Operator,
            outcome,
        )
        .await?;
    }
    Ok(run.steps)
}

/// Every command that can name an unknown instance answers `not-found`, and that answer is
/// recorded history like any other.
async fn unknown_instances(run: &mut Script<'_>) -> Result<()> {
    use Actor::{Operator, Supervisor};
    let unknown = "00000000-0000-4000-8000-000000000000";
    let goal = json!({"goal_id": unknown, "objective": "Unknown goal", "acceptance": "None",
        "max_workers": 1, "max_attempts": 1, "max_minutes": 1, "planner_model": "scripted-planner",
        "implementor_model": "scripted-implementor", "reviewer_model": "scripted-reviewer",
        "merge_authority": false});
    let planning = json!({"goal_id": unknown, "planning_revision": 1,
        "planning_fingerprint": "unknown", "planning_repository": unknown,
        "planning_worktree_id": "unknown", "planning_worktree_path": "unknown",
        "planning_reason": "", "planning_receipt": "", "planning_phase": "Idle"});
    let assignment = |fields: Value| {
        let mut body = fields;
        body["assignment_id"] = json!(unknown);
        body
    };
    let calls = [
        (
            "RemoveWorkspaceDirectory",
            json!({"directory_id": unknown}),
            Operator,
        ),
        (
            "ArchiveWorkspace",
            json!({"workspace_id": unknown}),
            Operator,
        ),
        (
            "DisableRepositoryRegistration",
            json!({"repository_id": unknown}),
            Operator,
        ),
        (
            "EnableRepositoryRegistration",
            json!({"repository_id": unknown}),
            Operator,
        ),
        (
            "ConfigureRepository",
            json!({"repository_id": unknown, "base_branch": "main", "test_command": "task check",
                "publish_command": ""}),
            Operator,
        ),
        ("StartGoal", json!({"goal_id": unknown}), Operator),
        ("PauseGoal", json!({"goal_id": unknown}), Operator),
        ("CancelGoal", json!({"goal_id": unknown}), Operator),
        ("DeleteGoal", json!({"goal_id": unknown}), Operator),
        ("UpdateGoal", goal, Operator),
        (
            "SatisfyGoal",
            json!({"goal_id": unknown, "satisfaction_receipt": "unknown"}),
            Supervisor,
        ),
        ("RecordPlanningProgress", planning, Supervisor),
        (
            "ClaimAssignment",
            assignment(
                json!({"worktree_id": "unknown", "implementor_run": "unknown",
                "base_revision": "unknown"}),
            ),
            Supervisor,
        ),
        (
            "ReviewAssignment",
            assignment(json!({"candidate": "unknown", "test_revision": "unknown"})),
            Supervisor,
        ),
        (
            "RepairAssignment",
            assignment(json!({"reason": "unknown", "implementor_run": "unknown"})),
            Supervisor,
        ),
        (
            "ReadyAssignment",
            assignment(json!({"reviewer_run": "unknown", "review_revision": "unknown"})),
            Supervisor,
        ),
        ("MergeAssignment", assignment(json!({})), Supervisor),
        (
            "CompleteAssignment",
            assignment(json!({"merge_receipt": "unknown"})),
            Supervisor,
        ),
        (
            "BlockAssignment",
            assignment(json!({"reason": "unknown"})),
            Supervisor,
        ),
        ("CancelAssignment", assignment(json!({})), Supervisor),
        (
            "ReconcileAssignment",
            assignment(json!({"merge_receipt": "unknown"})),
            Supervisor,
        ),
        (
            "MarkPublicationUncertain",
            json!({"publication_id": unknown}),
            Supervisor,
        ),
        (
            "ConfirmPublication",
            json!({"publication_id": unknown, "receipt": "unknown"}),
            Supervisor,
        ),
    ];
    for (command, body, actor) in calls {
        run.expect(command, body, actor, "not-found").await?;
    }
    Ok(())
}

fn with(assignment: &str, mut body: Value) -> Value {
    body["assignment_id"] = json!(assignment);
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only a new view or an added field that replays as `null` is tolerated; anything that
    /// changes, removes or adds recorded state is a different state.
    #[test]
    fn replayed_state_tolerates_only_new_views_and_added_null_fields() {
        let committed = json!({"GoalList": [{"goal_id": "g", "revision": 2}]});
        let same = |replayed: Value| same_state(&replayed, &committed);
        assert!(same(json!({"GoalList": [{"goal_id": "g", "revision": 2}]})));
        assert!(same(json!({
            "GoalList": [{"goal_id": "g", "revision": 2}],
            "GoalSummary": [{"running": 0}]
        })));
        assert!(same(
            json!({"GoalList": [{"goal_id": "g", "revision": 2, "note": null}]})
        ));
        assert!(!same(
            json!({"GoalList": [{"goal_id": "g", "revision": 2, "note": "set"}]})
        ));
        assert!(!same(
            json!({"GoalList": [{"goal_id": "g", "revision": 3}]})
        ));
        assert!(!same(json!({"GoalList": [{"goal_id": "g"}]})));
        assert!(!same(json!({"GoalList": []})));
        assert!(!same(json!({"GoalList": [
            {"goal_id": "g", "revision": 2},
            {"goal_id": "h", "revision": 1}
        ]})));
        assert!(!same(json!({})));
    }
}
