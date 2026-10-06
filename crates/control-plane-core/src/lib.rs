//! Durable host for ESS-generated behavior. No external effect can precede a successful append.
mod directories;
mod discovery;
mod guards;
mod memory;

use anyhow::{Context, Result, bail, ensure};
pub use controlplane_model::actor::Actor;
use controlplane_model::{
    actor::Caller,
    behaviour::Generated,
    ports::control_plane::ControlPlane,
    server::{control_plane::dispatch, http::Request},
    system::System,
};
pub use discovery::{DiscoveredRepository, DiscoveredWorkspace, discover};
use eventlog_core::{CommandMeta, EventStore, Expected, NewEvent, StreamId, TenantId};
use eventlog_sqlite::SqliteEventStore;
use fs2::FileExt;
use memory::{Journal, Memory, Ports};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::{Arc, Mutex},
};

/// Bytes kept of each planning field other than the receipt when progress is recorded.
const PLANNING_FIELD_BYTES: usize = 4 * 1024;

pub use memory::bounded_text;

/// A single service owns a database; its persisted assignments own repositories across workspaces.
/// The generated actor is selected by trusted Rust code, never decoded from an HTTP request.
pub struct Store {
    log: SqliteEventStore,
    stream: StreamId,
    version: u64,
    committed: tokio::sync::watch::Sender<u64>,
    memory: Memory,
    appended: u64,
    /// See [`Store::replayed_progress`].
    replayed_progress: u64,
    _lock: File,
}

impl Drop for Store {
    fn drop(&mut self) {
        // Explicit unlock also releases a descriptor briefly inherited by a concurrent fork.
        // Closing the descriptor remains the OS fallback if unlock itself fails.
        let _ = FileExt::unlock(&self._lock);
    }
}

#[cfg(test)]
mod notification_tests {
    use super::*;

    #[tokio::test]
    async fn subscribers_observe_only_committed_state_and_replay_position() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("state.sqlite");
        let mut store = Store::open(&path).await.unwrap();
        let mut changed = store.subscribe();
        assert_eq!(*changed.borrow_and_update(), 0);
        store
            .execute(
                "RegisterWorkspace",
                json!({"path":temp.path(),"name":"one"}),
                Actor::Operator,
            )
            .await
            .unwrap();
        assert!(changed.has_changed().unwrap());
        let committed = *changed.borrow_and_update();
        assert!(committed > 0);
        assert_eq!(store.query("WorkspaceList").unwrap()[0]["name"], "one");
        assert!(
            store
                .execute("RecordPlanningProgress", json!({}), Actor::Operator)
                .await
                .is_err()
        );
        assert!(!changed.has_changed().unwrap());

        // Force Eventlog's expected-version refusal: no speculative view or notification
        // may escape if the append fails, even after the generated decision was staged.
        store.version += 100;
        assert!(
            store
                .apply(
                    "RegisterWorkspace",
                    json!({"path":"different","name":"not committed"}),
                    Actor::Operator
                )
                .await
                .is_err()
        );
        assert!(!changed.has_changed().unwrap());
        assert_eq!(
            store
                .query("WorkspaceList")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        drop(store);
        let reopened = Store::open(&path).await.unwrap();
        assert_eq!(*reopened.subscribe().borrow(), committed);
    }
}

/// Durable generated contract, below filesystem and operational admission.
///
/// This is the state-machine adapter used by `Store`, also exposed for ESS conformance.
/// An HTTP server must expose `Store` instead; clients cannot select this boundary.
pub mod contract {
    use super::*;

    pub struct ContractStore(Store);
    impl ContractStore {
        pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
            Ok(Self(Store::open(path).await?))
        }
        pub async fn execute(&mut self, command: &str, body: Value, actor: Actor) -> Result<Value> {
            self.0
                .apply(
                    command
                        .strip_prefix("controlplane.host.")
                        .unwrap_or(command),
                    body,
                    actor,
                )
                .await
        }
        pub fn query(&self, view: &str) -> Result<Value> {
            self.0.query(view)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Decision {
    command: String,
    body: Value,
    actor: String,
    ids: Vec<String>,
    outcome: Value,
}

impl Store {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent)?;
        let path = if path.exists() {
            path.canonicalize()?
        } else {
            parent
                .canonicalize()?
                .join(path.file_name().context("database needs a filename")?)
        };
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("lock"))?;
        lock.try_lock_exclusive()
            .context("another control-plane service owns this database")?;
        let log = SqliteEventStore::open(
            path.to_str().context("database path is not UTF-8")?,
            "control_plane",
        )
        .await?;
        let stream = StreamId::new(TenantId::new("local")?, "host", "state")?;
        let mut store = Self {
            log,
            stream,
            version: 0,
            committed: tokio::sync::watch::channel(0).0,
            memory: Memory::default(),
            appended: 0,
            replayed_progress: 0,
            _lock: lock,
        };
        loop {
            let slice = store
                .log
                .read_stream(&store.stream, store.version, 1000)
                .await?;
            for event in slice.events {
                ensure!(
                    event.name == "HostDecision"
                        && event.schema_version == 1
                        && !event.is_redacted(),
                    "unsupported or redacted host history"
                );
                ensure!(
                    event.version == store.version + 1,
                    "host history is not contiguous"
                );
                let decision: Decision = serde_json::from_value(event.data)?;
                let actor = match decision.actor.as_str() {
                    "Operator" => Actor::Operator,
                    "Supervisor" => Actor::Supervisor,
                    _ => bail!("unknown recorded actor"),
                };
                // Opening fails on any refused decision, so replay moves its state forward.
                let (next, outcome) = invoke(
                    std::mem::take(&mut store.memory),
                    &decision.command,
                    &decision.body,
                    actor,
                    Some(decision.ids),
                )?;
                ensure!(
                    outcome == decision.outcome,
                    "generated behavior disagrees with durable history; migration required"
                );
                store.memory = next;
                store.version = event.version;
            }
            if slice.end_of_stream {
                break;
            }
        }
        // The replay boundary: progress recorded up to here was recorded by earlier processes.
        store.replayed_progress = store.memory.progress_records;
        store.committed.send_replace(store.version);
        Ok(store)
    }

    /// The sequence number of the newest progress record replayed when this store opened
    /// (0 for a new store). A journal step whose `recorded` is at most this was recorded by
    /// an earlier process, so no model call it opened is still running: a process's runtime
    /// holds its model calls, and the runtime opens fresh cases when it starts.
    pub fn replayed_progress(&self) -> u64 {
        self.replayed_progress
    }

    /// Subscribe to successfully committed host decisions, coalescing slow observers to
    /// the newest version. The receiver initially names the replayed durable state.
    /// Notifications are hints to re-query views, not an independent event journal.
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<u64> {
        self.committed.subscribe()
    }

    /// Serialized event data this instance has appended since it opened the store.
    pub fn appended_event_bytes(&self) -> u64 {
        self.appended
    }

    /// Execute one generated command and return its generated outcome envelope.
    /// Refused host preconditions return errors; declared ESS outcomes retain their wire shape.
    pub async fn execute(&mut self, command: &str, mut body: Value, actor: Actor) -> Result<Value> {
        let command = command
            .strip_prefix("controlplane.host.")
            .unwrap_or(command);
        let caller = Caller { actor };
        ensure!(
            caller.may(&format!("controlplane.host.{command}")),
            "actor is not granted this command"
        );
        if command == "AddWorkspaceDirectory" {
            return self
                .add_workspace_directory(
                    body["workspace_id"]
                        .as_str()
                        .context("workspace_id is required")?,
                    Path::new(body["path"].as_str().context("path is required")?),
                )
                .await;
        }
        if command == "RemoveWorkspaceDirectory" {
            return self
                .remove_workspace_directory(
                    body["directory_id"]
                        .as_str()
                        .context("directory_id is required")?,
                )
                .await;
        }
        if command == "RecordPlanningProgress" {
            self.bound_progress(&mut body);
        }
        if let Some(existing) = self.prepare(command, &mut body)? {
            return Ok(existing);
        }
        self.guard(command, &body)?;
        self.apply(command, body, actor).await
    }

    async fn apply(&mut self, command: &str, body: Value, actor: Actor) -> Result<Value> {
        let mut next = self.memory.clone();
        let mut decisions = Vec::new();
        let outcome = stage(&mut next, &mut decisions, command, body, actor)?;
        self.commit(next, decisions, actor).await?;
        Ok(outcome)
    }

    async fn commit(&mut self, next: Memory, decisions: Vec<Decision>, actor: Actor) -> Result<()> {
        ensure!(!decisions.is_empty(), "transaction has no decisions");
        let data = decisions
            .into_iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let events = data
            .iter()
            .map(|value| NewEvent::new("HostDecision", 1, value.clone()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let bytes = data
            .iter()
            .map(|value| serde_json::to_vec(value).map(|bytes| bytes.len() as u64))
            .sum::<std::result::Result<u64, _>>()?;
        let id = uuid::Uuid::new_v4().to_string();
        let meta = CommandMeta {
            idempotency_key: id.clone(),
            request_hash: eventlog_core::request_hash(&data)?,
            subject: "local-operator".into(),
            actor: actor.name().into(),
            request_id: id.clone(),
            trace_id: id,
            causation_id: None,
            causation_depth: 0,
            occurred_at: time::OffsetDateTime::now_utc(),
            claim: None,
        };
        let expected = if self.version == 0 {
            Expected::NoStream
        } else {
            Expected::Exact(self.version)
        };
        let result = self
            .log
            .append(&self.stream, expected, &events, &meta)
            .await
            .context("host decision was not committed; no dependent effect may run")?;
        self.version = result.last_version;
        self.appended += bytes;
        self.memory = next;
        self.committed.send_replace(self.version);
        Ok(())
    }

    /// Record one progress activity on a goal's planning receipt, keeping every other planning
    /// field. The decision stores this activity alone; the goal's journal keeps the history.
    pub async fn record_activity(&mut self, goal_id: &str, activity: Value) -> Result<Value> {
        let goal = self
            .query("GoalList")?
            .as_array()
            .context("goals view is not an array")?
            .iter()
            .find(|goal| goal["goal_id"] == goal_id)
            .context("progress goal missing")?
            .clone();
        let mut body = goal
            .as_object()
            .context("goal row is not an object")?
            .clone();
        body.retain(|key, _| key.starts_with("planning_") || key == "goal_id");
        let stored = body
            .get("planning_receipt")
            .and_then(Value::as_str)
            .unwrap_or("{}");
        let mut receipt = serde_json::from_str::<Value>(stored).unwrap_or_else(|_| json!({}));
        if !receipt.is_object() {
            receipt = json!({ "planning": receipt });
        }
        receipt["last_activity"] = activity;
        body.insert("planning_receipt".into(), json!(receipt.to_string()));
        self.execute(
            "RecordPlanningProgress",
            Value::Object(body),
            Actor::Supervisor,
        )
        .await
    }

    /// The host rule for progress recorded from now on. Every planning field but the receipt
    /// is capped at 4 KiB. An object receipt is bounded: its activity is bounded, and history,
    /// fleet entries and unchanged planner evidence and acceptance records are left to the
    /// goal's journal instead of being repeated in every decision.
    fn bound_progress(&self, body: &mut Value) {
        let Some(fields) = body.as_object_mut() else {
            return;
        };
        for (key, value) in fields.iter_mut() {
            if key.starts_with("planning_")
                && key != "planning_receipt"
                && let Some(text) = value.as_str()
                && text.len() > PLANNING_FIELD_BYTES
            {
                *value = Value::from(memory::bounded_text(text, PLANNING_FIELD_BYTES));
            }
        }
        let Some(goal) = body["goal_id"].as_str() else {
            return;
        };
        let Some(previous) = self.memory.goals.get(goal) else {
            return;
        };
        let Ok(Value::Object(mut receipt)) =
            serde_json::from_str::<Value>(body["planning_receipt"].as_str().unwrap_or_default())
        else {
            return;
        };
        receipt.remove("activity");
        receipt.remove("fleet");
        let journal = Journal::of(&self.memory, goal, &previous.data.planning_receipt);
        for key in memory::LATEST {
            if receipt.get(key).is_some() && receipt.get(key) == journal.latest(key) {
                receipt.remove(key);
            }
        }
        if let Some(activity) = receipt.get_mut("last_activity") {
            *activity = memory::bounded_activity(activity);
        }
        receipt.insert(
            memory::RECEIPT_FORMAT.into(),
            json!(memory::BOUNDED_RECEIPT),
        );
        body["planning_receipt"] = json!(Value::Object(receipt).to_string());
    }

    /// A goal's progress history, which bounded receipts no longer repeat: the newest 64
    /// activities (oldest first), each assignment's newest activity, the newest activity
    /// without an assignment (`planner_activity`), each lane's newest activity other than a
    /// streamed `loom.event` with its progress record number (`planner_step`, `fleet_steps`;
    /// compare [`Store::replayed_progress`]), and the newest planner evidence and acceptance
    /// record. For a receipt recorded before bounding, these are the receipt's own fields, and
    /// it has no steps.
    pub fn activity_history(&self, goal_id: &str) -> Result<Value> {
        let goal = self
            .memory
            .goals
            .get(goal_id)
            .context("goal was not found")?;
        Ok(Journal::of(&self.memory, goal_id, &goal.data.planning_receipt).history())
    }

    /// Views exactly as ESS defines them.
    pub fn query(&self, view: &str) -> Result<Value> {
        let name = view.strip_prefix("controlplane.host.").unwrap_or(view);
        // Views read generated rows only; host bookkeeping is not cloned per query.
        let stage = Arc::new(Mutex::new(self.memory.rows()));
        let mut system = System::new(ControlPlane::new(Generated::new(Ports(stage))));
        let response = dispatch(
            &mut system,
            None,
            &Request {
                method: "GET".into(),
                path: format!("/host/views/{name}"),
                ..Request::default()
            },
        );
        ensure!(response.status == 200, "view refused: {}", response.body);
        let response: Value = serde_json::from_str(&response.body)?;
        Ok(response["rows"].clone())
    }

    /// Canonical, idempotent workspace registration plus immediate repository discovery.
    pub async fn register_workspace(&mut self, path: &Path, name: &str) -> Result<Value> {
        let found = discover(path)?;
        let outcome = self
            .execute(
                "RegisterWorkspace",
                json!({"path": found.path, "name": name}),
                Actor::Operator,
            )
            .await?;
        let workspace_id = outcome["published"][0]["payload"]["workspace_id"]
            .as_str()
            .context("workspace creation has no identity")?;
        if !self
            .memory
            .directories
            .values()
            .any(|directory| directory.data.workspace_id.0 == workspace_id)
        {
            self.add_workspace_directory(workspace_id, &found.path)
                .await?;
        }
        Ok(outcome)
    }
}

fn stage(
    memory: &mut Memory,
    decisions: &mut Vec<Decision>,
    command: &str,
    body: Value,
    actor: Actor,
) -> Result<Value> {
    // A failed stage is discarded by every caller, so the transaction copy is moved, not cloned.
    let (next, outcome) = invoke(std::mem::take(memory), command, &body, actor, None)?;
    decisions.push(Decision {
        command: command.to_owned(),
        body,
        actor: format!("{actor:?}"),
        ids: next.ids.clone(),
        outcome: outcome.clone(),
    });
    *memory = next;
    Ok(outcome)
}

fn invoke(
    mut staged: Memory,
    name: &str,
    body: &Value,
    actor: Actor,
    replay: Option<Vec<String>>,
) -> Result<(Memory, Value)> {
    let progress = (name == "RecordPlanningProgress")
        .then(|| body["goal_id"].as_str())
        .flatten()
        .and_then(|goal| {
            let receipt = &staged.goals.get(goal)?.data.planning_receipt;
            Some((goal.to_owned(), receipt.clone()))
        });
    staged.replay = replay.is_some();
    staged.ids = replay.unwrap_or_default();
    staged.cursor = 0;
    let shared = Arc::new(Mutex::new(staged));
    let mut system = System::new(ControlPlane::new(Generated::new(Ports(shared.clone()))));
    let response = dispatch(
        &mut system,
        Some(&Caller { actor }),
        &Request {
            method: "POST".into(),
            path: format!("/host/commands/{name}"),
            body: serde_json::to_vec(body)?,
            ..Request::default()
        },
    );
    let outcome: Value = serde_json::from_str(&response.body)?;
    ensure!(
        outcome.get("outcome").is_some(),
        "generated command refused ({}): {}",
        response.status,
        response.body
    );
    drop(system);
    let poisoned = || anyhow::anyhow!("memory stage poisoned");
    let mut next = match Arc::try_unwrap(shared) {
        Ok(stage) => stage.into_inner().map_err(|_| poisoned())?,
        Err(shared) => shared.lock().map_err(|_| poisoned())?.clone(),
    };
    ensure!(
        next.cursor == next.ids.len(),
        "recorded UUID sequence was not consumed exactly"
    );
    if let Some((goal, previous)) = progress
        && outcome["outcome"] == "applied"
    {
        advance_journal(&mut next, &goal, &previous);
    }
    if name == "DeleteGoal"
        && let Some(goal) = body["goal_id"].as_str()
        && !next.goals.contains_key(goal)
    {
        next.journals.remove(goal);
    }
    if let Some(key) = guards::registration_key(name, body) {
        next.registration_receipts.insert(key, outcome.clone());
    }
    if matches!(name, "ClaimAssignment" | "RepairAssignment")
        && outcome["outcome"] == "applied"
        && let Some(assignment) = body["assignment_id"]
            .as_str()
            .and_then(|id| next.assignments.get(id))
        && let Some(repository) = next.repositories.get(&assignment.data.repository_id.0)
    {
        next.assignment_configs.insert(
            assignment.data.assignment_id.0.clone(),
            repository.data.clone(),
        );
    }
    Ok((next, outcome))
}

/// Keep a goal's progress journal in step with the receipt just recorded for it.
/// Replay runs this for every recorded decision, so the journal never needs storing.
fn advance_journal(next: &mut Memory, goal: &str, previous: &str) {
    let Some(receipt) = next
        .goals
        .get(goal)
        .map(|row| row.data.planning_receipt.clone())
    else {
        return;
    };
    match memory::bounded_receipt(&receipt) {
        Some(fields) => {
            let mut journal = Journal::take(next, goal, previous);
            next.progress_records += 1;
            journal.record(&receipt, &fields, next.progress_records);
            next.journals.insert(goal.to_owned(), journal);
        }
        None => {
            next.journals.remove(goal);
        }
    }
}

#[cfg(test)]
mod tests;
