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
/// newest activity and the newest planner evidence and acceptance record. Nothing here is
/// stored on its own; `Store::activity_history` reads it.
#[derive(Clone, Default)]
pub(crate) struct Journal {
    /// The stored receipt this journal belongs to; for any other receipt it is stale.
    receipt: Arc<str>,
    activity: VecDeque<Arc<Value>>,
    fleet: BTreeMap<String, Arc<Value>>,
    latest: [Option<Arc<Value>>; 2],
}

impl Journal {
    /// The journal a receipt implies when it was not recorded bounded: an older receipt
    /// that repeats its own history, an empty one, or one recorded through another path.
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

    /// Advance to a newly recorded bounded receipt.
    pub fn record(&mut self, receipt: &str, fields: &Map<String, Value>) {
        if let Some(activity) = fields
            .get("last_activity")
            .filter(|event| event.is_object())
            && self.activity.back().map(|event| &**event) != Some(activity)
        {
            let event = Arc::new(activity.clone());
            if let Some(assignment) = activity.get("assignment_id").and_then(Value::as_str) {
                self.fleet.insert(assignment.to_owned(), event.clone());
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
    /// activity, and the newest planner evidence and acceptance record.
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
