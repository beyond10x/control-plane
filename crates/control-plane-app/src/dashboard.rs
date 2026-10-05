//! Compact projections of committed observations; refreshing this document never reloads forms.
use super::*;
use axum::response::Html;
use std::fmt::Write;
use web::escape;

pub async fn live(State(state): State<AppState>) -> Response {
    render(state.snapshot().await)
}
pub async fn workspace_live(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    render(state.workspace_detail(&id).await)
}
fn render(view: Result<Value>) -> Response {
    let content = match view.and_then(|view| operations(&view)) {
        Ok(content) => content,
        Err(error) => format!(
            "<section class=\"panel error\" role=\"alert\"><h2>State unavailable</h2><p>{}</p><p>Retrying in one second. Work progress cannot be confirmed.</p></section>",
            escape(&error.to_string())
        ),
    };
    Html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"refresh\" content=\"1\"><title>Live operations</title><style>{}</style></head><body class=\"live\">{content}</body></html>", web::STYLE)).into_response()
}
pub async fn evidence(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let result = async {
        let view = state.snapshot().await?;
        let goal = rows(&view, "goals")?
            .iter()
            .find(|g| field(g, "goal_id") == id)
            .context("goal not found")?;
        let assignments: Vec<_> = rows(&view, "assignments")?
            .iter()
            .filter(|a| field(a, "goal_id") == id)
            .collect();
        let publications: Vec<_> = rows(&view, "publications")?
            .iter()
            .filter(|p| {
                assignments
                    .iter()
                    .any(|a| a["assignment_id"] == p["assignment_id"])
            })
            .collect();
        Ok(json!({"goal":goal,"assignments":assignments,"publications":publications}))
    }
    .await;
    api_answer(result)
}
fn short(value: &str, limit: usize) -> String {
    let mut text: String = value.chars().take(limit).collect();
    if value.chars().count() > limit {
        text.push('…');
    }
    escape(&text)
}
fn badge(state: &str) -> String {
    let class = match state {
        "Running" | "Implementing" | "Reviewing" | "Merging" | "running" => "active",
        "Blocked" | "failed" => "blocked",
        "Merged" | "Satisfied" | "completed" => "done",
        _ => "neutral",
    };
    format!("<span class=\"badge {class}\">{}</span>", escape(state))
}
fn elapsed(at: &str) -> String {
    match time::OffsetDateTime::parse(at, &time::format_description::well_known::Rfc3339) {
        Ok(at) => {
            let seconds = (time::OffsetDateTime::now_utc() - at)
                .whole_seconds()
                .max(0);
            if seconds < 60 {
                format!("{seconds}s since observation")
            } else if seconds < 3600 {
                format!("{}m {}s since observation", seconds / 60, seconds % 60)
            } else {
                format!(
                    "{}h {}m since observation",
                    seconds / 3600,
                    seconds % 3600 / 60
                )
            }
        }
        Err(_) => "Observation time unavailable".into(),
    }
}
fn activity_detail(detail: &Value) -> String {
    if let Some(text) = detail.as_str() {
        return short(text, 560);
    }
    if let Some(summary) = detail["summary"].as_str() {
        return short(summary, 560);
    }
    // Only selected operational fields enter HTML. Receipts, transcripts and
    // arbitrary process output remain behind the evidence endpoint.
    let bounded = |text: &str, limit| text.chars().take(limit).collect::<String>();
    let mut parts = Vec::new();
    if let Some(command) = detail["command"].as_str() {
        parts.push(bounded(command, 180));
    } else if let Some(program) = detail["program"].as_str() {
        let mut command = bounded(program, 80);
        if let Some(args) = detail["args"].as_array() {
            for arg in args.iter().take(12).filter_map(Value::as_str) {
                command.push(' ');
                command.push_str(&bounded(arg, 80));
            }
        }
        parts.push(bounded(&command, 240));
    }
    for (key, label, limit) in [
        ("reason", "Reason", 200),
        ("path", "Path", 120),
        ("worktree", "Worktree", 160),
        ("target", "Target", 80),
        ("candidate", "Candidate", 64),
    ] {
        if let Some(text) = detail[key].as_str().filter(|text| !text.is_empty()) {
            parts.push(format!("{label}: {}", bounded(text, limit)));
        }
    }
    short(&parts.join(" · "), 560)
}
fn event(event: &Value, current: bool) -> String {
    let at = field(event, "at");
    let status = if current {
        field(event, "status").to_owned()
    } else {
        format!("Recorded {}", field(event, "status"))
    };
    format!(
        "<div class=\"event\"><div class=\"event-top\"><strong>{}</strong>{}</div><p>{}</p><div class=\"meta\">{} · {}<br><time datetime=\"{}\">{}</time></div>{}</div>",
        short(field(event, "action"), 120),
        badge(&status),
        activity_detail(&event["detail"]),
        short(field(event, "role"), 80),
        elapsed(at),
        escape(at),
        escape(at),
        if field(event, "worktree").is_empty() {
            String::new()
        } else {
            format!(
                "<div class=\"path\">{}</div>",
                short(field(event, "worktree"), 180)
            )
        }
    )
}
pub(crate) fn operations(view: &Value) -> Result<String> {
    let goals = rows(view, "goals")?;
    let assignments = rows(view, "assignments")?;
    let current_assignment = |a: &&Value| {
        goals.iter().any(|g| {
            g["goal_id"] == a["goal_id"]
                && field(g, "state") == "Running"
                && g["revision"] == a["goal_revision"]
        })
    };
    let running = goals
        .iter()
        .filter(|g| field(g, "state") == "Running")
        .count();
    let active = assignments
        .iter()
        .filter(current_assignment)
        .filter(|a| matches!(field(a, "state"), "Implementing" | "Reviewing" | "Merging"))
        .count();
    let queued = assignments
        .iter()
        .filter(current_assignment)
        .filter(|a| matches!(field(a, "state"), "Queued" | "ReadyToMerge"))
        .count();
    let blocked = assignments
        .iter()
        .filter(|a| field(a, "state") == "Blocked")
        .count()
        + goals
            .iter()
            .filter(|g| field(g, "state") == "Running" && field(g, "planning_phase") == "Blocked")
            .count();
    let mut out = format!(
        "<div class=\"health\"><span class=\"health-label\">LOCAL CONTROL PLANE</span><span>Server observed <time>{}</time> · refresh 1s</span></div><p class=\"freshness\">Server freshness only; work timestamps below change when activity is recorded. If this view stops refreshing or cannot load, the connection is unavailable.</p>",
        escape(field(view, "server_observed_at"))
    );
    if let Some(error) = view["runtime_error"].as_str() {
        write!(
            out,
            "<section class=\"panel error\" role=\"alert\"><h2>Autonomous processing stopped</h2><p>{}</p><p>The console remains available. Resolve the runtime error and restart the service to resume.</p></section>",
            short(error, 600)
        )?;
    }
    out.push_str("<div class=\"metrics\">");
    for (value, label, detail) in [
        (running, "Running goals", "Planner supervision"),
        (
            active,
            "Active assignments",
            "Implementation · review · merge",
        ),
        (
            queued,
            "Waiting assignments",
            "Queued or ready for publication",
        ),
        (blocked, "Need attention", "Recorded blockers"),
    ] {
        write!(
            out,
            "<div class=\"metric\"><span>{label}</span><strong>{value:02}</strong><small>{detail}</small></div>"
        )?;
    }
    out.push_str("</div><div class=\"section-heading\"><h2>Planner operations</h2><span class=\"muted\">Committed state · live observations</span></div>");
    if running == 0 {
        out.push_str("<div class=\"empty\"><strong>No running goals</strong><p>Paused goals wait for Start. Cancelled goals stay stopped. An empty queue does not establish completion.</p></div>");
    }
    let mut timeline = Vec::new();
    for goal in goals {
        let receipt: Value =
            serde_json::from_str(field(goal, "planning_receipt")).unwrap_or(Value::Null);
        let state = field(goal, "state");
        write!(
            out,
            "<article class=\"operation\"><div class=\"operation-head\"><div><span class=\"eyebrow\">PLANNER · {}</span><h3>{}</h3></div>{}</div><div class=\"operation-meta\"><span>Stage <strong>{}</strong></span><span>Revision {}</span><span>Merge authority <strong>{}</strong></span></div>",
            short(field(goal, "planner_model"), 80),
            short(field(goal, "objective"), 240),
            badge(state),
            escape(field(goal, "planning_phase")),
            goal["revision"],
            if goal["merge_authority"] == true {
                "Granted"
            } else {
                "Withheld"
            }
        )?;
        if state == "Running" {
            match field(goal,"planning_phase") {
            "Idle"=>out.push_str("<p>Waiting for the planner to begin.</p>"),
            "Queued"=>out.push_str("<p>Planning has yielded to the assignment queue. Worker status appears below.</p>"),
            "Blocked"=>out.push_str("<p class=\"attention\">Planner needs attention. Review the reason, then revise settings or pause this goal.</p>"),
            _=>{}
        }
        } else {
            write!(
                out,
                "<p class=\"muted\">{} · previous observations retained below.</p>",
                match state {
                    "Paused" => "Paused; Start this goal to resume",
                    "Cancelled" => "Cancelled; no further work will start",
                    "Satisfied" => "Acceptance recorded",
                    _ => "Not running",
                }
            )?;
        }
        if !field(goal, "planning_reason").is_empty() {
            write!(out, "<p>{}</p>", short(field(goal, "planning_reason"), 600))?;
        }
        if receipt["last_activity"].is_object() {
            let observation = &receipt["last_activity"];
            out.push_str(&event(
                observation,
                state == "Running"
                    && field(goal, "planning_phase") != "Blocked"
                    && observation["goal_revision"] == goal["revision"]
                    && !goal["revision"].is_null(),
            ));
        } else if !field(&receipt, "kind").is_empty() {
            write!(
                out,
                "<p class=\"muted\">Last recorded result: {} · observation time unavailable</p>",
                short(field(&receipt, "kind"), 160)
            )?;
        } else {
            out.push_str("<p class=\"muted\">No timed activity recorded yet.</p>");
        }
        write!(
            out,
            "<div class=\"operation-foot\"><span class=\"path\">{}</span><a target=\"_top\" href=\"/goals/{}/evidence\">Inspect evidence ↗</a></div></article>",
            short(field(goal, "planning_worktree_path"), 180),
            escape(field(goal, "goal_id"))
        )?;
        if let Some(events) = receipt["activity"].as_array() {
            for item in events.iter().rev().take(20) {
                timeline.push((
                    field(item, "at").to_owned(),
                    field(goal, "objective").to_owned(),
                    item.clone(),
                ));
            }
        }
        // Each assignment keeps its own latest event; one worker cannot replace another.
        if let Some(fleet) = receipt["fleet"].as_object() {
            for (id, activity) in fleet {
                if state != "Running"
                    || activity["goal_revision"] != goal["revision"]
                    || goal["revision"].is_null()
                {
                    continue;
                }
                if let Some(assignment) = assignments
                    .iter()
                    .find(|a| field(a, "assignment_id") == id && a["goal_id"] == goal["goal_id"])
                {
                    write!(
                        out,
                        "<article class=\"operation worker\"><div class=\"operation-head\"><h3>Worker · {}</h3>{}</div>{}</article>",
                        short(field(assignment, "story_id"), 180),
                        badge(field(assignment, "state")),
                        event(
                            activity,
                            matches!(
                                field(assignment, "state"),
                                "Implementing" | "Reviewing" | "Merging"
                            ) && assignment["goal_revision"] == goal["revision"]
                        )
                    )?;
                }
            }
        }
    }
    out.push_str("<div class=\"section-heading\"><h2>Assignment queue</h2><span class=\"muted\">Real execution state</span></div><div class=\"panel table-wrap\"><table><thead><tr><th>Assignment / repository</th><th>Stage</th><th>Attempt</th><th>Current reason</th></tr></thead><tbody>");
    for assignment in assignments {
        let repository = rows(view, "repositories")?
            .iter()
            .find(|r| r["repository_id"] == assignment["repository_id"])
            .map(|r| field(r, "name"))
            .unwrap_or("Repository unavailable");
        write!(
            out,
            "<tr><td><strong>{}</strong><small>{}</small></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            short(field(assignment, "story_id"), 120),
            short(repository, 120),
            badge(field(assignment, "state")),
            assignment["attempt"],
            short(field(assignment, "reason"), 300)
        )?;
    }
    if assignments.is_empty() {
        out.push_str("<tr><td colspan=\"4\" class=\"muted\">No assignments recorded. Goal acceptance has not been inferred from an empty queue.</td></tr>");
    }
    out.push_str("</tbody></table></div><div class=\"section-heading\"><h2>Activity history</h2><span class=\"muted\">Latest 24 recorded events</span></div><section class=\"panel timeline\">");
    timeline.sort_by(|a, b| b.0.cmp(&a.0));
    if timeline.is_empty() {
        out.push_str("<p class=\"muted\">No timestamped events recorded yet. Historical receipts remain available through Inspect evidence.</p>");
    }
    for (_, objective, item) in timeline.iter().take(24) {
        write!(
            out,
            "<div class=\"timeline-item\"><span class=\"eyebrow\">{}</span>{}</div>",
            short(objective, 100),
            event(item, false)
        )?;
    }
    out.push_str("</section>");
    Ok(out)
}
