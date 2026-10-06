use super::*;
use axum::{
    extract::Form,
    response::{Html, Redirect},
};
use std::{collections::HashMap, fmt::Write};

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
    Html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Control plane</title><style>{STYLE}</style></head><body><header class=\"brand\"><a href=\"/\"><span class=\"brand-icon\">▦</span> control plane</a><span>ENGINEERING OPERATIONS</span></header><main>{body}</main></body></html>",escape(title))).into_response()
}
pub(crate) const STYLE: &str = include_str!("dashboard.css");

fn console_page(body: String) -> Response {
    // An immediately readable first response remains useful without JavaScript. Vue
    // takes over once, then updates keyed components from SSE without navigation.
    let fallback = format!("<div id=\"app\"><main>{body}</main></div>");
    Html(
        include_str!("../../../frontend/dist/index.html")
            .replace("<div id=\"app\"></div>", &fallback),
    )
    .into_response()
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

fn navigation(view: &Value, selected: &str) -> Result<String> {
    let mut out = String::from(
        "<nav class=\"sidebar\" aria-label=\"Workspaces\"><a class=\"overview\" href=\"/\">◈ &nbsp; Operations overview</a><p class=\"nav-label\">WORKSPACES</p>",
    );
    for workspace in rows(view, "workspaces")? {
        write!(
            out,
            "<a class=\"workspace-link {}\" href=\"/workspaces/{}\"><span class=\"nav-dot\"></span>{}</a>",
            if field(workspace, "workspace_id") == selected {
                "selected"
            } else {
                ""
            },
            escape(field(workspace, "workspace_id")),
            escape(field(workspace, "name"))
        )?;
    }
    out.push_str("<div class=\"sidebar-footer\"><strong>Local & governed</strong><p>Actions follow your authority.<br>Evidence stays with the work.</p></div></nav>");
    Ok(out)
}
fn token(state: &AppState) -> String {
    format!(
        "<input type=\"hidden\" name=\"csrf\" value=\"{}\">",
        escape(&state.csrf)
    )
}
fn input(label: &str, name: &str, value: &str) -> String {
    format!(
        "<label>{}<input name=\"{}\" value=\"{}\" required></label>",
        escape(label),
        escape(name),
        escape(value)
    )
}
fn control(state: &AppState, path: &str, label: &str) -> String {
    format!(
        "<form method=\"post\" action=\"{}\">{}<button class=\"secondary\">{}</button></form>",
        escape(path),
        token(state),
        escape(label)
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
    match home_body(&state).await {
        Ok(body) => console_page(body),
        Err(e) => error(e),
    }
}
async fn home_body(state: &AppState) -> Result<String> {
    let view = state.snapshot().await?;
    let mut body = navigation(&view, "")?;
    body.push_str("<div class=\"page-heading\"><div><p class=\"eyebrow\">ALL WORKSPACES</p><h1>Operations overview</h1><p class=\"muted\">Your autonomous engineering, in view.</p></div><a class=\"button secondary\" href=\"#workspace-settings\">Manage workspaces ↓</a></div>");
    body.push_str(&dashboard::operations(&view)?);
    body.push_str("<details id=\"workspace-settings\" class=\"settings\"><summary>Manage workspaces · Add workspace</summary><div class=\"grid\">");
    for ws in rows(&view, "workspaces")? {
        write!(
            body,
            "<article><h2><a href=\"/workspaces/{}\">{}</a></h2><p class=\"muted\">{}</p><span class=\"status\">{}</span></article>",
            escape(field(ws, "workspace_id")),
            escape(field(ws, "name")),
            escape(field(ws, "path")),
            escape(field(ws, "state"))
        )?;
    }
    write!(
        body,
        "</div><section><h2>Add workspace</h2><form method=\"post\" action=\"/workspaces\">{}{}{}<button>Add workspace</button></form></section></details>",
        token(state),
        input("Name", "name", ""),
        input("Directory", "path", "")
    )?;
    Ok(body)
}
pub async fn workspace(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match workspace_body(&state, &id).await {
        Ok(body) => console_page(body),
        Err(e) => error(e),
    }
}
async fn workspace_body(state: &AppState, id: &str) -> Result<String> {
    let view = state.snapshot().await?;
    let ws = rows(&view, "workspaces")?
        .iter()
        .find(|ws| field(ws, "workspace_id") == id)
        .context("workspace was not found")?;
    let mut body = navigation(&view, id)?;
    write!(
        body,
        "<div class=\"page-heading\"><div><p class=\"eyebrow\">WORKSPACE OPERATIONS</p><h1>{}</h1><p class=\"muted path\">{}</p></div><a class=\"button secondary\" href=\"#goal-controls\">Goal controls ↓</a></div>",
        escape(field(ws, "name")),
        escape(field(ws, "path"))
    )?;
    body.push_str(&dashboard::operations(&state.workspace_detail(id).await?)?);
    body.push_str(
        "<details class=\"settings\"><summary>Workspace settings · Directories</summary>",
    );
    body.push_str("<h2>Directories</h2><p>Directories provide workspace context, including folders without Git. Repositories in each directory and its immediate children appear below.</p>");
    for directory in rows(&view, "directories")?.iter().filter(|directory| {
        field(directory, "workspace_id") == id && field(directory, "state") == "Registered"
    }) {
        write!(
            body,
            "<article><p><code>{}</code></p>{}</article>",
            escape(field(directory, "path")),
            control(
                state,
                &format!(
                    "/workspaces/{id}/directories/{}/remove",
                    field(directory, "directory_id")
                ),
                "Remove directory"
            )
        )?;
    }
    write!(
        body,
        "<form method=\"post\" action=\"/workspaces/{}/directories\">{}{}<button>Add directory</button></form>",
        escape(id),
        token(state),
        input("Directory path", "path", "")
    )?;
    body.push_str("</details><div id=\"goal-controls\" class=\"section-heading\"><h2>Goal controls</h2><span class=\"muted\">Changes here remain until you submit</span></div>");
    for goal in rows(&view, "goals")?
        .iter()
        .filter(|g| field(g, "workspace_id") == id)
    {
        let goal_id = field(goal, "goal_id");
        write!(
            body,
            "<article><span class=\"status\">{}</span><h3>{}</h3><p>{}</p><div class=\"actions\">",
            escape(field(goal, "state")),
            escape(field(goal, "objective")),
            escape(field(goal, "acceptance"))
        )?;
        match field(goal, "state") {
            "Paused" => body.push_str(&control(state, &format!("/goals/{goal_id}/start"), "Start")),
            "Running" => {
                body.push_str(&control(state, &format!("/goals/{goal_id}/pause"), "Pause"))
            }
            _ => {}
        }
        if matches!(field(goal, "state"), "Paused" | "Running") {
            body.push_str(&control(
                state,
                &format!("/goals/{goal_id}/cancel"),
                "Cancel",
            ));
        }
        body.push_str("</div>");
        write!(
            body,
            "<p class=\"muted\">Planning at page load: {}</p><p>{}</p><p class=\"path\">{}</p><a href=\"/goals/{}/evidence\">Inspect planning evidence ↗</a>",
            escape(field(goal, "planning_phase")),
            escape(field(goal, "planning_reason")),
            escape(field(goal, "planning_worktree_path")),
            escape(goal_id)
        )?;
        if matches!(field(goal, "state"), "Paused" | "Running") {
            write!(
                body,
                "<details><summary>Edit goal and limits</summary><form method=\"post\" action=\"/goals/{}/edit\">{}{}<button>Save goal</button></form></details>",
                escape(goal_id),
                token(state),
                goal_fields(goal)
            )?;
        }
        let assignments: Vec<_> = rows(&view, "assignments")?
            .iter()
            .filter(|a| field(a, "goal_id") == goal_id)
            .collect();
        if assignments.is_empty() {
            let reason = match field(goal, "state") {
                "Paused" => "This goal is paused. Start it when you want the planner to work.",
                "Cancelled" => "This goal is cancelled; no further work will start.",
                "Running" if field(goal, "planning_phase") == "Idle" => {
                    "Waiting for the planner to begin. Live operations above update automatically."
                }
                "Running" => {
                    "No assignments are queued. The planning status above shows progress or the reason work cannot continue."
                }
                _ => "No assignments were recorded for this goal.",
            };
            write!(body, "<p class=\"muted\">{reason}</p>")?;
        }
        for assignment in assignments {
            write!(
                body,
                "<section><h3>{}</h3><span class=\"status\">{}</span><p>{}</p><details><summary>Evidence and execution details</summary><dl><dt>Candidate</dt><dd><code>{}</code></dd><dt>Merge receipt</dt><dd>Available through Inspect planning evidence</dd><dt>Implementation run</dt><dd><code>{}</code></dd><dt>Review run</dt><dd><code>{}</code></dd></dl>",
                escape(field(assignment, "story_id")),
                escape(field(assignment, "state")),
                escape(field(assignment, "reason")),
                escape(field(assignment, "candidate")),
                escape(field(assignment, "implementor_run")),
                escape(field(assignment, "reviewer_run"))
            )?;
            for publication in rows(&view, "publications")?
                .iter()
                .filter(|p| p["assignment_id"] == assignment["assignment_id"])
            {
                write!(
                    body,
                    "<p>Publication: {} · Receipt available through the state API</p>",
                    escape(field(publication, "state"))
                )?;
            }
            body.push_str("</details></section>");
        }
        if !field(goal, "satisfaction_receipt").is_empty() {
            body.push_str("<p>Acceptance receipt recorded; inspect evidence for details.</p>");
        }
        body.push_str("</article>");
    }
    write!(
        body,
        "<details class=\"settings\"><summary>Create a new goal</summary><section><h2>New goal</h2><form method=\"post\" action=\"/goals\">{}<input type=\"hidden\" name=\"workspace_id\" value=\"{}\">{}<button>Create paused goal</button></form></section></details><details class=\"settings\"><summary>Repository settings</summary>",
        token(state),
        escape(id),
        goal_fields(&json!({}))
    )?;
    for repo in rows(&view, "repositories")?
        .iter()
        .filter(|r| field(r, "workspace_id") == id)
    {
        let repo_id = field(repo, "repository_id");
        write!(
            body,
            "<section><h3>{}</h3><p class=\"muted\">{}</p><span class=\"status\">{}</span><form method=\"post\" action=\"/repositories/{}\">{}{}{}{}<button>Save repository settings</button></form><div class=\"actions\">{}</div></section>",
            escape(field(repo, "name")),
            escape(field(repo, "path")),
            escape(field(repo, "state")),
            escape(repo_id),
            token(state),
            input("Base branch", "base_branch", field(repo, "base_branch")),
            input("Test command", "test_command", field(repo, "test_command")),
            format_args!(
                "<label>Publish command<input name=\"publish_command\" value=\"{}\"></label><small>Use this repository’s approved publishing helper.</small>",
                escape(field(repo, "publish_command"))
            ),
            control(
                state,
                &format!(
                    "/repositories/{repo_id}/{}",
                    if field(repo, "state") == "Registered" {
                        "disable"
                    } else {
                        "enable"
                    }
                ),
                if field(repo, "state") == "Registered" {
                    "Disable"
                } else {
                    "Enable"
                }
            )
        )?;
    }
    write!(
        body,
        "<section><h3>Add repository</h3><form method=\"post\" action=\"/workspaces/{}/repositories\">{}{}<button>Add repository</button></form></section>",
        escape(id),
        token(state),
        input("Repository directory", "path", "")
    )?;
    body.push_str("</details>");
    Ok(body)
}
fn goal_fields(goal: &Value) -> String {
    let mut body = format!(
        "{}<label>Acceptance<textarea name=\"acceptance\" required>{}</textarea></label><div class=\"grid\">",
        input("Objective", "objective", field(goal, "objective")),
        escape(field(goal, "acceptance"))
    );
    for (label, key) in [
        ("Planner model", "planner_model"),
        ("Implementation model", "implementor_model"),
        ("Review model", "reviewer_model"),
    ] {
        let value = goal[key].as_str().unwrap_or("gpt-5.6-sol");
        body.push_str(&input(label, key, value));
    }
    for (label, key, default) in [
        ("Workers", "max_workers", 3),
        ("Attempts per assignment", "max_attempts", 3),
        ("Minutes per assignment", "max_minutes", 60),
    ] {
        let value = goal[key].as_i64().unwrap_or(default);
        write!(body,"<label>{label}<input type=\"number\" min=\"1\" name=\"{key}\" value=\"{value}\" required></label>").unwrap();
    }
    write!(body,"</div><label><input type=\"checkbox\" name=\"merge_authority\" value=\"true\" {}>Allow verified merges</label><small>When disabled, merging waits for your authorization.</small>",if goal["merge_authority"]==true{"checked"}else{""}).unwrap();
    body
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
