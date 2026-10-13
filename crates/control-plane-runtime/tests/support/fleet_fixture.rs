//! The fleet fixture: registered repositories with local origins, an AEP plan holding one active
//! story, managed worktrees and a started goal, all in one root under the tree's `.scratch`.
//! Shared by `tests/fleet.rs` and, by path, by the console's runtime stream test.

use control_plane_core::{Actor, Store};
use control_plane_runtime::{RuntimeConfig, SharedStore};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

pub fn cmd(cwd: &Path, program: &str, args: &[&str], env: &[(String, String)]) -> String {
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
pub struct Fixture {
    /// Deletes the fixture root when the fixture is dropped: its repositories, remotes, store
    /// and the managed worktrees with their records (`XDG_STATE_HOME`, `XDG_CONFIG_HOME`).
    _scratch: tempfile::TempDir,
    pub root: PathBuf,
    pub store: SharedStore,
    pub config: RuntimeConfig,
    pub goal: Value,
    pub repositories: Vec<Value>,
}
pub async fn fixture(count: usize) -> Fixture {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/fleet");
    std::fs::create_dir_all(&scratch).unwrap();
    let scratch = tempfile::tempdir_in(scratch).unwrap();
    let root = scratch.path().to_path_buf();
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
        _scratch: scratch,
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        goal,
        repositories,
    }
}
