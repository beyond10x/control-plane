//! `control-plane-xtask eval report` states the goal's bounds: its workers, attempts and minutes,
//! and whether they are within one worker, one attempt and ten minutes. The store holds a goal
//! the operator created and started; no model runs.
use control_plane_core::{Actor, Store};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A started goal with these limits in a store under the tree's `.scratch`, closed on return.
async fn started(
    workers: i64,
    attempts: i64,
    minutes: i64,
) -> (tempfile::TempDir, PathBuf, String, PathBuf) {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/eval-verify");
    std::fs::create_dir_all(&parent).unwrap();
    let scratch = tempfile::tempdir_in(parent).unwrap();
    let repo = scratch.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "--initial-branch=main"]);
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    git(&repo, &["commit", "--allow-empty", "-m", "fixture"]);
    let state = scratch.path().join("state.sqlite");
    let mut host = Store::open(&state).await.unwrap();
    let workspace = host
        .register_workspace(&repo, "bounds fixture")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let goal = host
        .execute(
            "CreateGoal",
            json!({"workspace_id":workspace,"objective":"Bounds","acceptance":"Bounds are reported","max_workers":workers,"max_attempts":attempts,"max_minutes":minutes,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false}),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let started = host
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    assert_eq!(started["outcome"], "applied", "{started}");
    drop(host);
    (scratch, state, goal, repo)
}

fn report(state: &Path, goal: &str, repo: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_control-plane-xtask"))
        .args(["eval", "report", "--state"])
        .arg(state)
        .args(["--goal", goal, "--repo"])
        .arg(repo)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn eval_report_states_goal_bounds() {
    let (_within, state, goal, repo) = started(1, 1, 10).await;
    let stdout = report(&state, &goal, &repo);
    for line in [
        "workers: 1\n",
        "attempts: 1\n",
        "minutes: 10\n",
        "within one worker, one attempt and ten minutes: yes\n",
    ] {
        assert!(stdout.contains(line), "missing {line:?} in:\n{stdout}");
    }

    for (workers, attempts, minutes) in [(2, 1, 10), (1, 2, 10), (1, 1, 11)] {
        let (_beyond, state, goal, repo) = started(workers, attempts, minutes).await;
        let stdout = report(&state, &goal, &repo);
        for line in [
            format!("workers: {workers}\n"),
            format!("attempts: {attempts}\n"),
            format!("minutes: {minutes}\n"),
            "within one worker, one attempt and ten minutes: no\n".to_owned(),
        ] {
            assert!(stdout.contains(&line), "missing {line:?} in:\n{stdout}");
        }
    }
}
