//! Adversary pass 2 for story:console-evidence-page: the `test_revision` semantics the view's
//! "tests passed on candidate" line rests on, driven through the real store and the public router.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use control_plane_app::{AppState, router};
use control_plane_core::{Actor, Store};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tower::ServiceExt;

async fn fixture() -> (tempfile::TempDir, AppState) {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    (
        temp,
        AppState::new(
            Arc::new(Mutex::new(store)),
            "127.0.0.1:8787".parse().unwrap(),
            Arc::new(Notify::new()),
        ),
    )
}

async fn execute(state: &AppState, command: &str, body: Value, actor: Actor) -> Value {
    let result = state
        .store
        .lock()
        .await
        .execute(command, body, actor)
        .await
        .unwrap();
    assert!(
        matches!(
            result["outcome"].as_str(),
            Some("applied" | "created" | "rebased")
        ),
        "{command}: {result}"
    );
    result
}

async fn evidence(state: &AppState, goal: &str) -> Value {
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/goals/{goal}/evidence"))
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn goal_with_assignments(
    state: &AppState,
    parent: &std::path::Path,
    n: usize,
) -> (String, Vec<String>) {
    let directory = parent.join("pass2");
    std::fs::create_dir(&directory).unwrap();
    let workspace = execute(
        state,
        "RegisterWorkspace",
        json!({"path":directory,"name":"pass2"}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["workspace_id"]
        .clone();
    // One repository per assignment: the store admits one active change per repository.
    let mut repositories = Vec::new();
    for index in 0..n {
        let path = directory.join(format!("repository-{index}"));
        assert!(
            std::process::Command::new("git")
                .args(["init", "--initial-branch=main"])
                .arg(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
        repositories.push(execute(state, "RegisterRepository", json!({"workspace_id":workspace,"name":format!("repository-{index}"),"path":path,"common_dir":"","base_branch":"main","test_command":"task check","publish_command":"true"}), Actor::Operator).await["published"][0]["payload"]["repository_id"].clone());
    }
    let goal = execute(state, "CreateGoal", json!({"workspace_id":workspace,"objective":"pass2","acceptance":"verified","max_workers":2,"max_attempts":3,"max_minutes":10,"planner_model":"p","implementor_model":"i","reviewer_model":"r","merge_authority":true}), Actor::Operator).await["published"][0]["payload"]["goal_id"].as_str().unwrap().to_owned();
    execute(state, "StartGoal", json!({"goal_id":goal}), Actor::Operator).await;
    let mut ids = Vec::new();
    for (index, repository) in repositories.iter().enumerate().take(n) {
        ids.push(
            execute(state, "QueueAssignment", json!({"goal_id":goal,"repository_id":repository,"story_id":format!("story:{index}"),"case_id":format!("story:{index}/case"),"worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":1}), Actor::Supervisor).await["published"][0]["payload"]["assignment_id"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    (goal, ids)
}

fn test_revisions(body: &Value) -> Vec<(String, String, String)> {
    body["assignments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["story_id"].as_str().unwrap().to_owned(),
                row["state"].as_str().unwrap().to_owned(),
                row["test_revision"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// The view prints "tests passed on candidate <test_revision>" for every assignment with a
/// non-empty test_revision. That claim holds only if test_revision is set solely by
/// ReviewAssignment (after the checks pass, fleet.rs) and cleared by every repair, so that a
/// repaired assignment never shows its earlier candidate as the passed one.
#[tokio::test]
async fn adversary_test_revision_is_cleared_by_repair_and_never_set_without_review() {
    let (temp, state) = fixture().await;
    let (goal, ids) = goal_with_assignments(&state, temp.path(), 3).await;
    let claim = |id: &str| json!({"assignment_id":id,"worktree_id":"tree","implementor_run":"implementor","base_revision":"base"});
    // story:0 passes checks on c1, the reviewer rejects, it is repaired (rebased), and blocked.
    execute(&state, "ClaimAssignment", claim(&ids[0]), Actor::Supervisor).await;
    execute(
        &state,
        "ReviewAssignment",
        json!({"assignment_id":ids[0],"candidate":"c1","test_revision":"c1"}),
        Actor::Supervisor,
    )
    .await;
    execute(&state, "RepairAssignment", json!({"assignment_id":ids[0],"reason":"rejected","implementor_run":"implementor-2","base_revision":"base-2"}), Actor::Supervisor).await;
    execute(
        &state,
        "BlockAssignment",
        json!({"assignment_id":ids[0],"reason":"checks failed"}),
        Actor::Supervisor,
    )
    .await;
    // story:1 passes checks on c2 and is blocked while reviewing: c2 did pass.
    execute(&state, "ClaimAssignment", claim(&ids[1]), Actor::Supervisor).await;
    execute(
        &state,
        "ReviewAssignment",
        json!({"assignment_id":ids[1],"candidate":"c2","test_revision":"c2"}),
        Actor::Supervisor,
    )
    .await;
    execute(
        &state,
        "BlockAssignment",
        json!({"assignment_id":ids[1],"reason":"reviewer unavailable"}),
        Actor::Supervisor,
    )
    .await;
    // story:2 is claimed and its checks fail: never reviewed.
    execute(&state, "ClaimAssignment", claim(&ids[2]), Actor::Supervisor).await;
    execute(
        &state,
        "BlockAssignment",
        json!({"assignment_id":ids[2],"reason":"checks failed"}),
        Actor::Supervisor,
    )
    .await;
    // A mismatched test revision is refused, so no candidate is recorded as tested on another.
    let retried = state
        .store
        .lock()
        .await
        .execute(
            "RepairAssignment",
            json!({"assignment_id":ids[2],"reason":"retry","implementor_run":"implementor-3"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(retried["outcome"], "applied", "{retried}");
    let mismatch = state
        .store
        .lock()
        .await
        .execute(
            "ReviewAssignment",
            json!({"assignment_id":ids[2],"candidate":"c3","test_revision":"c-old"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_ne!(mismatch["outcome"], "applied", "{mismatch}");

    let mut rows = test_revisions(&evidence(&state, &goal).await);
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("story:0".into(), "Blocked".into(), String::new()),
            ("story:1".into(), "Blocked".into(), "c2".into()),
            ("story:2".into(), "Implementing".into(), String::new()),
        ]
    );
}
