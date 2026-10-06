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

async fn projection(state: &AppState, workspace: Option<&str>) -> Result<(u64, Value)> {
    let view = match workspace {
        Some(id) => state.workspace_detail(id).await?,
        None => state.snapshot().await?,
    };
    let view = with_history(state, view).await?;
    Ok((
        view["committed_version"]
            .as_u64()
            .context("missing committed version")?,
        compact(view),
    ))
}

pub async fn console(State(state): State<AppState>) -> Response {
    let view = async { with_history(&state, state.snapshot().await?).await }.await;
    api_answer(view.map(|view| clocked(compact(view))))
}

/// Add `server_time`, the server clock as the projection is sent, as RFC 3339 UTC with exactly
/// three fractional digits (`2026-10-06T10:00:00.250Z`), which is also ECMAScript's date
/// format. A connection adds it after comparing the view, so time alone sends no frame.
fn clocked(mut view: Value) -> Value {
    let now = time::OffsetDateTime::now_utc();
    view["server_time"] = json!(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.millisecond()
    ));
    view
}

/// The goals' activity history comes from the store; bounded receipts do not repeat it.
/// `replayed_progress` is the store's replay boundary, which [`compact`] reads and removes; it
/// is fixed for the life of the store, so reading it under a later lock than the view is exact.
async fn with_history(state: &AppState, mut view: Value) -> Result<Value> {
    let store = state.store.lock().await;
    dashboard::attach_history(&store, &mut view["goals"])?;
    view["replayed_progress"] = json!(store.replayed_progress());
    Ok(view)
}

/// Keep raw model/tool receipts behind the evidence endpoint. Browser projections
/// carry bounded operational observations, never the accumulated model transcript.
///
/// Besides the committed rows, a projection carries these fields, each derived from records
/// that already exist. No field name contains `receipt`, and none carries receipt contents.
///
/// | Field | Type | Present |
/// |---|---|---|
/// | `server_time` (top level) | string: RFC 3339, UTC, three fractional digits | On every SSE frame and every `/api/console` answer: the server clock as it was sent ([`clocked`]). |
/// | `id` on every activity object (goal `last_activity`, `planner_activity`, `activity[]` and `fleet{}`) | string | Always. The id recorded with the entry; an entry recorded without one gets `entry-` and 16 hex digits hashing its goal and content ([`Entries`]). The same entry has the same id wherever it appears, in every projection of one store state, and after later entries are appended. |
/// | goal `planner_activity` | activity object, or null | The newest entry recorded without an `assignment_id`: the planner's own, roles `planner` and `critic`. Worker entries always carry their assignment and appear under `fleet`. The store's progress journal keeps this entry however much worker activity follows (its `planner_activity`, rebuilt from the recorded entries on replay). Null when the goal has none. `last_activity` stays the newest entry of any role. |
/// | goal `waiting` | `{role, model, since}`, or null | A goal-level model call that is open: the planner's or plan critic's ([`planner_wait`]), else the final goal review's (role `goal_reviewer`, [`assignment_wait`]). Null otherwise. |
/// | assignment `waiting` | `{role, model, since}`, or null | The assignment's open implementor call (while Implementing) or reviewer call (while Reviewing) ([`assignment_wait`]). Null otherwise. |
/// | goal `acceptance_recorded` | bool | Always: true when the goal is Satisfied with a recorded satisfaction receipt. |
/// | assignment `merged_at` | string, or null | The `observed_at` of a Merged assignment's merge receipt (`fleet.rs` `observe_merge`); null otherwise. |
///
/// A model call is open while all of these hold: the goal is Running; the runtime has not
/// stopped (`runtime_error` is not set); the newest step of the call's lane (the planner's, or
/// the assignment's) is the call's request, where a step is the lane's newest entry other than
/// a streamed `loom.event` and the store's journal keeps it however many stream events follow;
/// the request was recorded at the goal's revision; and it was recorded since the store last
/// opened (its record number is above `Store::replayed_progress`), because a call does not
/// outlive the process whose runtime holds it.
///
/// In both `waiting` objects, `role` is the call's role, `model` the goal's model field for
/// that role (`planner` → `planner_model`; `implementor` → `implementor_model`; `critic`,
/// `reviewer` and `goal_reviewer` → `reviewer_model`, which is what the runtime sends each of
/// them), and `since` the `at` of the call's request.
fn compact(mut view: Value) -> Value {
    view.as_object_mut().unwrap().remove("server_observed_at");
    view.as_object_mut().unwrap().remove("committed_version");
    let replayed = view
        .as_object_mut()
        .unwrap()
        .remove("replayed_progress")
        .and_then(|replayed| replayed.as_u64())
        .unwrap_or(0);
    // A stopped runtime holds no model call.
    let runtime_holds_calls = !view["runtime_error"].is_string();
    // Each assignment's goal and state, in a stable order.
    let assignments: std::collections::BTreeMap<_, _> = view["assignments"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|assignment| {
            (
                field(assignment, "assignment_id").to_owned(),
                (
                    field(assignment, "goal_id").to_owned(),
                    field(assignment, "state").to_owned(),
                ),
            )
        })
        .collect();
    let mut assignment_waits = std::collections::BTreeMap::new();
    for goal in view["goals"].as_array_mut().into_iter().flatten() {
        // Only the fields shown are materialized; planner evidence is skipped unparsed. The
        // history (with planner evidence) stays behind the evidence endpoint.
        let mut receipt: Shown =
            serde_json::from_str(field(goal, "planning_receipt")).unwrap_or_default();
        if let Some(history) = goal
            .as_object_mut()
            .and_then(|row| row.remove("activity_history"))
            .filter(Value::is_object)
        {
            receipt.activity = history["activity"].clone();
            receipt.fleet = history["fleet"].clone();
            receipt.planner_activity = history["planner_activity"].clone();
            receipt.planner_step = history["planner_step"].clone();
            receipt.fleet_steps = history["fleet_steps"].clone();
        }
        let goal_id = field(goal, "goal_id").to_owned();
        let entries = Entries::new(&goal_id, &receipt.activity);
        goal["last_activity"] = entries.observe(&receipt.last_activity);
        goal["activity"] = Value::Array(
            entries
                .listed
                .iter()
                .rev()
                .take(24)
                .map(|(event, id)| observation(event, id))
                .collect(),
        );
        // The planner's entries, newest first. `last_activity` is the newest entry of all and
        // the journal's `planner_activity` the newest planner entry, so whichever of them is
        // the planner's comes before every listed planner entry.
        let planner = || {
            std::iter::once(&receipt.last_activity)
                .chain(std::iter::once(&receipt.planner_activity))
                .chain(entries.listed.iter().rev().map(|(event, _)| *event))
                .filter(|event| planner_entry(event))
        };
        let planner_activity = planner()
            .next()
            .map_or(Value::Null, |event| entries.observe(event));
        let mut wait = None;
        if runtime_holds_calls && field(goal, "state") == "Running" {
            wait = planner_wait(goal, &receipt.planner_step, replayed);
            let owned = assignments
                .iter()
                .filter(|(_, (owner, _))| *owner == goal_id);
            for (id, (_, state)) in owned {
                let step = &receipt.fleet_steps[id.as_str()];
                match assignment_wait(goal, state, step, replayed) {
                    Some(call) if call["role"] == "goal_reviewer" => {
                        wait.get_or_insert(call);
                    }
                    Some(call) => {
                        assignment_waits.insert(id.clone(), call);
                    }
                    None => {}
                }
            }
        }
        let accepted =
            field(goal, "state") == "Satisfied" && !field(goal, "satisfaction_receipt").is_empty();
        goal["planner_activity"] = planner_activity;
        goal["waiting"] = wait.unwrap_or(Value::Null);
        goal["acceptance_recorded"] = json!(accepted);
        let fleet = receipt
            .fleet
            .as_object()
            .map(|fleet| {
                fleet
                    .iter()
                    .filter(|(id, _)| {
                        assignments
                            .get(*id)
                            .is_some_and(|(owner, _)| *owner == goal_id)
                    })
                    .map(|(id, event)| (id.clone(), entries.observe(event)))
                    .collect()
            })
            .unwrap_or_default();
        goal["fleet"] = Value::Object(fleet);
    }
    for assignment in view["assignments"].as_array_mut().into_iter().flatten() {
        let merged = merged_at(assignment);
        let wait = assignment_waits
            .remove(field(assignment, "assignment_id"))
            .unwrap_or(Value::Null);
        assignment["merged_at"] = merged;
        assignment["waiting"] = wait;
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
    /// This and the steps are taken from the store's history only, never from a receipt.
    #[serde(skip)]
    planner_activity: Value,
    #[serde(skip)]
    planner_step: Value,
    #[serde(skip)]
    fleet_steps: Value,
}

/// Whether an entry is the planner's own: the fleet records every worker entry with its
/// `assignment_id`, and the planner's progress path never does. The journal uses this rule.
fn planner_entry(event: &Value) -> bool {
    event.is_object() && event.get("assignment_id").and_then(Value::as_str).is_none()
}

/// A goal's retained activity, oldest first, each entry with the id it is projected under.
///
/// An entry's id is the one recorded with it. Entries recorded before the runtime gave each
/// one an id get `entry-` and 16 hex digits of a 64-bit FNV-1a hash of the goal id and the
/// entry's JSON; a later byte-identical entry in the same history gets `-2`, `-3` and so on,
/// counted from the oldest retained entry. Appending an entry therefore changes no earlier id;
/// only the retained history (64 entries) dropping an identical older one can.
struct Entries<'a> {
    goal: &'a str,
    listed: Vec<(&'a Value, String)>,
}

impl<'a> Entries<'a> {
    fn new(goal: &'a str, activity: &'a Value) -> Self {
        let mut seen = std::collections::HashMap::<String, usize>::new();
        let listed = activity
            .as_array()
            .into_iter()
            .flatten()
            .map(|event| {
                let id = entry_id(goal, event);
                let count = seen.entry(id.clone()).or_default();
                *count += 1;
                let id = match *count {
                    1 => id,
                    count => format!("{id}-{count}"),
                };
                (event, id)
            })
            .collect();
        Self { goal, listed }
    }

    /// An entry shown outside the list (`last_activity`, `planner_activity`, `fleet`) has the
    /// id of the newest identical listed entry, so it reads as the same entry in both places.
    fn observe(&self, event: &Value) -> Value {
        let id = self
            .listed
            .iter()
            .rev()
            .find(|(listed, _)| *listed == event)
            .map_or_else(|| entry_id(self.goal, event), |(_, id)| id.clone());
        observation(event, &id)
    }
}

fn entry_id(goal: &str, event: &Value) -> String {
    if let Some(id) = event["id"].as_str().filter(|id| !id.is_empty()) {
        return id.chars().take(240).collect();
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in goal.bytes().chain([0]).chain(event.to_string().bytes()) {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    format!("entry-{hash:016x}")
}

/// The call a lane's step opened, if it is still open: the step (`{recorded, entry}`, see
/// `Store::activity_history`) is one of `requests` and was recorded by this process, after
/// progress record `replayed`. Any later entry of the lane but a stream event replaces the
/// step and so ends the call.
fn open_call<'a>(step: &'a Value, replayed: u64, requests: &[&str]) -> Option<&'a Value> {
    let call = &step["entry"];
    (step["recorded"]
        .as_u64()
        .is_some_and(|recorded| recorded > replayed)
        && requests.contains(&field(call, "action")))
    .then_some(call)
}

/// `{role, model, since}` of an open call, if `call` was recorded at the goal's revision.
/// The model is the goal's field for the role, as the runtime sends it (see [`compact`]).
fn wait(goal: &Value, role: &str, call: &Value) -> Option<Value> {
    if goal["revision"].is_null() || call["goal_revision"] != goal["revision"] {
        return None;
    }
    let model = match role {
        "planner" => goal["planner_model"].clone(),
        "implementor" => goal["implementor_model"].clone(),
        "critic" | "reviewer" | "goal_reviewer" => goal["reviewer_model"].clone(),
        _ => Value::Null,
    };
    let role: String = role.chars().take(240).collect();
    let since: String = field(call, "at").chars().take(240).collect();
    Some(json!({"role":role,"model":model,"since":since}))
}

/// The planner's open call, from the planner lane's step. engine.rs `Planning::respond`
/// records `model.requested` before each planner or plan-critic call, `loom.event` entries
/// with the call's role while it streams, and `model.completed` or `model.failed` after it.
/// The caller checks that the goal is Running and the runtime has not stopped.
fn planner_wait(goal: &Value, step: &Value, replayed: u64) -> Option<Value> {
    let call = open_call(step, replayed, &["model.requested"])?;
    wait(goal, field(call, "role"), call)
}

/// An assignment's open call, from the assignment's step. fleet.rs records `model.request`
/// (role `implementor`) before each implementor call while Implementing, `review.request`
/// (`reviewer`) before the review while Reviewing, and `goal.review` (`goal_reviewer`) on a
/// Merged assignment before the final goal review; while the model streams it records
/// `loom.event` entries with role `runtime`. It records no completion: the assignment's next
/// entry of any other action ends the call, and so does the assignment leaving the state its
/// call belongs to. The caller checks that the goal is Running and the runtime has not stopped.
fn assignment_wait(goal: &Value, state: &str, step: &Value, replayed: u64) -> Option<Value> {
    let role = match state {
        "Implementing" => "implementor",
        "Reviewing" => "reviewer",
        "Merged" => "goal_reviewer",
        _ => return None,
    };
    let call = open_call(
        step,
        replayed,
        &["model.request", "review.request", "goal.review"],
    )?;
    if field(call, "role") != role {
        return None;
    }
    wait(goal, role, call)
}

/// When a Merged assignment's merge receipt observed the merge. Only `observed_at` is read.
fn merged_at(assignment: &Value) -> Value {
    #[derive(serde::Deserialize)]
    struct Observed {
        observed_at: String,
    }
    if field(assignment, "state") != "Merged" {
        return Value::Null;
    }
    serde_json::from_str::<Observed>(field(assignment, "merge_receipt"))
        .map_or(Value::Null, |receipt| {
            json!(receipt.observed_at.chars().take(64).collect::<String>())
        })
}

fn observation(value: &Value, id: &str) -> Value {
    if !value.is_object() {
        return Value::Null;
    }
    let mut result = json!({});
    result["id"] = json!(id);
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
                Ok((version, view)) => {
                    let shown = view.to_string();
                    if connection.previous.as_ref() == Some(&shown) {
                        continue;
                    }
                    // Reconnection always gets the complete current projection, including
                    // when Last-Event-ID names an old process. No replay is promised. The
                    // clock joins the frame after the comparison above.
                    let event = Event::default()
                        .event("operations")
                        .id(version.to_string())
                        .retry(Duration::from_secs(1))
                        .data(clocked(view).to_string());
                    connection.previous = Some(shown);
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

    /// Run one supervisor command; it must apply.
    async fn supervise(state: &AppState, command: &str, body: Value) -> Value {
        let result = state
            .store
            .lock()
            .await
            .execute(command, body, Actor::Supervisor)
            .await
            .unwrap();
        assert!(
            matches!(result["outcome"].as_str(), Some("applied" | "created")),
            "{command}: {result}"
        );
        result
    }
    /// A started goal whose roles use distinct models, with its revision.
    async fn running_goal(state: &AppState, workspace: &str) -> (String, Value) {
        let created = state.command("CreateGoal", json!({"workspace_id":workspace,"objective":"projection goal","acceptance":"verified","max_workers":2,"max_attempts":2,"max_minutes":10,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"reviewer-model","merge_authority":true})).await.unwrap();
        let id: String = created["published"][0]["payload"]["goal_id"]
            .as_str()
            .unwrap()
            .into();
        state
            .command("StartGoal", json!({"goal_id":id}))
            .await
            .unwrap();
        let goals = state.store.lock().await.query("GoalList").unwrap();
        let revision = goals
            .as_array()
            .unwrap()
            .iter()
            .find(|goal| goal["goal_id"] == id)
            .unwrap()["revision"]
            .clone();
        (id, revision)
    }
    /// Record one activity through the planner's progress path, as `RecordPlanningProgress`.
    async fn record_planner(state: &AppState, goal: &str, revision: &Value, entry: Value) {
        supervise(state, "RecordPlanningProgress", json!({"goal_id":goal,"planning_revision":revision,"planning_fingerprint":"fingerprint","planning_repository":"","planning_worktree_id":"tree","planning_worktree_path":"tree","planning_phase":"Planning","planning_reason":"","planning_receipt":json!({"last_activity":entry}).to_string()})).await;
    }
    /// A planner entry shaped as the supervisor records it: its own id and no assignment.
    async fn planner_event(
        state: &AppState,
        goal: &str,
        revision: &Value,
        action: &str,
        at: &str,
    ) -> Value {
        let entry = json!({"id":uuid::Uuid::new_v4().to_string(),"action":action,"role":"planner","status":"running","detail":format!("{action} observed"),"at":at,"worktree":"tree","goal_revision":revision});
        record_planner(state, goal, revision, entry.clone()).await;
        entry
    }
    /// A Git repository registered in the workspace directory `parent`.
    async fn repository(state: &AppState, parent: &std::path::Path, workspace: &str) -> String {
        let path = parent.join("repository");
        assert!(
            std::process::Command::new("git")
                .args(["init", "--initial-branch=main"])
                .arg(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
        let result = state
            .store
            .lock()
            .await
            .execute(
                "RegisterRepository",
                json!({"workspace_id":workspace,"name":"repository","path":path,"common_dir":"","base_branch":"main","test_command":"true","publish_command":"true"}),
                Actor::Operator,
            )
            .await
            .unwrap();
        result["published"][0]["payload"]["repository_id"]
            .as_str()
            .unwrap()
            .into()
    }
    /// An assignment queued at the goal's current revision.
    async fn queued(
        state: &AppState,
        goal: &str,
        revision: &Value,
        repository: &str,
        story: &str,
    ) -> String {
        let result = supervise(state, "QueueAssignment", json!({"goal_id":goal,"repository_id":repository,"story_id":story,"case_id":format!("{story}/case"),"worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":revision})).await;
        result["published"][0]["payload"]["assignment_id"]
            .as_str()
            .unwrap()
            .into()
    }
    /// The supervisor whose fleet progress path records worker activity.
    fn supervisor(state: &AppState) -> control_plane_runtime::Supervisor {
        struct Unused;
        impl control_plane_runtime::AgentModel for Unused {
            fn respond(&self, _: &control_plane_runtime::ModelRequest) -> anyhow::Result<Value> {
                anyhow::bail!("progress recording does not call the model")
            }
        }
        control_plane_runtime::Supervisor::new(
            state.store.clone(),
            Arc::new(Notify::new()),
            control_plane_runtime::RuntimeConfig::default(),
            Arc::new(Unused),
        )
    }
    /// One worker activity of `assignment`, recorded through the fleet's progress path.
    async fn worker_event(state: &AppState, goal: &str, revision: &Value, assignment: &str) {
        supervisor(state)
            .record_progress(
                &json!({"goal_id":goal,"assignment_id":assignment,"goal_revision":revision,"worktree_id":"cp-impl-projection"}),
                "tool.run",
                "implementor",
                json!({"program":"cargo","args":["test"]}),
            )
            .await
            .unwrap();
    }
    fn row<'a>(view: &'a Value, kind: &str, key: &str, id: &str) -> &'a Value {
        view[kind]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row[key] == id)
            .unwrap_or_else(|| panic!("{kind} has no {id}"))
    }
    fn ids(goal: &Value) -> Vec<String> {
        goal["activity"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["id"].as_str().expect("activity id").to_owned())
            .collect()
    }
    /// A planner event, then a worker event of one assignment, on one goal: the goal as the
    /// browser receives it, the assignment id and the planner entry.
    async fn planner_then_worker() -> (Value, String, Value) {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "lanes").await;
        let repo = repository(&state, &temp.path().join("lanes"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:worker").await;
        let planner = planner_event(
            &state,
            &goal,
            &revision,
            "planner.intent",
            "2026-10-06T10:00:00Z",
        )
        .await;
        worker_event(&state, &goal, &revision, &assignment).await;
        let mut body = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        let view = data(&frame(&mut body).await);
        (
            row(&view, "goals", "goal_id", &goal).clone(),
            assignment,
            planner,
        )
    }

    #[tokio::test]
    async fn planner_activity_is_the_planner_event() {
        let (goal, _, planner) = planner_then_worker().await;
        assert_eq!(goal["planner_activity"]["id"], planner["id"]);
        assert_eq!(goal["planner_activity"]["action"], "planner.intent");
        assert_eq!(goal["planner_activity"]["role"], "planner");
        assert_eq!(
            goal["planner_activity"]["detail"],
            "planner.intent observed"
        );
        // `last_activity` keeps its meaning: the newest entry of any role.
        assert_eq!(goal["last_activity"]["action"], "tool.run");
    }

    #[tokio::test]
    async fn worker_event_is_listed_under_its_assignment() {
        let (goal, assignment, planner) = planner_then_worker().await;
        let worker = &goal["fleet"][&assignment];
        assert_eq!(worker["action"], "tool.run");
        assert_eq!(worker["role"], "implementor");
        assert_eq!(worker["detail"], "cargo");
        let id = worker["id"].as_str().expect("worker entry id");
        assert_eq!(goal["activity"][0]["id"], id);
        assert_ne!(goal["planner_activity"]["id"], id);
        assert_eq!(goal["planner_activity"]["id"], planner["id"]);
    }

    #[tokio::test]
    async fn model_wait_is_projected() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "model-wait").await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let mut body = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        let mut waiting = async || {
            let view = data(&frame(&mut body).await);
            row(&view, "goals", "goal_id", &goal)["waiting"].clone()
        };
        assert_eq!(waiting().await, Value::Null);
        let entry = |action: &str, role: &str, detail: &str, at: &str| json!({"id":uuid::Uuid::new_v4().to_string(),"action":action,"role":role,"detail":detail,"status":"running","at":at,"worktree":"tree","goal_revision":revision});
        // engine.rs `Planning::respond` records these around every model call it makes.
        record_planner(
            &state,
            &goal,
            &revision,
            entry(
                "model.requested",
                "planner",
                "Waiting for planner response from planner-model",
                "2026-10-06T10:00:00Z",
            ),
        )
        .await;
        let planner_wait =
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"});
        assert_eq!(waiting().await, planner_wait);
        record_planner(
            &state,
            &goal,
            &revision,
            entry(
                "loom.event",
                "planner",
                "planner model turn started",
                "2026-10-06T10:00:04Z",
            ),
        )
        .await;
        assert_eq!(waiting().await, planner_wait);
        record_planner(
            &state,
            &goal,
            &revision,
            entry(
                "model.completed",
                "planner",
                "planner response received",
                "2026-10-06T10:00:09Z",
            ),
        )
        .await;
        assert_eq!(waiting().await, Value::Null);
        // The plan critic waits for the reviewer model; a failed call also ends its wait.
        record_planner(
            &state,
            &goal,
            &revision,
            entry(
                "model.requested",
                "critic",
                "Waiting for critic response from reviewer-model",
                "2026-10-06T10:00:10Z",
            ),
        )
        .await;
        assert_eq!(
            waiting().await,
            json!({"role":"critic","model":"reviewer-model","since":"2026-10-06T10:00:10Z"})
        );
        record_planner(
            &state,
            &goal,
            &revision,
            entry(
                "model.failed",
                "critic",
                "critic: provider unavailable",
                "2026-10-06T10:00:12Z",
            ),
        )
        .await;
        assert_eq!(waiting().await, Value::Null);
    }

    #[tokio::test]
    async fn model_wait_outlasts_worker_events_and_its_request_leaving_history() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "long-wait").await;
        let repo = repository(&state, &temp.path().join("long-wait"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:worker").await;
        let console = async || {
            let response = subscribe(&state, "/api/console", None).await;
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            let view: Value = serde_json::from_slice(&bytes).unwrap();
            row(&view, "goals", "goal_id", &goal)["waiting"].clone()
        };
        record_planner(&state, &goal, &revision, json!({"id":"request","action":"model.requested","role":"planner","detail":"Waiting for planner response from planner-model","status":"running","at":"2026-10-06T10:00:00Z","worktree":"tree","goal_revision":revision})).await;
        // A worker's activity does not end the planner's wait.
        worker_event(&state, &goal, &revision, &assignment).await;
        assert_eq!(
            console().await,
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"})
        );
        // Streamed model events push the request out of the retained history (64 entries);
        // the call is still in progress, and the journal still knows when it started.
        for index in 0..70 {
            record_planner(&state, &goal, &revision, json!({"id":format!("stream-{index}"),"action":"loom.event","role":"planner","detail":format!("Receiving model response ({index} streamed events)"),"status":"running","at":"2026-10-06T10:01:00Z","worktree":"tree","goal_revision":revision})).await;
        }
        assert_eq!(
            console().await,
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"})
        );
        // A paused goal waits for nothing.
        state
            .command("PauseGoal", json!({"goal_id":goal}))
            .await
            .unwrap();
        assert_eq!(console().await, Value::Null);
    }

    #[tokio::test]
    async fn projection_carries_server_clock() {
        use time::{OffsetDateTime, format_description::well_known::Rfc3339};
        let (temp, state) = fixture().await;
        let within = |frame: &str, before: OffsetDateTime| {
            let clock = data(frame)["server_time"]
                .as_str()
                .expect("server_time")
                .to_owned();
            let clock = OffsetDateTime::parse(&clock, &Rfc3339).unwrap();
            // Whole milliseconds: the clock may round down below `before`.
            assert!(
                clock >= before - time::Duration::milliseconds(1),
                "{clock} < {before}"
            );
            assert!(
                clock <= OffsetDateTime::now_utc(),
                "{clock} is in the future"
            );
        };
        let before = OffsetDateTime::now_utc();
        let mut body = subscribe(&state, "/events", None).await.into_body();
        within(&frame(&mut body).await, before);
        let before = OffsetDateTime::now_utc();
        let ws = workspace(&state, temp.path(), "clocked").await;
        within(&frame(&mut body).await, before);
        let before = OffsetDateTime::now_utc();
        let mut scoped = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        within(&frame(&mut scoped).await, before);
        let before = OffsetDateTime::now_utc();
        goal(&state, &ws, "clocked-goal").await;
        within(&frame(&mut scoped).await, before);
    }

    #[tokio::test]
    async fn recorded_acceptance_reaches_the_browser() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "accepted").await;
        let (goal, _) = running_goal(&state, &ws).await;
        let mut body = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        let running = data(&frame(&mut body).await);
        assert_eq!(
            row(&running, "goals", "goal_id", &goal)["acceptance_recorded"],
            false
        );
        let receipt = json!({"kind":"goal_acceptance","review":"ACCEPTANCE-RECEIPT-PRIVATE"});
        supervise(
            &state,
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt.to_string()}),
        )
        .await;
        let update = frame(&mut body).await;
        assert!(!update.contains("ACCEPTANCE-RECEIPT-PRIVATE"));
        assert!(!update.contains("receipt"));
        let view = data(&update);
        let satisfied = row(&view, "goals", "goal_id", &goal);
        assert_eq!(satisfied["state"], "Satisfied");
        assert_eq!(satisfied["acceptance_recorded"], true);
    }

    #[tokio::test]
    async fn merge_time_reaches_the_browser() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "merged").await;
        let repo = repository(&state, &temp.path().join("merged"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:merged").await;
        let mut body = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        let queued_view = data(&frame(&mut body).await);
        assert_eq!(
            row(&queued_view, "assignments", "assignment_id", &assignment)["merged_at"],
            Value::Null
        );
        let with = |mut body: Value| {
            body["assignment_id"] = json!(assignment);
            body
        };
        for (command, body) in [
            (
                "ClaimAssignment",
                json!({"worktree_id":"tree","implementor_run":"implementor-run","base_revision":"base"}),
            ),
            (
                "ReviewAssignment",
                json!({"candidate":"candidate","test_revision":"candidate"}),
            ),
            (
                "ReadyAssignment",
                json!({"reviewer_run":"reviewer-run","review_revision":"candidate"}),
            ),
        ] {
            supervise(&state, command, with(body)).await;
        }
        let prepared = supervise(
            &state,
            "PreparePublication",
            with(json!({"candidate":"candidate","target":"main","expected_base":"base"})),
        )
        .await;
        let publication = prepared["published"][0]["payload"]["publication_id"].clone();
        supervise(&state, "MergeAssignment", with(json!({}))).await;
        // fleet.rs `observe_merge` writes this receipt shape.
        let receipt = json!({"kind":"git_merge_observation","operation_id":publication,"candidate":"candidate","expected_base":"base","target":"main","observed_head":"candidate","origin":"MERGE-RECEIPT-PRIVATE","observed_at":"2026-10-06T11:22:33Z"}).to_string();
        supervise(
            &state,
            "ConfirmPublication",
            json!({"publication_id":publication,"receipt":receipt}),
        )
        .await;
        supervise(
            &state,
            "CompleteAssignment",
            with(json!({"merge_receipt":receipt})),
        )
        .await;
        // Bursts coalesce: read until the committed merge reaches the stream.
        let update = loop {
            let update = frame(&mut body).await;
            if row(&data(&update), "assignments", "assignment_id", &assignment)["state"] == "Merged"
            {
                break update;
            }
        };
        assert!(!update.contains("MERGE-RECEIPT-PRIVATE"));
        assert!(!update.contains("receipt"));
        let view = data(&update);
        assert_eq!(
            row(&view, "assignments", "assignment_id", &assignment)["merged_at"],
            "2026-10-06T11:22:33Z"
        );
    }

    #[tokio::test]
    async fn activity_entries_have_distinct_ids() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "distinct").await;
        let repo = repository(&state, &temp.path().join("distinct"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:distinct").await;
        let second = "2026-10-06T10:00:00Z";
        // Recorded in one second, each with the id the runtime gives it ...
        planner_event(&state, &goal, &revision, "planner.intent", second).await;
        planner_event(&state, &goal, &revision, "planner.observation", second).await;
        // ... and recorded in one second without one, as entries were before ids existed.
        for action in ["tool.run", "tool.run.completed"] {
            record_planner(&state, &goal, &revision, json!({"action":action,"role":"implementor","status":"running","detail":"same second","at":second,"assignment_id":assignment,"goal_revision":revision})).await;
        }
        // Through the fleet's own path, twice in quick succession.
        worker_event(&state, &goal, &revision, &assignment).await;
        worker_event(&state, &goal, &revision, &assignment).await;
        let mut body = subscribe(&state, &format!("/workspaces/{ws}/events"), None)
            .await
            .into_body();
        let view = data(&frame(&mut body).await);
        let ids = ids(row(&view, "goals", "goal_id", &goal));
        assert_eq!(ids.len(), 6);
        assert!(ids.iter().all(|id| !id.is_empty()));
        let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(distinct.len(), 6, "{ids:?}");
    }

    #[tokio::test]
    async fn activity_ids_are_stable_across_projections() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "stable").await;
        let repo = repository(&state, &temp.path().join("stable"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:stable").await;
        let at = "2026-10-06T10:00:00Z";
        let planner = planner_event(&state, &goal, &revision, "planner.intent", at).await;
        // Entries without a recorded id, one recorded twice with identical content.
        let unnamed = |action: &str| json!({"action":action,"role":"planner","status":"running","detail":"unnamed","at":at,"goal_revision":revision});
        for action in [
            "planning.Planning",
            "planner.observation",
            "planning.Planning",
        ] {
            record_planner(&state, &goal, &revision, unnamed(action)).await;
        }
        worker_event(&state, &goal, &revision, &assignment).await;
        let path = format!("/workspaces/{ws}/events");
        let mut first = subscribe(&state, &path, None).await.into_body();
        let one = data(&frame(&mut first).await);
        let mut second = subscribe(&state, &path, None).await.into_body();
        let two = data(&frame(&mut second).await);
        let (one, two) = (
            row(&one, "goals", "goal_id", &goal).clone(),
            row(&two, "goals", "goal_id", &goal).clone(),
        );
        let before = ids(&one);
        assert_eq!(before.len(), 5);
        assert_eq!(before, ids(&two));
        let distinct: std::collections::BTreeSet<_> = before.iter().collect();
        assert_eq!(distinct.len(), 5, "{before:?}");
        assert_eq!(before[4], planner["id"].as_str().unwrap());
        // The same entry carries the same id wherever the projection shows it.
        assert_eq!(one["last_activity"]["id"], one["activity"][0]["id"]);
        assert_eq!(one["fleet"][&assignment]["id"], one["activity"][0]["id"]);
        assert_eq!(one["planner_activity"]["id"], one["activity"][1]["id"]);
        planner_event(&state, &goal, &revision, "planner.accept-story", at).await;
        let three = data(&frame(&mut first).await);
        let after = ids(row(&three, "goals", "goal_id", &goal));
        assert_eq!(after.len(), 6);
        assert_eq!(after[1..], before[..]);
    }

    /// The browser's `/api/console` projection.
    async fn console_view(state: &AppState) -> Value {
        let response = subscribe(state, "/api/console", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }
    /// The `at` of the newest projected entry with `action`.
    fn requested_at(goal: &Value, action: &str) -> Value {
        goal["activity"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["action"] == action)
            .unwrap_or_else(|| panic!("no {action} listed"))["at"]
            .clone()
    }

    #[tokio::test]
    async fn planner_activity_survives_worker_activity_and_restart_replay() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "replayed").await;
        let repo = repository(&state, &temp.path().join("replayed"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:replayed").await;
        let planner = planner_event(
            &state,
            &goal,
            &revision,
            "planning.Queued",
            "2026-10-06T10:00:00Z",
        )
        .await;
        // More worker entries than the goal's retained history (64) holds.
        for _ in 0..70 {
            worker_event(&state, &goal, &revision, &assignment).await;
        }
        let check = |view: Value| {
            let shown = row(&view, "goals", "goal_id", &goal);
            assert!(
                shown["activity"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|entry| entry["id"] != planner["id"])
            );
            assert_eq!(shown["planner_activity"]["id"], planner["id"]);
            assert_eq!(shown["planner_activity"]["action"], "planning.Queued");
        };
        check(console_view(&state).await);
        drop(state);
        // Reopening replays the recorded decisions; nothing else is stored.
        let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
        let state = AppState::new(
            Arc::new(Mutex::new(reopened)),
            "127.0.0.1:8787".parse().unwrap(),
            Arc::new(Notify::new()),
        );
        check(console_view(&state).await);
    }

    #[tokio::test]
    async fn assignment_model_waits_are_projected() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "assignment-wait").await;
        let repo = repository(&state, &temp.path().join("assignment-wait"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:waits").await;
        let with = |mut body: Value| {
            body["assignment_id"] = json!(assignment);
            body
        };
        supervise(&state, "ClaimAssignment", with(json!({"worktree_id":"tree","implementor_run":"implementor-run","base_revision":"base"}))).await;
        let fleet = supervisor(&state);
        let worker = json!({"goal_id":goal,"assignment_id":assignment,"goal_revision":revision,"worktree_id":"tree"});
        // The fleet's own progress path, with the actions and roles fleet.rs records.
        let record = async |action: &str, role: &str| {
            fleet
                .record_progress(&worker, action, role, json!({"execution_context":"run"}))
                .await
                .unwrap();
        };
        let waits = async || {
            let view = console_view(&state).await;
            let shown = row(&view, "goals", "goal_id", &goal).clone();
            let waiting =
                row(&view, "assignments", "assignment_id", &assignment)["waiting"].clone();
            (shown, waiting)
        };
        record("model.request", "implementor").await;
        record("loom.event", "runtime").await;
        let (shown, waiting) = waits().await;
        assert_eq!(
            waiting,
            json!({"role":"implementor","model":"implementor-model","since":requested_at(&shown, "model.request")})
        );
        // A worker call is the assignment's wait, not the goal's.
        assert_eq!(shown["waiting"], Value::Null);
        // The next entry of the assignment that is not a stream event ends the call.
        record("tool.run", "implementor").await;
        assert_eq!(waits().await.1, Value::Null);
        // A call whose request streamed out of the retained history is still open, and keeps
        // its start time.
        record("model.request", "implementor").await;
        let started = waits().await.0["fleet"][&assignment]["at"].clone();
        assert!(started.is_string());
        for _ in 0..70 {
            record("loom.event", "runtime").await;
        }
        assert_eq!(
            waits().await.1,
            json!({"role":"implementor","model":"implementor-model","since":started})
        );
        record("checks.run", "host").await;
        assert_eq!(waits().await.1, Value::Null);
        supervise(
            &state,
            "ReviewAssignment",
            with(json!({"candidate":"candidate","test_revision":"candidate"})),
        )
        .await;
        record("review.request", "reviewer").await;
        let (shown, waiting) = waits().await;
        assert_eq!(
            waiting,
            json!({"role":"reviewer","model":"reviewer-model","since":requested_at(&shown, "review.request")})
        );
        // Leaving the state the call belongs to ends it, even before another entry.
        supervise(
            &state,
            "ReadyAssignment",
            with(json!({"reviewer_run":"reviewer-run","review_revision":"candidate"})),
        )
        .await;
        assert_eq!(waits().await.1, Value::Null);
    }

    #[tokio::test]
    async fn model_wait_ends_when_the_goal_revision_changes() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "revised").await;
        let repo = repository(&state, &temp.path().join("revised"), &ws).await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let assignment = queued(&state, &goal, &revision, &repo, "story:revised").await;
        supervise(&state, "ClaimAssignment", json!({"assignment_id":assignment,"worktree_id":"tree","implementor_run":"implementor-run","base_revision":"base"})).await;
        record_planner(&state, &goal, &revision, json!({"id":"request","action":"model.requested","role":"planner","detail":"Waiting for planner response from planner-model","status":"running","at":"2026-10-06T10:00:00Z","worktree":"tree","goal_revision":revision})).await;
        supervisor(&state)
            .record_progress(
                &json!({"goal_id":goal,"assignment_id":assignment,"goal_revision":revision,"worktree_id":"tree"}),
                "model.request",
                "implementor",
                json!({"execution_context":"run"}),
            )
            .await
            .unwrap();
        let view = console_view(&state).await;
        assert_eq!(
            row(&view, "goals", "goal_id", &goal)["waiting"],
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"})
        );
        assert_eq!(
            row(&view, "assignments", "assignment_id", &assignment)["waiting"]["role"],
            "implementor"
        );
        // An operator edit bumps the revision while both calls are open.
        state.command("UpdateGoal", json!({"goal_id":goal,"objective":"projection goal","acceptance":"verified","max_workers":2,"max_attempts":2,"max_minutes":20,"planner_model":"planner-model","implementor_model":"implementor-model","reviewer_model":"reviewer-model","merge_authority":true})).await.unwrap();
        let view = console_view(&state).await;
        let shown = row(&view, "goals", "goal_id", &goal);
        assert_eq!(shown["state"], "Running");
        assert_ne!(shown["revision"], revision);
        assert_eq!(shown["waiting"], Value::Null);
        assert_eq!(
            row(&view, "assignments", "assignment_id", &assignment)["waiting"],
            Value::Null
        );
    }

    #[tokio::test]
    async fn planner_wait_does_not_survive_a_restart_and_a_new_call_opens() {
        let (temp, state) = fixture().await;
        let ws = workspace(&state, temp.path(), "reopened").await;
        let (goal, revision) = running_goal(&state, &ws).await;
        let request = |id: &str, at: &str| json!({"id":id,"action":"model.requested","role":"planner","detail":"Waiting for planner response from planner-model","status":"running","at":at,"worktree":"tree","goal_revision":revision});
        record_planner(
            &state,
            &goal,
            &revision,
            request("before", "2026-10-06T10:00:00Z"),
        )
        .await;
        let waiting = |view: Value| row(&view, "goals", "goal_id", &goal)["waiting"].clone();
        assert_eq!(
            waiting(console_view(&state).await),
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"})
        );
        drop(state);
        let reopened = Store::open(temp.path().join("state.sqlite")).await.unwrap();
        let state = AppState::new(
            Arc::new(Mutex::new(reopened)),
            "127.0.0.1:8787".parse().unwrap(),
            Arc::new(Notify::new()),
        );
        // The process that held the call is gone; the goal is still Running.
        let view = console_view(&state).await;
        assert_eq!(row(&view, "goals", "goal_id", &goal)["state"], "Running");
        assert_eq!(waiting(view), Value::Null);
        // A call this process requests is open, with the same fixed-time records as before.
        record_planner(
            &state,
            &goal,
            &revision,
            request("after", "2026-10-06T10:00:00Z"),
        )
        .await;
        assert_eq!(
            waiting(console_view(&state).await),
            json!({"role":"planner","model":"planner-model","since":"2026-10-06T10:00:00Z"})
        );
    }
}
