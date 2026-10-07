// generated from controlplane v1
// model digest 897a3414c77f9f4e1cf2364f3618ac52b1f26e3c84b3ff542878e9e78509dbd7
// contract digest b6fee6bf66c237bc1382d9b6b607570b4f096d13d3d91c101d1efb3fde6bb9b7
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `controlplane.host.Assignment`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAssignmentStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::AssignmentSnapshot>>>);
impl crate::behaviour::AssignmentStorage for MemoryAssignmentStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::AssignmentSnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.assignment_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::AssignmentSnapshot) { self.delete(&snapshot.data.assignment_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.assignment_id).0.clone()).cmp(&MemoryKey::Text((&other.data.assignment_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.assignment_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::AssignmentSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::AssignmentStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::AssignmentSnapshot> { crate::behaviour::AssignmentStorage::get(&self.assignment_storage, identity) }
fn put(&mut self, snapshot: crate::host::AssignmentSnapshot) { crate::behaviour::AssignmentStorage::put(&mut self.assignment_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::AssignmentStorage::delete(&mut self.assignment_storage, identity); }
fn list(&self) -> Vec<crate::host::AssignmentSnapshot> { crate::behaviour::AssignmentStorage::list(&self.assignment_storage) }
}

/// Ephemeral storage of `controlplane.host.Goal`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryGoalStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::GoalSnapshot>>>);
impl crate::behaviour::GoalStorage for MemoryGoalStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::GoalSnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.goal_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::GoalSnapshot) { self.delete(&snapshot.data.goal_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.goal_id).0.clone()).cmp(&MemoryKey::Text((&other.data.goal_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.goal_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::GoalSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::GoalStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::GoalSnapshot> { crate::behaviour::GoalStorage::get(&self.goal_storage, identity) }
fn put(&mut self, snapshot: crate::host::GoalSnapshot) { crate::behaviour::GoalStorage::put(&mut self.goal_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::GoalStorage::delete(&mut self.goal_storage, identity); }
fn list(&self) -> Vec<crate::host::GoalSnapshot> { crate::behaviour::GoalStorage::list(&self.goal_storage) }
}

/// Ephemeral storage of `controlplane.host.PublicationIntent`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryPublicationIntentStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::PublicationIntentSnapshot>>>);
impl crate::behaviour::PublicationIntentStorage for MemoryPublicationIntentStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::PublicationIntentSnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.publication_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::PublicationIntentSnapshot) { self.delete(&snapshot.data.publication_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.publication_id).0.clone()).cmp(&MemoryKey::Text((&other.data.publication_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.publication_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::PublicationIntentSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::PublicationIntentStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::PublicationIntentSnapshot> { crate::behaviour::PublicationIntentStorage::get(&self.publication_intent_storage, identity) }
fn put(&mut self, snapshot: crate::host::PublicationIntentSnapshot) { crate::behaviour::PublicationIntentStorage::put(&mut self.publication_intent_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::PublicationIntentStorage::delete(&mut self.publication_intent_storage, identity); }
fn list(&self) -> Vec<crate::host::PublicationIntentSnapshot> { crate::behaviour::PublicationIntentStorage::list(&self.publication_intent_storage) }
}

/// Ephemeral storage of `controlplane.host.RepositoryRegistration`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryRepositoryRegistrationStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::RepositoryRegistrationSnapshot>>>);
impl crate::behaviour::RepositoryRegistrationStorage for MemoryRepositoryRegistrationStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::RepositoryRegistrationSnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.repository_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::RepositoryRegistrationSnapshot) { self.delete(&snapshot.data.repository_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.repository_id).0.clone()).cmp(&MemoryKey::Text((&other.data.repository_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.repository_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::RepositoryRegistrationSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::RepositoryRegistrationStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::RepositoryRegistrationSnapshot> { crate::behaviour::RepositoryRegistrationStorage::get(&self.repository_registration_storage, identity) }
fn put(&mut self, snapshot: crate::host::RepositoryRegistrationSnapshot) { crate::behaviour::RepositoryRegistrationStorage::put(&mut self.repository_registration_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::RepositoryRegistrationStorage::delete(&mut self.repository_registration_storage, identity); }
fn list(&self) -> Vec<crate::host::RepositoryRegistrationSnapshot> { crate::behaviour::RepositoryRegistrationStorage::list(&self.repository_registration_storage) }
}

/// Ephemeral storage of `controlplane.host.Workspace`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryWorkspaceStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::WorkspaceSnapshot>>>);
impl crate::behaviour::WorkspaceStorage for MemoryWorkspaceStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceSnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.workspace_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::WorkspaceSnapshot) { self.delete(&snapshot.data.workspace_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.workspace_id).0.clone()).cmp(&MemoryKey::Text((&other.data.workspace_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.workspace_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::WorkspaceSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::WorkspaceStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceSnapshot> { crate::behaviour::WorkspaceStorage::get(&self.workspace_storage, identity) }
fn put(&mut self, snapshot: crate::host::WorkspaceSnapshot) { crate::behaviour::WorkspaceStorage::put(&mut self.workspace_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::WorkspaceStorage::delete(&mut self.workspace_storage, identity); }
fn list(&self) -> Vec<crate::host::WorkspaceSnapshot> { crate::behaviour::WorkspaceStorage::list(&self.workspace_storage) }
}

/// Ephemeral storage of `controlplane.host.WorkspaceDirectory`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryWorkspaceDirectoryStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::host::WorkspaceDirectorySnapshot>>>);
impl crate::behaviour::WorkspaceDirectoryStorage for MemoryWorkspaceDirectoryStorage {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceDirectorySnapshot> { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow().iter().find(|row| MemoryKey::Text((&row.data.directory_id).0.clone()) == key).cloned() }
fn put(&mut self, snapshot: crate::host::WorkspaceDirectorySnapshot) { self.delete(&snapshot.data.directory_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| MemoryKey::Text((&row.data.directory_id).0.clone()).cmp(&MemoryKey::Text((&other.data.directory_id).0.clone()))); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { let key = MemoryKey::Text((identity).0.clone()); self.0.borrow_mut().retain(|row| MemoryKey::Text((&row.data.directory_id).0.clone()) != key); }
fn list(&self) -> Vec<crate::host::WorkspaceDirectorySnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::WorkspaceDirectoryStorage for MemoryPorts {
fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceDirectorySnapshot> { crate::behaviour::WorkspaceDirectoryStorage::get(&self.workspace_directory_storage, identity) }
fn put(&mut self, snapshot: crate::host::WorkspaceDirectorySnapshot) { crate::behaviour::WorkspaceDirectoryStorage::put(&mut self.workspace_directory_storage, snapshot); }
fn delete(&mut self, identity: &crate::primitives::Uuid) { crate::behaviour::WorkspaceDirectoryStorage::delete(&mut self.workspace_directory_storage, identity); }
fn list(&self) -> Vec<crate::host::WorkspaceDirectorySnapshot> { crate::behaviour::WorkspaceDirectoryStorage::list(&self.workspace_directory_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `controlplane.host.Assignment`.
pub assignment_storage: MemoryAssignmentStorage,
/// Storage of `controlplane.host.Goal`.
pub goal_storage: MemoryGoalStorage,
/// Storage of `controlplane.host.PublicationIntent`.
pub publication_intent_storage: MemoryPublicationIntentStorage,
/// Storage of `controlplane.host.RepositoryRegistration`.
pub repository_registration_storage: MemoryRepositoryRegistrationStorage,
/// Storage of `controlplane.host.Workspace`.
pub workspace_storage: MemoryWorkspaceStorage,
/// Storage of `controlplane.host.WorkspaceDirectory`.
pub workspace_directory_storage: MemoryWorkspaceDirectoryStorage,
}

impl crate::behaviour::TryContext for MemoryPorts {
fn try_generate_uuid(&mut self) -> Result<crate::primitives::Uuid, crate::obligation::UnmetObligation> { Ok(crate::primitives::Uuid(uuid::Uuid::new_v4().to_string())) }
}

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

