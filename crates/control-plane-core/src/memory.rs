//! The generated infallible ports are pure memory. Cloning `Memory` is a deep transaction stage.
use controlplane_model::{behaviour::*, host::*, obligation::UnmetObligation, primitives::Uuid};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub(crate) struct Memory {
    pub registration_receipts: BTreeMap<String, serde_json::Value>,
    pub assignment_configs: BTreeMap<String, RepositoryRegistrationData>,
    pub assignments: BTreeMap<String, AssignmentSnapshot>,
    pub goals: BTreeMap<String, GoalSnapshot>,
    pub publications: BTreeMap<String, PublicationIntentSnapshot>,
    pub repositories: BTreeMap<String, RepositoryRegistrationSnapshot>,
    pub workspaces: BTreeMap<String, WorkspaceSnapshot>,
    pub directories: BTreeMap<String, WorkspaceDirectorySnapshot>,
    /// Progress journals of goals with bounded receipts, rebuilt from recorded decisions.
    pub journals: BTreeMap<String, Journal>,
    /// Bounded progress records journaled so far, counted again by every replay: the
    /// sequence number of the newest one.
    pub progress_records: u64,
    pub ids: Vec<String>,
    pub replay: bool,
    pub cursor: usize,
}

impl Memory {
    /// The generated rows alone: what a view reads. Host bookkeeping is not cloned per query.
    pub fn rows(&self) -> Self {
        Self {
            assignments: self.assignments.clone(),
            goals: self.goals.clone(),
            publications: self.publications.clone(),
            repositories: self.repositories.clone(),
            workspaces: self.workspaces.clone(),
            directories: self.directories.clone(),
            ..Self::default()
        }
    }
}

/// Top-level key and value marking a bounded planning receipt.
pub(crate) const RECEIPT_FORMAT: &str = "receipt_format";
pub(crate) const BOUNDED_RECEIPT: u64 = 2;
/// Receipt keys whose newest value the journal keeps, recorded only when they change.
pub(crate) const LATEST: [&str; 2] = ["planner", "acceptance"];
/// Activities a goal's journal keeps; readers show the newest 24.
const HISTORY: usize = 64;
/// Characters of a text activity detail kept: what the console shows.
const DETAIL_CHARS: usize = 560;
/// Serialized bytes of a structured detail kept as it is.
const DETAIL_BYTES: usize = 1024;
/// Bytes kept of every activity field and of every string a bounded detail keeps.
const FIELD_BYTES: usize = 240;
/// Items kept of an array in a bounded detail: the arguments the dashboard shows.
const DETAIL_ITEMS: usize = 12;
const ACTIVITY_FIELDS: [&str; 9] = [
    "id",
    "at",
    "action",
    "role",
    "status",
    "worktree",
    "assignment_id",
    "goal_revision",
    "detail",
];
/// Detail members the console and dashboard show; a bounded detail keeps them.
const SHOWN: [&str; 9] = [
    "summary",
    "program",
    "args",
    "command",
    "path",
    "reason",
    "worktree",
    "target",
    "candidate",
];
/// Bookkeeping a bounded detail adds: the original size and the members it left out.
const BOUNDED: &str = "_bounded";

/// The progress history of a goal whose planning receipt is bounded.
///
/// A bounded receipt (`"receipt_format": 2`) stores its newest activity and, only when they
/// change, the planner evidence and the acceptance record. Replaying those decisions rebuilds
/// what older receipts repeated in every decision: the newest activities, each assignment's
/// newest activity and the newest planner evidence and acceptance record. It also keeps the
/// newest activity without an `assignment_id` (the planner's own), which worker activity can
/// push out of the newest activities, and each lane's newest [`Step`]. Nothing here is stored
/// on its own; replay derives it from the recorded activities, and `Store::activity_history`
/// reads it.
#[derive(Clone, Default)]
pub(crate) struct Journal {
    /// The stored receipt this journal belongs to; for any other receipt it is stale.
    receipt: Arc<str>,
    activity: VecDeque<Arc<Value>>,
    fleet: BTreeMap<String, Arc<Value>>,
    /// The newest activity without an `assignment_id`, as `fleet` keeps each assignment's.
    planner_activity: Option<Arc<Value>>,
    /// The planner lane's newest step: activities without an `assignment_id`.
    planner_step: Option<Step>,
    /// Each assignment's newest step.
    fleet_steps: BTreeMap<String, Step>,
    latest: [Option<Arc<Value>>; 2],
}

/// A lane's newest activity other than a streamed `loom.event`, kept however many stream
/// events follow, with the number of the progress record that recorded it
/// (`Memory::progress_records`). A model call opens with such an activity and ends with the
/// lane's next one; the stream events between them only show it is running.
#[derive(Clone)]
struct Step {
    recorded: u64,
    entry: Arc<Value>,
}

impl Step {
    fn shown(&self) -> Value {
        json!({"recorded":self.recorded,"entry":*self.entry})
    }
}

/// Whether an activity is the planner's own: the fleet records every worker activity with
/// its `assignment_id`, and the planner's progress path never does.
fn planner_activity(event: &Value) -> bool {
    event.is_object() && event.get("assignment_id").and_then(Value::as_str).is_none()
}

impl Journal {
    /// The journal a receipt implies when it was not recorded bounded: an older receipt
    /// that repeats its own history, an empty one, or one recorded through another path.
    /// It has no steps: such a receipt was written before bounding, by an earlier process.
    pub fn seed(receipt: &str) -> Self {
        let mut journal = Self {
            receipt: receipt.into(),
            ..Self::default()
        };
        let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(receipt) else {
            return journal;
        };
        if let Some(history) = fields.get("activity").and_then(Value::as_array) {
            journal.activity = history[history.len().saturating_sub(HISTORY)..]
                .iter()
                .map(|event| Arc::new(bounded_activity(event)))
                .collect();
            journal.planner_activity = history
                .iter()
                .rev()
                .find(|event| planner_activity(event))
                .map(|event| Arc::new(bounded_activity(event)));
        }
        if let Some(fleet) = fields.get("fleet").and_then(Value::as_object) {
            journal.fleet = fleet
                .iter()
                .map(|(id, event)| (id.clone(), Arc::new(bounded_activity(event))))
                .collect();
        }
        journal.latest = LATEST.map(|key| fields.get(key).cloned().map(Arc::new));
        journal
    }

    /// The journal of `goal` while its stored receipt is `stored`.
    pub fn of(memory: &Memory, goal: &str, stored: &str) -> Self {
        match memory.journals.get(goal) {
            Some(journal) if &*journal.receipt == stored => journal.clone(),
            _ => Self::seed(stored),
        }
    }

    /// Take the journal of `goal` while its stored receipt is `previous`.
    pub fn take(memory: &mut Memory, goal: &str, previous: &str) -> Self {
        match memory.journals.remove(goal) {
            Some(journal) if &*journal.receipt == previous => journal,
            _ => Self::seed(previous),
        }
    }

    /// The newest value of one of the [`LATEST`] keys.
    pub fn latest(&self, key: &str) -> Option<&Value> {
        let index = LATEST.iter().position(|name| *name == key)?;
        self.latest[index].as_deref()
    }

    /// Advance to a newly recorded bounded receipt, progress record number `recorded`.
    pub fn record(&mut self, receipt: &str, fields: &Map<String, Value>, recorded: u64) {
        if let Some(activity) = fields
            .get("last_activity")
            .filter(|event| event.is_object())
            && self.activity.back().map(|event| &**event) != Some(activity)
        {
            let event = Arc::new(activity.clone());
            let step =
                (activity.get("action").and_then(Value::as_str) != Some("loom.event")).then(|| {
                    Step {
                        recorded,
                        entry: event.clone(),
                    }
                });
            if let Some(assignment) = activity.get("assignment_id").and_then(Value::as_str) {
                self.fleet.insert(assignment.to_owned(), event.clone());
                if let Some(step) = step {
                    self.fleet_steps.insert(assignment.to_owned(), step);
                }
            } else {
                self.planner_activity = Some(event.clone());
                if step.is_some() {
                    self.planner_step = step;
                }
            }
            self.activity.push_back(event);
            while self.activity.len() > HISTORY {
                self.activity.pop_front();
            }
        }
        for (index, key) in LATEST.iter().enumerate() {
            if let Some(value) = fields.get(*key) {
                self.latest[index] = Some(Arc::new(value.clone()));
            }
        }
        self.receipt = receipt.into();
    }

    /// The history as readers see it: activities oldest first, each assignment's newest
    /// activity, the newest activity without an assignment (`planner_activity`, or null), each
    /// lane's newest step as `{recorded, entry}` (`planner_step`, or null, and `fleet_steps`
    /// by assignment), and the newest planner evidence and acceptance record.
    pub fn history(&self) -> Value {
        let mut history = Map::new();
        history.insert(
            "activity".into(),
            Value::Array(
                self.activity
                    .iter()
                    .map(|event| (**event).clone())
                    .collect(),
            ),
        );
        history.insert(
            "fleet".into(),
            Value::Object(
                self.fleet
                    .iter()
                    .map(|(id, event)| (id.clone(), (**event).clone()))
                    .collect(),
            ),
        );
        history.insert(
            "planner_activity".into(),
            self.planner_activity
                .as_deref()
                .cloned()
                .unwrap_or(Value::Null),
        );
        history.insert(
            "planner_step".into(),
            self.planner_step.as_ref().map_or(Value::Null, Step::shown),
        );
        history.insert(
            "fleet_steps".into(),
            Value::Object(
                self.fleet_steps
                    .iter()
                    .map(|(id, step)| (id.clone(), step.shown()))
                    .collect(),
            ),
        );
        for (index, key) in LATEST.iter().enumerate() {
            history.insert(
                (*key).into(),
                self.latest[index]
                    .as_deref()
                    .cloned()
                    .unwrap_or(Value::Null),
            );
        }
        Value::Object(history)
    }
}

/// The top-level fields of a bounded receipt; `None` for every other receipt.
pub(crate) fn bounded_receipt(receipt: &str) -> Option<Map<String, Value>> {
    // Older receipts are never parsed here: none carries the marker at its top level.
    if !receipt.contains("\"receipt_format\":2") {
        return None;
    }
    match serde_json::from_str::<Value>(receipt) {
        Ok(Value::Object(fields))
            if fields.get(RECEIPT_FORMAT) == Some(&json!(BOUNDED_RECEIPT)) =>
        {
            Some(fields)
        }
        _ => None,
    }
}

/// One activity as it is recorded: the fields readers use, each bounded.
pub(crate) fn bounded_activity(activity: &Value) -> Value {
    let Some(fields) = activity.as_object() else {
        return activity.clone();
    };
    let mut bounded = Map::new();
    for key in ACTIVITY_FIELDS {
        let Some(value) = fields.get(key) else {
            continue;
        };
        let value = match value {
            _ if key == "detail" => bounded_detail(value),
            other => bounded_member(other),
        };
        bounded.insert(key.into(), value);
    }
    Value::Object(bounded)
}

/// Text keeps what the console shows, and a small structured detail stays as it is. A larger
/// one keeps every member bounded: strings to 240 bytes, arrays to their first 12 items,
/// nested values to the start of their JSON text. If that is still over 1 KiB, it keeps the
/// members readers show and names the ones it left out under `_bounded`.
fn bounded_detail(detail: &Value) -> Value {
    let Value::Object(members) = detail else {
        return match detail {
            Value::String(text) => Value::from(text.chars().take(DETAIL_CHARS).collect::<String>()),
            other => bounded_member(other),
        };
    };
    let size = detail.to_string().len();
    if size <= DETAIL_BYTES {
        return detail.clone();
    }
    let mut kept = members
        .iter()
        .filter(|(key, _)| key.as_str() != BOUNDED)
        .map(|(key, value)| (prefix(key, FIELD_BYTES).to_owned(), bounded_member(value)))
        .collect::<Map<String, Value>>();
    // A detail bounded before keeps its first account; the bound is applied once.
    let earlier = members.get(BOUNDED).and_then(Value::as_object);
    let mut dropped = earlier
        .and_then(|account| account.get("dropped"))
        .and_then(Value::as_array)
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut count = earlier
        .and_then(|account| account.get("dropped_count"))
        .and_then(Value::as_u64)
        .unwrap_or(dropped.len() as u64);
    if Value::Object(kept.clone()).to_string().len() > DETAIL_BYTES {
        let names = kept
            .keys()
            .filter(|key| !SHOWN.contains(&key.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for name in names {
            kept.remove(&name);
            count += 1;
            dropped.push(name);
        }
    }
    let mut account = Map::new();
    account.insert(
        "bytes".into(),
        earlier
            .and_then(|account| account.get("bytes"))
            .and_then(Value::as_u64)
            .map_or_else(|| json!(size), |bytes| json!(bytes)),
    );
    if count > 0 {
        dropped.truncate(DETAIL_ITEMS);
        account.insert(
            "dropped".into(),
            Value::Array(
                dropped
                    .iter()
                    .map(|name| Value::from(prefix(name, 64)))
                    .collect(),
            ),
        );
        account.insert("dropped_count".into(), json!(count));
    }
    kept.insert(BOUNDED.into(), Value::Object(account));
    Value::Object(kept)
}

/// One member of a bounded detail.
fn bounded_member(value: &Value) -> Value {
    match value {
        Value::String(text) => Value::from(prefix(text, FIELD_BYTES)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(DETAIL_ITEMS)
                .map(|item| match item {
                    Value::Array(_) | Value::Object(_) => {
                        Value::from(prefix(&item.to_string(), FIELD_BYTES))
                    }
                    other => bounded_member(other),
                })
                .collect(),
        ),
        Value::Object(_) => Value::from(prefix(&value.to_string(), FIELD_BYTES)),
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
    }
}

/// Text cut to at most `limit` bytes, ending in a marker that says how much was kept.
pub fn bounded_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let marker = |kept: usize| format!(" [truncated: {kept} of {} bytes]", text.len());
    let kept = prefix(text, limit.saturating_sub(marker(limit).len())).len();
    format!("{}{}", &text[..kept], marker(kept))
}

fn prefix(text: &str, bytes: usize) -> &str {
    if text.len() <= bytes {
        return text;
    }
    let mut end = bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[derive(Clone)]
pub(crate) struct Ports(pub Arc<Mutex<Memory>>);
macro_rules! storage {
    ($trait:ident, $snapshot:ident, $rows:ident, $id:ident) => {
        impl $trait for Ports {
            fn get(&self, id: &Uuid) -> Option<$snapshot> {
                self.0
                    .lock()
                    .expect("pure memory lock poisoned")
                    .$rows
                    .get(&id.0)
                    .cloned()
            }
            fn put(&mut self, row: $snapshot) {
                self.0
                    .lock()
                    .expect("pure memory lock poisoned")
                    .$rows
                    .insert(row.data.$id.0.clone(), row);
            }
            fn delete(&mut self, id: &Uuid) {
                self.0
                    .lock()
                    .expect("pure memory lock poisoned")
                    .$rows
                    .remove(&id.0);
            }
            fn list(&self) -> Vec<$snapshot> {
                self.0
                    .lock()
                    .expect("pure memory lock poisoned")
                    .$rows
                    .values()
                    .cloned()
                    .collect()
            }
        }
    };
}
storage!(
    AssignmentStorage,
    AssignmentSnapshot,
    assignments,
    assignment_id
);
storage!(GoalStorage, GoalSnapshot, goals, goal_id);
storage!(
    WorkspaceDirectoryStorage,
    WorkspaceDirectorySnapshot,
    directories,
    directory_id
);
storage!(
    PublicationIntentStorage,
    PublicationIntentSnapshot,
    publications,
    publication_id
);
storage!(
    RepositoryRegistrationStorage,
    RepositoryRegistrationSnapshot,
    repositories,
    repository_id
);
storage!(
    WorkspaceStorage,
    WorkspaceSnapshot,
    workspaces,
    workspace_id
);
impl TryContext for Ports {
    fn try_generate_uuid(&mut self) -> Result<Uuid, UnmetObligation> {
        let mut memory = self.0.lock().expect("pure memory lock poisoned");
        let id = if memory.replay {
            memory
                .ids
                .get(memory.cursor)
                .cloned()
                .ok_or(UnmetObligation {
                    capability: "durable UUID replay",
                    source: "recorded UUID sequence exhausted",
                })?
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            memory.ids.push(id.clone());
            id
        };
        memory.cursor += 1;
        Ok(Uuid(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Record `activity` as the newest activity of a bounded receipt, as progress record
    /// number `recorded`.
    fn record_as(journal: &mut Journal, activity: &Value, recorded: u64) {
        let receipt =
            json!({"last_activity":activity,"receipt_format":BOUNDED_RECEIPT}).to_string();
        let fields = bounded_receipt(&receipt).expect("bounded receipt");
        journal.record(&receipt, &fields, recorded);
    }
    fn record(journal: &mut Journal, activity: &Value) {
        record_as(journal, activity, 1);
    }

    #[test]
    fn journal_keeps_the_newest_planner_activity_past_its_history() {
        let planner = json!({"id":"plan","action":"planning.Queued","role":"planner","status":"waiting","at":"2026-10-06T10:00:00Z","goal_revision":1,"detail":"Planning phase: Queued"});
        let worker = |index: usize| json!({"id":format!("stream-{index}"),"assignment_id":"assignment","action":"loom.event","role":"runtime","status":"running","at":"2026-10-06T10:01:00Z","goal_revision":1,"detail":format!("Receiving model response ({index} streamed events)")});
        let mut journal = Journal::default();
        record(&mut journal, &planner);
        for index in 0..HISTORY + 6 {
            record(&mut journal, &worker(index));
        }
        let history = journal.history();
        // The planner entry has left the retained activities; its own slot keeps it, the way
        // `fleet` keeps each assignment's newest activity.
        assert!(
            history["activity"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["id"] != "plan")
        );
        assert_eq!(history["fleet"]["assignment"], worker(HISTORY + 5));
        assert_eq!(history["planner_activity"], planner);
        let newer = json!({"id":"plan-2","action":"planner.intent","role":"planner","status":"running","at":"2026-10-06T10:02:00Z","goal_revision":1,"detail":"Reading the repository"});
        record(&mut journal, &newer);
        record(&mut journal, &worker(HISTORY + 6));
        assert_eq!(journal.history()["planner_activity"], newer);
        // A receipt recorded before bounding carries its own history, and seeds the slot.
        let seeded = Journal::seed(&json!({"activity":[planner,worker(1),worker(2)]}).to_string());
        assert_eq!(seeded.history()["planner_activity"], planner);
        assert_eq!(
            Journal::default().history()["planner_activity"],
            Value::Null
        );
    }

    #[test]
    fn journal_keeps_each_lanes_newest_step_with_its_record_number() {
        let entry = |id: &str, assignment: Option<&str>, action: &str| {
            let mut entry = json!({"id":id,"action":action,"role":"planner","status":"running","at":"2026-10-06T10:00:00Z","goal_revision":1,"detail":id});
            if let Some(assignment) = assignment {
                entry["assignment_id"] = json!(assignment);
                entry["role"] = json!("implementor");
            }
            entry
        };
        let planner_request = entry("planner-request", None, "model.requested");
        let worker_request = entry("worker-request", Some("assignment"), "model.request");
        let mut journal = Journal::default();
        let mut recorded = 0;
        let mut next = |journal: &mut Journal, activity: &Value| {
            recorded += 1;
            record_as(journal, activity, recorded);
        };
        next(&mut journal, &planner_request);
        next(&mut journal, &worker_request);
        // Both calls stream longer than the retained activities reach back.
        for index in 0..HISTORY + 6 {
            let stream = |assignment| entry(&format!("stream-{index}"), assignment, "loom.event");
            next(&mut journal, &stream(Some("assignment")));
            next(&mut journal, &stream(None));
        }
        let history = journal.history();
        assert!(
            history["activity"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["action"] == "loom.event")
        );
        // A lane's step is its newest entry that is not a stream event, with its record number.
        assert_eq!(
            history["planner_step"],
            json!({"recorded":1,"entry":planner_request})
        );
        assert_eq!(
            history["fleet_steps"],
            json!({"assignment":{"recorded":2,"entry":worker_request}})
        );
        let ran = entry("tool-run", Some("assignment"), "tool.run");
        next(&mut journal, &ran);
        let recorded_ran = 2 + 2 * (HISTORY as u64 + 6) + 1;
        assert_eq!(
            journal.history()["fleet_steps"]["assignment"],
            json!({"recorded":recorded_ran,"entry":ran})
        );
        assert_eq!(journal.history()["planner_step"]["entry"], planner_request);
        // A receipt recorded before bounding comes from an earlier process: it opens no step.
        let seeded =
            Journal::seed(&json!({"activity":[planner_request,worker_request]}).to_string());
        assert_eq!(seeded.history()["planner_step"], Value::Null);
        assert_eq!(seeded.history()["fleet_steps"], json!({}));
    }
}
