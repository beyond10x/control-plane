//! Local operator surface. Browser and CLI mutations share the same admitted Store.
mod cli;
mod web;
use anyhow::{Context, Result, ensure};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
pub use cli::{Cli, Client, run};
use control_plane_core::{Actor, Store};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, Notify};

pub type SharedStore = Arc<Mutex<Store>>;
#[derive(Debug)]
struct CsrfRefused;
impl std::fmt::Display for CsrfRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("request token is missing or expired; reload the page")
    }
}
impl std::error::Error for CsrfRefused {}
#[derive(Clone)]
pub struct AppState {
    pub store: SharedStore,
    pub wake: Arc<Notify>,
    listen: SocketAddr,
    csrf: String,
}
impl AppState {
    pub fn new(store: SharedStore, listen: SocketAddr, wake: Arc<Notify>) -> Self {
        Self {
            store,
            listen,
            wake,
            csrf: uuid::Uuid::new_v4().to_string(),
        }
    }
    fn host_allowed(&self, host: &str) -> bool {
        host == self.listen.to_string() || host == format!("localhost:{}", self.listen.port())
    }
    fn check_csrf(&self, token: &str) -> Result<()> {
        if token != self.csrf {
            return Err(CsrfRefused.into());
        }
        Ok(())
    }
    async fn snapshot(&self) -> Result<Value> {
        let store = self.store.lock().await;
        Ok(json!({
            "workspaces":store.query("WorkspaceList")?,
            "repositories":store.query("RepositoryRegistrationList")?,
            "goals":store.query("GoalList")?,
            "assignments":store.query("AssignmentList")?,
            "publications":store.query("PublicationIntentList")?,
        }))
    }
    async fn command(&self, command: &str, body: Value) -> Result<Value> {
        ensure!(
            OPERATOR_COMMANDS.contains(&command),
            "this command is not available to the operator"
        );
        ensure!(
            body.get("actor").is_none(),
            "requests cannot select an actor"
        );
        let result = self
            .store
            .lock()
            .await
            .execute(command, body, Actor::Operator)
            .await?;
        ensure!(
            matches!(result["outcome"].as_str(), Some("applied" | "created")),
            "command did not apply: {result}"
        );
        self.wake.notify_one();
        Ok(result)
    }
    async fn add_workspace(&self, request: WorkspaceInput) -> Result<Value> {
        let result = self
            .store
            .lock()
            .await
            .register_workspace(std::path::Path::new(&request.path), &request.name)
            .await?;
        self.wake.notify_one();
        Ok(result)
    }
}
const OPERATOR_COMMANDS: &[&str] = &[
    "ArchiveWorkspace",
    "RegisterRepository",
    "ConfigureRepository",
    "DisableRepositoryRegistration",
    "EnableRepositoryRegistration",
    "CreateGoal",
    "UpdateGoal",
    "StartGoal",
    "PauseGoal",
    "CancelGoal",
];

/// Routes intentionally contain no supervisor command or request-selected actor.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(web::home))
        .route("/workspaces", post(web::add_workspace))
        .route("/workspaces/{id}", get(web::workspace))
        .route("/workspaces/{id}/repositories", post(web::add_repository))
        .route("/repositories/{id}", post(web::configure_repository))
        .route("/repositories/{id}/{action}", post(web::repository_action))
        .route("/goals", post(web::create_goal))
        .route("/goals/{id}/edit", post(web::edit_goal))
        .route("/goals/{id}/{action}", post(web::goal_action))
        .route("/api/session", get(session))
        .route("/api/state", get(snapshot))
        .route("/api/workspaces", post(add_workspace))
        .route("/api/commands/{command}", post(command))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), request_guard))
        .with_state(state)
}

/// The listener must already be loopback-bound. Reusable by the supervisor bootstrap.
pub async fn serve(listener: tokio::net::TcpListener, state: AppState) -> Result<()> {
    let address = listener.local_addr()?;
    ensure!(
        address.ip().is_loopback(),
        "control-plane serves loopback addresses only"
    );
    ensure!(
        address == state.listen,
        "application authority must match the bound listener"
    );
    axum::serve(listener, router(state)).await?;
    Ok(())
}

async fn request_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let host = request.headers().get("host").and_then(|v| v.to_str().ok());
    let valid_host = host.is_some_and(|host| state.host_allowed(host));
    let valid_origin = request.headers().get("origin").is_none_or(|origin| {
        origin
            .to_str()
            .ok()
            .is_some_and(|origin| host.is_some_and(|host| origin == format!("http://{host}")))
    });
    let cross_site = request
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|v| v == "cross-site");
    if !valid_host || !valid_origin || cross_site {
        return (
            StatusCode::FORBIDDEN,
            "Request does not belong to this local console.",
        )
            .into_response();
    }
    if request.method() != Method::GET
        && request.method() != Method::HEAD
        && request.uri().path().starts_with("/api/")
    {
        let token = request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if state.check_csrf(token).is_err() {
            return (
                StatusCode::FORBIDDEN,
                "Request token is missing or expired.",
            )
                .into_response();
        }
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-store".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    // Native form navigation with no-referrer sends Origin: null in Chromium.
    // Keep local form origins verifiable without sharing referrers cross-origin.
    headers.insert("referrer-policy", "same-origin".parse().unwrap());
    headers.insert("content-security-policy","default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'".parse().unwrap());
    response
}

async fn session(State(state): State<AppState>) -> Json<Value> {
    Json(json!({"csrf_token":state.csrf}))
}
async fn snapshot(State(state): State<AppState>) -> Response {
    api_answer(state.snapshot().await)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceInput {
    path: String,
    name: String,
}
async fn add_workspace(
    State(state): State<AppState>,
    Json(input): Json<WorkspaceInput>,
) -> Response {
    api_answer(state.add_workspace(input).await)
}
async fn command(
    State(state): State<AppState>,
    Path(command): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    if !OPERATOR_COMMANDS.contains(&command.as_str()) || body.get("actor").is_some() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"operator command refused"})),
        )
            .into_response();
    }
    api_answer(state.command(&command, body).await)
}
fn api_answer(result: Result<Value>) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(json!({"error":format!("{error:#}")})),
        )
            .into_response(),
    }
}
fn field<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name].as_str().unwrap_or_default()
}
fn rows<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    value[name]
        .as_array()
        .with_context(|| format!("missing {name} view"))
}

#[cfg(test)]
mod tests;
