//! SSE is a presentation of committed ESS views, never a second runtime or event journal.
use super::*;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream;
use std::{convert::Infallible, time::Duration};

pub async fn events(State(state): State<AppState>) -> Response {
    connect(state, None).await
}

pub async fn workspace_events(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    // Refuse unknown scopes before opening a successful stream.
    if let Err(error) = state.workspace_detail(&id).await {
        return (StatusCode::NOT_FOUND, error.to_string()).into_response();
    }
    connect(state, Some(id)).await
}

async fn projection(state: &AppState, workspace: Option<&str>) -> Result<(u64, String)> {
    let view = match workspace {
        Some(id) => state.workspace_detail(id).await?,
        None => state.snapshot().await?,
    };
    Ok((
        view["committed_version"]
            .as_u64()
            .context("missing committed version")?,
        serde_json::to_string(&compact(view))?,
    ))
}

pub async fn console(State(state): State<AppState>) -> Response {
    api_answer(state.snapshot().await.map(compact))
}

/// Keep raw model/tool receipts behind the evidence endpoint. Browser projections
/// carry bounded operational observations, never the accumulated model transcript.
fn compact(mut view: Value) -> Value {
    view.as_object_mut().unwrap().remove("server_observed_at");
    view.as_object_mut().unwrap().remove("committed_version");
    let assignment_owners: std::collections::HashMap<_, _> = view["assignments"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|assignment| {
            (
                field(assignment, "assignment_id").to_owned(),
                field(assignment, "goal_id").to_owned(),
            )
        })
        .collect();
    for goal in view["goals"].as_array_mut().into_iter().flatten() {
        // Only the fields shown are materialized; planner evidence is skipped unparsed.
        let receipt: Shown =
            serde_json::from_str(field(goal, "planning_receipt")).unwrap_or_default();
        goal["last_activity"] = observation(&receipt.last_activity);
        goal["activity"] = Value::Array(
            receipt
                .activity
                .as_array()
                .into_iter()
                .flatten()
                .rev()
                .take(24)
                .map(observation)
                .collect(),
        );
        let fleet = receipt
            .fleet
            .as_object()
            .map(|fleet| {
                fleet
                    .iter()
                    .filter(|(id, _)| {
                        assignment_owners
                            .get(*id)
                            .is_some_and(|owner| owner == field(goal, "goal_id"))
                    })
                    .map(|(id, event)| (id.clone(), observation(event)))
                    .collect()
            })
            .unwrap_or_default();
        goal["fleet"] = Value::Object(fleet);
    }
    for kind in ["goals", "assignments", "publications"] {
        for row in view[kind].as_array_mut().into_iter().flatten() {
            row.as_object_mut()
                .unwrap()
                .retain(|key, _| !key.contains("receipt"));
        }
    }
    view
}

/// The receipt fields the console shows.
#[derive(Default, serde::Deserialize)]
#[serde(default)]
struct Shown {
    last_activity: Value,
    activity: Value,
    fleet: Value,
}

fn observation(value: &Value) -> Value {
    if !value.is_object() {
        return Value::Null;
    }
    let mut result = json!({});
    for key in ["at", "action", "role", "status", "worktree"] {
        result[key] = json!(field(value, key).chars().take(240).collect::<String>());
    }
    result["goal_revision"] = value["goal_revision"].clone();
    let detail = &value["detail"];
    let summary = detail
        .as_str()
        .or_else(|| detail["summary"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            ["command", "program", "path", "reason", "candidate"]
                .into_iter()
                .filter_map(|key| detail[key].as_str())
                .map(|part| part.chars().take(180).collect::<String>())
                .collect::<Vec<_>>()
                .join(" · ")
        });
    result["detail"] = json!(summary.chars().take(560).collect::<String>());
    result
}

struct Connection {
    state: AppState,
    workspace: Option<String>,
    committed: tokio::sync::watch::Receiver<u64>,
    runtime_error: tokio::sync::watch::Receiver<Option<String>>,
    previous: Option<String>,
    close: bool,
}

async fn connect(state: AppState, workspace: Option<String>) -> Response {
    let committed = state.store.lock().await.subscribe();
    let runtime_error = state.runtime_error.subscribe();
    let connection = Connection {
        state,
        workspace,
        committed,
        runtime_error,
        previous: None,
        close: false,
    };
    let events = stream::unfold(connection, |mut connection| async move {
        if connection.close {
            return None;
        }
        loop {
            if connection.previous.is_some() {
                tokio::select! {
                    _ = connection.state.live_cancel.cancelled() => return None,
                    result = connection.committed.changed() => { result.ok()?; },
                    result = connection.runtime_error.changed() => { result.ok()?; },
                }
                // One latest-value slot, no event queue. Bursts are coalesced, and each
                // connection renders at most four times per second, only after changes.
                tokio::select! {
                    _ = connection.state.live_cancel.cancelled() => return None,
                    _ = tokio::time::sleep(Duration::from_millis(250)) => {},
                }
            }
            connection.committed.borrow_and_update();
            connection.runtime_error.borrow_and_update();
            match projection(&connection.state, connection.workspace.as_deref()).await {
                Ok((version, html)) => {
                    if connection.previous.as_ref() == Some(&html) {
                        continue;
                    }
                    // Reconnection always gets the complete current projection, including
                    // when Last-Event-ID names an old process. No replay is promised.
                    let event = Event::default()
                        .event("operations")
                        .id(version.to_string())
                        .retry(Duration::from_secs(1))
                        .data(&html);
                    connection.previous = Some(html);
                    return Some((Ok::<_, Infallible>(event), connection));
                }
                Err(_) => {
                    connection.close = true;
                    let event = Event::default().event("unavailable").data(
                        "State unavailable; retained observations may be stale. Reconnecting.",
                    );
                    return Some((Ok(event), connection));
                }
            }
        }
    });
    Sse::new(events)
        // Transport liveness only: comments never enter operational activity or state.
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[test]
    fn compact_projection_omits_foreign_fleet_entries_and_raw_evidence() {
        let event = |detail| json!({"action":"tool.run","detail":detail});
        let view = compact(
            json!({"goals":[{"goal_id":"g","planning_receipt":json!({"fleet":{"owned":event("visible"),"foreign":event("FOREIGN")},"receipt":"RAW"}).to_string()}],"assignments":[{"assignment_id":"owned","goal_id":"g"}],"publications":[]}),
        );
        let serialized = view.to_string();
        assert!(serialized.contains("visible"));
        assert!(!serialized.contains("FOREIGN"));
        assert!(!serialized.contains("RAW"));
    }

    async fn fixture() -> (tempfile::TempDir, AppState) {
        let temp = tempfile::tempdir().unwrap();
        let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
        (
            temp,
            AppState::new(
                Arc::new(Mutex::new(store)),
                "127.0.0.1:8787".parse().unwrap(),
                Arc::new(Notify::new()),
            ),
        )
    }
    async fn subscribe(state: &AppState, path: &str, last: Option<&str>) -> Response {
        let mut request = Request::builder()
            .uri(path)
            .header("host", "127.0.0.1:8787");
        if let Some(last) = last {
            request = request.header("last-event-id", last);
        }
        router(state.clone())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }
    async fn frame(body: &mut Body) -> String {
        let frame = tokio::time::timeout(Duration::from_secs(3), body.frame())
            .await
            .expect("SSE update timed out")
            .expect("stream closed")
            .unwrap();
        String::from_utf8(frame.into_data().expect("SSE data").to_vec()).unwrap()
    }
    fn data(frame: &str) -> Value {
        serde_json::from_str(
            frame
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
                .expect("SSE data line"),
        )
        .unwrap()
    }
    async fn workspace(state: &AppState, parent: &std::path::Path, name: &str) -> String {
        let path = parent.join(name);
        std::fs::create_dir(&path).unwrap();
        let result = state
            .store
            .lock()
            .await
            .execute(
                "RegisterWorkspace",
                json!({"path":path, "name":name}),
                Actor::Operator,
            )
            .await
            .unwrap();
        result["published"][0]["payload"]["workspace_id"]
            .as_str()
            .unwrap()
            .into()
    }
    async fn goal(state: &AppState, workspace: &str, name: &str) -> String {
        let result = state.command("CreateGoal", json!({"workspace_id":workspace,"objective":name,"acceptance":"verified","max_workers":1,"max_attempts":1,"max_minutes":10,"planner_model":"model","implementor_model":"model","reviewer_model":"model","merge_authority":false})).await.unwrap();
        result["published"][0]["payload"]["goal_id"]
            .as_str()
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn stream_sends_initial_state_commits_and_full_reconnect_without_polling() {
        let (temp, state) = fixture().await;
        let response = subscribe(&state, "/events", None).await;
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        let mut body = response.into_body();
        let initial = frame(&mut body).await;
        assert!(initial.contains("event: operations"));
        assert!(data(&initial)["workspaces"].as_array().unwrap().is_empty());
        assert!(
            tokio::time::timeout(Duration::from_millis(40), body.frame())
                .await
                .is_err(),
            "idle connection must not poll snapshots"
        );
        let ws = workspace(&state, temp.path(), "first").await;
        let update = frame(&mut body).await;
        assert_eq!(data(&update)["workspaces"][0]["workspace_id"], ws);
        drop(body);
        let second = workspace(&state, temp.path(), "while-disconnected").await;
        let mut reconnected = subscribe(&state, "/events", Some("0")).await.into_body();
        let view = data(&frame(&mut reconnected).await);
        assert!(
            view["workspaces"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w["workspace_id"] == second)
        );
        state.live_cancel.cancel();
        assert!(
            tokio::time::timeout(Duration::from_secs(1), reconnected.frame())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn workspace_stream_filters_foreign_changes_and_recovers_durable_progress() {
        let (temp, state) = fixture().await;
        let visible = workspace(&state, temp.path(), "visible").await;
        let other = workspace(&state, temp.path(), "foreign-workspace").await;
        let id = goal(&state, &visible, "visible-goal").await;
        let mut body = subscribe(&state, &format!("/workspaces/{visible}/events"), None)
            .await
            .into_body();
        let first = frame(&mut body).await;
        assert!(first.contains("visible-goal"));
        assert!(!first.contains("foreign-workspace"));
        goal(&state, &other, "FOREIGN-PRIVATE-GOAL").await;
        assert!(
            tokio::time::timeout(Duration::from_millis(350), body.frame())
                .await
                .is_err(),
            "foreign commit must not send a scoped update"
        );
        state
            .command("StartGoal", json!({"goal_id":id}))
            .await
            .unwrap();
        let event = json!({"at":"2026-10-06T10:00:00Z","action":"provider.stream","role":"planner","status":"running","goal_revision":1,"detail":{"summary":"streamed 18 events", "raw":"PRIVATE-RAW"}});
        state.store.lock().await.execute("RecordPlanningProgress",json!({"goal_id":id,"planning_revision":1,"planning_fingerprint":"fingerprint","planning_repository":"","planning_worktree_id":"tree","planning_worktree_path":"tree","planning_phase":"Planning","planning_reason":"receiving model output","planning_receipt":json!({"last_activity":event,"activity":[event],"receipt":"PRIVATE-RAW".repeat(10000)}).to_string()}),Actor::Supervisor).await.unwrap();
        let update = frame(&mut body).await;
        assert!(update.contains("streamed 18 events"));
        assert!(!update.contains("PRIVATE-RAW"));
        assert!(!update.contains("FOREIGN-PRIVATE-GOAL"));
        assert!(update.len() < 10000);
        drop(body);
        drop(state);
        let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
        let state = AppState::new(
            Arc::new(Mutex::new(reopened)),
            "127.0.0.1:8787".parse().unwrap(),
            Arc::new(Notify::new()),
        );
        let mut restored = subscribe(
            &state,
            &format!("/workspaces/{visible}/events"),
            Some("999999"),
        )
        .await
        .into_body();
        assert!(frame(&mut restored).await.contains("streamed 18 events"));
    }

    #[tokio::test]
    async fn console_activity_survives_bounding() {
        use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, Supervisor};
        struct NoModel;
        impl AgentModel for NoModel {
            fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
                anyhow::bail!("progress recording does not call the model")
            }
        }
        let (temp, state) = fixture().await;
        let visible = workspace(&state, temp.path(), "bounded").await;
        let id = goal(&state, &visible, "bounded-goal").await;
        state
            .command("StartGoal", json!({"goal_id":id}))
            .await
            .unwrap();
        let supervisor = Supervisor::new(
            state.store.clone(),
            Arc::new(Notify::new()),
            RuntimeConfig::default(),
            Arc::new(NoModel),
        );
        let assignment = json!({"goal_id":id,"assignment_id":uuid::Uuid::new_v4().to_string(),"goal_revision":1,"worktree_id":"cp-impl-console"});
        let mut sent = Vec::new();
        for index in 0..100 {
            let (action, role, status) = match index % 4 {
                0 => ("tool.run", "implementor", "running"),
                1 => ("tool.run.completed", "implementor", "completed"),
                2 => ("loom.event", "runtime", "running"),
                _ => ("blocked", "host", "failed"),
            };
            // Every fifth detail is longer than the 560 characters the console shows.
            let detail = format!(
                "event {index:03} \"{action}\" {}",
                "observed ".repeat(if index % 5 == 0 { 120 } else { index % 7 + 1 })
            );
            supervisor
                .record_progress(&assignment, action, role, json!(detail))
                .await
                .unwrap();
            sent.push((action, role, status, detail));
        }
        let check = |view: Value| {
            let goal = &view["goals"][0];
            let shown = goal["activity"].as_array().unwrap();
            assert_eq!(shown.len(), 24);
            for (event, (action, role, status, detail)) in shown.iter().zip(sent.iter().rev()) {
                assert_eq!(event["action"], *action);
                assert_eq!(event["role"], *role);
                assert_eq!(event["status"], *status);
                assert_eq!(
                    event["detail"],
                    detail.chars().take(560).collect::<String>()
                );
            }
            assert_eq!(goal["last_activity"], shown[0]);
        };
        let mut body = subscribe(&state, &format!("/workspaces/{visible}/events"), None)
            .await
            .into_body();
        check(data(&frame(&mut body).await));
        drop(body);
        drop(supervisor);
        drop(state);
        let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
        let state = AppState::new(
            Arc::new(Mutex::new(reopened)),
            "127.0.0.1:8787".parse().unwrap(),
            Arc::new(Notify::new()),
        );
        let mut restored = subscribe(&state, &format!("/workspaces/{visible}/events"), None)
            .await
            .into_body();
        check(data(&frame(&mut restored).await));
    }

    #[tokio::test]
    async fn unavailable_runtime_is_pushed_without_fabricating_work_and_assets_are_local() {
        let (_temp, state) = fixture().await;
        let mut body = subscribe(&state, "/events", None).await.into_body();
        frame(&mut body).await;
        state
            .runtime_error
            .send_replace(Some("executor unavailable".into()));
        let update = data(&frame(&mut body).await);
        assert_eq!(update["runtime_error"], "executor unavailable");
        assert!(update["goals"].as_array().unwrap().is_empty());
        assert_eq!(
            subscribe(&state, "/workspaces/missing/events", None)
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        for (path, kind) in [("/app.js", "text/javascript"), ("/index.css", "text/css")] {
            let asset = subscribe(&state, path, None).await;
            assert_eq!(asset.status(), StatusCode::OK);
            assert!(
                asset.headers()["content-type"]
                    .to_str()
                    .unwrap()
                    .starts_with(kind)
            );
            assert!(
                !asset
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .is_empty()
            );
        }
    }
}
