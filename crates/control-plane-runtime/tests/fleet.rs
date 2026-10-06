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
/// The goal's progress history (`Store::activity_history`), including its newest acceptance
/// record, which bounded receipts record once per review.
async fn history(store: &SharedStore) -> Value {
    let store = store.lock().await;
    let goal = store.query("GoalList").unwrap()[0]["goal_id"].clone();
    store.activity_history(goal.as_str().unwrap()).unwrap()
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

struct InspectionWritesOutsideScope {
    inner: Scripted,
    attempted: std::sync::atomic::AtomicBool,
    output: &'static str,
}
impl AgentModel for InspectionWritesOutsideScope {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "implementor" && !self.attempted.swap(true, Ordering::SeqCst) {
            // git diff is admitted as inspection, but --output truncates any named file.
            // This fixture's accepted story scope is src/, never tests/.
            return Ok(
                json!({"action":"run","program":"git","args":["diff",format!("--output={}",self.output)]}),
            );
        }
        self.inner.respond(request)
    }
}

#[tokio::test]
async fn inspection_commands_cannot_overwrite_tests_outside_accepted_scope() {
    let fixture = fixture(1).await;
    let model = Arc::new(InspectionWritesOutsideScope {
        inner: Scripted::new(),
        attempted: std::sync::atomic::AtomicBool::new(false),
        output: "tests/acceptance.rs",
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    assert!(model.attempted.load(Ordering::SeqCst));
    let primary = fixture.root.join("repos/repo0");
    let published = cmd(
        &primary,
        "git",
        &["show", "refs/remotes/origin/main:tests/acceptance.rs"],
        &[],
    );
    assert!(
        published.contains("requested_answer"),
        "An admitted inspection command erased and published acceptance tests outside src/ scope: {published:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn inspection_commands_cannot_write_through_repository_symlinks() {
    let fixture = fixture(1).await;
    let sentinel = fixture.root.join("outside-worktree.txt");
    std::fs::write(&sentinel, "operator-owned data").unwrap();
    let primary = fixture.root.join("repos/repo0");
    std::os::unix::fs::symlink(&sentinel, primary.join("tests/external.txt")).unwrap();
    cmd(&primary, "git", &["add", "tests/external.txt"], &[]);
    cmd(
        &primary,
        "git",
        &["commit", "-m", "fixture tracked context symlink"],
        &[],
    );
    cmd(&primary, "git", &["push", "origin", "HEAD:main"], &[]);
    let model = Arc::new(InspectionWritesOutsideScope {
        inner: Scripted::new(),
        attempted: std::sync::atomic::AtomicBool::new(false),
        output: "tests/external.txt",
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    assert!(model.attempted.load(Ordering::SeqCst));
    assert_eq!(
        std::fs::read_to_string(sentinel).unwrap(),
        "operator-owned data",
        "admitted inspection wrote outside managed worktree through a tracked symlink"
    );
}

#[tokio::test]
async fn restart_marks_exhausted_inflight_attempt_as_blocked() {
    let fixture = fixture(1).await;
    let model = Arc::new(Scripted::new());
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model,
    );
    supervisor.tick().await.unwrap();
    let mut store = fixture.store.lock().await;
    let assignment = store.query("AssignmentList").unwrap()[0].clone();
    let id = assignment["assignment_id"].clone();
    let base = cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &["rev-parse", "HEAD"],
        &[],
    )
    .trim()
    .to_owned();
    // Exactly the durable transitions deliver() emits before an interrupted retry.
    for (command, payload) in [
        (
            "ClaimAssignment",
            json!({"assignment_id":id,"worktree_id":"interrupted-tree","implementor_run":"attempt-one","base_revision":base}),
        ),
        (
            "BlockAssignment",
            json!({"assignment_id":id,"reason":"first attempt interrupted"}),
        ),
        (
            "RepairAssignment",
            json!({"assignment_id":id,"reason":"retry","implementor_run":"attempt-two"}),
        ),
    ] {
        assert_eq!(
            store
                .execute(command, payload, Actor::Supervisor)
                .await
                .unwrap()["outcome"],
            "applied"
        );
    }
    assert_eq!(store.query("AssignmentList").unwrap()[0]["attempt"], 2);
    drop(store);
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
    assert_eq!(
        model.calls.load(Ordering::SeqCst),
        0,
        "exhausted attempts cannot restart model work"
    );
    let assignment = reopened.lock().await.query("AssignmentList").unwrap()[0].clone();
    assert_eq!(
        assignment["state"], "Blocked",
        "exhausted persisted attempt must not remain falsely active after restart: {assignment}"
    );
    assert!(!assignment["reason"].as_str().unwrap().is_empty());
}
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
    let mut fixture = fixture(1).await;
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
    let receipt = history(&fixture.store).await;
    assert_eq!(receipt["acceptance"]["status"], "failed");
    assert!(
        receipt["acceptance"]["reason"]
            .as_str()
            .unwrap()
            .contains("acceptance obligation")
    );
    let calls = model.0.calls.load(Ordering::SeqCst);
    drop(supervisor);
    drop(fixture.store);
    fixture.store = Arc::new(tokio::sync::Mutex::new(
        Store::open(fixture.root.join("host.sqlite3"))
            .await
            .unwrap(),
    ));
    let restarted = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    restarted.tick().await.unwrap();
    restarted.fleet_tick().await.unwrap();
    assert_eq!(model.0.calls.load(Ordering::SeqCst), calls);
    let repo = fixture.root.join("repos/repo0");
    let origin = cmd(&repo, "git", &["remote", "get-url", "origin"], &[]);
    cmd(
        &repo,
        "git",
        &[
            "remote",
            "set-url",
            "origin",
            fixture.root.join("unavailable.git").to_str().unwrap(),
        ],
        &[],
    );
    restarted.tick().await.unwrap();
    restarted.fleet_tick().await.unwrap();
    assert_eq!(
        model.0.calls.load(Ordering::SeqCst),
        calls,
        "unavailable remote is not evidence permitting more model spend"
    );
    cmd(
        &repo,
        "git",
        &["remote", "set-url", "origin", origin.trim()],
        &[],
    );
    revise(&fixture, true).await;
    restarted.tick().await.unwrap();
    assert!(model.0.calls.load(Ordering::SeqCst) > calls);
}

/// A goal reviewer that rejects with one line per unmet obligation, about 2.9 KiB in all.
struct DetailedGoalRejection(Scripted);
impl AgentModel for DetailedGoalRejection {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "goal_reviewer" {
            let mut reason = String::new();
            for index in 1..=40 {
                reason.push_str(&format!(
                    "Obligation {index:02} is not demonstrated by the observed checks or diff.\n"
                ));
            }
            reason.push_str(
                "Unmet obligation 41: requested_answer is not shown on the reviewed target.",
            );
            return Ok(json!({"approved":false,"reason":reason}));
        }
        self.0.respond(request)
    }
}

#[tokio::test]
async fn adversary_rejected_goal_acceptance_keeps_the_reviewers_whole_reason() {
    // The operator reads why acceptance failed from the goal's durable acceptance record
    // (its progress history, shown by the evidence page). The reviewer's reason must
    // reach it whole; the blocked activity keeps only a short summary of it.
    let fixture = fixture(1).await;
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        Arc::new(DetailedGoalRejection(Scripted::new())),
    );
    supervisor.tick().await.unwrap();
    let result = supervisor.fleet_tick().await.unwrap();
    assert!(
        result
            .blockers
            .iter()
            .any(|reason| reason.contains("Unmet obligation 41")),
        "{result:?}"
    );
    let receipt = history(&fixture.store).await;
    assert_eq!(receipt["acceptance"]["status"], "failed");
    let recorded = receipt["acceptance"]["reason"].as_str().unwrap();
    assert!(
        recorded.contains("Unmet obligation 41"),
        "the durable acceptance reason ends: {}",
        &recorded[recorded.len().saturating_sub(240)..]
    );
}

/// A goal reviewer that rejects with 120 unmet obligations, about 8.5 KiB, and notes the event
/// data the store had recorded when it was asked.
struct ThoroughGoalRejection {
    inner: Scripted,
    store: SharedStore,
    asked: Mutex<Option<(u64, u64)>>,
}
impl AgentModel for ThoroughGoalRejection {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        if request.role == "goal_reviewer" {
            let mark = tokio::runtime::Handle::current().block_on(async {
                let store = self.store.lock().await;
                let version = *store.subscribe().borrow();
                (store.appended_event_bytes(), version)
            });
            *self.asked.lock().unwrap() = Some(mark);
            let mut reason = String::new();
            for index in 1..=120 {
                reason.push_str(&format!(
                    "Obligation {index:03} is not demonstrated by the observed checks or diff.\n"
                ));
            }
            return Ok(json!({"approved":false,"reason":reason}));
        }
        self.inner.respond(request)
    }
}

#[tokio::test]
async fn adversary_rejected_acceptance_record_decision_stays_under_16_kib() {
    // story:bounded-progress-records outcome: at most 16 KiB of event data per progress
    // decision. A rejected goal review is recorded with RecordPlanningProgress
    // (fleet.rs:114-123) and keeps up to 8 KiB of the reason (ACCEPTANCE_REASON_BYTES). Each
    // decision stores the receipt in its body and again in its outcome.
    //
    // After the reviewer answers, the fleet records two decisions: the acceptance record and
    // the blocked activity. The blocked activity is measured by recording its twin (same
    // assignment, action, role and detail) through the same progress path afterwards.
    let fixture = fixture(1).await;
    let model = Arc::new(ThoroughGoalRejection {
        inner: Scripted::new(),
        store: fixture.store.clone(),
        asked: Mutex::new(None),
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    let result = supervisor.fleet_tick().await.unwrap();
    let reason = result
        .blockers
        .iter()
        .find(|reason| reason.contains("final goal review rejected"))
        .cloned()
        .unwrap_or_else(|| panic!("{result:?}"));
    let (asked_bytes, asked_version) = model.asked.lock().unwrap().expect("goal review ran");
    let (bytes, version, assignment) = {
        let store = fixture.store.lock().await;
        let version = *store.subscribe().borrow();
        let assignment = store.query("AssignmentList").unwrap()[0].clone();
        (store.appended_event_bytes(), version, assignment)
    };
    assert_eq!(
        version - asked_version,
        2,
        "after the goal review: the acceptance record and the blocked activity"
    );
    let before = fixture.store.lock().await.appended_event_bytes();
    supervisor
        .record_progress(
            &assignment,
            "blocked",
            "goal_reviewer",
            json!({"reason":reason}),
        )
        .await
        .unwrap();
    let blocked = fixture.store.lock().await.appended_event_bytes() - before;
    let acceptance = bytes - asked_bytes - blocked;
    assert!(
        acceptance <= 16 * 1024,
        "the rejected acceptance record is one decision of {acceptance} bytes of event data \
         (reviewer reason {} bytes; blocked activity {blocked} bytes)",
        reason.len()
    );
}

const PUBLISH_CREDENTIAL: &str = "CONTROL_PLANE_PUBLISH_PROBE";

/// Deliver one assignment while the test and publish commands record their environments
/// outside the candidate tree. Returns (test environment, publish environment).
async fn delivered_command_environments() -> (String, String) {
    let mut fixture = fixture(1).await;
    let checks = fixture.root.join("test-environment");
    let publish = fixture.root.join("publish-environment");
    let repository = fixture.repositories[0]["repository_id"].clone();
    fixture.store.lock().await.execute("ConfigureRepository",json!({"repository_id":repository,"base_branch":"main","test_command":format!("sh -c 'env > {} && cargo test --quiet'",checks.display()),"publish_command":format!("sh -c 'env > {} && git push --force-with-lease=refs/heads/{{target}}:{{expected_base}} origin {{candidate}}:refs/heads/{{target}}'",publish.display())}),Actor::Operator).await.unwrap();
    fixture
        .config
        .credentials
        .push((PUBLISH_CREDENTIAL.into(), "publish-secret".into()));
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        Arc::new(Scripted::new()),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(
        fixture.store.lock().await.query("AssignmentList").unwrap()[0]["state"],
        "Merged"
    );
    (
        std::fs::read_to_string(checks).unwrap(),
        std::fs::read_to_string(publish).unwrap(),
    )
}

#[tokio::test]
async fn publish_command_receives_only_named_credentials() {
    let (_, publish) = delivered_command_environments().await;
    assert!(publish.contains(&format!("{PUBLISH_CREDENTIAL}=publish-secret")));
    for line in publish.lines() {
        let name = line.split_once('=').map_or(line, |(name, _)| name);
        assert!(
            control_plane_runtime::process::INHERITED_ENVIRONMENT.contains(&name)
                || name == PUBLISH_CREDENTIAL
                || name.starts_with("CONTROL_PLANE_")
                || ["XDG_STATE_HOME", "XDG_CONFIG_HOME", "CARGO_TARGET_DIR"].contains(&name)
                // Set by `sh` itself, not inherited from the service.
                || ["PWD", "OLDPWD", "SHLVL", "_"].contains(&name),
            "publish command received unnamed variable {name}"
        );
    }
}

#[tokio::test]
async fn named_credentials_stay_out_of_other_commands() {
    let (checks, _) = delivered_command_environments().await;
    assert!(checks.contains("PATH="), "{checks}");
    assert!(!checks.contains(PUBLISH_CREDENTIAL), "{checks}");
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
async fn restart_reconciles_effects() {
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
async fn repository_execution_is_exclusive() {
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

#[tokio::test]
async fn native_loom_delivers_candidate_through_checks_review_and_observed_publication() {
    native_read_recovery("absolute").await;
}
#[tokio::test]
async fn native_loom_parent_read_is_refused_and_corrected() {
    native_read_recovery("parent").await;
}
#[tokio::test]
async fn native_loom_repeated_malformed_read_exhausts_refusal_budget() {
    native_read_recovery("repeat").await;
}
#[tokio::test]
async fn native_loom_symlink_read_remains_fatal() {
    native_read_recovery("symlink").await;
}
async fn native_read_recovery(mode: &'static str) {
    use llm_core::{BoxFuture, Capabilities, Model, Provenance, TurnObservation};
    struct Provider {
        binding: Provenance,
        caps: Capabilities,
        mode: &'static str,
        absolute_paths: Vec<String>,
        implementation_calls: AtomicUsize,
    }
    impl Model for Provider {
        fn provenance(&self) -> &Provenance {
            &self.binding
        }
        fn capabilities(&self) -> &Capabilities {
            &self.caps
        }
        fn turn<'a>(
            &'a self,
            request: &'a llm_core::TurnRequest,
            _: &'a mut dyn llm_core::StreamSink,
            _: &'a llm_core::Cancel,
        ) -> BoxFuture<'a, Result<llm_core::TurnOutcome, llm_core::Error>> {
            Box::pin(async move {
                request.validate_for(&self.binding, &self.caps)?;
                let previous = request
                    .items
                    .iter()
                    .filter(|item| matches!(item, llm_core::Item::ToolCall(_)))
                    .count();
                let briefs = request.items.iter().filter(|item| matches!(item, llm_core::Item::UserText{text} if text.contains("\"goal\""))).count();
                assert!(
                    briefs <= 1,
                    "native session repeated the role brief {briefs} times"
                );
                assert!(!request.items.iter().any(|item|matches!(item,llm_core::Item::UserText{text} if text.contains("planning_receipt"))), "runtime bookkeeping leaked into native role session");
                let arguments = if request
                    .instructions
                    .contains("Implement the accepted AEP story")
                {
                    self.implementation_calls.fetch_add(1, Ordering::SeqCst);
                    assert!(!request.items.iter().any(|item| matches!(item,llm_core::Item::UserText{text} if text.contains("EXTERNAL_SECRET_MUST_NOT_BE_READ"))), "confined read leaked external contents");
                    if previous > 0 {
                        assert!(request.items.iter().any(|item| matches!(item,llm_core::Item::UserText{text} if text.contains("read_path_syntax") && text.contains("worktree-relative"))), "native continuation did not receive typed corrective feedback");
                    }
                    if previous == 0 || self.mode == "repeat" {
                        let paths = match self.mode {
                            "parent" => vec!["../outside-secret.txt".into()],
                            "symlink" => vec!["external.txt".into()],
                            _ => self.absolute_paths.clone(),
                        };
                        json!({"action":"read","paths":paths})
                    } else if previous == 1 {
                        json!({"action":"read","paths":["Cargo.toml"]})
                    } else if previous == 2 {
                        assert!(request.items.iter().any(|item| matches!(item,llm_core::Item::UserText{text} if text.contains("fleet_fixture_0"))), "corrected relative read did not reach model");
                        json!({"action":"write","path":"src/lib.rs","contents":"pub fn answer() -> u32 { 42 }\n"})
                    } else {
                        json!({"action":"finish","summary":"Candidate ready for actual checks and independent review"})
                    }
                } else if request.instructions.contains("Independently") {
                    assert_eq!(previous, 0, "each reviewer needs a separate session");
                    json!({"approved":true,"reason":"Trusted test output and exact diff establish the accepted answer"})
                } else {
                    json!({"action":"finish","stories":["story:deliver"],"summary":"Select existing accepted scope"})
                };
                Ok(llm_core::TurnOutcome {
                    stop_reason: llm_core::StopReason::ToolCalls,
                    items: vec![
                        llm_core::Item::Opaque {
                            provenance: self.binding.clone(),
                            payload: json!({"retained":true}),
                        },
                        llm_core::Item::ToolCall(llm_core::ToolCall {
                            call_id: llm_core::CallId::new(format!("call-{previous}")).unwrap(),
                            name: request.tools[0].name.clone(),
                            arguments,
                        }),
                    ],
                    observation: TurnObservation {
                        usage: Some(llm_core::Usage {
                            input_tokens: Some(20),
                            output_tokens: Some(10),
                            cached_input_tokens: Some(0),
                            ..Default::default()
                        }),
                        final_usage: true,
                        ..TurnObservation::new(self.binding.clone())
                    },
                })
            })
        }
    }
    let fixture = fixture(1).await;
    let primary = fixture.root.join("repos/repo0");
    let sentinel = fixture.root.join("outside-secret.txt");
    std::fs::write(&sentinel, "EXTERNAL_SECRET_MUST_NOT_BE_READ").unwrap();
    if mode == "symlink" {
        std::os::unix::fs::symlink(&sentinel, primary.join("external.txt")).unwrap();
        cmd(&primary, "git", &["add", "external.txt"], &[]);
        cmd(
            &primary,
            "git",
            &["commit", "-m", "tracked external symlink"],
            &[],
        );
        cmd(&primary, "git", &["push", "origin", "HEAD:main"], &[]);
    }
    let absolute_paths = [
        primary.join("AGENTS.md"),
        primary.join("TASK.md"),
        primary.join("Cargo.toml"),
        sentinel.clone(),
    ]
    .iter()
    .map(|p| p.display().to_string())
    .collect();
    let provider=Arc::new(Provider{mode,absolute_paths,implementation_calls:AtomicUsize::new(0),binding:serde_json::from_value(json!({"protocol":"responses","provider":"fixture","account":"test","endpoint":"offline","model":"scripted","binding_revision":"one"})).unwrap(),caps:Capabilities{tools:true,tool_choice:true,temperature:false,top_p:false,reasoning_efforts:vec![],context_window:128000,max_output_tokens:32000}});
    let sessions = tempfile::tempdir().unwrap();
    let model = Arc::new(control_plane_runtime::CodexAgentModel::with_provider(
        sessions.path().into(),
        provider.clone(),
    ));
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config,
        model,
    );
    let planning = supervisor.tick().await.unwrap();
    assert_eq!(planning.queued, 1, "{planning:?}");
    let fleet = supervisor.fleet_tick().await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&sentinel).unwrap(),
        "EXTERNAL_SECRET_MUST_NOT_BE_READ"
    );
    if mode == "repeat" || mode == "symlink" {
        let expected = if mode == "repeat" {
            "refusal budget exhausted after 5 consecutive refused actions"
        } else {
            "symlink paths are not tool inputs"
        };
        assert!(
            fleet.blockers.iter().any(|b| b.contains(expected)),
            "{fleet:?}"
        );
        assert_eq!(
            provider.implementation_calls.load(Ordering::SeqCst),
            if mode == "repeat" { 5 } else { 1 }
        );
        let store = fixture.store.lock().await;
        assert_eq!(
            store.query("AssignmentList").unwrap()[0]["state"],
            "Blocked"
        );
        assert_eq!(store.query("PublicationIntentList").unwrap(), json!([]));
        return;
    }
    assert!(fleet.blockers.is_empty(), "{fleet:?}");
    let store = fixture.store.lock().await;
    let assignments = store.query("AssignmentList").unwrap();
    assert_eq!(assignments[0]["state"], "Merged", "{assignments}");
    assert_eq!(assignments[0]["candidate"], assignments[0]["test_revision"]);
    assert_eq!(
        assignments[0]["candidate"],
        assignments[0]["review_revision"]
    );
    let remote = cmd(
        Path::new(fixture.repositories[0]["path"].as_str().unwrap()),
        "git",
        &["ls-remote", "origin", "refs/heads/main"],
        &[],
    );
    assert!(remote.starts_with(assignments[0]["candidate"].as_str().unwrap()));
    assert_eq!(store.query("GoalList").unwrap()[0]["state"], "Satisfied");
    assert_eq!(
        std::fs::read_dir(sessions.path()).unwrap().count(),
        5,
        "planner, critic, implementor, reviewer and goal reviewer sessions"
    );
}

/// One implementor turn: the proposed action, and the refusal code the next turn's input
/// must carry when the host refuses it.
type Turn = (Value, Option<&'static str>);

struct RefusalProvider {
    binding: llm_core::Provenance,
    caps: llm_core::Capabilities,
    turns: Vec<Turn>,
    implementation_calls: AtomicUsize,
    unseen: Mutex<Vec<&'static str>>,
    inputs: Mutex<Vec<String>>,
}
impl llm_core::Model for RefusalProvider {
    fn provenance(&self) -> &llm_core::Provenance {
        &self.binding
    }
    fn capabilities(&self) -> &llm_core::Capabilities {
        &self.caps
    }
    fn turn<'a>(
        &'a self,
        request: &'a llm_core::TurnRequest,
        _: &'a mut dyn llm_core::StreamSink,
        _: &'a llm_core::Cancel,
    ) -> llm_core::BoxFuture<'a, Result<llm_core::TurnOutcome, llm_core::Error>> {
        Box::pin(async move {
            request.validate_for(&self.binding, &self.caps)?;
            let previous = request
                .items
                .iter()
                .filter(|item| matches!(item, llm_core::Item::ToolCall(_)))
                .count();
            let arguments = if request
                .instructions
                .contains("Implement the accepted AEP story")
            {
                self.implementation_calls.fetch_add(1, Ordering::SeqCst);
                let input = request
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        llm_core::Item::UserText { text } => Some(text.clone()),
                        llm_core::Item::ToolResult { output, .. } => Some(output.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(code) = previous
                    .checked_sub(1)
                    .and_then(|index| self.turns[index].1)
                    && !input.contains(code)
                {
                    self.unseen.lock().unwrap().push(code);
                }
                self.inputs.lock().unwrap().push(input);
                self.turns
                    .get(previous)
                    .map(|turn| turn.0.clone())
                    .unwrap_or_else(|| json!({"action":"finish","summary":"Script exhausted"}))
            } else if request.instructions.contains("Independently") {
                json!({"approved":true,"reason":"Trusted test output and exact diff establish the accepted answer"})
            } else {
                json!({"action":"finish","stories":["story:deliver"],"summary":"Select existing accepted scope"})
            };
            Ok(llm_core::TurnOutcome {
                stop_reason: llm_core::StopReason::ToolCalls,
                items: vec![
                    llm_core::Item::Opaque {
                        provenance: self.binding.clone(),
                        payload: json!({"retained":true}),
                    },
                    llm_core::Item::ToolCall(llm_core::ToolCall {
                        call_id: llm_core::CallId::new(format!("call-{previous}")).unwrap(),
                        name: request.tools[0].name.clone(),
                        arguments,
                    }),
                ],
                observation: llm_core::TurnObservation {
                    usage: Some(llm_core::Usage {
                        input_tokens: Some(20),
                        output_tokens: Some(10),
                        cached_input_tokens: Some(0),
                        ..Default::default()
                    }),
                    final_usage: true,
                    ..llm_core::TurnObservation::new(self.binding.clone())
                },
            })
        })
    }
}

struct RefusalRun {
    fixture: Fixture,
    provider: Arc<RefusalProvider>,
    report: control_plane_runtime::TickReport,
}

/// Plan once, then run the implementor through `turns` with the real Loom runtime,
/// Commission, checks, independent review and publication. The primary repository gains
/// a 3 MB text file and, in its last commit, a Latin-1 file.
async fn native_refusal_run(turns: Vec<Turn>, local_eval: bool) -> RefusalRun {
    let mut fixture = fixture(1).await;
    let primary = fixture.root.join("repos/repo0");
    std::fs::write(
        primary.join("huge.txt"),
        format!("{}\n", "a".repeat(99)).repeat(30_000),
    )
    .unwrap();
    cmd(&primary, "git", &["add", "huge.txt"], &[]);
    cmd(
        &primary,
        "git",
        &["commit", "-m", "large context file"],
        &[],
    );
    std::fs::write(primary.join("latin1.txt"), b"caf\xe9\n").unwrap();
    cmd(&primary, "git", &["add", "latin1.txt"], &[]);
    cmd(
        &primary,
        "git",
        &["commit", "-m", "Latin-1 context file"],
        &[],
    );
    cmd(&primary, "git", &["push", "origin", "HEAD:main"], &[]);
    if local_eval {
        fixture.config.local_eval_root = Some(fixture.root.canonicalize().unwrap());
    }
    let provider = Arc::new(RefusalProvider {
        turns,
        implementation_calls: AtomicUsize::new(0),
        unseen: Mutex::new(Vec::new()),
        inputs: Mutex::new(Vec::new()),
        binding: serde_json::from_value(json!({"protocol":"responses","provider":"fixture","account":"test","endpoint":"offline","model":"scripted","binding_revision":"one"})).unwrap(),
        caps: llm_core::Capabilities {
            tools: true,
            tool_choice: true,
            temperature: false,
            top_p: false,
            reasoning_efforts: vec![],
            // Large refused proposals (a 300 KiB write) must not trigger Loom compaction here.
            context_window: 4_000_000,
            max_output_tokens: 32000,
        },
    });
    let sessions = tempfile::tempdir().unwrap();
    let model = Arc::new(control_plane_runtime::CodexAgentModel::with_provider(
        sessions.path().into(),
        provider.clone(),
    ));
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model,
    );
    assert_eq!(supervisor.tick().await.unwrap().queued, 1);
    let report = supervisor.fleet_tick().await.unwrap();
    RefusalRun {
        fixture,
        provider,
        report,
    }
}

const FIX: &str = "pub fn answer() -> u32 { 42 }\n";

async fn assert_merged_after_every_refusal_was_seen(run: &RefusalRun) {
    assert!(
        run.provider.unseen.lock().unwrap().is_empty(),
        "refusals that did not reach the next model turn: {:?}",
        run.provider.unseen.lock().unwrap()
    );
    assert!(run.report.blockers.is_empty(), "{:?}", run.report);
    assert_eq!(
        run.fixture
            .store
            .lock()
            .await
            .query("AssignmentList")
            .unwrap()[0]["state"],
        "Merged"
    );
}

#[tokio::test]
async fn admission_refusals_reach_the_model() {
    let read = |paths: Value| json!({"action":"read","paths":paths});
    let write =
        |path: &str, contents: String| json!({"action":"write","path":path,"contents":contents});
    let admitted = (read(json!(["Cargo.toml"])), None);
    let turns = vec![
        // Loom refuses these two before the host sees them: the published schema bounds
        // reads at 32 paths, and tool arguments are bounded at 65,536 bytes.
        (
            read(json!(vec!["Cargo.toml"; 33])),
            Some("published schema"),
        ),
        (read(json!(["huge.txt"])), Some("read_too_large")),
        (read(json!(["latin1.txt"])), Some("read_not_utf8")),
        (
            read(json!(["workspace:unregistered-directory/AGENTS.md"])),
            Some("context_directory_unknown"),
        ),
        admitted.clone(),
        (
            write("tests/extra.rs", "#[test]\nfn extra() {}\n".into()),
            Some("write_outside_scope"),
        ),
        (
            write("src/tool.py", "print(1)\n".into()),
            Some("write_language_policy"),
        ),
        (
            write("src/huge.rs", "a".repeat(300 * 1024)),
            Some("65536 byte bound"),
        ),
        (write("../escape.rs", FIX.into()), Some("write_path_syntax")),
        admitted.clone(),
        (
            write(".engineering/planning/story/forged.md", "forged\n".into()),
            Some("planning_store_host_owned"),
        ),
        (
            json!({"action":"delete","path":"tests/acceptance.rs"}),
            Some("write_outside_scope"),
        ),
        (
            json!({"action":"delete","path":"src/missing.rs"}),
            Some("delete_missing"),
        ),
        (
            json!({"action":"run","program":"git","args":["push","origin","HEAD"]}),
            Some("command_not_admitted"),
        ),
        admitted,
        (
            json!({"action":"run","program":"go","args":["run","main.go"]}),
            Some("command_not_admitted"),
        ),
        (
            json!({"action":"finish","summary":"  "}),
            Some("finish_summary_missing"),
        ),
        (write("src/lib.rs", FIX.into()), None),
        (
            json!({"action":"finish","summary":"Return the requested answer"}),
            None,
        ),
    ];
    let run = native_refusal_run(turns, true).await;
    assert_merged_after_every_refusal_was_seen(&run).await;
    let worktrees = run.fixture.root.join("state");
    for name in ["escape.rs", "forged.md", "extra.rs", "tool.py"] {
        let found = Command::new("find")
            .arg(&worktrees)
            .arg(run.fixture.root.join("repos"))
            .args(["-name", name])
            .output()
            .unwrap();
        assert!(
            found.stdout.is_empty(),
            "refused write of {name} reached disk"
        );
    }
}

#[tokio::test]
async fn execution_refusals_reach_the_model() {
    let turns = vec![
        (
            json!({"action":"run","program":"git","args":["show","HEAD:huge.txt"]}),
            Some("command_output_too_large"),
        ),
        (
            json!({"action":"run","program":"git","args":["show","HEAD"]}),
            Some("command_output_not_utf8"),
        ),
        (
            json!({"action":"write","path":"src/lib.rs","contents":FIX}),
            None,
        ),
        (
            json!({"action":"finish","summary":"Return the requested answer"}),
            None,
        ),
    ];
    let run = native_refusal_run(turns, false).await;
    assert_merged_after_every_refusal_was_seen(&run).await;
}

#[tokio::test]
async fn fifth_consecutive_refusal_blocks_the_attempt() {
    let refused = (
        json!({"action":"write","path":"tests/extra.rs","contents":"#[test]\nfn extra() {}\n"}),
        Some("write_outside_scope"),
    );
    let mut turns = vec![refused.clone(); 4];
    turns.push((json!({"action":"read","paths":["Cargo.toml"]}), None));
    turns.extend(vec![refused; 5]);
    let run = native_refusal_run(turns, false).await;
    // Without the reset after the admitted read, the fifth refusal overall (call 6) would block.
    assert_eq!(run.provider.implementation_calls.load(Ordering::SeqCst), 10);
    assert!(
        run.report.blockers.iter().any(|reason| reason
            .contains("refusal budget exhausted after 5 consecutive refused actions")
            && reason.contains("write_outside_scope")),
        "{:?}",
        run.report
    );
    let store = run.fixture.store.lock().await;
    assert_eq!(
        store.query("AssignmentList").unwrap()[0]["state"],
        "Blocked"
    );
    assert_eq!(store.query("PublicationIntentList").unwrap(), json!([]));
}

#[tokio::test]
async fn host_failures_stay_fatal() {
    let turns = vec![(json!({"action":"read","paths":[".git/config"]}), None)];
    let run = native_refusal_run(turns, false).await;
    assert_eq!(run.provider.implementation_calls.load(Ordering::SeqCst), 1);
    assert!(
        run.report
            .blockers
            .iter()
            .any(|reason| reason.contains("Git administrative paths are not tool inputs")),
        "{:?}",
        run.report
    );
    assert_eq!(
        run.fixture
            .store
            .lock()
            .await
            .query("AssignmentList")
            .unwrap()[0]["state"],
        "Blocked"
    );
}

#[tokio::test]
async fn round_six_gofmt_proposal_recovers() {
    let turns = vec![
        (
            json!({"action":"run","program":"gofmt","args":["-w","cmd/server/main.go","internal/app/app.go","internal/app/app_test.go","web/assets.go"]}),
            Some("command_not_admitted"),
        ),
        (
            json!({"action":"write","path":"src/lib.rs","contents":FIX}),
            None,
        ),
        (
            json!({"action":"finish","summary":"Return the requested answer"}),
            None,
        ),
    ];
    let run = native_refusal_run(turns, false).await;
    assert_merged_after_every_refusal_was_seen(&run).await;
    assert!(
        run.provider.inputs.lock().unwrap()[1].contains("Commands that write files"),
        "the refusal did not carry the admitted command grammar"
    );
}
