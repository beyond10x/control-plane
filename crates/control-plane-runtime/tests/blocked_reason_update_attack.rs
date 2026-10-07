//! Adversary cases for story:blocked-reason-update.
//!
//! The planner's input fingerprint (`supervisor.rs` `fingerprint`) digests every related
//! AssignmentList row whole, `reason` included, and the service runs the planner tick before
//! every fleet tick (`Supervisor::run`). Until this story a Blocked assignment's reason never
//! changed while it stayed Blocked (the `block` route refused Blocked), so a changed blocker of a
//! Blocked assignment recorded progress only, which the fingerprint leaves out (`planning_*`).
//! These cases measure whether replacing the reason now starts planner runs that the planner
//! cannot use: its engine input is the goal and the repository, never an assignment.
use anyhow::Result;
use control_plane_core::{Actor, Store};
use control_plane_runtime::{
    AgentModel, ModelRequest, PUBLICATION_GRACE, RuntimeConfig, SharedStore, Supervisor,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

fn cmd(cwd: &Path, program: &str, args: &[&str], env: &[(String, String)]) -> String {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .envs(env.iter().cloned())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

struct Fixture {
    root: PathBuf,
    store: SharedStore,
    config: RuntimeConfig,
}

/// One registered repository with a story ready for implementation and a bare origin, under a
/// Running goal; the same setup as `fixture(1)` in tests/fleet.rs.
async fn fixture() -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/blocked-reason-update-attack");
    std::fs::create_dir_all(&scratch).unwrap();
    let root = tempfile::tempdir_in(scratch).unwrap().keep();
    std::fs::create_dir(root.join("repos")).unwrap();
    std::fs::create_dir(root.join("remotes")).unwrap();
    let config = RuntimeConfig {
        environment: vec![
            (
                "XDG_STATE_HOME".into(),
                root.join("state").display().to_string(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                root.join("config").display().to_string(),
            ),
            (
                "CARGO_TARGET_DIR".into(),
                root.join("cargo-target").display().to_string(),
            ),
        ],
        commit_command: vec!["git".into(), "commit".into(), "-m".into()],
        ..RuntimeConfig::default()
    };
    let profile = root.join("profile.toml");
    std::fs::write(
        &profile,
        "version = 1\nname = 'blocked-reason-attack'\nexpire_after_seconds = 604800\n\
         protect_workspace_root = false\n",
    )
    .unwrap();
    cmd(
        &root,
        "worktree",
        &[
            "activate",
            "--profile",
            profile.to_str().unwrap(),
            "--workspace",
            root.join("repos").to_str().unwrap(),
        ],
        &config.environment,
    );
    let repo = root.join("repos/repo0");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::create_dir(repo.join("tests")).unwrap();
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = 'blocked_reason_fixture'\nversion = '0.1.0'\nedition = '2024'\n\
         [workspace]\n",
    )
    .unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
    std::fs::write(
        repo.join("tests/acceptance.rs"),
        "#[test]\nfn requested_answer() { assert_eq!(blocked_reason_fixture::answer(), 42); }\n",
    )
    .unwrap();
    std::fs::write(repo.join(".gitignore"), "target\nCargo.lock\n").unwrap();
    std::fs::write(
        repo.join("ess/system.yaml"),
        "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("ess/domains/item.yaml"),
        "domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, \
         type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: \
         [Present]\n      terminal: [Present]\n",
    )
    .unwrap();
    cmd(&repo, "git", &["init", "--initial-branch=main"], &[]);
    cmd(&repo, "git", &["config", "user.name", "Fixture"], &[]);
    cmd(
        &repo,
        "git",
        &["config", "user.email", "fixture@example.invalid"],
        &[],
    );
    for args in [
        vec![
            "plan",
            "reverse",
            "init",
            "--protocols",
            &config.aep_protocols,
            "--profile",
            "development.standard",
        ],
        vec![
            "plan",
            "artifact",
            "new",
            "story",
            "deliver",
            "--title",
            "Return the requested answer",
        ],
        vec![
            "plan",
            "artifact",
            "scope",
            "story:deliver",
            "--add",
            "src/",
            "--inferred",
        ],
        vec![
            "plan",
            "artifact",
            "move",
            "story:deliver",
            "--to",
            "proposed",
        ],
        vec![
            "plan",
            "artifact",
            "move",
            "story:deliver",
            "--to",
            "active",
        ],
    ] {
        cmd(&repo, "aep", &args, &config.environment);
    }
    cmd(&repo, "git", &["add", "."], &[]);
    cmd(
        &repo,
        "git",
        &["commit", "-m", "fixture before requested behavior"],
        &[],
    );
    let remote = root.join("remotes/repo0.git");
    cmd(
        &root,
        "git",
        &["init", "--bare", remote.to_str().unwrap()],
        &[],
    );
    cmd(
        &repo,
        "git",
        &["remote", "add", "origin", remote.to_str().unwrap()],
        &[],
    );
    cmd(&repo, "git", &["push", "origin", "HEAD:main"], &[]);
    let mut host = Store::open(root.join("host.sqlite3")).await.unwrap();
    let workspace = host
        .register_workspace(&repo, "blocked reason fixture")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repositories = host
        .query("RepositoryRegistrationList")
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    // The publisher exits without pushing, so the delivery's intent is left Uncertain.
    host.execute(
        "ConfigureRepository",
        json!({"repository_id":repositories[0]["repository_id"],"base_branch":"main",
            "test_command":"cargo test --quiet","publish_command":"git --version"}),
        Actor::Operator,
    )
    .await
    .unwrap();
    let goal = host
        .execute(
            "CreateGoal",
            json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository",
                "acceptance":"requested_answer passes on each reviewed merged target",
                "max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted",
                "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    Fixture {
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
    }
}

/// The scripted model of tests/fleet.rs, counting its calls per role.
struct Counting {
    steps: Mutex<BTreeMap<String, usize>>,
    calls: Mutex<BTreeMap<String, usize>>,
}
impl Counting {
    fn new() -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
            calls: Mutex::new(BTreeMap::new()),
        }
    }
    fn calls(&self, role: &str) -> usize {
        self.calls.lock().unwrap().get(role).copied().unwrap_or(0)
    }
}
impl AgentModel for Counting {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        *self
            .calls
            .lock()
            .unwrap()
            .entry(request.role.clone())
            .or_default() += 1;
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => Ok(
                json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
            ),
            "implementor" => {
                let mut steps = self.steps.lock().unwrap();
                let count = steps.entry(request.execution_context.clone()).or_default();
                *count += 1;
                Ok(if *count == 1 {
                    json!({"action":"write","path":"src/lib.rs","contents":"pub fn answer() -> u32 { 42 }\n"})
                } else {
                    json!({"action":"finish","summary":"Requested behavior implemented"})
                })
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
}

async fn rows(store: &SharedStore, view: &str) -> Vec<Value> {
    store
        .lock()
        .await
        .query(view)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

/// Decisions the store has committed so far.
async fn committed(store: &SharedStore) -> u64 {
    *store.lock().await.subscribe().borrow()
}

/// One delivery left Uncertain by its publisher, its assignment Blocked, and the planner settled
/// on that state: the service's planner tick runs once more after the fleet blocked the
/// assignment, then stays idle while nothing it reads changes.
async fn settled_unresolved_publication() -> (Fixture, Supervisor, Arc<Counting>) {
    let fixture = fixture().await;
    let model = Arc::new(Counting::new());
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        RuntimeConfig {
            publication_grace: PUBLICATION_GRACE,
            ..fixture.config.clone()
        },
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    assert_eq!(intents.len(), 1, "{intents:?}");
    assert_eq!(intents[0]["state"], "Uncertain", "{intents:?}");
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(assignment["state"], "Blocked", "{assignment}");
    supervisor.tick().await.unwrap();
    let settled = model.calls("planner");
    supervisor.tick().await.unwrap();
    assert_eq!(
        model.calls("planner"),
        settled,
        "fixture: the planner tick is not idle over an unchanged state"
    );
    (fixture, supervisor, model)
}

/// Every member of an assignment row but its reason.
fn without_reason(row: &Value) -> Value {
    let mut row = row.clone();
    row.as_object_mut().unwrap().remove("reason");
    row
}

/// The cause of a Blocked assignment changes once (its remote becomes unreachable while its
/// publication is unresolved). The fleet records the new blocker; the planner's next tick has no
/// new input it can plan from, so it must not run the planner model again.
#[tokio::test]
async fn replaced_blocked_reason_starts_no_planner_run() {
    let (fixture, supervisor, model) = settled_unresolved_publication().await;
    let before = rows(&fixture.store, "AssignmentList").await[0].clone();
    let planner = model.calls("planner");

    let remote = fixture.root.join("remotes/repo0.git");
    std::fs::rename(&remote, fixture.root.join("remotes/repo0-unreachable.git")).unwrap();
    supervisor.fleet_tick().await.unwrap();
    let after = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(after["state"], "Blocked", "{after}");

    supervisor.tick().await.unwrap();
    assert_eq!(
        model.calls("planner") - planner,
        0,
        "one changed blocker of a Blocked assignment started {} planner run(s). The assignment row \
         differs only in its reason ({}): {:?} -> {:?}",
        model.calls("planner") - planner,
        if without_reason(&before) == without_reason(&after) {
            "every other member is equal"
        } else {
            "other members changed too"
        },
        before["reason"],
        after["reason"]
    );
}

/// The service loop (planner tick, then fleet tick) over a remote that answers on every other
/// tick, while the publication is unresolved within its grace period: the blocker alternates
/// between "observation unavailable" and "outcome unresolved". Nothing the planner reads changes
/// apart from the assignment's reason, so the planner model is not run again.
#[tokio::test]
async fn alternating_blocker_starts_no_planner_run_per_cycle() {
    let (fixture, supervisor, model) = settled_unresolved_publication().await;
    let planner = model.calls("planner");
    let critic = model.calls("critic");
    let start = committed(&fixture.store).await;
    let remote = fixture.root.join("remotes/repo0.git");
    let away = fixture.root.join("remotes/repo0-unreachable.git");
    let cycles = 6;
    let mut reasons = Vec::new();
    for cycle in 0..cycles {
        if cycle % 2 == 0 {
            std::fs::rename(&remote, &away).unwrap();
        } else {
            std::fs::rename(&away, &remote).unwrap();
        }
        supervisor.tick().await.unwrap();
        supervisor.fleet_tick().await.unwrap();
        let row = rows(&fixture.store, "AssignmentList").await[0].clone();
        assert_eq!(row["state"], "Blocked", "{row}");
        reasons.push(
            row["reason"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .take(48)
                .collect::<String>(),
        );
    }
    supervisor.tick().await.unwrap();
    let plans = cmd(
        &fixture.root,
        "find",
        &[".", "-maxdepth", "8", "-type", "d", "-name", "cp-plan-*"],
        &[],
    )
    .lines()
    .count();
    assert_eq!(
        rows(&fixture.store, "PublicationIntentList").await[0]["state"],
        "Uncertain"
    );
    assert_eq!(
        model.calls("planner") - planner,
        0,
        "{cycles} service cycles over an alternating blocker ran the planner model {} more times \
         and the critic {} more times, committed {} decisions, and left {plans} planner worktrees; \
         reasons per cycle: {reasons:?}",
        model.calls("planner") - planner,
        model.calls("critic") - critic,
        committed(&fixture.store).await - start
    );
}
