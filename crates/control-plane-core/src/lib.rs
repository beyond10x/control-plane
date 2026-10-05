//! Durable host for ESS-generated behavior. No external effect can precede a successful append.
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
use memory::{Memory, Ports};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::{Arc, Mutex},
};

/// A single service owns a database; its persisted assignments own repositories across workspaces.
/// The generated actor is selected by trusted Rust code, never decoded from an HTTP request.
pub struct Store {
    log: SqliteEventStore,
    stream: StreamId,
    version: u64,
    memory: Memory,
    _lock: File,
}

impl Drop for Store {
    fn drop(&mut self) {
        // Explicit unlock also releases a descriptor briefly inherited by a concurrent fork.
        // Closing the descriptor remains the OS fallback if unlock itself fails.
        let _ = FileExt::unlock(&self._lock);
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
            memory: Memory::default(),
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
                let (next, outcome) = invoke(
                    &store.memory,
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
        Ok(store)
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
        if let Some(existing) = self.prepare(command, &mut body)? {
            return Ok(existing);
        }
        self.guard(command, &body)?;
        self.apply(command, body, actor).await
    }

    async fn apply(&mut self, command: &str, body: Value, actor: Actor) -> Result<Value> {
        let (next, outcome) = invoke(&self.memory, command, &body, actor, None)?;
        let decision = Decision {
            command: command.to_owned(),
            body,
            actor: format!("{actor:?}"),
            ids: next.ids.clone(),
            outcome: outcome.clone(),
        };
        let data = serde_json::to_value(decision)?;
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
            .append(
                &self.stream,
                expected,
                &[NewEvent::new("HostDecision", 1, data)?],
                &meta,
            )
            .await
            .context("host decision was not committed; no dependent effect may run")?;
        self.version = result.last_version;
        self.memory = next;
        Ok(outcome)
    }

    pub fn query(&self, view: &str) -> Result<Value> {
        let name = view.strip_prefix("controlplane.host.").unwrap_or(view);
        let stage = Arc::new(Mutex::new(self.memory.clone()));
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
        for repo in found.repositories {
            self.execute("RegisterRepository", json!({"workspace_id":workspace_id,"name":repo.name,"path":repo.path,"common_dir":repo.common_dir,"base_branch":repo.base_branch,"test_command":"task check","publish_command":""}), Actor::Operator).await?;
        }
        Ok(outcome)
    }
}

fn invoke(
    memory: &Memory,
    name: &str,
    body: &Value,
    actor: Actor,
    replay: Option<Vec<String>>,
) -> Result<(Memory, Value)> {
    let mut staged = memory.clone();
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
    let mut next = shared
        .lock()
        .map_err(|_| anyhow::anyhow!("memory stage poisoned"))?
        .clone();
    ensure!(
        next.cursor == next.ids.len(),
        "recorded UUID sequence was not consumed exactly"
    );
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

#[cfg(test)]
mod tests;
