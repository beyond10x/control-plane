//! Adversary pass 2 for story:console-projection: worker model waits and the runtime that holds
//! the calls they describe. Worker activity is recorded through the runtime's own fleet progress
//! path; the projection is read through the public router.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use control_plane_app::{AppState, router, serve_with_runtime};
use control_plane_core::{Actor, Store};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, Notify};
use tower::ServiceExt;

const LISTEN: &str = "127.0.0.1:8787";

fn state_at(store: Store, listen: SocketAddr) -> AppState {
    AppState::new(Arc::new(Mutex::new(store)), listen, Arc::new(Notify::new()))
}

async fn fixture() -> (tempfile::TempDir, AppState) {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    (temp, state_at(store, LISTEN.parse().unwrap()))
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

/// The browser's `/api/console` projection, asked of the server listening at `host`.
async fn console(state: &AppState, host: &str) -> Value {
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/console")
                .header("host", host)
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

fn id(result: &Value, key: &str) -> String {
    result["published"][0]["payload"][key]
        .as_str()
        .unwrap_or_else(|| panic!("no {key} in {result}"))
        .to_owned()
}

/// A workspace registered at `parent/name`, without repository discovery.
async fn workspace(state: &AppState, parent: &std::path::Path, name: &str) -> String {
    let directory = parent.join(name);
    std::fs::create_dir(&directory).unwrap();
    let result = execute(
        state,
        "RegisterWorkspace",
        json!({"path":directory,"name":name}),
        Actor::Operator,
    )
    .await;
    id(&result, "workspace_id")
}

/// A Git repository at `directory/name`, registered in `workspace`.
async fn repository(
    state: &AppState,
    directory: &std::path::Path,
    workspace: &str,
    name: &str,
) -> String {
    let path = directory.join(name);
    assert!(
        std::process::Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );
    let result = execute(
        state,
        "RegisterRepository",
        json!({"workspace_id":workspace,"name":name,"path":path,"common_dir":"","base_branch":"main","test_command":"true","publish_command":"true"}),
        Actor::Operator,
    )
    .await;
    id(&result, "repository_id")
}

/// A started goal whose roles use distinct models: (goal id, revision).
async fn running_goal(state: &AppState, workspace: &str) -> (String, Value) {
    let created = execute(
        state,
        "CreateGoal",
        json!({"workspace_id":workspace,"objective":"pass 2 goal","acceptance":"verified","max_workers":2,"max_attempts":2,"max_minutes":10,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"reviewer-model","merge_authority":true}),
        Actor::Operator,
    )
    .await;
    let goal = id(&created, "goal_id");
    execute(state, "StartGoal", json!({"goal_id":goal}), Actor::Operator).await;
    let goals = json!({"goals":state.store.lock().await.query("GoalList").unwrap()});
    let revision = row(&goals, "goals", "goal_id", &goal)["revision"].clone();
    (goal, revision)
}

/// An assignment queued at the goal's revision and claimed by an implementor, as its
/// AssignmentList row: what fleet.rs passes to its progress path.
async fn implementing(
    state: &AppState,
    goal: &str,
    revision: &Value,
    repository: &str,
    story: &str,
) -> Value {
    let queued = execute(
        state,
        "QueueAssignment",
        json!({"goal_id":goal,"repository_id":repository,"story_id":story,"case_id":format!("{story}/case"),"worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":revision}),
        Actor::Supervisor,
    )
    .await;
    let assignment = id(&queued, "assignment_id");
    execute(
        state,
        "ClaimAssignment",
        json!({"assignment_id":assignment,"worktree_id":format!("tree-{story}"),"implementor_run":format!("run-{story}"),"base_revision":"base"}),
        Actor::Supervisor,
    )
    .await;
    let rows = json!({"assignments":state.store.lock().await.query("AssignmentList").unwrap()});
    let claimed = row(&rows, "assignments", "assignment_id", &assignment).clone();
    assert_eq!(claimed["state"], "Implementing");
    claimed
}

struct Unused;
impl control_plane_runtime::AgentModel for Unused {
    fn respond(&self, _: &control_plane_runtime::ModelRequest) -> anyhow::Result<Value> {
        anyhow::bail!("these cases never reach a model call")
    }
}

/// The runtime supervisor; only its fleet progress path is used, never a model.
fn supervisor(state: &AppState) -> control_plane_runtime::Supervisor {
    control_plane_runtime::Supervisor::new(
        state.store.clone(),
        Arc::new(Notify::new()),
        control_plane_runtime::RuntimeConfig::default(),
        Arc::new(Unused),
    )
}

/// One streamed model event, as loom_model.rs reports it (at most one per visible second)
/// and fleet.rs records it: action `loom.event`, role `runtime`.
fn streamed(events: u64) -> Value {
    json!({"summary":format!("Receiving model response ({events} streamed events)"),"event":{"kind":"model-stream","events":events}})
}

/// Two workers of one goal (max_workers 2, one per repository) stream their implementor calls
/// at the same time, each recording one `loom.event` per second. After about half a minute,
/// worker A's `model.request` has left the goal's 64 retained entries, and A's `waiting` loses
/// its start time although A's call is still open. The acceptance (`model_wait_is_projected`)
/// says the projection carries the wait "with role, model and start time" until completion.
#[tokio::test]
async fn worker_wait_keeps_its_start_while_two_workers_stream() {
    let (temp, state) = fixture().await;
    let ws = workspace(&state, temp.path(), "pair").await;
    let first = repository(&state, &temp.path().join("pair"), &ws, "first").await;
    let second = repository(&state, &temp.path().join("pair"), &ws, "second").await;
    let (goal, revision) = running_goal(&state, &ws).await;
    let a = implementing(&state, &goal, &revision, &first, "story:first").await;
    let b = implementing(&state, &goal, &revision, &second, "story:second").await;
    let a_id = a["assignment_id"].as_str().unwrap().to_owned();
    let fleet = supervisor(&state);
    fleet
        .record_progress(
            &a,
            "model.request",
            "implementor",
            json!({"execution_context":"run-a"}),
        )
        .await
        .unwrap();
    // The request's own recorded time, read from the assignment's newest entry.
    let view = console(&state, LISTEN).await;
    let newest = &row(&view, "goals", "goal_id", &goal)["fleet"][&a_id];
    assert_eq!(newest["action"], "model.request");
    let requested = newest["at"].clone();
    assert!(requested.is_string());
    fleet
        .record_progress(
            &b,
            "model.request",
            "implementor",
            json!({"execution_context":"run-b"}),
        )
        .await
        .unwrap();
    for events in 1..=32 {
        for worker in [&a, &b] {
            fleet
                .record_progress(worker, "loom.event", "runtime", streamed(events))
                .await
                .unwrap();
        }
    }
    let view = console(&state, LISTEN).await;
    let shown = row(&view, "assignments", "assignment_id", &a_id);
    assert_eq!(shown["state"], "Implementing");
    assert_eq!(
        shown["waiting"],
        json!({"role":"implementor","model":"implementor-model","since":requested}),
        "worker A after 32 streamed events per worker"
    );
}

/// serve_with_runtime keeps the console up after a fatal runtime error ("Autonomous processing
/// stopped"). An implementor call recorded as open when the runtime stopped is not open: the
/// runtime that would hold it has returned, and nothing records its end until a restart. Here
/// the runtime stops on another goal whose stored planning receipt is not JSON (the planner's
/// `record` refuses it); a store write failing inside the worker's own call (fleet.rs
/// `worker.block(..)?`) stops it the same way, leaving the same recorded state.
#[tokio::test]
async fn no_model_wait_once_the_runtime_has_stopped() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let host = address.to_string();
    let state = state_at(store, address);
    let ws = workspace(&state, temp.path(), "worked").await;
    let repo = repository(&state, &temp.path().join("worked"), &ws, "repository").await;
    let (goal, revision) = running_goal(&state, &ws).await;
    let worker = implementing(&state, &goal, &revision, &repo, "story:worked").await;
    let assignment = worker["assignment_id"].as_str().unwrap().to_owned();
    supervisor(&state)
        .record_progress(
            &worker,
            "model.request",
            "implementor",
            json!({"execution_context":"run-worked"}),
        )
        .await
        .unwrap();
    let before = console(&state, &host).await;
    assert_eq!(
        row(&before, "assignments", "assignment_id", &assignment)["waiting"]["role"],
        "implementor"
    );
    // A second goal, in a workspace without repositories, whose receipt the planner cannot
    // read: its planning record fails and the supervisor's tick returns the error.
    let other = workspace(&state, temp.path(), "unreadable").await;
    let (broken, broken_revision) = running_goal(&state, &other).await;
    execute(
        &state,
        "RecordPlanningProgress",
        json!({"goal_id":broken,"planning_revision":broken_revision,"planning_fingerprint":"fingerprint","planning_repository":"","planning_worktree_id":"","planning_worktree_path":"","planning_reason":"","planning_receipt":"not-json","planning_phase":"Planning"}),
        Actor::Supervisor,
    )
    .await;
    let shutdown = tokio_util::sync::CancellationToken::new();
    let server = tokio::spawn(serve_with_runtime(
        listener,
        state.clone(),
        control_plane_runtime::RuntimeConfig {
            poll_interval: Duration::from_secs(3600),
            ..control_plane_runtime::RuntimeConfig::default()
        },
        Arc::new(Unused),
        shutdown.clone(),
    ));
    let stopped = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let view = console(&state, &host).await;
            if view["runtime_error"].is_string() {
                break view;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    shutdown.cancel();
    tokio::time::timeout(Duration::from_secs(10), server)
        .await
        .expect("service did not shut down")
        .unwrap()
        .unwrap();
    let view = stopped.expect("the runtime did not stop");
    let shown = row(&view, "assignments", "assignment_id", &assignment);
    assert_eq!(shown["state"], "Implementing");
    assert_eq!(
        shown["waiting"],
        Value::Null,
        "runtime_error: {}",
        view["runtime_error"]
    );
}

/// A model call does not survive its process: the Store is opened by one process at a time,
/// and the runtime runs inside it. After a restart, the implementor call recorded as open by
/// the previous process is still projected as open until the first fleet tick blocks the
/// assignment, which runs only after the planning tick over every Running goal.
#[tokio::test]
async fn a_wait_recorded_before_a_restart_is_not_open_after_it() {
    let (temp, state) = fixture().await;
    let ws = workspace(&state, temp.path(), "restarted").await;
    let repo = repository(&state, &temp.path().join("restarted"), &ws, "repository").await;
    let (goal, revision) = running_goal(&state, &ws).await;
    let worker = implementing(&state, &goal, &revision, &repo, "story:restarted").await;
    let assignment = worker["assignment_id"].as_str().unwrap().to_owned();
    supervisor(&state)
        .record_progress(
            &worker,
            "model.request",
            "implementor",
            json!({"execution_context":"run-restarted"}),
        )
        .await
        .unwrap();
    let before = console(&state, LISTEN).await;
    assert_eq!(
        row(&before, "assignments", "assignment_id", &assignment)["waiting"]["role"],
        "implementor"
    );
    drop(state);
    let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let state = state_at(reopened, LISTEN.parse().unwrap());
    let view = console(&state, LISTEN).await;
    let shown = row(&view, "assignments", "assignment_id", &assignment);
    assert_eq!(shown["state"], "Implementing");
    assert_eq!(shown["waiting"], Value::Null);
}
