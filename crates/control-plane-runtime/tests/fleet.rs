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
        atomic::{AtomicUsize, Ordering},
    },
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
    goal: Value,
    repositories: Vec<Value>,
}
async fn fixture(count: usize) -> Fixture {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/fleet");
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
    std::fs::write(&profile,"version = 1\nname = 'fleet-fixture'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
    for index in 0..count {
        let repo = root.join(format!("repos/repo{index}"));
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::create_dir(repo.join("tests")).unwrap();
        std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
        std::fs::write(repo.join("Cargo.toml"),format!("[package]\nname = 'fleet_fixture_{index}'\nversion = '0.1.0'\nedition = '2024'\n[workspace]\n")).unwrap();
        std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
        std::fs::write(repo.join("tests/acceptance.rs"),format!("#[test]\nfn requested_answer() {{ assert_eq!(fleet_fixture_{index}::answer(), 42); }}\n")).unwrap();
        std::fs::write(repo.join(".gitignore"), "target\nCargo.lock\n").unwrap();
        std::fs::write(
            repo.join("ess/system.yaml"),
            "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
        )
        .unwrap();
        std::fs::write(repo.join("ess/domains/item.yaml"),"domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
        cmd(&repo, "git", &["init", "--initial-branch=main"], &[]);
        cmd(&repo, "git", &["config", "user.name", "Fixture"], &[]);
        cmd(
            &repo,
            "git",
            &["config", "user.email", "fixture@example.invalid"],
            &[],
        );
        cmd(
            &repo,
            "aep",
            &[
                "plan",
                "reverse",
                "init",
                "--protocols",
                &config.aep_protocols,
                "--profile",
                "development.standard",
            ],
            &config.environment,
        );
        cmd(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "new",
                "story",
                "deliver",
                "--title",
                "Return the requested answer",
            ],
            &config.environment,
        );
        cmd(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "scope",
                "story:deliver",
                "--add",
                "src/",
                "--inferred",
            ],
            &config.environment,
        );
        cmd(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "move",
                "story:deliver",
                "--to",
                "proposed",
            ],
            &config.environment,
        );
        cmd(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "move",
                "story:deliver",
                "--to",
                "active",
            ],
            &config.environment,
        );
        cmd(&repo, "git", &["add", "."], &[]);
        cmd(
            &repo,
            "git",
            &["commit", "-m", "fixture before requested behavior"],
            &[],
        );
        let remote = root.join(format!("remotes/repo{index}.git"));
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
    }
    let mut host = Store::open(root.join("host.sqlite3")).await.unwrap();
    let workspace = host
        .register_workspace(&root.join("repos/repo0"), "fleet fixture")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    for index in 1..count {
        host.add_workspace_directory(
            workspace.as_str().unwrap(),
            &root.join(format!("repos/repo{index}")),
        )
        .await
        .unwrap();
    }
    let repositories = host
        .query("RepositoryRegistrationList")
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    for repo in &repositories {
        host.execute("ConfigureRepository",json!({"repository_id":repo["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git push --force-with-lease=refs/heads/{target}:{expected_base} origin {candidate}:refs/heads/{target}"}),Actor::Operator).await.unwrap();
    }
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    Fixture {
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        goal,
        repositories,
    }
}
struct Scripted {
    steps: Mutex<BTreeMap<String, usize>>,
    contexts: Mutex<Vec<(String, String)>>,
    calls: AtomicUsize,
}
impl Scripted {
    fn new() -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
            contexts: Mutex::new(Vec::new()),
            calls: AtomicUsize::new(0),
        }
    }
}
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.contexts
            .lock()
            .unwrap()
            .push((request.role.clone(), request.execution_context.clone()));
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

#[tokio::test]
async fn review_is_independent_and_two_repositories_reach_observed_goal_completion() {
    let fixture = fixture(2).await;
    let model = Arc::new(Scripted::new());
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    let planning = supervisor.tick().await.unwrap();
    assert_eq!(planning.queued, 2, "{planning:?}");
    supervisor.fleet_tick().await.unwrap();
    let assignments = fixture.store.lock().await.query("AssignmentList").unwrap();
    assert_eq!(assignments.as_array().unwrap().len(), 2);
    for assignment in assignments.as_array().unwrap() {
        assert_eq!(assignment["state"], "Merged", "{assignment}");
        assert_ne!(assignment["implementor_run"], assignment["reviewer_run"]);
        assert_eq!(assignment["candidate"], assignment["test_revision"]);
        assert_eq!(assignment["candidate"], assignment["review_revision"]);
        assert!(!assignment["merge_receipt"].as_str().unwrap().is_empty());
        let repo = fixture
            .repositories
            .iter()
            .find(|r| r["repository_id"] == assignment["repository_id"])
            .unwrap();
        let remote = cmd(
            Path::new(repo["path"].as_str().unwrap()),
            "git",
            &["ls-remote", "origin", "refs/heads/main"],
            &[],
        );
        assert!(remote.starts_with(assignment["candidate"].as_str().unwrap()));
    }
    let goals = fixture.store.lock().await.query("GoalList").unwrap();
    assert_eq!(goals[0]["state"], "Satisfied", "{goals}");
    assert!(
        model
            .contexts
            .lock()
            .unwrap()
            .iter()
            .any(|(role, _)| role == "goal_reviewer")
    );
    let calls = model.calls.load(Ordering::SeqCst);
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(model.calls.load(Ordering::SeqCst), calls);
}

struct RejectGoalReview(Scripted);
impl AgentModel for RejectGoalReview {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "goal_reviewer" {
            self.0.calls.fetch_add(1, Ordering::SeqCst);
            return Ok(
                json!({"approved":false,"reason":"A stated acceptance obligation is not demonstrated"}),
            );
        }
        self.0.respond(request)
    }
}

#[tokio::test]
async fn rejected_goal_acceptance_remains_durable_and_idle_until_inputs_change() {
    let fixture = fixture(1).await;
    let model = Arc::new(RejectGoalReview(Scripted::new()));
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    let result = supervisor.fleet_tick().await.unwrap();
    assert!(
        result
            .blockers
            .iter()
            .any(|r| r.contains("final goal review rejected")),
        "{result:?}"
    );
    let goals = fixture.store.lock().await.query("GoalList").unwrap();
    assert_eq!(goals[0]["state"], "Running");
    let receipt: Value =
        serde_json::from_str(goals[0]["planning_receipt"].as_str().unwrap()).unwrap();
    assert_eq!(receipt["acceptance"]["status"], "failed");
    assert!(
        receipt["acceptance"]["reason"]
            .as_str()
            .unwrap()
            .contains("acceptance obligation")
    );
    let calls = model.0.calls.load(Ordering::SeqCst);
    drop(supervisor);
    let restarted = Supervisor::new(
        fixture.store,
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    restarted.fleet_tick().await.unwrap();
    assert_eq!(model.0.calls.load(Ordering::SeqCst), calls);
}

async fn revise(fixture: &Fixture, authority: bool) {
    fixture.store.lock().await.execute("UpdateGoal",json!({"goal_id":fixture.goal,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":authority}),Actor::Operator).await.unwrap();
}

#[tokio::test]
async fn merge_requires_current_authority() {
    let fixture = fixture(1).await;
    revise(&fixture, false).await;
    let model = Arc::new(Scripted::new());
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model,
    );
    supervisor.tick().await.unwrap();
    let initial = cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &["ls-remote", "origin", "refs/heads/main"],
        &[],
    );
    let report = supervisor.fleet_tick().await.unwrap();
    assert!(!report.blockers.is_empty());
    assert_eq!(
        fixture
            .store
            .lock()
            .await
            .query("PublicationIntentList")
            .unwrap(),
        json!([])
    );
    assert_eq!(
        cmd(
            &fixture.root.join("repos/repo0"),
            "git",
            &["ls-remote", "origin", "refs/heads/main"],
            &[]
        ),
        initial
    );
    assert_eq!(
        fixture.store.lock().await.query("AssignmentList").unwrap()[0]["state"],
        "Blocked"
    );
}
struct ChangeDuringReview {
    inner: Scripted,
    store: SharedStore,
    goal: Value,
    changed: std::sync::atomic::AtomicBool,
}
impl AgentModel for ChangeDuringReview {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "reviewer" && !self.changed.swap(true, Ordering::SeqCst) {
            tokio::runtime::Handle::current().block_on(async {
                self.store.lock().await.execute("UpdateGoal",json!({"goal_id":self.goal,"objective":"Changed objective invalidates old review","acceptance":"Different acceptance","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false}),Actor::Operator).await
            })?;
        }
        self.inner.respond(request)
    }
}
#[tokio::test]
async fn changed_revision_invalidates_evidence() {
    let fixture = fixture(1).await;
    let model = Arc::new(ChangeDuringReview {
        inner: Scripted::new(),
        store: fixture.store.clone(),
        goal: fixture.goal.clone(),
        changed: std::sync::atomic::AtomicBool::new(false),
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model,
    );
    supervisor.tick().await.unwrap();
    let report = supervisor.fleet_tick().await.unwrap();
    assert!(!report.blockers.is_empty());
    let assignments = fixture.store.lock().await.query("AssignmentList").unwrap();
    assert_eq!(assignments[0]["state"], "Blocked");
    assert_eq!(assignments[0]["review_revision"], "");
    assert_eq!(
        fixture
            .store
            .lock()
            .await
            .query("PublicationIntentList")
            .unwrap(),
        json!([])
    );
}

#[tokio::test]
async fn pause_and_limits_stop_dispatch() {
    let mut fixture = fixture(1).await;
    let model = Arc::new(Scripted::new());
    let planning = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    planning.tick().await.unwrap();
    drop(planning);
    fixture
        .store
        .lock()
        .await
        .execute(
            "PauseGoal",
            json!({"goal_id":fixture.goal}),
            Actor::Operator,
        )
        .await
        .unwrap();
    fixture.config.max_steps = 1;
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    let calls = model.calls.load(Ordering::SeqCst);
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(model.calls.load(Ordering::SeqCst), calls);
    assert_eq!(
        fixture.store.lock().await.query("AssignmentList").unwrap()[0]["attempt"],
        0
    );
    fixture
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
    supervisor.fleet_tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let calls = model.calls.load(Ordering::SeqCst);
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(model.calls.load(Ordering::SeqCst), calls);
    let assignments = fixture.store.lock().await.query("AssignmentList").unwrap();
    assert_eq!(assignments[0]["attempt"], 2);
    assert_eq!(assignments[0]["state"], "Blocked");
}

#[tokio::test]
async fn restart_reconciles_effects_and_zero_exit_is_not_a_merge_receipt() {
    let fixture = fixture(1).await;
    fixture.store.lock().await.execute("ConfigureRepository",json!({"repository_id":fixture.repositories[0]["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git --version"}),Actor::Operator).await.unwrap();
    let model = Arc::new(Scripted::new());
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model,
    );
    supervisor.tick().await.unwrap();
    let report = supervisor.fleet_tick().await.unwrap();
    assert!(!report.blockers.is_empty());
    let intent = fixture
        .store
        .lock()
        .await
        .query("PublicationIntentList")
        .unwrap()[0]
        .clone();
    assert_eq!(intent["state"], "Uncertain");
    assert_ne!(
        fixture.store.lock().await.query("AssignmentList").unwrap()[0]["state"],
        "Merged"
    );
    fixture
        .store
        .lock()
        .await
        .execute(
            "PauseGoal",
            json!({"goal_id":fixture.goal}),
            Actor::Operator,
        )
        .await
        .unwrap();
    cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &[
            "push",
            "origin",
            &format!("{}:refs/heads/main", intent["candidate"].as_str().unwrap()),
        ],
        &[],
    );
    drop(supervisor);
    drop(fixture.store);
    let reopened = Arc::new(tokio::sync::Mutex::new(
        Store::open(fixture.root.join("host.sqlite3"))
            .await
            .unwrap(),
    ));
    let model = Arc::new(Scripted::new());
    let recovered = Supervisor::new(
        reopened.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    recovered.fleet_tick().await.unwrap();
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        reopened
            .lock()
            .await
            .query("PublicationIntentList")
            .unwrap()[0]["state"],
        "Confirmed"
    );
    let assignment = reopened.lock().await.query("AssignmentList").unwrap()[0].clone();
    assert_eq!(assignment["state"], "Merged");
    assert!(!assignment["merge_receipt"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn repository_execution_is_exclusive_across_workspace_aliases() {
    let fixture = fixture(1).await;
    let primary = fixture.root.join("repos/repo0");
    let alias_id = format!("alias-{}", uuid::Uuid::new_v4());
    let created: Value = serde_json::from_str(&cmd(
        &primary,
        "worktree",
        &[
            "create",
            "--json",
            "--repo",
            primary.to_str().unwrap(),
            "--base",
            "main",
            "--id",
            &alias_id,
            "--purpose",
            "cross workspace fixture",
        ],
        &fixture.config.environment,
    ))
    .unwrap();
    let alias = Path::new(created["evidence"]["path"].as_str().unwrap());
    let mut host = fixture.store.lock().await;
    let workspace=host.register_workspace(alias,"alias").await.unwrap()["published"][0]["payload"]["workspace_id"].clone();
    let repository = host
        .query("RepositoryRegistrationList")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["workspace_id"] == workspace)
        .unwrap()["repository_id"]
        .clone();
    host.execute("ConfigureRepository",json!({"repository_id":repository,"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git push --force-with-lease=refs/heads/{target}:{expected_base} origin {candidate}:refs/heads/{target}"}),Actor::Operator).await.unwrap();
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Same repository through an alias","acceptance":"requested_answer passes after observed merge","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    drop(host);
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        Arc::new(Scripted::new()),
    );
    assert_eq!(supervisor.tick().await.unwrap().queued, 2);
    supervisor.fleet_tick().await.unwrap();
    let assignments = fixture.store.lock().await.query("AssignmentList").unwrap();
    let rows = assignments.as_array().unwrap();
    assert_eq!(
        rows.iter().filter(|a| a["state"] == "Merged").count(),
        1,
        "{assignments}"
    );
    assert_eq!(
        rows.iter().filter(|a| a["state"] == "Queued").count(),
        1,
        "{assignments}"
    );
}
