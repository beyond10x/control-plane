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
    Html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Control plane</title><style>{STYLE}</style></head><body><header><a href=\"/\">Control plane</a><span>Local engineering workspace</span></header><main>{body}</main></body></html>",escape(title))).into_response()
}
const STYLE: &str = "*{box-sizing:border-box}body{margin:0;background:#f6f7f9;color:#182430;font:16px system-ui,sans-serif}header{display:flex;justify-content:space-between;padding:20px max(24px,calc((100vw - 1120px)/2));background:#142b3a;color:#cadbe5}header a{color:white;font-weight:700;text-decoration:none}main{max-width:1120px;margin:32px auto;padding:0 24px}h1{font-size:30px}h2{font-size:22px}h3{font-size:18px}section,article{background:white;padding:24px;border:1px solid #dde3e8;border-radius:10px;margin:18px 0}label{display:block;font-size:14px;font-weight:600;margin:12px 0}input,textarea{display:block;width:100%;margin-top:6px;padding:10px;border:1px solid #bac6cf;border-radius:5px;font:inherit}input[type=checkbox]{display:inline;width:auto;margin-right:8px}textarea{min-height:80px}button{border:0;border-radius:5px;background:#145c78;color:white;padding:10px 16px;font:inherit;cursor:pointer}button.secondary{background:#e8eef2;color:#183a4c}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:16px}.actions{display:flex;gap:10px;flex-wrap:wrap}.actions form{margin:0}.muted{color:#566b7a;font-size:14px}.status{display:inline-block;border-radius:20px;background:#e8f1f5;padding:4px 10px;font-size:13px}code,pre{overflow-wrap:anywhere;white-space:pre-wrap}a{color:#145c78}details{margin-top:18px}summary{cursor:pointer}table{width:100%;border-collapse:collapse}td,th{padding:10px;text-align:left;border-bottom:1px solid #e1e7ec}.error{border-left:4px solid #b13939}small{display:block;color:#566b7a;margin:8px 0}";
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
        Ok(body) => page("Workspaces", body),
        Err(e) => error(e),
    }
}
async fn home_body(state: &AppState) -> Result<String> {
    let view = state.snapshot().await?;
    let mut body = String::from(
        "<h1>Workspaces</h1><p>Choose a repository or a directory of repositories, set a goal, and follow its progress.</p><div class=\"grid\">",
    );
    body.push_str(&runtime_notice(&view));
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
        "</div><section><h2>Add workspace</h2><form method=\"post\" action=\"/workspaces\">{}{}{}<button>Add workspace</button></form></section>",
        token(state),
        input("Name", "name", ""),
        input("Directory", "path", "")
    )?;
    Ok(body)
}
pub async fn workspace(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match workspace_body(&state, &id).await {
        Ok(body) => page("Workspace", body),
        Err(e) => error(e),
    }
}
async fn workspace_body(state: &AppState, id: &str) -> Result<String> {
    let view = state.snapshot().await?;
    let ws = rows(&view, "workspaces")?
        .iter()
        .find(|ws| field(ws, "workspace_id") == id)
        .context("workspace was not found")?;
    let mut body = format!(
        "<p><a href=\"/\">All workspaces</a></p><h1>{}</h1><p class=\"muted\">{}</p>",
        escape(field(ws, "name")),
        escape(field(ws, "path"))
    );
    body.push_str(&runtime_notice(&view));
    write!(
        body,
        "<p><a href=\"/workspaces/{}\">Refresh status</a> <span class=\"muted\">Reload to see the latest planning and assignment progress.</span></p>",
        escape(id)
    )?;
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
    body.push_str("<h2>Goals</h2>");
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
            "<p>Planning: <span class=\"status\">{}</span></p><p>{}</p><details><summary>Planning evidence</summary><dl><dt>Workspace</dt><dd><code>{}</code></dd><dt>Receipt</dt><dd><code>{}</code></dd></dl></details>",
            escape(field(goal, "planning_phase")),
            escape(field(goal, "planning_reason")),
            escape(field(goal, "planning_worktree_path")),
            escape(field(goal, "planning_receipt"))
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
                    "Waiting for the planner to begin. Refresh this page to see progress."
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
                "<section><h3>{}</h3><span class=\"status\">{}</span><p>{}</p><details><summary>Evidence and execution details</summary><dl><dt>Candidate</dt><dd><code>{}</code></dd><dt>Merge receipt</dt><dd><code>{}</code></dd><dt>Implementation run</dt><dd><code>{}</code></dd><dt>Review run</dt><dd><code>{}</code></dd></dl>",
                escape(field(assignment, "story_id")),
                escape(field(assignment, "state")),
                escape(field(assignment, "reason")),
                escape(field(assignment, "candidate")),
                escape(field(assignment, "merge_receipt")),
                escape(field(assignment, "implementor_run")),
                escape(field(assignment, "reviewer_run"))
            )?;
            for publication in rows(&view, "publications")?
                .iter()
                .filter(|p| p["assignment_id"] == assignment["assignment_id"])
            {
                write!(
                    body,
                    "<p>Publication: {} · <code>{}</code></p>",
                    escape(field(publication, "state")),
                    escape(field(publication, "receipt"))
                )?;
            }
            body.push_str("</details></section>");
        }
        if !field(goal, "satisfaction_receipt").is_empty() {
            write!(
                body,
                "<p>Acceptance receipt: <code>{}</code></p>",
                escape(field(goal, "satisfaction_receipt"))
            )?;
        }
        body.push_str("</article>");
    }
    write!(
        body,
        "<section><h2>New goal</h2><form method=\"post\" action=\"/goals\">{}<input type=\"hidden\" name=\"workspace_id\" value=\"{}\">{}<button>Create paused goal</button></form></section><h2>Repositories</h2>",
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
    Ok(body)
}

fn runtime_notice(view: &Value) -> String {
    view["runtime_error"].as_str().map(|error|format!("<section class=\"error\" role=\"alert\"><h2>Autonomous processing needs attention</h2><p>{}</p></section>",escape(error))).unwrap_or_default()
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
