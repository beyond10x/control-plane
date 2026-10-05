//! The generated infallible ports are pure memory. Cloning `Memory` is a deep transaction stage.
use controlplane_model::{behaviour::*, host::*, obligation::UnmetObligation, primitives::Uuid};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub(crate) struct Memory {
    pub registration_receipts: BTreeMap<String, serde_json::Value>,
    pub assignments: BTreeMap<String, AssignmentSnapshot>,
    pub goals: BTreeMap<String, GoalSnapshot>,
    pub publications: BTreeMap<String, PublicationIntentSnapshot>,
    pub repositories: BTreeMap<String, RepositoryRegistrationSnapshot>,
    pub workspaces: BTreeMap<String, WorkspaceSnapshot>,
    pub ids: Vec<String>,
    pub replay: bool,
    pub cursor: usize,
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
