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
