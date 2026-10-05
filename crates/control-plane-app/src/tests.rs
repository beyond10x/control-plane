use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

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
            .contains("frame-ancestors 'none'")
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
