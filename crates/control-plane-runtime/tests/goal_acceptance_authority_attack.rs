//! Adversarial cases for story:goal-acceptance-authority, wave 3 pass 1 (attacks 95274aa).
//!
//! The unit closes the window between goal acceptance's final check and `SatisfyGoal` for one
//! condition, the goal revision, by checking it again inside `Store::execute`. Goal acceptance
//! (`satisfy_goals`, fleet.rs) re-checks two more store facts after its review, the goal state
//! and the workspace's repository configuration, and records any failure in the durable
//! acceptance latch (`acceptance_is_unchanged`). These cases drive the fleet through both.
use anyhow::Result;
use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, SharedStore, Supervisor};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;

const PUBLISH: &str = "git push --force-with-lease=refs/heads/{target}:{expected_base} origin {candidate}:refs/heads/{target}";

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
    store: SharedStore,
    config: RuntimeConfig,
    goal: Value,
    repository: Value,
}

/// One registered repository with a story, a bare `origin` and a running goal: the shape of the
/// `fixture(1)` in tests/fleet.rs, kept apart under `.scratch/goal-acceptance-attack`.
async fn fixture() -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/goal-acceptance-attack");
    std::fs::create_dir_all(&scratch).unwrap();
    let root: PathBuf = tempfile::tempdir_in(scratch).unwrap().keep();
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
    std::fs::write(&profile, "version = 1\nname = 'fleet-fixture'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
        "[package]\nname = 'fleet_fixture_0'\nversion = '0.1.0'\nedition = '2024'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
    std::fs::write(
        repo.join("tests/acceptance.rs"),
        "#[test]\nfn requested_answer() { assert_eq!(fleet_fixture_0::answer(), 42); }\n",
    )
    .unwrap();
    std::fs::write(repo.join(".gitignore"), "target\nCargo.lock\n").unwrap();
    std::fs::write(
        repo.join("ess/system.yaml"),
        "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
    )
    .unwrap();
    std::fs::write(repo.join("ess/domains/item.yaml"), "domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
    cmd(&repo, "git", &["init", "--initial-branch=main"], &[]);
    cmd(&repo, "git", &["config", "user.name", "Fixture"], &[]);
    cmd(
        &repo,
        "git",
        &["config", "user.email", "fixture@example.invalid"],
        &[],
    );
    let aep = |args: &[&str]| cmd(&repo, "aep", args, &config.environment);
    aep(&[
        "plan",
        "reverse",
        "init",
        "--protocols",
        &config.aep_protocols,
        "--profile",
        "development.standard",
    ]);
    aep(&[
        "plan",
        "artifact",
        "new",
        "story",
        "deliver",
        "--title",
        "Return the requested answer",
    ]);
    aep(&[
        "plan",
        "artifact",
        "scope",
        "story:deliver",
        "--add",
        "src/",
        "--inferred",
    ]);
    aep(&[
        "plan",
        "artifact",
        "move",
        "story:deliver",
        "--to",
        "proposed",
    ]);
    aep(&[
        "plan",
        "artifact",
        "move",
        "story:deliver",
        "--to",
        "active",
    ]);
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
        .register_workspace(&repo, "fleet fixture")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0].clone();
    host.execute(
        "ConfigureRepository",
        json!({"repository_id":repository["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":PUBLISH}),
        Actor::Operator,
    )
    .await
    .unwrap();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0].clone();
    let goal = host
        .execute(
            "CreateGoal",
            json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    Fixture {
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        goal,
        repository,
    }
}

/// The scripted roles of tests/fleet.rs: plan the existing story, write the answer, approve.
struct Scripted {
    steps: Mutex<BTreeMap<String, usize>>,
    reviews: Mutex<usize>,
}
impl Scripted {
    fn new() -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
            reviews: Mutex::new(0),
        }
    }
    fn goal_reviews(&self) -> usize {
        *self.reviews.lock().unwrap()
    }
}
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => {
                if request.role == "goal_reviewer" {
                    *self.reviews.lock().unwrap() += 1;
                }
                Ok(
                    json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
                )
            }
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

/// During the goal review, slows the post-review `git ls-remote` of `origin` (its upload-pack
/// sleeps first), and starts an operator who, one second later, replaces the repository's
/// configured check with one that fails. Acceptance compares the repository configuration
/// right after the review, then observes the target with that slowed `ls-remote`, then sends
/// `SatisfyGoal`; the reconfiguration lands between the comparison and the command.
struct ReconfigureAfterFinalCheck {
    inner: Scripted,
    store: SharedStore,
    repository: Value,
    operator: Mutex<Option<std::thread::JoinHandle<(Value, Value)>>>,
}
impl AgentModel for ReconfigureAfterFinalCheck {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        let mut operator = self.operator.lock().unwrap();
        if request.role == "goal_reviewer" && operator.is_none() {
            cmd(
                Path::new(self.repository["path"].as_str().unwrap()),
                "git",
                &[
                    "config",
                    "remote.origin.uploadpack",
                    "sleep 4; git-upload-pack",
                ],
                &[],
            );
            let store = self.store.clone();
            let handle = tokio::runtime::Handle::current();
            let edit = json!({"repository_id":self.repository["repository_id"],"base_branch":"main","test_command":"false","publish_command":PUBLISH});
            *operator = Some(std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(1500));
                let mut store = store.blocking_lock();
                let goal = store.query("GoalList").unwrap()[0].clone();
                let outcome = handle
                    .block_on(store.execute("ConfigureRepository", edit, Actor::Operator))
                    .unwrap();
                (goal, outcome)
            }));
        }
        drop(operator);
        self.inner.respond(request)
    }
}

/// The repository's check changes after acceptance last compared the configuration and before
/// `SatisfyGoal`. Acceptance itself treats a configuration change during acceptance as a reason
/// not to satisfy ("workspace membership or repository configuration changed during
/// acceptance"), and the unit's own rule for the goal revision is that a change in this window
/// leaves the goal Running. The goal is Satisfied under a check that is no longer configured.
#[tokio::test]
async fn configuration_change_after_final_check_is_not_satisfied() {
    let fixture = fixture().await;
    let model = Arc::new(ReconfigureAfterFinalCheck {
        inner: Scripted::new(),
        store: fixture.store.clone(),
        repository: fixture.repository.clone(),
        operator: Mutex::new(None),
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    let report = supervisor.fleet_tick().await;
    let (at_edit, edit) = model
        .operator
        .lock()
        .unwrap()
        .take()
        .expect("goal acceptance reached its review")
        .join()
        .unwrap();
    assert_eq!(edit["outcome"], "applied", "{edit}");
    assert_eq!(
        at_edit["state"], "Running",
        "precondition: the reconfiguration landed before SatisfyGoal: {at_edit}"
    );
    assert_eq!(at_edit["satisfaction_receipt"], "", "{at_edit}");
    let repository = fixture
        .store
        .lock()
        .await
        .query("RepositoryRegistrationList")
        .unwrap()[0]
        .clone();
    assert_eq!(repository["test_command"], "false", "{repository}");
    let goal = fixture.store.lock().await.query("GoalList").unwrap()[0].clone();
    assert_eq!(
        goal["state"], "Running",
        "goal satisfied although its repository's check changed after the final comparison; \
         report {report:?}; goal {goal}"
    );
}

/// Pauses the goal while its reviewer runs, the way an operator does during a long review.
struct PauseDuringGoalReview {
    inner: Scripted,
    store: SharedStore,
    goal: Value,
    paused: AtomicBool,
}
impl AgentModel for PauseDuringGoalReview {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "goal_reviewer" && !self.paused.swap(true, Ordering::SeqCst) {
            let mut store = self.store.blocking_lock();
            let outcome = tokio::runtime::Handle::current()
                .block_on(store.execute("PauseGoal", json!({"goal_id":self.goal}), Actor::Operator))
                .unwrap();
            assert_eq!(outcome["outcome"], "applied", "{outcome}");
        }
        self.inner.respond(request)
    }
}

/// A pause during acceptance is an interruption, not a rejection: the reviewer approved, and
/// nothing about the goal, its repositories or its targets changes. Acceptance nevertheless
/// records it as a `failed` acceptance for the unchanged fingerprint, which the durable latch
/// treats as a rejection. After the operator resumes, neither planning nor acceptance runs
/// again, and the goal stays Running with no way forward short of an edit.
#[tokio::test]
async fn pause_during_goal_review_does_not_latch_acceptance_after_resume() {
    let fixture = fixture().await;
    let model = Arc::new(PauseDuringGoalReview {
        inner: Scripted::new(),
        store: fixture.store.clone(),
        goal: fixture.goal.clone(),
        paused: AtomicBool::new(false),
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(model.inner.goal_reviews(), 1);
    let goal = fixture.store.lock().await.query("GoalList").unwrap()[0].clone();
    assert_eq!(goal["state"], "Paused", "precondition: {goal}");
    assert_eq!(goal["revision"], 1, "{goal}");
    let resumed = fixture
        .store
        .lock()
        .await
        .execute(
            "StartGoal",
            json!({"goal_id":fixture.goal}),
            Actor::Operator,
        )
        .await
        .unwrap();
    assert_eq!(resumed["outcome"], "applied", "{resumed}");
    for _ in 0..2 {
        supervisor.tick().await.unwrap();
        supervisor.fleet_tick().await.unwrap();
    }
    let goal = fixture.store.lock().await.query("GoalList").unwrap()[0].clone();
    let acceptance = {
        let store = fixture.store.lock().await;
        store
            .activity_history(fixture.goal.as_str().unwrap())
            .unwrap()["acceptance"]
            .clone()
    };
    assert_eq!(
        goal["state"],
        "Satisfied",
        "resumed goal never re-reviewed ({} goal reviews); acceptance {acceptance}",
        model.inner.goal_reviews()
    );
}
