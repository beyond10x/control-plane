//! `control-plane-xtask eval report`: whether a recorded goal ran unattended, read from the
//! store a scripted-model run leaves behind, against a disposable repository and its origin.
use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, Supervisor};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
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

/// The planner finishes on the fixture's story, every reviewer approves, and the implementor
/// writes the requested answer once.
struct Scripted {
    implementor_steps: Mutex<std::collections::BTreeMap<String, usize>>,
    calls: AtomicUsize,
}
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => Ok(
                json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
            ),
            "implementor" => {
                let mut steps = self.implementor_steps.lock().unwrap();
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

/// A recorded run: the state store the run left behind, its goal and the repository it merged into.
struct Run {
    state: PathBuf,
    goal: String,
    repo: PathBuf,
    candidate: String,
    base: String,
}

/// One disposable repository with a local origin, one goal started by the operator, and
/// `between` run as the operator after StartGoal before the scripted fleet takes the goal to
/// Satisfied. The store is closed when this returns, as after a stopped service.
async fn recorded_run(between: &[&str]) -> Run {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/unattended-run-report");
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
    std::fs::write(&profile, "version = 1\nname = 'unattended-run-report'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
        "[package]\nname = 'report_fixture'\nversion = '0.1.0'\nedition = '2024'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
    std::fs::write(
        repo.join("tests/acceptance.rs"),
        "#[test]\nfn requested_answer() { assert_eq!(report_fixture::answer(), 42); }\n",
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

    let state = root.join("state.sqlite");
    let mut host = Store::open(&state).await.unwrap();
    let workspace = host
        .register_workspace(&repo, "report fixture")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0]["repository_id"].clone();
    host.execute("ConfigureRepository", json!({"repository_id":repository,"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git push --force-with-lease=refs/heads/{target}:{expected_base} origin {candidate}:refs/heads/{target}"}), Actor::Operator).await.unwrap();
    let goal = host.execute("CreateGoal", json!({"workspace_id":workspace,"objective":"Return 42","acceptance":"requested_answer passes on the reviewed merged target","max_workers":1,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}), Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    for command in between {
        let outcome = host
            .execute(command, json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        assert_eq!(outcome["outcome"], "applied", "{command}: {outcome}");
    }
    let store = Arc::new(tokio::sync::Mutex::new(host));
    let model = Arc::new(Scripted {
        implementor_steps: Mutex::default(),
        calls: AtomicUsize::new(0),
    });
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let (goal_row, assignment) = {
        let store = store.lock().await;
        (
            store.query("GoalList").unwrap()[0].clone(),
            store.query("AssignmentList").unwrap()[0].clone(),
        )
    };
    assert_eq!(goal_row["state"], "Satisfied", "{goal_row}");
    assert_eq!(assignment["state"], "Merged", "{assignment}");
    drop(supervisor);
    drop(
        Arc::try_unwrap(store)
            .ok()
            .expect("the store is closed after the run"),
    );
    Run {
        state,
        goal,
        repo,
        candidate: assignment["candidate"].as_str().unwrap().to_owned(),
        base: assignment["base_revision"].as_str().unwrap().to_owned(),
    }
}

fn report(run: &Run) -> Output {
    Command::new(env!("CARGO_BIN_EXE_control-plane-xtask"))
        .args(["eval", "report", "--state"])
        .arg(&run.state)
        .args(["--goal", &run.goal, "--repo"])
        .arg(&run.repo)
        .output()
        .unwrap()
}

fn text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Every file of the store, with its bytes, so a report that wrote to the store is caught.
fn store_files(run: &Run) -> Vec<(PathBuf, Vec<u8>)> {
    let directory = run.state.parent().unwrap();
    let mut files = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[tokio::test(flavor = "multi_thread")]
async fn unattended_run_report_counts_operator_commands() {
    // The operator pauses and resumes the running goal: two Operator commands after StartGoal.
    let run = recorded_run(&["PauseGoal", "StartGoal"]).await;
    let output = report(&run);
    let (stdout, stderr) = text(&output);
    assert!(!output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("operator commands after start: 2"),
        "{stdout}"
    );
    assert!(stdout.contains("unattended: no"), "{stdout}");
    assert!(stderr.contains("2 Operator command"), "{stderr}");
    // The goal is otherwise clean, so the count is the only reason named.
    assert!(!stderr.contains("not on the target"), "{stderr}");
    assert!(!stderr.contains("not Satisfied"), "{stderr}");
}

#[tokio::test(flavor = "multi_thread")]
async fn unattended_run_report_requires_a_merged_commit_on_target() {
    let run = recorded_run(&[]).await;
    // Rewind the origin's target past the merge: the goal stays Satisfied in the store, but
    // its merged commit is no longer on the target branch.
    cmd(
        &run.repo,
        "git",
        &[
            "push",
            "--force",
            "origin",
            &format!("{}:refs/heads/main", run.base),
        ],
        &[],
    );
    let output = report(&run);
    let (stdout, stderr) = text(&output);
    assert!(!output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("state: Satisfied"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "merged commit: {} on origin/main: no",
            run.candidate
        )),
        "{stdout}"
    );
    assert!(stdout.contains("unattended: no"), "{stdout}");
    assert!(
        stderr.contains(&format!("{} is not on the target", run.candidate)),
        "{stderr}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unattended_run_report_passes_a_clean_run() {
    let run = recorded_run(&[]).await;

    // A service that owns the store is not interrupted: the report refuses and names it.
    let held = Store::open(&run.state).await.unwrap();
    let output = report(&run);
    let (stdout, stderr) = text(&output);
    assert!(!output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.contains("service owns this store"), "{stderr}");
    drop(held);

    let before = store_files(&run);
    let output = report(&run);
    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains(&format!("goal: {}", run.goal)), "{stdout}");
    assert!(stdout.contains("state: Satisfied"), "{stdout}");
    assert!(stdout.contains("receipt revision: 1"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "merged commit: {} on origin/main: yes",
            run.candidate
        )),
        "{stdout}"
    );
    assert!(stdout.contains("elapsed: "), "{stdout}");
    assert!(
        stdout.contains("operator commands after start: 0"),
        "{stdout}"
    );
    assert!(stdout.contains("unattended: yes"), "{stdout}");
    assert_eq!(store_files(&run), before, "the report wrote to the store");

    // A missing store is refused, not created.
    let missing = Run {
        state: run.state.with_file_name("absent.sqlite"),
        goal: run.goal.clone(),
        repo: run.repo.clone(),
        candidate: run.candidate.clone(),
        base: run.base.clone(),
    };
    assert!(!report(&missing).status.success());
    assert!(!missing.state.exists());
    assert!(!missing.state.with_extension("lock").exists());
}
