//! Local operator surface. Browser and CLI mutations share the same admitted Store.
mod cli;
mod dashboard;
mod live;
mod web;
use anyhow::{Context, Result, ensure};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
pub use cli::{Cli, Client, run};
use control_plane_core::{Actor, Store};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, Notify};

pub type SharedStore = Arc<Mutex<Store>>;
pub async fn initialize_workspaces(
    store: &mut Store,
    cwd: &std::path::Path,
    roots: &[std::path::PathBuf],
) -> Result<()> {
    store.backfill_workspace_directories().await?;
    let paths = if roots.is_empty() {
        vec![cwd.to_owned()]
    } else {
        roots.to_vec()
    };
    for path in paths {
        let canonical = path.canonicalize()?;
        let name = canonical.file_name().unwrap_or_default().to_string_lossy();
        store.register_workspace(&canonical, &name).await?;
    }
    Ok(())
}
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
    runtime_error: tokio::sync::watch::Sender<Option<String>>,
    live_cancel: tokio_util::sync::CancellationToken,
}
impl AppState {
    pub fn new(store: SharedStore, listen: SocketAddr, wake: Arc<Notify>) -> Self {
        Self {
            store,
            listen,
            wake,
            csrf: uuid::Uuid::new_v4().to_string(),
            runtime_error: tokio::sync::watch::channel(None).0,
            live_cancel: tokio_util::sync::CancellationToken::new(),
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
        // Bounded receipts no longer repeat the activity history; every reader of a
        // snapshot (dashboard, SSE, state API) gets it from the store, per goal.
        let mut goals = store.query("GoalList")?;
        dashboard::attach_history(&store, &mut goals)?;
        Ok(json!({
            "committed_version":*store.subscribe().borrow(),
            "workspaces":store.query("WorkspaceList")?,
            "directories":store.query("WorkspaceDirectoryList")?,
            "repositories":store.query("RepositoryRegistrationList")?,
            "goals":goals,
            "assignments":store.query("AssignmentList")?,
            "publications":store.query("PublicationIntentList")?,
            "runtime_error":self.runtime_error.borrow().clone(),
            "server_observed_at": time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?,
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
    "AddWorkspaceDirectory",
    "RemoveWorkspaceDirectory",
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
    "DeleteGoal",
];

/// Routes intentionally contain no supervisor command or request-selected actor.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(web::home))
        .route("/live", get(dashboard::live))
        .route("/workspaces/{id}/live", get(dashboard::workspace_live))
        .route("/events", get(live::events))
        .route("/workspaces/{id}/events", get(live::workspace_events))
        .route("/goals/{id}/evidence", get(web::evidence))
        .route("/workspaces", post(web::add_workspace))
        .route("/workspaces/{id}", get(web::workspace))
        .route("/workspaces/{id}/repositories", post(web::add_repository))
        .route("/workspaces/{id}/directories", post(web::add_directory))
        .route(
            "/workspaces/{id}/directories/{directory}/remove",
            post(web::remove_directory),
        )
        .route("/repositories/{id}", post(web::configure_repository))
        .route("/repositories/{id}/{action}", post(web::repository_action))
        .route("/goals", post(web::create_goal))
        .route("/goals/{id}/edit", post(web::edit_goal))
        .route("/goals/{id}/{action}", post(web::goal_action))
        .route("/api/session", get(session))
        .route("/api/state", get(snapshot))
        .route("/api/console", get(live::console))
        .route("/api/goals/{id}/evidence", get(dashboard::evidence))
        .route("/app.js", get(web::javascript))
        .route("/index.css", get(web::stylesheet))
        .route("/api/workspaces", get(list_workspaces).post(add_workspace))
        .route("/api/workspaces/{id}", get(workspace_detail))
        .route(
            "/api/workspaces/{id}/directories",
            get(list_directories).post(add_directory),
        )
        .route(
            "/api/workspaces/{id}/directories/{directory}",
            delete(remove_directory),
        )
        .route(
            "/api/workspaces/{id}/directories/{directory}/remove",
            post(remove_directory),
        )
        .route("/api/commands/{command}", post(command))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), request_guard))
        .with_state(state)
}

/// The listener must already be loopback-bound. Reusable by the supervisor bootstrap.
pub async fn serve(listener: tokio::net::TcpListener, state: AppState) -> Result<()> {
    check_listener(&listener, &state)?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}

fn check_listener(listener: &tokio::net::TcpListener, state: &AppState) -> Result<()> {
    let address = listener.local_addr()?;
    ensure!(
        address.ip().is_loopback(),
        "control-plane serves loopback addresses only"
    );
    ensure!(
        address == state.listen,
        "application authority must match the bound listener"
    );
    Ok(())
}

/// Production service: one durable Store, operator surface and supervised runtime.
/// Ordinary planner refusals are persisted by the runtime; a fatal runtime error leaves the
/// console available for inspection and reports why autonomous processing stopped.
pub async fn serve_with_runtime(
    listener: tokio::net::TcpListener,
    state: AppState,
    config: control_plane_runtime::RuntimeConfig,
    model: Arc<dyn control_plane_runtime::AgentModel>,
    shutdown: tokio_util::sync::CancellationToken,
) -> Result<()> {
    check_listener(&listener, &state)?;
    let supervisor = control_plane_runtime::Supervisor::new(
        state.store.clone(),
        state.wake.clone(),
        config,
        model,
    );
    let runtime_shutdown = shutdown.clone();
    let server_shutdown = shutdown.clone();
    let runtime_error = state.runtime_error.clone();
    let runtime = async {
        if let Err(error) = supervisor.run(runtime_shutdown).await {
            let message = format!(
                "Autonomous processing stopped: {error:#}. Restart the service after resolving this problem."
            );
            eprintln!("{message}");
            runtime_error.send_replace(Some(message));
        }
    };
    let server = async {
        let live_cancel = state.live_cancel.clone();
        let result = axum::serve(listener, router(state))
            .with_graceful_shutdown(async move {
                server_shutdown.cancelled().await;
                live_cancel.cancel();
            })
            .await;
        shutdown.cancel();
        result
    };
    let (result, ()) = tokio::join!(server, runtime);
    result.map_err(Into::into)
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
    headers.insert("content-security-policy","default-src 'none'; script-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; frame-ancestors 'self'; base-uri 'none'".parse().unwrap());
    headers.insert("x-frame-options", "SAMEORIGIN".parse().unwrap());
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

impl AppState {
    async fn workspace_detail(&self, id: &str) -> Result<Value> {
        let mut snapshot = self.snapshot().await?;
        let workspace = rows(&snapshot, "workspaces")?
            .iter()
            .find(|w| field(w, "workspace_id") == id)
            .context("workspace was not found")?
            .clone();
        for kind in ["directories", "repositories", "goals"] {
            snapshot[kind] = Value::Array(
                rows(&snapshot, kind)?
                    .iter()
                    .filter(|row| {
                        field(row, "workspace_id") == id
                            && (kind != "directories" || row["state"] == "Registered")
                    })
                    .cloned()
                    .collect(),
            );
        }
        let goals: Vec<_> = rows(&snapshot, "goals")?
            .iter()
            .map(|g| g["goal_id"].clone())
            .collect();
        snapshot["assignments"] = Value::Array(
            rows(&snapshot, "assignments")?
                .iter()
                .filter(|a| goals.contains(&a["goal_id"]))
                .cloned()
                .collect(),
        );
        let assignments: Vec<_> = rows(&snapshot, "assignments")?
            .iter()
            .map(|a| a["assignment_id"].clone())
            .collect();
        snapshot["publications"] = Value::Array(
            rows(&snapshot, "publications")?
                .iter()
                .filter(|p| assignments.contains(&p["assignment_id"]))
                .cloned()
                .collect(),
        );
        snapshot
            .as_object_mut()
            .expect("snapshot object")
            .remove("workspaces");
        snapshot["workspace"] = workspace;
        Ok(snapshot)
    }
    async fn add_directory(&self, workspace: &str, input: DirectoryInput) -> Result<Value> {
        let answer = self
            .store
            .lock()
            .await
            .add_workspace_directory(workspace, std::path::Path::new(&input.path))
            .await?;
        self.wake.notify_one();
        Ok(answer)
    }
    async fn remove_directory(&self, workspace: &str, directory: &str) -> Result<Value> {
        let mut store = self.store.lock().await;
        let rows = store.query("WorkspaceDirectoryList")?;
        ensure!(
            rows.as_array()
                .context("directory rows")?
                .iter()
                .any(|d| field(d, "directory_id") == directory
                    && field(d, "workspace_id") == workspace),
            "directory does not belong to this workspace"
        );
        let answer = store.remove_workspace_directory(directory).await?;
        ensure!(
            answer["outcome"] == "applied",
            "directory removal did not apply: {answer}"
        );
        self.wake.notify_one();
        Ok(answer)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryInput {
    path: String,
}
async fn list_workspaces(State(state): State<AppState>) -> Response {
    api_answer(
        state
            .snapshot()
            .await
            .map(|v| json!({"workspaces":v["workspaces"]})),
    )
}
async fn workspace_detail(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    api_answer(state.workspace_detail(&id).await)
}
async fn list_directories(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    api_answer(
        state
            .workspace_detail(&id)
            .await
            .map(|v| json!({"directories":v["directories"]})),
    )
}
async fn add_directory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<DirectoryInput>,
) -> Response {
    api_answer(state.add_directory(&id, input).await)
}
async fn remove_directory(
    State(state): State<AppState>,
    Path((id, directory)): Path<(String, String)>,
) -> Response {
    api_answer(state.remove_directory(&id, &directory).await)
}

#[cfg(test)]
mod tests;
