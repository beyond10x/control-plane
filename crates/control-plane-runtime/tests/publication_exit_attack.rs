//! Adversarial cases for story:publication-exit, wave 4 pass 1, driven through the real fleet.
//!
//! The fixture is the one `tests/fleet.rs` builds (a bare remote per repository, a managed
//! worktree workspace and an active AEP story), kept under its own scratch directory.
use anyhow::Result;
use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, SharedStore, Supervisor};
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

struct Fixture {
    root: PathBuf,
    store: SharedStore,
    config: RuntimeConfig,
    workspace: Value,
}

/// One repository with a bare remote and one started goal, as `tests/fleet.rs` `fixture(1)`
/// builds it, except that the publish command exits without pushing (`git --version`).
async fn fixture() -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/publication-exit-attack");
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
    std::fs::write(&profile,"version = 1\nname = 'publication-exit-attack'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
    std::fs::write(repo.join("ess/domains/item.yaml"),"domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
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
        .register_workspace(&repo, "publication exit attack")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0].clone();
    host.execute("ConfigureRepository",json!({"repository_id":repository["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git --version"}),Actor::Operator).await.unwrap();
    start_goal(&mut host, &workspace).await;
    Fixture {
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        workspace,
    }
}

async fn start_goal(host: &mut Store, workspace: &Value) -> Value {
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    goal
}

/// The fleet tests' scripted model, except that a second implementation run writes the same
/// behaviour in other words, as a fresh model run does: its candidate is a new commit.
struct Scripted {
    steps: Mutex<BTreeMap<String, usize>>,
}
impl Scripted {
    fn new() -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
        }
    }
}
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => Ok(
                json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
            ),
            "implementor" => {
                let mut steps = self.steps.lock().unwrap();
                let runs = steps.len();
                let count = steps.entry(request.execution_context.clone()).or_default();
                *count += 1;
                let contents = if runs == 0 || (runs == 1 && *count > 1) {
                    "pub fn answer() -> u32 { 42 }\n"
                } else {
                    "pub fn answer() -> u32 { 40 + 2 }\n"
                };
                Ok(if *count == 1 {
                    json!({"action":"write","path":"src/lib.rs","contents":contents})
                } else {
                    json!({"action":"finish","summary":"Requested behavior implemented"})
                })
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
}

/// The fixture's configuration with no publication grace: an Uncertain intent whose target still
/// lacks its candidate closes at the next observation (correction round 1, decision F1).
fn no_grace(fixture: &Fixture) -> RuntimeConfig {
    RuntimeConfig {
        publication_grace: std::time::Duration::ZERO,
        ..fixture.config.clone()
    }
}

fn remote_main(fixture: &Fixture) -> String {
    cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &["ls-remote", "origin", "refs/heads/main"],
        &[],
    )
}

/// A publisher whose merge becomes visible on the target after it exited (a merge queue, an
/// auto-merge, a fetch mirror behind the push URL): the first candidate lands one fleet tick
/// later than `published_candidate_still_reconciles` lands it. The reviewed, tested candidate
/// of this assignment is then on its target, which is what the assignment exists to achieve.
#[tokio::test]
async fn candidate_landing_after_its_close_still_reconciles() {
    let fixture = fixture().await;
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        Arc::new(Scripted::new()),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    assert_eq!(intents.len(), 1, "{intents:?}");
    assert_eq!(intents[0]["state"], "Uncertain", "{intents:?}");
    let candidate = intents[0]["candidate"].as_str().unwrap().to_owned();

    // The next observation: the merge is not visible yet.
    supervisor.fleet_tick().await.unwrap();
    // Now it is: the first publisher's merge lands on main.
    cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &["push", "origin", &format!("{candidate}:refs/heads/main")],
        &[],
    );
    for _ in 0..2 {
        supervisor.fleet_tick().await.unwrap();
    }
    let main = remote_main(&fixture);
    assert!(main.starts_with(&candidate), "{main}");
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        assignment["state"], "Merged",
        "candidate {candidate} is on main, yet the assignment is {} ({}) with intents {intents:?}",
        assignment["state"], assignment["reason"]
    );
}

/// After an assignment's publications are closed and its attempts are spent, it stays Blocked,
/// and the store keeps refusing any other change on that repository until it is cancelled
/// (`unpublished_intent_closes_and_frees_repository`). A second workspace's assignment on the
/// same repository must then wait, not be attempted and refused on every tick.
#[tokio::test]
async fn second_workspace_waits_quietly_while_a_closed_assignment_holds_the_repository() {
    let fixture = fixture().await;
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        no_grace(&fixture),
        Arc::new(Scripted::new()),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let first = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(first["state"], "Blocked", "{first}");

    // A second workspace registers the same repository and plans its own goal on it.
    let other = fixture.root.join("second");
    std::fs::create_dir(&other).unwrap();
    let second_goal = {
        let mut store = fixture.store.lock().await;
        let workspace = store
            .execute(
                "RegisterWorkspace",
                json!({"path":other,"name":"second workspace"}),
                Actor::Operator,
            )
            .await
            .unwrap()["published"][0]["payload"]["workspace_id"]
            .clone();
        assert_ne!(workspace, fixture.workspace);
        store.execute("RegisterRepository",json!({"workspace_id":workspace,"path":fixture.root.join("repos/repo0"),"name":"repo0","common_dir":"","base_branch":"main","test_command":"cargo test --quiet","publish_command":"git --version"}),Actor::Operator).await.unwrap();
        start_goal(&mut store, &workspace).await
    };
    supervisor.tick().await.unwrap();
    let second = rows(&fixture.store, "AssignmentList")
        .await
        .into_iter()
        .find(|a| a["goal_id"] == second_goal)
        .expect("the second workspace's goal queued an assignment");
    let shared: Vec<_> = rows(&fixture.store, "RepositoryRegistrationList")
        .await
        .into_iter()
        .map(|r| r["common_dir"].clone())
        .collect();
    assert_eq!(shared.len(), 2, "{shared:?}");
    assert_eq!(shared[0], shared[1], "{shared:?}");

    // Close, retry, close again: the first assignment's attempts are spent.
    for _ in 0..2 {
        supervisor.fleet_tick().await.unwrap();
    }
    let first = rows(&fixture.store, "AssignmentList")
        .await
        .into_iter()
        .find(|a| a["assignment_id"] == first["assignment_id"])
        .unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    assert_eq!(first["state"], "Blocked", "{first}");
    assert_eq!(first["attempt"], 2, "{first}");
    assert!(
        intents.iter().all(|i| i["state"] == "NotPublished"),
        "{intents:?}"
    );

    let blocked = |history: &Value| {
        history["activity"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["assignment_id"] == second["assignment_id"] && a["action"] == "blocked")
            .count()
    };
    let history = |store: SharedStore, goal: Value| async move {
        store
            .lock()
            .await
            .activity_history(goal.as_str().unwrap())
            .unwrap()
    };
    let before = blocked(&history(fixture.store.clone(), second_goal.clone()).await);
    for _ in 0..10 {
        supervisor.fleet_tick().await.unwrap();
    }
    let after = history(fixture.store.clone(), second_goal.clone()).await;
    let now = rows(&fixture.store, "AssignmentList")
        .await
        .into_iter()
        .find(|a| a["assignment_id"] == second["assignment_id"])
        .unwrap();
    assert!(
        blocked(&after).saturating_sub(before) <= 1,
        "ten fleet ticks recorded {} blockers for the second workspace's assignment, now {} ({}); \
         last: {}",
        blocked(&after).saturating_sub(before),
        now["state"],
        now["reason"],
        after["fleet"][second["assignment_id"].as_str().unwrap()]
    );
}

/// The review's probe through the real fleet: the first workspace's goal is cancelled while its
/// publication is unresolved. One tick closes the intent, cancels the superseded assignment and
/// lets the second workspace's assignment on the same repository claim it.
#[tokio::test]
async fn cancelled_goal_frees_the_repository_for_a_second_workspace_in_one_tick() {
    let fixture = fixture().await;
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        no_grace(&fixture),
        Arc::new(Scripted::new()),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let first = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(first["state"], "Blocked", "{first}");
    let other = fixture.root.join("second");
    std::fs::create_dir(&other).unwrap();
    let second_goal = {
        let mut store = fixture.store.lock().await;
        let workspace = store
            .execute(
                "RegisterWorkspace",
                json!({"path":other,"name":"second workspace"}),
                Actor::Operator,
            )
            .await
            .unwrap()["published"][0]["payload"]["workspace_id"]
            .clone();
        store.execute("RegisterRepository",json!({"workspace_id":workspace,"path":fixture.root.join("repos/repo0"),"name":"repo0","common_dir":"","base_branch":"main","test_command":"cargo test --quiet","publish_command":"git --version"}),Actor::Operator).await.unwrap();
        start_goal(&mut store, &workspace).await
    };
    supervisor.tick().await.unwrap();
    fixture
        .store
        .lock()
        .await
        .execute(
            "CancelGoal",
            json!({"goal_id":first["goal_id"]}),
            Actor::Operator,
        )
        .await
        .unwrap();
    supervisor.fleet_tick().await.unwrap();
    let assignments = rows(&fixture.store, "AssignmentList").await;
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let first = assignments
        .iter()
        .find(|a| a["assignment_id"] == first["assignment_id"])
        .unwrap();
    let second = assignments
        .iter()
        .find(|a| a["goal_id"] == second_goal)
        .expect("the second workspace's goal queued an assignment");
    assert_eq!(first["state"], "Cancelled", "{assignments:?}");
    assert!(
        intents
            .iter()
            .any(|i| i["assignment_id"] == first["assignment_id"] && i["state"] == "NotPublished"),
        "{intents:?}"
    );
    assert_eq!(second["attempt"], 1, "{assignments:?}");
    assert!(
        intents
            .iter()
            .any(|i| i["assignment_id"] == second["assignment_id"]),
        "the second workspace's assignment did not reach publication: {assignments:?}"
    );
}
