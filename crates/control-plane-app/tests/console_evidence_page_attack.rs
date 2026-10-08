//! Adversary cases for story:console-evidence-page, driven through the public router and the
//! store, with worker activity recorded through the runtime's own fleet progress path.
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
        matches!(result["outcome"].as_str(), Some("applied" | "created")),
        "{command}: {result}"
    );
    result
}

async fn get(
    state: &AppState,
    uri: &str,
    headers: &[(&str, &str)],
) -> (StatusCode, String, String) {
    let mut request = Request::builder().uri(uri).header("host", "127.0.0.1:8787");
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = router(state.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        content_type,
        String::from_utf8(bytes.to_vec()).unwrap(),
    )
}

async fn evidence(state: &AppState, goal: &str) -> Value {
    let (status, _, body) = get(state, &format!("/api/goals/{goal}/evidence"), &[]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    serde_json::from_str(&body).unwrap()
}

fn supervisor(state: &AppState) -> control_plane_runtime::Supervisor {
    struct Unused;
    impl control_plane_runtime::AgentModel for Unused {
        fn respond(&self, _: &control_plane_runtime::ModelRequest) -> anyhow::Result<Value> {
            anyhow::bail!("progress recording does not call the model")
        }
    }
    control_plane_runtime::Supervisor::new(
        state.store.clone(),
        Arc::new(Notify::new()),
        control_plane_runtime::RuntimeConfig::default(),
        Arc::new(Unused),
    )
}

async fn assignment_row(state: &AppState, assignment: &str) -> Value {
    let rows = state.store.lock().await.query("AssignmentList").unwrap();
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["assignment_id"] == assignment)
        .unwrap()
        .clone()
}

/// A running goal with one queued assignment: (goal id, assignment id).
async fn running_assignment(state: &AppState, parent: &std::path::Path) -> (String, String) {
    let directory = parent.join("attack");
    std::fs::create_dir(&directory).unwrap();
    let workspace = execute(
        state,
        "RegisterWorkspace",
        json!({"path":directory,"name":"attack"}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = directory.join("repository");
    assert!(
        std::process::Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );
    let repository = execute(
        state,
        "RegisterRepository",
        json!({"workspace_id":workspace,"name":"repository","path":path,"common_dir":"","base_branch":"main","test_command":"task check","publish_command":"true"}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["repository_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let goal = execute(
        state,
        "CreateGoal",
        json!({"workspace_id":workspace,"objective":"attack goal","acceptance":"verified","max_workers":1,"max_attempts":2,"max_minutes":10,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"reviewer-model","merge_authority":true}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    execute(state, "StartGoal", json!({"goal_id":goal}), Actor::Operator).await;
    let goals = state.store.lock().await.query("GoalList").unwrap();
    let revision = goals
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["goal_id"] == goal.as_str())
        .unwrap()["revision"]
        .clone();
    let assignment = execute(
        state,
        "QueueAssignment",
        json!({"goal_id":goal,"repository_id":repository,"story_id":"story:attack","case_id":"story:attack/case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":revision}),
        Actor::Supervisor,
    )
    .await["published"][0]["payload"]["assignment_id"]
        .as_str()
        .unwrap()
        .to_owned();
    (goal, assignment)
}

/// fleet.rs: `checks.run` (fleet.rs:1287) is recorded before the checks, then the reviewer
/// is asked (`review.request`, fleet.rs:1357) and the reviewer model streams; fleet.rs:78
/// records one `loom.event` per visible streamed second (loom_model.rs throttles to one per
/// second). A review that streams for about a minute is ordinary. The story's view must show
/// "the latest check command and the candidate it ran on" from the evidence JSON.
#[tokio::test]
async fn adversary_latest_check_survives_a_minute_of_review_streaming() {
    let (temp, state) = fixture().await;
    let (goal, assignment) = running_assignment(&state, temp.path()).await;
    execute(&state, "ClaimAssignment", json!({"assignment_id":assignment,"worktree_id":"impl-tree","implementor_run":"implementor-run","base_revision":"base"}), Actor::Supervisor).await;
    let fleet = supervisor(&state);
    let worker = assignment_row(&state, &assignment).await;
    fleet
        .record_progress(
            &worker,
            "checks.run",
            "host",
            json!({"candidate":"cand1234","command":"task check"}),
        )
        .await
        .unwrap();
    execute(
        &state,
        "ReviewAssignment",
        json!({"assignment_id":assignment,"candidate":"cand1234","test_revision":"cand1234"}),
        Actor::Supervisor,
    )
    .await;
    let reviewing = assignment_row(&state, &assignment).await;
    fleet
        .record_progress(
            &reviewing,
            "review.request",
            "reviewer",
            json!({"execution_context":"reviewer-run"}),
        )
        .await
        .unwrap();
    for streamed in 1..=64 {
        fleet
            .record_progress(
                &reviewing,
                "loom.event",
                "runtime",
                json!({"summary":format!("Receiving model response ({streamed} streamed events)"),"event":{"kind":"model-stream","events":streamed}}),
            )
            .await
            .unwrap();
    }
    execute(&state, "ReadyAssignment", json!({"assignment_id":assignment,"reviewer_run":"reviewer-run","review_revision":"cand1234"}), Actor::Supervisor).await;

    let body = evidence(&state, &goal).await;
    // Re-pinned (correction 1, F1): evidence.js `testedCandidates` reads the tested candidate
    // from the assignment's durable `test_revision`, not from the bounded history. The view case
    // `evidence_view_shows_the_tested_candidate_after_the_history_drops_its_check` renders it.
    let tested: Vec<_> = body["assignments"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["assignment_id"] == assignment.as_str())
        .map(|row| row["test_revision"].clone())
        .collect();
    assert_eq!(
        tested,
        vec![json!("cand1234")],
        "the checks ran on cand1234, yet the evidence JSON the view reads does not carry the \
         tested candidate after a minute of review streaming"
    );
}

/// The Vue fixture `frontend/src/fixtures/evidence-merged.json` is hand-written. Every field
/// the view reads must exist in the server's real JSON for a merged assignment, with the same
/// JSON type, or the view is tested against a shape the server never sends.
#[tokio::test]
async fn adversary_merged_fixture_matches_the_server_shape() {
    let (temp, state) = fixture().await;
    let (goal, assignment) = running_assignment(&state, temp.path()).await;
    let with = |mut body: Value| {
        body["assignment_id"] = json!(assignment);
        body
    };
    for (command, body) in [
        (
            "ClaimAssignment",
            json!({"worktree_id":"impl-tree","implementor_run":"implementor-run","base_revision":"base"}),
        ),
        (
            "ReviewAssignment",
            json!({"candidate":"candidate","test_revision":"candidate"}),
        ),
        (
            "ReadyAssignment",
            json!({"reviewer_run":"reviewer-run","review_revision":"candidate"}),
        ),
    ] {
        execute(&state, command, with(body), Actor::Supervisor).await;
    }
    let prepared = execute(
        &state,
        "PreparePublication",
        with(json!({"candidate":"candidate","target":"main","expected_base":"base"})),
        Actor::Supervisor,
    )
    .await;
    let publication = prepared["published"][0]["payload"]["publication_id"].clone();
    execute(
        &state,
        "MergeAssignment",
        with(json!({})),
        Actor::Supervisor,
    )
    .await;
    let receipt = json!({"kind":"git_merge_observation","operation_id":publication,"candidate":"candidate","expected_base":"base","target":"main","observed_head":"head","origin":"origin","observed_at":"2026-10-06T11:22:33Z"}).to_string();
    execute(
        &state,
        "ConfirmPublication",
        json!({"publication_id":publication,"receipt":receipt}),
        Actor::Supervisor,
    )
    .await;
    execute(
        &state,
        "CompleteAssignment",
        with(json!({"merge_receipt":receipt})),
        Actor::Supervisor,
    )
    .await;
    let merged = assignment_row(&state, &assignment).await;
    supervisor(&state)
        .record_progress(
            &merged,
            "merge.completed",
            "host",
            json!({"candidate":"candidate"}),
        )
        .await
        .unwrap();

    let real = evidence(&state, &goal).await;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../frontend/src/fixtures/evidence-merged.json"
    ))
    .unwrap();
    let kind = |value: &Value| match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };
    let mut drift = Vec::new();
    for pointer in [
        "/goal/goal_id",
        "/goal/objective",
        "/goal/state",
        "/goal/satisfaction_receipt",
        "/history/activity",
        "/history/activity/0/id",
        "/history/activity/0/at",
        "/history/activity/0/action",
        "/history/activity/0/role",
        "/history/activity/0/detail",
        "/assignments/0/assignment_id",
        "/assignments/0/story_id",
        "/assignments/0/state",
        "/assignments/0/candidate",
        "/assignments/0/reviewer_run",
        "/assignments/0/review_revision",
        "/assignments/0/merge_receipt",
    ] {
        let (r, f) = (real.pointer(pointer), fixture.pointer(pointer));
        match (r, f) {
            (Some(r), Some(f)) if kind(r) == kind(f) => {}
            _ => drift.push(format!("{pointer}: real {r:?} fixture {f:?}")),
        }
    }
    let real_receipt: Value =
        serde_json::from_str(real["assignments"][0]["merge_receipt"].as_str().unwrap()).unwrap();
    for key in ["observed_head", "target", "candidate"] {
        if !real_receipt[key].is_string() {
            drift.push(format!("merge_receipt.{key} missing from the real receipt"));
        }
    }
    assert!(drift.is_empty(), "fixture drift: {drift:#?}");
}

/// The API moved under `/api/`: the same host, origin and fetch-site guard must refuse a
/// cross-site read of the JSON, and a missing goal answers JSON the view can show.
#[tokio::test]
async fn adversary_api_path_keeps_the_guard_and_answers_a_missing_goal() {
    let (_temp, state) = fixture().await;
    let (status, _, _) = get(
        &state,
        "/api/goals/nope/evidence",
        &[("sec-fetch-site", "cross-site")],
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = get(
        &state,
        "/api/goals/nope/evidence",
        &[("origin", "http://evil.example")],
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, content_type, body) = get(&state, "/api/goals/nope/evidence", &[]).await;
    assert!(status.is_client_error(), "{status}");
    assert!(
        content_type.starts_with("application/json"),
        "{content_type}"
    );
    assert!(serde_json::from_str::<Value>(&body).unwrap()["error"].is_string());
    let (status, content_type, _) = get(&state, "/goals/nope/evidence", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/html"));
}

/// main.js decodes the console path's id with `decodeURIComponent`; the server serves the
/// console for any id, including one with a bare `%`. Whether the shell is served at all
/// decides whether the frontend case for a bare `%` is reachable.
#[tokio::test]
async fn adversary_console_is_served_for_a_bare_percent_id() {
    let (_temp, state) = fixture().await;
    let (status, content_type, _) = get(&state, "/goals/50%/evidence", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/html"), "{content_type}");
}
