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
/// Activities a goal's journal keeps; readers show the newest 24.
const HISTORY: usize = 64;
/// Receipt keys a bounded receipt leaves to the journal instead of repeating them.
const JOURNALED: [&str; 3] = ["activity", "fleet", "planner"];
/// Characters of a text activity detail kept: what the console shows.
const DETAIL_CHARS: usize = 560;
/// Serialized bytes of a structured detail kept verbatim; a larger one keeps its summary.
const DETAIL_BYTES: usize = 1024;
/// Bytes kept of every other activity field.
const FIELD_BYTES: usize = 240;
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

/// The progress history of a goal whose planning receipt is bounded.
///
/// A bounded receipt (`"receipt_format": 2`) stores its newest activity and, only when it
/// changed, the planner evidence. Replaying those decisions rebuilds what older receipts
/// repeated in every decision: the newest activities, each assignment's newest activity and
/// the newest planner evidence. Views expand the stored receipt with them, so readers keep
/// the receipt shape they always had. Nothing here is stored on its own.
#[derive(Clone, Default)]
pub(crate) struct Journal {
    /// The stored receipt this journal expands; for any other receipt it is stale.
    receipt: Arc<str>,
    activity: VecDeque<Arc<str>>,
    fleet: BTreeMap<String, Arc<str>>,
    planner: Option<Arc<str>>,
    /// Which of the journaled keys the stored receipt carries itself.
    inline: [bool; 3],
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
                .map(|event| serialized(&bounded_activity(event)))
                .collect();
        }
        if let Some(fleet) = fields.get("fleet").and_then(Value::as_object) {
            journal.fleet = fleet
                .iter()
                .map(|(id, event)| (id.clone(), serialized(&bounded_activity(event))))
                .collect();
        }
        journal.planner = fields.get("planner").map(serialized);
        journal
    }

    /// Take the journal of `goal` while its stored receipt is `previous`.
    pub fn take(memory: &mut Memory, goal: &str, previous: &str) -> Self {
        match memory.journals.remove(goal) {
            Some(journal) if &*journal.receipt == previous => journal,
            _ => Self::seed(previous),
        }
    }

    /// The planner evidence `goal` shows while its stored receipt is `previous`.
    pub fn planner_of(memory: &Memory, goal: &str, previous: &str) -> Option<Arc<str>> {
        match memory.journals.get(goal) {
            Some(journal) if &*journal.receipt == previous => journal.planner.clone(),
            _ => Self::seed(previous).planner,
        }
    }

    /// Advance to a newly recorded bounded receipt.
    pub fn record(&mut self, receipt: &str, fields: &Map<String, Value>) {
        if let Some(activity) = fields
            .get("last_activity")
            .filter(|event| event.is_object())
        {
            let event = serialized(activity);
            if self.activity.back() != Some(&event) {
                if let Some(assignment) = activity.get("assignment_id").and_then(Value::as_str) {
                    self.fleet.insert(assignment.to_owned(), event.clone());
                }
                self.activity.push_back(event);
                while self.activity.len() > HISTORY {
                    self.activity.pop_front();
                }
            }
        }
        if let Some(planner) = fields.get("planner") {
            self.planner = Some(serialized(planner));
        }
        self.inline = JOURNALED.map(|key| fields.contains_key(key));
        self.receipt = receipt.into();
    }

    /// The stored receipt, or that receipt with the history it leaves to the journal.
    pub fn expand(&self, stored: &str) -> Option<String> {
        if stored != &*self.receipt {
            return None;
        }
        let open = stored.trim_end().strip_suffix('}')?;
        let mut text = String::with_capacity(
            stored.len()
                + self
                    .activity
                    .iter()
                    .map(|event| event.len() + 1)
                    .sum::<usize>()
                + self
                    .fleet
                    .values()
                    .map(|event| event.len() + 48)
                    .sum::<usize>()
                + self.planner.as_ref().map_or(0, |planner| planner.len())
                + 64,
        );
        text.push_str(open);
        if !self.inline[0] && !self.activity.is_empty() {
            text.push_str(",\"activity\":[");
            for (index, event) in self.activity.iter().enumerate() {
                if index > 0 {
                    text.push(',');
                }
                text.push_str(event);
            }
            text.push(']');
        }
        if !self.inline[1] && !self.fleet.is_empty() {
            text.push_str(",\"fleet\":{");
            for (index, (assignment, event)) in self.fleet.iter().enumerate() {
                if index > 0 {
                    text.push(',');
                }
                text.push_str(&Value::from(assignment.as_str()).to_string());
                text.push(':');
                text.push_str(event);
            }
            text.push('}');
        }
        if !self.inline[2]
            && let Some(planner) = &self.planner
        {
            text.push_str(",\"planner\":");
            text.push_str(planner);
        }
        text.push('}');
        Some(text)
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
            Value::String(text) => Value::from(prefix(text, FIELD_BYTES)),
            Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
            other => bounded_detail(other),
        };
        bounded.insert(key.into(), value);
    }
    Value::Object(bounded)
}

/// Text keeps what the console shows. A small structured detail stays as it is; a larger one
/// keeps the summary the console would show, its kind and how many bytes were left out.
fn bounded_detail(detail: &Value) -> Value {
    match detail {
        Value::String(text) => Value::from(text.chars().take(DETAIL_CHARS).collect::<String>()),
        Value::Null | Value::Bool(_) | Value::Number(_) => detail.clone(),
        _ => {
            let size = detail.to_string().len();
            if size <= DETAIL_BYTES {
                return detail.clone();
            }
            let summary = detail
                .get("summary")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    ["command", "program", "path", "reason", "candidate"]
                        .into_iter()
                        .filter_map(|key| detail.get(key).and_then(Value::as_str))
                        .map(|part| part.chars().take(180).collect::<String>())
                        .collect::<Vec<_>>()
                        .join(" · ")
                });
            let mut projected = Map::new();
            projected.insert(
                "summary".into(),
                Value::from(summary.chars().take(DETAIL_CHARS).collect::<String>()),
            );
            if let Some(kind) = detail
                .get("kind")
                .or_else(|| detail.get("event").and_then(|event| event.get("kind")))
                .and_then(Value::as_str)
            {
                projected.insert("kind".into(), Value::from(prefix(kind, 80)));
            }
            projected.insert(
                "omitted_bytes".into(),
                detail
                    .get("omitted_bytes")
                    .cloned()
                    .unwrap_or_else(|| json!(size)),
            );
            Value::Object(projected)
        }
    }
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

fn serialized(value: &Value) -> Arc<str> {
    value.to_string().into()
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
