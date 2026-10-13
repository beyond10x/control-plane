use super::*;
use axum::{
    extract::Form,
    response::{Html, Redirect},
};
use std::collections::HashMap;

pub(crate) fn escape(value: &str) -> String {
    value.chars().fold(String::new(), |mut out, c| {
        out.push_str(match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#39;",
            _ => {
                out.push(c);
                return out;
            }
        });
        out
    })
}
fn page(title: &str, body: String) -> Response {
    Html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Control plane</title><link rel=\"stylesheet\" href=\"/index.css\"></head><body><header class=\"brand\"><a href=\"/\"><span class=\"brand-icon\">▦</span> control plane</a><span>ENGINEERING OPERATIONS</span></header><main>{body}</main></body></html>",escape(title))).into_response()
}

/// The console shell with the page's console payload (`live::snapshot`) embedded as
/// `<script type="application/json" id="console-snapshot">`, ahead of the application, which
/// mounts from it and then follows the stream. Vue is the only renderer; nothing of the
/// payload is written as markup. `<`, `>` and `&` are written as JSON unicode escapes, so no
/// text in the payload can close the element or open markup.
fn console_page(snapshot: &Value) -> Response {
    let json = snapshot
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let shell = include_str!("../../../frontend/dist/index.html");
    let head = shell.find("<script").unwrap_or(shell.len());
    Html(format!(
        "{}<script type=\"application/json\" id=\"console-snapshot\">{json}</script>{}",
        &shell[..head],
        &shell[head..]
    ))
    .into_response()
}

/// The console shell for a goal's evidence: Vue renders the view from
/// `GET /api/goals/{id}/evidence`; nothing about the goal is rendered here.
pub async fn evidence(Path(_id): Path<String>) -> Response {
    Html(include_str!("../../../frontend/dist/index.html")).into_response()
}

pub async fn javascript() -> impl IntoResponse {
    (
        [("content-type", "text/javascript; charset=utf-8")],
        include_str!("../../../frontend/dist/app.js"),
    )
}

pub async fn stylesheet() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../frontend/dist/index.css"),
    )
}

fn error(error: anyhow::Error) -> Response {
    let status = if error.is::<CsrfRefused>() {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::CONFLICT
    };
    (status,page("Request needs attention",format!("<section class=\"error\"><h1>Request needs attention</h1><p>{}</p><a href=\"/\">Return to workspaces</a></section>",escape(&format!("{error:#}"))))).into_response()
}
fn answer(result: Result<String>) -> Response {
    match result {
        Ok(path) => Redirect::to(&path).into_response(),
        Err(e) => error(e),
    }
}
fn verify_form(state: &AppState, form: &HashMap<String, String>) -> Result<()> {
    state.check_csrf(form.get("csrf").map(String::as_str).unwrap_or_default())
}
fn required<'a>(form: &'a HashMap<String, String>, key: &str) -> Result<&'a str> {
    form.get(key)
        .map(String::as_str)
        .with_context(|| format!("{key} is missing"))
}

pub async fn home(State(state): State<AppState>) -> Response {
    match live::snapshot(&state, None).await {
        Ok(snapshot) => console_page(&snapshot),
        Err(e) => error(e),
    }
}
pub async fn workspace(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match live::snapshot(&state, Some(&id)).await {
        Ok(snapshot) => console_page(&snapshot),
        Err(e) => error(e),
    }
}
fn goal_input(form: &HashMap<String, String>) -> Result<Value> {
    let mut body = json!({});
    for field in [
        "objective",
        "acceptance",
        "planner_model",
        "implementor_model",
        "reviewer_model",
    ] {
        body[field] = json!(required(form, field)?);
    }
    for field in ["max_workers", "max_attempts", "max_minutes"] {
        body[field] = json!(required(form, field)?.parse::<i64>()?);
    }
    body["merge_authority"] = json!(
        form.get("merge_authority")
            .is_some_and(|value| value == "true")
    );
    Ok(body)
}
pub async fn add_workspace(
    State(state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            let result = state
                .add_workspace(WorkspaceInput {
                    path: required(&form, "path")?.into(),
                    name: required(&form, "name")?.into(),
                })
                .await?;
            Ok(format!(
                "/workspaces/{}",
                result["published"][0]["payload"]["workspace_id"]
                    .as_str()
                    .context("workspace identity missing")?
            ))
        }
        .await,
    )
}
pub async fn create_goal(
    State(state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            let id = required(&form, "workspace_id")?;
            let mut body = goal_input(&form)?;
            body["workspace_id"] = json!(id);
            state.command("CreateGoal", body).await?;
            Ok(format!("/workspaces/{id}"))
        }
        .await,
    )
}
pub async fn edit_goal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            let mut body = goal_input(&form)?;
            body["goal_id"] = json!(id);
            state.command("UpdateGoal", body).await?;
            goal_location(&state, &id).await
        }
        .await,
    )
}
pub async fn goal_action(
    State(state): State<AppState>,
    Path((id, action)): Path<(String, String)>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            let command = match action.as_str() {
                "start" => "StartGoal",
                "pause" => "PauseGoal",
                "cancel" => "CancelGoal",
                _ => anyhow::bail!("unknown goal action"),
            };
            state.command(command, json!({"goal_id":id})).await?;
            goal_location(&state, &id).await
        }
        .await,
    )
}
async fn goal_location(state: &AppState, id: &str) -> Result<String> {
    let view = state.snapshot().await?;
    let goal = rows(&view, "goals")?
        .iter()
        .find(|goal| field(goal, "goal_id") == id)
        .context("goal not found")?;
    Ok(format!("/workspaces/{}", field(goal, "workspace_id")))
}
pub async fn configure_repository(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(async{verify_form(&state,&form)?;state.command("ConfigureRepository",json!({"repository_id":id,"base_branch":required(&form,"base_branch")?,"test_command":required(&form,"test_command")?,"publish_command":required(&form,"publish_command")?})).await?;repository_location(&state,&id).await}.await)
}
pub async fn repository_action(
    State(state): State<AppState>,
    Path((id, action)): Path<(String, String)>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            let command = match action.as_str() {
                "enable" => "EnableRepositoryRegistration",
                "disable" => "DisableRepositoryRegistration",
                _ => anyhow::bail!("unknown repository action"),
            };
            state.command(command, json!({"repository_id":id})).await?;
            repository_location(&state, &id).await
        }
        .await,
    )
}
async fn repository_location(state: &AppState, id: &str) -> Result<String> {
    let view = state.snapshot().await?;
    let repo = rows(&view, "repositories")?
        .iter()
        .find(|repo| field(repo, "repository_id") == id)
        .context("repository not found")?;
    Ok(format!("/workspaces/{}", field(repo, "workspace_id")))
}
pub async fn add_repository(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(async{verify_form(&state,&form)?;let found=control_plane_core::discover(std::path::Path::new(required(&form,"path")?))?;ensure!(found.repositories.len()==1,"select one Git repository");let repo=&found.repositories[0];state.command("RegisterRepository",json!({"workspace_id":id,"name":repo.name,"path":repo.path,"common_dir":repo.common_dir,"base_branch":repo.base_branch,"test_command":"task check","publish_command":""})).await?;Ok(format!("/workspaces/{id}"))}.await)
}

pub async fn add_directory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            state
                .add_directory(
                    &id,
                    DirectoryInput {
                        path: required(&form, "path")?.into(),
                    },
                )
                .await?;
            Ok(format!("/workspaces/{id}"))
        }
        .await,
    )
}
pub async fn remove_directory(
    State(state): State<AppState>,
    Path((id, directory)): Path<(String, String)>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    answer(
        async {
            verify_form(&state, &form)?;
            state.remove_directory(&id, &directory).await?;
            Ok(format!("/workspaces/{id}"))
        }
        .await,
    )
}
