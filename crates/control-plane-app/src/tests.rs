use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn dashboard_embeds_vue_assets_without_refresh_or_iframe() {
    let (_temp, state) = fixture().await;
    let app = router(state);
    let request = |path| {
        Request::builder()
            .uri(path)
            .header("host", "127.0.0.1:8787")
            .body(Body::empty())
            .unwrap()
    };
    let shell = app.clone().oneshot(request("/")).await.unwrap();
    let policy = shell.headers()["content-security-policy"].to_str().unwrap();
    assert!(policy.contains("script-src 'self'"));
    assert!(policy.contains("connect-src 'self'"));
    assert!(policy.contains("frame-ancestors 'self'"));
    assert_eq!(shell.headers()["x-frame-options"], "SAMEORIGIN");
    let shell = body(shell).await;
    assert!(shell.contains("src=\"/app.js\""));
    assert!(shell.contains("href=\"/index.css\""));
    assert!(!shell.contains("<iframe"));
    assert!(!shell.contains("http-equiv=\"refresh\""));
    assert!(shell.contains("action=\"/workspaces\""));
    let live = app.oneshot(request("/live")).await.unwrap();
    assert_eq!(live.status(), StatusCode::OK);
    let live = body(live).await;
    assert!(!live.contains("http-equiv=\"refresh\""));
    assert!(live.contains("Server observed"));
    assert!(live.contains("No running goals"));
    assert!(live.contains("src=\"/app.js\""));
}

#[tokio::test]
async fn live_activity_is_durable_scoped_and_does_not_inline_model_receipts() {
    let (temp, state) = fixture().await;
    let mut identities = Vec::new();
    for name in ["visible-workspace", "other-workspace"] {
        let path = temp.path().join(name);
        std::fs::create_dir(&path).unwrap();
        let created = state
            .add_workspace(WorkspaceInput {
                path: path.to_string_lossy().into_owned(),
                name: name.into(),
            })
            .await
            .unwrap();
        let ws = created["published"][0]["payload"]["workspace_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let created = state.command("CreateGoal",json!({"workspace_id":ws,"objective":name,"acceptance":"verified","max_workers":2,"max_attempts":3,"max_minutes":10,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"review-model","merge_authority":false})).await.unwrap();
        let id = created["published"][0]["payload"]["goal_id"]
            .as_str()
            .unwrap()
            .to_owned();
        state
            .command("StartGoal", json!({"goal_id":id}))
            .await
            .unwrap();
        let event = json!({"at":"2026-10-05T09:00:00Z","action":format!("Model waiting {name}"),"role":"planner","worktree":"isolated/plan","detail":"<script>unsafe</script>","status":"running"});
        let receipt=json!({"last_activity":event,"activity":[event],"receipt":"MODEL-RAW-SECRET".repeat(40000)}).to_string();
        state.store.lock().await.execute("RecordPlanningProgress",json!({"goal_id":id,"planning_revision":1,"planning_fingerprint":"fingerprint","planning_repository":"","planning_worktree_id":"plan","planning_worktree_path":"isolated/plan","planning_phase":"Planning","planning_reason":"Model request in flight","planning_receipt":receipt}),Actor::Supervisor).await.unwrap();
        identities.push((ws, id));
    }
    drop(state);
    let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let state = AppState::new(
        Arc::new(Mutex::new(reopened)),
        "127.0.0.1:8787".parse().unwrap(),
        Arc::new(Notify::new()),
    );
    let app = router(state);
    let request = |path: String| {
        Request::builder()
            .uri(path)
            .header("host", "127.0.0.1:8787")
            .body(Body::empty())
            .unwrap()
    };
    let html = body(
        app.clone()
            .oneshot(request(format!("/workspaces/{}/live", identities[0].0)))
            .await
            .unwrap(),
    )
    .await;
    assert!(html.contains("Model waiting visible-workspace"));
    assert!(html.contains("2026-10-05T09:00:00Z"));
    assert!(html.contains("since observation"));
    assert!(html.contains("&lt;script&gt;unsafe&lt;/script&gt;"));
    assert!(!html.contains("Model waiting other-workspace"));
    assert!(!html.contains("MODEL-RAW-SECRET"));
    assert!(html.len() < 30000);
    let evidence = body(
        app.oneshot(request(format!("/goals/{}/evidence", identities[0].1)))
            .await
            .unwrap(),
    )
    .await;
    assert!(evidence.contains("MODEL-RAW-SECRET"));
    assert!(!evidence.contains("other-workspace"));
}

#[test]
fn concurrent_workers_and_blocked_stopped_states_are_distinct() {
    let event = |action| json!({"at":"2026-10-05T09:00:00Z","action":action,"role":"implementor","worktree":"worktree","detail":"actual tool observation","status":"running","goal_revision":1});
    let receipt=json!({"fleet":{"a":event("worker one running"),"b":event("worker two running"),"foreign":event("must not leak")}}).to_string();
    let mut view = json!({"server_observed_at":"2026-10-05T10:00:00Z","runtime_error":"Executor unavailable","repositories":[],"goals":[{"goal_id":"g","revision":1,"state":"Running","planning_phase":"Blocked","planning_reason":"Repository unavailable","planning_receipt":receipt}],"assignments":[{"goal_id":"g","goal_revision":1,"assignment_id":"a","story_id":"first","state":"Implementing"},{"goal_id":"g","goal_revision":1,"assignment_id":"b","story_id":"second","state":"Reviewing"}]});
    let html = dashboard::operations(&view).unwrap();
    assert!(html.contains("worker one running"));
    assert!(html.contains("worker two running"));
    assert!(!html.contains("must not leak"));
    assert!(html.contains("Autonomous processing stopped"));
    assert!(html.contains("Planner needs attention"));
    assert!(html.contains("Repository unavailable"));
    assert!(html.contains("initial snapshot is static"));
    assert!(!html.contains("Connected"));
    view["goals"][0]["state"] = json!("Paused");
    let paused = dashboard::operations(&view).unwrap();
    assert!(!paused.contains("worker one running"));
    assert!(paused.contains("Paused; Start this goal to resume"));
    view["goals"][0]["state"] = json!("Running");
    view["goals"][0]["revision"] = json!(2);
    assert!(
        !dashboard::operations(&view)
            .unwrap()
            .contains("worker one running")
    );
}

#[test]
fn structured_worker_details_show_bounded_commands_and_paths_without_receipts() {
    let event = json!({"at":"2026-10-05T09:00:00Z","action":"tool.run","role":"implementor","status":"running","goal_revision":1,"detail":{"program":"cargo","args":["test","--package","<script>bad</script>"],"path":"src/lib.rs","worktree":"isolated/worker","receipt":"PRIVATE-RECEIPT".repeat(50000),"stdout":"PRIVATE-OUTPUT"}});
    let receipt = json!({"last_activity":event,"activity":[event]}).to_string();
    let mut view = json!({"server_observed_at":"2026-10-05T10:00:00Z","runtime_error":null,"repositories":[],"assignments":[],"goals":[{"goal_id":"g","revision":1,"state":"Running","planning_phase":"Planning","planning_receipt":receipt}]});
    let html = dashboard::operations(&view).unwrap();
    assert!(html.contains("cargo test --package &lt;script&gt;bad&lt;/script&gt;"));
    assert!(html.contains("src/lib.rs"));
    assert!(html.contains("isolated/worker"));
    assert!(!html.contains("PRIVATE-RECEIPT"));
    assert!(!html.contains("PRIVATE-OUTPUT"));
    assert!(!html.contains("<script>"));
    assert!(html.len() < 30000);
    let receipt=json!({"last_activity":{"action":"blocked","detail":{"reason":"Required checks refused","command":"cargo test --locked"}}}).to_string();
    view["goals"][0]["planning_receipt"] = json!(receipt);
    let html = dashboard::operations(&view).unwrap();
    assert!(html.contains("Required checks refused"));
    assert!(html.contains("cargo test --locked"));
}

#[tokio::test]
async fn adversary_recorded_tool_run_keeps_the_command_the_dashboard_shows() {
    // The case above shows what the dashboard renders for a structured tool run, from a
    // hand-built view. This sends the same kind of event through the runtime's progress path.
    use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, Supervisor};
    struct NoModel;
    impl AgentModel for NoModel {
        fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
            anyhow::bail!("progress recording does not call the model")
        }
    }
    let (temp, state) = fixture().await;
    let path = temp.path().join("tool-run");
    std::fs::create_dir(&path).unwrap();
    let created = state
        .add_workspace(WorkspaceInput {
            path: path.to_string_lossy().into_owned(),
            name: "tool-run".into(),
        })
        .await
        .unwrap();
    let ws = created["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let created = state.command("CreateGoal",json!({"workspace_id":ws,"objective":"tool run","acceptance":"verified","max_workers":1,"max_attempts":1,"max_minutes":10,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false})).await.unwrap();
    let id = created["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    state
        .command("StartGoal", json!({"goal_id":id}))
        .await
        .unwrap();
    // The detail fleet records for an admitted `cargo test` over the members of a large
    // workspace: `{"program":program,"args":args}`, here just over 1 KiB.
    let mut args = vec![json!("test"), json!("--locked")];
    for index in 0..40 {
        args.push(json!("--package"));
        args.push(json!(format!("workspace-member-{index:02}")));
    }
    let detail = json!({"program":"cargo","args":args});
    assert!(detail.to_string().len() > 1024);
    let shown = "cargo test --locked --package workspace-member-00";
    let event = json!({"at":"2026-10-06T09:00:00Z","action":"tool.run","role":"implementor","status":"running","goal_revision":1,"detail":detail});
    let hand_built = json!({"server_observed_at":"2026-10-06T09:00:01Z","runtime_error":null,"repositories":[],"assignments":[],"goals":[{"goal_id":"g","revision":1,"state":"Running","planning_phase":"Queued","planning_receipt":json!({"last_activity":event}).to_string()}]});
    assert!(
        dashboard::operations(&hand_built).unwrap().contains(shown),
        "control: the dashboard shows the command of this event"
    );
    let supervisor = Supervisor::new(
        state.store.clone(),
        Arc::new(Notify::new()),
        RuntimeConfig::default(),
        Arc::new(NoModel),
    );
    let assignment = json!({"goal_id":id,"assignment_id":uuid::Uuid::new_v4().to_string(),"goal_revision":1,"worktree_id":"cp-impl-tool-run"});
    supervisor
        .record_progress(&assignment, "tool.run", "implementor", detail)
        .await
        .unwrap();
    let view = state.snapshot().await.unwrap();
    let html = dashboard::operations(&view).unwrap();
    let receipt = view["goals"][0]["planning_receipt"].as_str().unwrap();
    assert!(
        html.contains(shown),
        "the recorded tool run lost its arguments; recorded detail: {}",
        serde_json::from_str::<Value>(receipt).unwrap()["last_activity"]["detail"]
    );
}

#[tokio::test]
async fn dashboard_and_evidence_read_activity_history_from_the_store() {
    let (temp, state) = fixture().await;
    let ws = state
        .store
        .lock()
        .await
        .execute(
            "RegisterWorkspace",
            json!({"path":temp.path(),"name":"history"}),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let created = state.command("CreateGoal",json!({"workspace_id":ws,"objective":"history","acceptance":"verified","max_workers":1,"max_attempts":1,"max_minutes":10,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false})).await.unwrap();
    let id = created["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    state
        .command("StartGoal", json!({"goal_id":id}))
        .await
        .unwrap();
    for (index, step) in ["first step", "second step", "third step"]
        .into_iter()
        .enumerate()
    {
        state.store.lock().await.record_activity(&id, json!({"id":index.to_string(),"assignment_id":"a","goal_revision":1,"at":format!("2026-10-06T09:00:0{index}Z"),"action":"tool.run","role":"implementor","status":"running","detail":step})).await.unwrap();
    }
    let mut view = state.snapshot().await.unwrap();
    // The recorded receipt carries the newest activity only.
    assert!(!field(&view["goals"][0], "planning_receipt").contains("first step"));
    dashboard::attach_history(&*state.store.lock().await, &mut view["goals"]).unwrap();
    let html = dashboard::operations(&view).unwrap();
    for step in ["first step", "second step", "third step"] {
        assert!(
            html.contains(step),
            "{step} missing from the activity history"
        );
    }
    let evidence: Value = serde_json::from_str(
        &body(
            router(state.clone())
                .oneshot(
                    Request::builder()
                        .uri(format!("/goals/{id}/evidence"))
                        .header("host", "127.0.0.1:8787")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await,
    )
    .unwrap();
    let activity = evidence["history"]["activity"].as_array().unwrap();
    assert_eq!(activity.len(), 3);
    assert_eq!(activity[0]["detail"], "first step");
    assert_eq!(evidence["history"]["fleet"]["a"]["detail"], "third step");
}

/// A running goal in a fresh workspace, for the adversary cases below.
async fn adversary_running_goal(state: &AppState, path: &std::path::Path) -> String {
    let ws = state
        .store
        .lock()
        .await
        .execute(
            "RegisterWorkspace",
            json!({"path":path,"name":"adversary"}),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let created = state.command("CreateGoal",json!({"workspace_id":ws,"objective":"adversary","acceptance":"verified","max_workers":1,"max_attempts":1,"max_minutes":10,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false})).await.unwrap();
    let id = created["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap()
        .to_owned();
    state
        .command("StartGoal", json!({"goal_id":id}))
        .await
        .unwrap();
    id
}

fn adversary_step(index: usize) -> Value {
    json!({"id":format!("step-{index:02}"),"assignment_id":"a","goal_revision":1,"at":format!("2026-10-06T09:{index:02}:00Z"),"action":"tool.run","role":"implementor","status":"running","detail":format!("step {index:02}")})
}

#[tokio::test]
async fn adversary_evidence_shows_one_state_of_the_goal() {
    // GET /goals/{id}/evidence takes the goal (and its attached activity_history) from
    // AppState::snapshot under one store lock, then takes the lock again for "history"
    // (dashboard.rs:31). A progress decision committed between the two is in "history" and
    // in neither the goal's recorded receipt nor its attached history.
    let (temp, state) = fixture().await;
    let id = adversary_running_goal(&state, temp.path()).await;
    state
        .store
        .lock()
        .await
        .record_activity(&id, adversary_step(0))
        .await
        .unwrap();
    let held = state.store.lock().await;
    let request = tokio::spawn(
        router(state.clone()).oneshot(
            Request::builder()
                .uri(format!("/goals/{id}/evidence"))
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        ),
    );
    // The evidence request now waits for the store; a progress decision queues behind it.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let writer = {
        let state = state.clone();
        let id = id.clone();
        tokio::spawn(async move {
            state
                .store
                .lock()
                .await
                .record_activity(&id, adversary_step(1))
                .await
                .unwrap();
        })
    };
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    drop(held);
    writer.await.unwrap();
    let evidence: Value =
        serde_json::from_str(&body(request.await.unwrap().unwrap()).await).unwrap();
    let recorded: Value =
        serde_json::from_str(evidence["goal"]["planning_receipt"].as_str().unwrap()).unwrap();
    let attached = evidence["goal"]["activity_history"]["activity"]
        .as_array()
        .unwrap()
        .last()
        .cloned();
    let history = evidence["history"]["activity"]
        .as_array()
        .unwrap()
        .last()
        .cloned();
    assert_eq!(
        (history.clone(), attached.clone()),
        (
            Some(recorded["last_activity"].clone()),
            Some(recorded["last_activity"].clone())
        ),
        "one evidence response: history ends with {}, the goal's attached history with {}, \
         its recorded receipt with {}",
        history.unwrap_or_default()["detail"],
        attached.unwrap_or_default()["detail"],
        recorded["last_activity"]["detail"]
    );
}

#[tokio::test]
async fn adversary_state_api_and_dashboard_keep_history_after_restart() {
    // console_activity_survives_bounding covers the SSE projection after a restart; /api/state
    // and the server-rendered dashboard read the same history through AppState::snapshot.
    let (temp, state) = fixture().await;
    let id = adversary_running_goal(&state, temp.path()).await;
    for index in 0..30 {
        state
            .store
            .lock()
            .await
            .record_activity(&id, adversary_step(index))
            .await
            .unwrap();
    }
    drop(state);
    let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let state = AppState::new(
        Arc::new(Mutex::new(reopened)),
        "127.0.0.1:8787".parse().unwrap(),
        Arc::new(Notify::new()),
    );
    let get = |path: String| {
        router(state.clone()).oneshot(
            Request::builder()
                .uri(path)
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
    };
    let api: Value =
        serde_json::from_str(&body(get("/api/state".into()).await.unwrap()).await).unwrap();
    let goal = api["goals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|goal| goal["goal_id"] == id.as_str())
        .unwrap();
    assert_eq!(
        goal["activity_history"]["activity"],
        json!((0..30).map(adversary_step).collect::<Vec<_>>())
    );
    assert_eq!(goal["activity_history"]["fleet"]["a"], adversary_step(29));
    let html = body(get("/".into()).await.unwrap()).await;
    for index in [10, 20, 29] {
        assert!(
            html.contains(&format!("step {index:02}")),
            "step {index:02} missing from the dashboard after restart"
        );
    }
}

async fn fixture() -> (tempfile::TempDir, AppState) {
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/control-plane-console");
    std::fs::create_dir_all(&root).unwrap();
    let temp = tempfile::tempdir_in(root).unwrap();
    let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let state = AppState::new(
        Arc::new(Mutex::new(store)),
        "127.0.0.1:8787".parse().unwrap(),
        Arc::new(Notify::new()),
    );
    (temp, state)
}

#[tokio::test]
async fn local_operator_controls() {
    let (_temp, state) = fixture().await;
    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/")
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    // Native same-origin POST forms need their origin preserved by the document policy.
    // no-referrer makes Chrome send Origin:null, which the security boundary must refuse.
    assert_eq!(response.headers()["referrer-policy"], "same-origin");
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Add workspace"));
}

#[tokio::test]
async fn cross_origin_mutation_refused() {
    let (_temp, state) = fixture().await;
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspaces")
                .header("host", "127.0.0.1:8787")
                .header("origin", "https://attacker.invalid")
                .header("x-csrf-token", &state.csrf)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn console_and_api_share_state() {
    let (temp, state) = fixture().await;
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspaces")
                .header("host", "127.0.0.1:8787")
                .header("x-csrf-token", &state.csrf)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"path":temp.path(),"name":"shared-workspace"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        state.store.lock().await.query("WorkspaceList").unwrap()[0]["name"],
        "shared-workspace"
    );
}

async fn body(response: Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn form_and_json_mutations_require_csrf_and_never_select_supervisor() {
    let (temp, state) = fixture().await;
    for (uri, content_type, payload) in [
        (
            "/api/workspaces",
            "application/json",
            json!({"path":temp.path(),"name":"forged"}).to_string(),
        ),
        (
            "/workspaces",
            "application/x-www-form-urlencoded",
            "path=.&name=forged".into(),
        ),
    ] {
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("host", "127.0.0.1:8787")
                    .header("content-type", content_type)
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "missing CSRF on {uri}"
        );
    }
    for command in [
        "QueueAssignment",
        "ClaimAssignment",
        "SatisfyGoal",
        "ConfirmPublication",
        "RecordPlanningProgress",
    ] {
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/commands/{command}"))
                    .header("host", "127.0.0.1:8787")
                    .header("x-csrf-token", &state.csrf)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/commands/CreateGoal")
                .header("host", "127.0.0.1:8787")
                .header("x-csrf-token", &state.csrf)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"actor":"Supervisor"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(
        state
            .store
            .lock()
            .await
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn host_and_origin_rebinding_are_refused_even_with_valid_token() {
    let (_temp, state) = fixture().await;
    for (host, origin) in [
        ("attacker.invalid:8787", "http://attacker.invalid:8787"),
        ("127.0.0.1:8787", "http://127.0.0.1:9000"),
        ("127.0.0.1:8787", "null"),
    ] {
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/session")
                    .header("host", host)
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(!body(response).await.contains(&state.csrf));
    }
}

#[tokio::test]
async fn user_content_is_escaped_in_console() {
    let (temp, state) = fixture().await;
    state
        .add_workspace(WorkspaceInput {
            path: temp.path().to_string_lossy().into_owned(),
            name: "<script>alert('x')</script>\"".into(),
        })
        .await
        .unwrap();
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/")
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'self'")
    );
    let html = body(response).await;
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;&quot;"));
}

#[tokio::test]
async fn cli_browser_and_store_share_goal_controls_over_real_http() {
    use clap::Parser;
    let (temp, state) = fixture().await;
    let repo = temp.path().join("repository");
    assert!(
        std::process::Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(&repo)
            .output()
            .unwrap()
            .status
            .success()
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = AppState::new(state.store.clone(), address, state.wake.clone());
    let server = tokio::spawn(serve(listener, app));
    let url = format!("http://{address}");
    let created = run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "workspace",
        "add",
        temp.path().to_str().unwrap(),
        "--name",
        "actual-cli-workspace",
    ])
    .unwrap())
    .await
    .unwrap()
    .unwrap();
    let ws = created["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let created = run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "goal",
        "create",
        ws,
        "--objective",
        "Deliver first change",
        "--acceptance",
        "All acceptance tests pass",
        "--allow-merges",
    ])
    .unwrap())
    .await
    .unwrap()
    .unwrap();
    let goal = created["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap();
    for action in ["start", "pause"] {
        run(Cli::try_parse_from(["control-plane", "--url", &url, "goal", action, goal]).unwrap())
            .await
            .unwrap();
    }
    run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "goal",
        "edit",
        goal,
        "--max-workers",
        "2",
        "--merge-authority",
        "false",
    ])
    .unwrap())
    .await
    .unwrap();
    let snapshot = Client::new(&url).unwrap().snapshot().await.unwrap();
    assert_eq!(snapshot["goals"][0]["state"], "Paused");
    assert_eq!(snapshot["goals"][0]["max_workers"], 2);
    assert_eq!(snapshot["goals"][0]["merge_authority"], false);
    assert_eq!(snapshot["goals"][0]["planner_model"], "gpt-5.6-sol");
    let html = reqwest::get(format!("{url}/workspaces/{ws}"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(html.contains("Deliver first change"));
    assert!(html.contains("Save repository settings"));
    assert!(html.contains("Allow verified merges"));
    assert!(html.contains("repository"));
    run(Cli::try_parse_from(["control-plane", "--url", &url, "goal", "cancel", goal]).unwrap())
        .await
        .unwrap();
    assert_eq!(
        state.store.lock().await.query("GoalList").unwrap()[0]["state"],
        "Cancelled"
    );
    tokio::time::timeout(std::time::Duration::from_secs(1), state.wake.notified())
        .await
        .unwrap();
    server.abort();
    let _ = server.await;
}

#[test]
fn cli_refuses_nonlocal_and_credential_bearing_service_urls() {
    for url in [
        "https://127.0.0.1:8787",
        "http://example.org:8787",
        "http://user@127.0.0.1:8787",
        "http://127.0.0.1:8787/api",
    ] {
        assert!(Client::new(url).is_err());
    }
    assert!(Client::new("http://127.0.0.1:8787").is_ok());
}

#[tokio::test]
async fn browser_forms_apply_goal_settings_and_controls() {
    let (temp, state) = fixture().await;
    let result = state
        .add_workspace(WorkspaceInput {
            path: temp.path().to_string_lossy().into_owned(),
            name: "browser".into(),
        })
        .await
        .unwrap();
    let ws = result["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let form = format!(
        "csrf={}&workspace_id={ws}&objective=Browser+goal&acceptance=Tests+pass&max_workers=2&max_attempts=4&max_minutes=30&planner_model=planner&implementor_model=builder&reviewer_model=reviewer&merge_authority=true",
        state.csrf
    );
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/goals")
                .header("host", "127.0.0.1:8787")
                .header("origin", "http://127.0.0.1:8787")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let goals = state.store.lock().await.query("GoalList").unwrap();
    let goal = goals[0]["goal_id"].as_str().unwrap();
    assert_eq!(goals[0]["max_workers"], 2);
    assert_eq!(goals[0]["max_attempts"], 4);
    assert_eq!(goals[0]["max_minutes"], 30);
    assert_eq!(goals[0]["merge_authority"], true);
    assert_eq!(goals[0]["reviewer_model"], "reviewer");
    for action in ["start", "pause", "cancel"] {
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/goals/{goal}/{action}"))
                    .header("host", "127.0.0.1:8787")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(format!("csrf={}", state.csrf)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }
    assert_eq!(
        state.store.lock().await.query("GoalList").unwrap()[0]["state"],
        "Cancelled"
    );
}

#[tokio::test]
async fn serve_refuses_non_loopback_before_opening_state() {
    use clap::Parser;
    let error =
        run(Cli::try_parse_from(["control-plane", "serve", "--listen", "0.0.0.0:8787"]).unwrap())
            .await
            .unwrap_err();
    assert!(error.to_string().contains("loopback"));
}

#[tokio::test]
async fn workspace_directory_api_lists_adds_and_isolates_multiple_workspaces() {
    let (temp, state) = fixture().await;
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    let context = temp.path().join("context");
    for path in [&one, &two, &context] {
        std::fs::create_dir_all(path).unwrap();
    }
    let first = state
        .add_workspace(WorkspaceInput {
            path: one.to_string_lossy().into_owned(),
            name: "one".into(),
        })
        .await
        .unwrap();
    let second = state
        .add_workspace(WorkspaceInput {
            path: two.to_string_lossy().into_owned(),
            name: "two".into(),
        })
        .await
        .unwrap();
    let first = first["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let second = second["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/workspaces")
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&body(response).await).unwrap()["workspaces"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{first}/directories"))
                .header("host", "127.0.0.1:8787")
                .header("x-csrf-token", &state.csrf)
                .header("content-type", "application/json")
                .body(Body::from(json!({"path":context}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let dir=serde_json::from_str::<Value>(&body(response).await).unwrap()["published"][0]["payload"]["directory_id"].as_str().unwrap().to_owned();
    let wrong = router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{second}/directories/{dir}"))
                .header("host", "127.0.0.1:8787")
                .header("x-csrf-token", &state.csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(!wrong.status().is_success());
    for (ws, count) in [(first, 2), (second, 1)] {
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/api/workspaces/{ws}/directories"))
                    .header("host", "127.0.0.1:8787")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<Value>(&body(response).await).unwrap()["directories"]
                .as_array()
                .unwrap()
                .len(),
            count
        );
    }
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{first}/directories/{dir}"))
                .header("host", "127.0.0.1:8787")
                .header("x-csrf-token", &state.csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let detail = router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/workspaces/{first}"))
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&body(detail).await).unwrap()["directories"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn startup_registers_current_directory_once() {
    let (temp, state) = fixture().await;
    let other = temp.path().join("other");
    std::fs::create_dir_all(&other).unwrap();
    let mut store = state.store.lock().await;
    initialize_workspaces(&mut store, temp.path(), &[])
        .await
        .unwrap();
    initialize_workspaces(&mut store, temp.path(), &[])
        .await
        .unwrap();
    assert_eq!(
        store
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        store
            .query("GoalList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    initialize_workspaces(&mut store, temp.path(), std::slice::from_ref(&other))
        .await
        .unwrap();
    assert_eq!(
        store
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        store
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["path"] == other.to_str().unwrap())
    );
}

#[tokio::test]
async fn directory_cli_and_browser_share_membership_over_real_http() {
    use clap::Parser;
    let (temp, state) = fixture().await;
    let root = temp.path().join("primary");
    let context = temp.path().join("context <&>");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&context).unwrap();
    let result = state
        .add_workspace(WorkspaceInput {
            path: root.to_string_lossy().into_owned(),
            name: "cli".into(),
        })
        .await
        .unwrap();
    let workspace = result["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = AppState::new(state.store.clone(), address, state.wake.clone());
    let token = app.csrf.clone();
    let server = tokio::spawn(serve(listener, app));
    let url = format!("http://{address}");
    let created = run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "workspace",
        "add-directory",
        workspace,
        context.to_str().unwrap(),
    ])
    .unwrap())
    .await
    .unwrap()
    .unwrap();
    let directory = created["published"][0]["payload"]["directory_id"]
        .as_str()
        .unwrap();
    let listed = run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "workspace",
        "directories",
        workspace,
    ])
    .unwrap())
    .await
    .unwrap()
    .unwrap();
    assert_eq!(listed["directories"].as_array().unwrap().len(), 2);
    let http = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let page = http
        .get(format!("{url}/workspaces/{workspace}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(page.contains("context &lt;&amp;&gt;"));
    assert!(!page.contains("context <&>"));
    assert!(page.contains("Add directory"));
    let removed = http
        .post(format!(
            "{url}/workspaces/{workspace}/directories/{directory}/remove"
        ))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!("csrf={token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), StatusCode::SEE_OTHER);
    let listed = Client::new(&url)
        .unwrap()
        .directories(workspace)
        .await
        .unwrap();
    assert_eq!(listed["directories"].as_array().unwrap().len(), 1);
    let created = Client::new(&url)
        .unwrap()
        .add_directory(workspace, &context)
        .await
        .unwrap();
    let second = created["published"][0]["payload"]["directory_id"]
        .as_str()
        .unwrap();
    assert_ne!(second, directory);
    run(Cli::try_parse_from([
        "control-plane",
        "--url",
        &url,
        "workspace",
        "remove-directory",
        workspace,
        second,
    ])
    .unwrap())
    .await
    .unwrap();
    assert_eq!(
        Client::new(&url)
            .unwrap()
            .directories(workspace)
            .await
            .unwrap()["directories"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    server.abort();
}

#[tokio::test]
async fn explicit_startup_paths_do_not_register_cwd_or_grant_authority() {
    let (temp, state) = fixture().await;
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let mut store = state.store.lock().await;
    initialize_workspaces(
        &mut store,
        temp.path(),
        &[first.clone(), second.clone(), first.clone()],
    )
    .await
    .unwrap();
    let workspaces = store.query("WorkspaceList").unwrap();
    assert_eq!(workspaces.as_array().unwrap().len(), 2);
    assert!(
        workspaces
            .as_array()
            .unwrap()
            .iter()
            .all(|w| w["path"] == first.to_str().unwrap() || w["path"] == second.to_str().unwrap())
    );
    assert_eq!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        store
            .query("GoalList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn service_start_wakes_real_supervisor_and_exposes_durable_planning_reason() {
    use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig};
    struct UnusedModel;
    impl AgentModel for UnusedModel {
        fn respond(&self, _request: &ModelRequest) -> anyhow::Result<Value> {
            anyhow::bail!("empty repository inventory must not call the model")
        }
    }
    let (temp, state) = fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = AppState::new(state.store.clone(), address, state.wake.clone());
    let shutdown = tokio_util::sync::CancellationToken::new();
    let server = tokio::spawn(serve_with_runtime(
        listener,
        app,
        RuntimeConfig {
            poll_interval: std::time::Duration::from_secs(3600),
            ..RuntimeConfig::default()
        },
        Arc::new(UnusedModel),
        shutdown.clone(),
    ));
    let client = Client::new(&format!("http://{address}")).unwrap();
    let workspace = client.add_workspace(temp.path(), "runtime").await.unwrap();
    let workspace = workspace["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    let make = || json!({"workspace_id":workspace,"objective":"User objective","acceptance":"Verified result","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"gpt-5.6-sol","implementor_model":"gpt-5.6-sol","reviewer_model":"gpt-5.6-sol","merge_authority":false});
    let goal = client.command("CreateGoal", make()).await.unwrap();
    let goal = goal["published"][0]["payload"]["goal_id"].as_str().unwrap();
    let cancelled = client.command("CreateGoal", make()).await.unwrap();
    let cancelled = cancelled["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap();
    client
        .command("CancelGoal", json!({"goal_id":cancelled}))
        .await
        .unwrap();
    let before = client.snapshot().await.unwrap();
    assert!(
        before["goals"]
            .as_array()
            .unwrap()
            .iter()
            .all(|g| g["planning_phase"] == "Idle")
    );
    client
        .command("StartGoal", json!({"goal_id":goal}))
        .await
        .unwrap();
    let progressed = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let view = client.snapshot().await.unwrap();
            let row = view["goals"]
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["goal_id"] == goal)
                .unwrap();
            if !field(row, "planning_reason").is_empty() {
                break view;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await;
    shutdown.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(3), server)
        .await
        .expect("service did not shut down")
        .unwrap()
        .unwrap();
    let view = progressed.expect("started goal stayed idle: service did not run the supervisor");
    let row = view["goals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["goal_id"] == goal)
        .unwrap();
    assert_eq!(row["state"], "Running");
    assert_eq!(row["merge_authority"], false);
    assert!(field(row, "planning_reason").contains("No ready story selected"));
    assert_eq!(
        view["goals"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["goal_id"] == cancelled)
            .unwrap()["state"],
        "Cancelled"
    );
    let html = web::workspace(
        State(AppState::new(
            state.store.clone(),
            address,
            state.wake.clone(),
        )),
        Path(workspace.to_owned()),
    )
    .await;
    assert!(body(html).await.contains("No ready story selected"));
}

#[tokio::test]
async fn service_persists_repository_tool_refusal_and_keeps_operator_controls_available() {
    use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig};
    struct UnexpectedModel;
    impl AgentModel for UnexpectedModel {
        fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
            anyhow::bail!("repository checks must happen before model execution")
        }
    }
    let (temp, state) = fixture().await;
    let repository = temp.path().join("repository");
    assert!(
        std::process::Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(&repository)
            .output()
            .unwrap()
            .status
            .success()
    );
    let registered = state
        .add_workspace(WorkspaceInput {
            path: repository.to_string_lossy().into_owned(),
            name: "offline repository".into(),
        })
        .await
        .unwrap();
    let workspace = registered["published"][0]["payload"]["workspace_id"]
        .as_str()
        .unwrap();
    std::fs::rename(&repository, temp.path().join("disconnected")).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = AppState::new(state.store.clone(), address, state.wake.clone());
    let shutdown = tokio_util::sync::CancellationToken::new();
    let server = tokio::spawn(serve_with_runtime(
        listener,
        app,
        RuntimeConfig::default(),
        Arc::new(UnexpectedModel),
        shutdown.clone(),
    ));
    let client = Client::new(&format!("http://{address}")).unwrap();
    let result=client.command("CreateGoal",json!({"workspace_id":workspace,"objective":"User objective","acceptance":"Verified result","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"gpt-5.6-sol","implementor_model":"gpt-5.6-sol","reviewer_model":"gpt-5.6-sol","merge_authority":false})).await.unwrap();
    let goal = result["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap();
    client
        .command("StartGoal", json!({"goal_id":goal}))
        .await
        .unwrap();
    let blocked = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let view = client.snapshot().await.unwrap();
            if view["goals"][0]["planning_phase"] == "Blocked" {
                break view["goals"][0].clone();
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await;
    client
        .command("CancelGoal", json!({"goal_id":goal}))
        .await
        .unwrap();
    shutdown.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let blocked = blocked.expect("repository tool failure was not persisted");
    assert!(!field(&blocked, "planning_reason").is_empty());
    assert_eq!(
        state.store.lock().await.query("GoalList").unwrap()[0]["state"],
        "Cancelled"
    );
    assert!(
        state
            .store
            .lock()
            .await
            .query("AssignmentList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn restored_service_preserves_inactive_goals_while_resuming_running_goal() {
    use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig};
    struct NoRepositoryModel;
    impl AgentModel for NoRepositoryModel {
        fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
            anyhow::bail!("a workspace with no repositories must not invoke a model")
        }
    }
    let scratch =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/app-runtime-review");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
    let inactive = temp.path().join("inactive");
    let active = temp.path().join("active");
    std::fs::create_dir_all(&inactive).unwrap();
    std::fs::create_dir_all(&active).unwrap();
    // A bare fixture has no working repository inventory and prevents Git from
    // discovering the enclosing review checkout through these scratch paths.
    for path in [&inactive, &active] {
        assert!(
            std::process::Command::new("git")
                .args(["init", "--bare"])
                .arg(path)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let database = temp.path().join("host.sqlite");

    let mut store = Store::open(&database).await.unwrap();
    let ws_inactive = store
        .register_workspace(&inactive, "inactive")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let ws_active = store.register_workspace(&active, "active").await.unwrap()["published"][0]["payload"]["workspace_id"].clone();
    let make_goal = |workspace: Value| json!({"workspace_id":workspace,"objective":"Preserve explicit operator lifecycle","acceptance":"No unapproved work starts","max_workers":1,"max_attempts":1,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false});
    let paused = store
        .execute(
            "CreateGoal",
            make_goal(ws_inactive.clone()),
            Actor::Operator,
        )
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .clone();
    store
        .execute("StartGoal", json!({"goal_id":paused}), Actor::Operator)
        .await
        .unwrap();
    store
        .execute("PauseGoal", json!({"goal_id":paused}), Actor::Operator)
        .await
        .unwrap();
    let cancelled = store
        .execute("CreateGoal", make_goal(ws_inactive), Actor::Operator)
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .clone();
    store
        .execute("CancelGoal", json!({"goal_id":cancelled}), Actor::Operator)
        .await
        .unwrap();
    let running = store
        .execute("CreateGoal", make_goal(ws_active), Actor::Operator)
        .await
        .unwrap()["published"][0]["payload"]["goal_id"]
        .clone();
    store
        .execute("StartGoal", json!({"goal_id":running}), Actor::Operator)
        .await
        .unwrap();
    drop(store);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let shared = Arc::new(Mutex::new(Store::open(&database).await.unwrap()));
    let state = AppState::new(shared.clone(), address, Arc::new(Notify::new()));
    let shutdown = tokio_util::sync::CancellationToken::new();
    let server = tokio::spawn(serve_with_runtime(
        listener,
        state,
        RuntimeConfig {
            poll_interval: std::time::Duration::from_secs(3600),
            ..RuntimeConfig::default()
        },
        Arc::new(NoRepositoryModel),
        shutdown.clone(),
    ));
    let client = Client::new(&format!("http://{address}")).unwrap();
    let progressed = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let view = client.snapshot().await.unwrap();
            if view["goals"]
                .as_array()
                .unwrap()
                .iter()
                .any(|g| g["goal_id"] == running && g["planning_phase"] == "Queued")
            {
                break view;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await;
    shutdown.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(3), server)
        .await
        .expect("graceful shutdown stalled")
        .unwrap()
        .unwrap();
    let view = progressed.expect("persisted running goal did not resume on service startup");
    assert!(view["runtime_error"].is_null());
    let goals = view["goals"].as_array().unwrap();
    for (id, expected) in [(&paused, "Paused"), (&cancelled, "Cancelled")] {
        let row = goals.iter().find(|row| row["goal_id"] == *id).unwrap();
        assert_eq!(row["state"], expected);
        assert_eq!(
            row["planning_phase"], "Idle",
            "inactive goal was processed on restart"
        );
        assert_eq!(row["planning_receipt"], "");
    }
    assert_eq!(
        goals.iter().find(|row| row["goal_id"] == running).unwrap()["state"],
        "Running"
    );
    assert_eq!(view["assignments"], json!([]));
    assert!(
        client.snapshot().await.is_err(),
        "shutdown left the HTTP listener active"
    );
    drop(shared);
    assert!(
        Store::open(&database).await.is_ok(),
        "shutdown retained the durable-store lock"
    );
}
