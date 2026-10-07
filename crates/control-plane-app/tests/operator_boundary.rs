use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use control_plane_app::{AppState, router};
use control_plane_core::Store;
use http_body_util::BodyExt;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tower::ServiceExt;

fn isolate_git_discovery(path: &std::path::Path) {
    // Scratch lives under the product checkout, which is detached in PR CI.
    // A bare boundary models a non-worktree directory without inheriting that checkout.
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

#[tokio::test]
async fn token_from_previous_process_cannot_mutate_restarted_service() {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
    isolate_git_discovery(temp.path());
    let store = Arc::new(Mutex::new(
        Store::open(temp.path().join("state.sqlite")).await.unwrap(),
    ));
    let address = "127.0.0.1:8787".parse().unwrap();
    let old = AppState::new(store.clone(), address, Arc::new(Notify::new()));
    let session = router(old)
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = session.into_body().collect().await.unwrap().to_bytes();
    let token: Value = serde_json::from_slice(&bytes).unwrap();
    let next = AppState::new(store.clone(), address, Arc::new(Notify::new()));
    let payload = serde_json::json!({"name":"stale request", "path":temp.path()}).to_string();
    let response = router(next.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspaces")
                .header("host", "127.0.0.1:8787")
                .header("content-type", "application/json")
                .header("x-csrf-token", token["csrf_token"].as_str().unwrap())
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        store.lock().await.query("WorkspaceList").unwrap(),
        serde_json::json!([])
    );
    let session = router(next.clone())
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let token: Value =
        serde_json::from_slice(&session.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let response = router(next)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspaces")
                .header("host", "127.0.0.1:8787")
                .header("content-type", "application/json")
                .header("x-csrf-token", token["csrf_token"].as_str().unwrap())
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        store.lock().await.query("WorkspaceList").unwrap()[0]["name"],
        "stale request"
    );
}

#[tokio::test]
async fn ipv6_loopback_client_and_server_agree_on_authority() {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
    isolate_git_discovery(temp.path());
    let store = Arc::new(Mutex::new(
        Store::open(temp.path().join("state.sqlite")).await.unwrap(),
    ));
    let listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = AppState::new(store, address, Arc::new(Notify::new()));
    let server = tokio::spawn(control_plane_app::serve(listener, app));
    let client = control_plane_app::Client::new(&format!("http://{address}")).unwrap();
    let result = client
        .add_workspace(temp.path(), "ipv6 workspace")
        .await
        .unwrap();
    assert_eq!(result["outcome"], "created");
    assert_eq!(
        client.snapshot().await.unwrap()["workspaces"][0]["name"],
        "ipv6 workspace"
    );
    server.abort();
    let _ = server.await;
}

#[cfg(unix)]
#[tokio::test]
async fn real_serve_command_starts_supervisor_and_releases_database_on_sigterm() {
    use control_plane_app::Client;
    use serde_json::json;
    use tokio::io::{AsyncBufReadExt, BufReader};
    let scratch = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/control-plane-console");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
    let db = temp.path().join("state.sqlite");
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_control-plane"))
        .args(["serve", "--listen", "127.0.0.1:0", "--state"])
        .arg(&db)
        .current_dir(temp.path())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    let line = tokio::time::timeout(std::time::Duration::from_secs(5), lines.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let url = line
        .strip_prefix("Control plane: ")
        .expect("service must announce its bound origin");
    let client = Client::new(url).unwrap();
    let view = client.snapshot().await.unwrap();
    assert_eq!(view["workspaces"].as_array().unwrap().len(), 1);
    assert_eq!(view["workspaces"][0]["path"], temp.path().to_str().unwrap());
    assert!(view["goals"].as_array().unwrap().is_empty());
    let result = client.command("CreateGoal", json!({"workspace_id":view["workspaces"][0]["workspace_id"],"objective":"Observe empty workspace","acceptance":"A visible planning result","max_workers":3,"max_attempts":3,"max_minutes":1,"planner_model":"gpt-5.6-sol","implementor_model":"gpt-5.6-sol","reviewer_model":"gpt-5.6-sol","merge_authority":false})).await.unwrap();
    let goal = result["published"][0]["payload"]["goal_id"]
        .as_str()
        .unwrap();
    client
        .command("StartGoal", json!({"goal_id":goal}))
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if client.snapshot().await.unwrap()["goals"][0]["planning_phase"] != "Idle" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("serve never started its supervisor");
    client
        .command("CancelGoal", json!({"goal_id":goal}))
        .await
        .unwrap();
    assert!(
        std::process::Command::new("kill")
            .args(["-TERM", &child.id().unwrap().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
    let store = Store::open(&db)
        .await
        .expect("shutdown retained the database lock");
    assert_eq!(store.query("GoalList").unwrap()[0]["state"], "Cancelled");
}
