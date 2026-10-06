//! Adversary cases for story:console-projection, driven through the public router and the
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

/// Run one command as `actor`; it must apply.
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

/// The browser's `/api/console` projection.
async fn console(state: &AppState) -> Value {
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/console")
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

fn row<'a>(view: &'a Value, kind: &str, key: &str, id: &str) -> &'a Value {
    view[kind]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row[key] == id)
        .unwrap_or_else(|| panic!("{kind} has no {id}"))
}

/// A Running goal whose roles use distinct models, in a workspace with one Git repository:
/// (goal id, goal revision, repository id).
async fn running_goal(state: &AppState, parent: &std::path::Path) -> (String, Value, String) {
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
        json!({"workspace_id":workspace,"name":"repository","path":path,"common_dir":"","base_branch":"main","test_command":"true","publish_command":"true"}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["repository_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let goal = execute(
        state,
        "CreateGoal",
        json!({"workspace_id":workspace,"objective":"attack goal","acceptance":"verified","max_workers":2,"max_attempts":2,"max_minutes":10,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"reviewer-model","merge_authority":true}),
        Actor::Operator,
    )
    .await["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    execute(state, "StartGoal", json!({"goal_id":goal}), Actor::Operator).await;
    let goals = json!({"goals":state.store.lock().await.query("GoalList").unwrap()});
    let revision = row(&goals, "goals", "goal_id", &goal)["revision"].clone();
    (goal, revision, repository)
}

/// An assignment queued by the planner at the goal's revision, as its AssignmentList row.
async fn queued(state: &AppState, goal: &str, revision: &Value, repository: &str) -> String {
    execute(
        state,
        "QueueAssignment",
        json!({"goal_id":goal,"repository_id":repository,"story_id":"story:attack","case_id":"story:attack/case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":revision}),
        Actor::Supervisor,
    )
    .await["published"][0]["payload"]["assignment_id"]
        .as_str()
        .unwrap()
        .to_owned()
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

/// One planner entry, shaped as the supervisor's planner progress path records it.
async fn planner_entry(state: &AppState, goal: &str, revision: &Value, action: &str) -> Value {
    let entry = json!({"id":uuid::Uuid::new_v4().to_string(),"action":action,"role":"planner","status":"waiting","detail":"Planning phase: Queued","at":"2026-10-06T10:00:00Z","worktree":"plan-tree","goal_revision":revision});
    execute(state, "RecordPlanningProgress", json!({"goal_id":goal,"planning_revision":revision,"planning_fingerprint":"fingerprint","planning_repository":"","planning_worktree_id":"plan-tree","planning_worktree_path":"plan-tree","planning_phase":"Queued","planning_reason":"","planning_receipt":json!({"last_activity":entry}).to_string()}), Actor::Supervisor).await;
    entry
}

/// The runtime supervisor; only its fleet progress path is used, never a model.
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

/// After the planner queues work, one implementing worker streams a model response for about
/// a minute: fleet.rs records a `loom.event` per visible streaming second (loom_model.rs
/// throttles to one per second). The planner's newest activity is still the planning entry;
/// the story's outcome is "the planner's own latest activity separate from each worker's".
#[tokio::test]
async fn planner_activity_survives_a_minute_of_worker_streaming() {
    let (temp, state) = fixture().await;
    let (goal, revision, repository) = running_goal(&state, temp.path()).await;
    let assignment = queued(&state, &goal, &revision, &repository).await;
    let planner = planner_entry(&state, &goal, &revision, "planning.Queued").await;
    execute(&state, "ClaimAssignment", json!({"assignment_id":assignment,"worktree_id":"impl-tree","implementor_run":"implementor-run","base_revision":"base"}), Actor::Supervisor).await;
    let worker = assignment_row(&state, &assignment).await;
    let fleet = supervisor(&state);
    for streamed in 1..=64 {
        fleet
            .record_progress(
                &worker,
                "loom.event",
                "runtime",
                json!({"summary":format!("Receiving model response ({streamed} streamed events)"),"event":{"kind":"model-stream","events":streamed}}),
            )
            .await
            .unwrap();
    }
    let view = console(&state).await;
    let projected = row(&view, "goals", "goal_id", &goal);
    // The worker's activity is where it belongs.
    assert_eq!(projected["fleet"][&assignment]["action"], "loom.event");
    // The planner card's activity is still the planner's own newest entry.
    assert_eq!(
        projected["planner_activity"]["id"], planner["id"],
        "planner_activity after 64 worker entries: {}",
        projected["planner_activity"]
    );
}

/// The final goal review is a model call: fleet.rs records `goal.review` (role
/// `goal_reviewer`) on the goal's merged assignment, then calls the reviewer model, whose
/// stream records `loom.event` entries, and only afterwards `goal.acceptance.completed` or
/// `blocked`. No assignment is active and no planner call is open, so without `waiting` the
/// console has nothing to say but idle while the goal waits on the reviewer model.
#[tokio::test]
async fn final_goal_review_wait_is_projected() {
    let (temp, state) = fixture().await;
    let (goal, revision, repository) = running_goal(&state, temp.path()).await;
    let assignment = queued(&state, &goal, &revision, &repository).await;
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
    let receipt = json!({"kind":"git_merge_observation","operation_id":publication,"candidate":"candidate","expected_base":"base","target":"main","observed_head":"candidate","origin":"origin","observed_at":"2026-10-06T11:22:33Z"}).to_string();
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
    assert_eq!(merged["state"], "Merged");
    let fleet = supervisor(&state);
    // fleet.rs goal acceptance: the review request, then the reviewer model's stream.
    fleet
        .record_progress(
            &merged,
            "goal.review",
            "goal_reviewer",
            json!({"execution_context":"goal-reviewer-attack"}),
        )
        .await
        .unwrap();
    fleet
        .record_progress(
            &merged,
            "loom.event",
            "runtime",
            json!({"role":"goal_reviewer","summary":"goal_reviewer model turn started","event":{"kind":"turn-started"}}),
        )
        .await
        .unwrap();
    let view = console(&state).await;
    let projected = row(&view, "goals", "goal_id", &goal);
    assert_eq!(projected["state"], "Running");
    let requested = projected["activity"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "goal.review")
        .expect("goal.review is listed")["at"]
        .clone();
    assert_eq!(
        projected["waiting"],
        json!({"role":"goal_reviewer","model":"reviewer-model","since":requested}),
        "while the goal reviewer's model call is open"
    );
}
