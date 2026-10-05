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

#[tokio::test]
async fn token_from_previous_process_cannot_mutate_restarted_service() {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
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
