// generated from controlplane v1
// model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
// contract digest e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
// do not edit: regenerate with `ess synthesize --layout crate`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! preserves the trait; generated network entries supply an ephemeral store. `Context` carries the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `controlplane.host.Assignment` is stored — a port the implementor provides.
///
/// Keyed by the identity `assignment_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait AssignmentStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::AssignmentSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::AssignmentSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::AssignmentSnapshot>;
}

/// Where `controlplane.host.Goal` is stored — a port the implementor provides.
///
/// Keyed by the identity `goal_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait GoalStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::GoalSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::GoalSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::GoalSnapshot>;
}

/// Where `controlplane.host.PublicationIntent` is stored — a port the implementor provides.
///
/// Keyed by the identity `publication_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait PublicationIntentStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::PublicationIntentSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::PublicationIntentSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::PublicationIntentSnapshot>;
}

/// Where `controlplane.host.RepositoryRegistration` is stored — a port the implementor provides.
///
/// Keyed by the identity `repository_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait RepositoryRegistrationStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::RepositoryRegistrationSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::RepositoryRegistrationSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::RepositoryRegistrationSnapshot>;
}

/// Where `controlplane.host.Workspace` is stored — a port the implementor provides.
///
/// Keyed by the identity `workspace_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait WorkspaceStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::WorkspaceSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::WorkspaceSnapshot>;
}

/// Where `controlplane.host.WorkspaceDirectory` is stored — a port the implementor provides.
///
/// Keyed by the identity `directory_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait WorkspaceDirectoryStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::host::WorkspaceDirectorySnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::host::WorkspaceDirectorySnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::primitives::Uuid);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::host::WorkspaceDirectorySnapshot>;
}

/// What the specification leaves to the implementor's context — a port the implementor provides.
///
/// The caller's attributes, the values the model says the implementation assigns, and the answer
/// to each `external:` branch.
pub trait Context {
    /// A new `Uuid`, which the model says the implementation assigns — a created identity, a
    /// `{generated: true}` value, or an event field the model leaves undetermined.
    fn generate_uuid(&mut self) -> crate::primitives::Uuid;
}

/// Context answers that may be unavailable, without fabricated values.
/// Existing `Context` implementations receive the blanket adapter.
pub trait TryContext {
/// Assigns the value, or names the unavailable answer.
fn try_generate_uuid(&mut self) -> Result<crate::primitives::Uuid, UnmetObligation>;
}

impl<T: Context + ?Sized> TryContext for T {
fn try_generate_uuid(&mut self) -> Result<crate::primitives::Uuid, UnmetObligation> { Ok(Context::generate_uuid(self)) }
}

/// An unavailable runtime context answer, rather than a new planned capability.
pub fn unmet_context(source: &'static str) -> UnmetObligation { UnmetObligation { capability: "context answer", source } }

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

/// `controlplane.host.AddWorkspaceDirectory`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::AddWorkspaceDirectoryBehavior for Generated<P>
where
    P: TryContext + WorkspaceDirectoryStorage,
{
    fn add_workspace_directory(&mut self, input: crate::host::AddWorkspaceDirectory) -> Result<crate::host::AddWorkspaceDirectoryOutcome, UnmetObligation> {
        let _ = &input;
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::WorkspaceDirectoryData {
            directory_id: identity.clone(),
            workspace_id: input.workspace_id.clone(),
            path: input.path.clone(),
            repository_common_dirs: input.repository_common_dirs.clone(),
            managed_common_dirs: input.managed_common_dirs.clone(),
        };
        let answer = crate::host::AddWorkspaceDirectoryOutcome::Created { workspace_directory_created: crate::host::WorkspaceDirectoryCreated { directory_id: identity.clone(), workspace_id: input.workspace_id.clone(), path: input.path.clone(), repository_common_dirs: input.repository_common_dirs.clone(), managed_common_dirs: input.managed_common_dirs.clone() } };
        WorkspaceDirectoryStorage::put(&mut self.ports, crate::host::AnyWorkspaceDirectory::Registered(crate::host::WorkspaceDirectory::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.ArchiveWorkspace`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ArchiveWorkspaceBehavior for Generated<P>
where
    P: WorkspaceStorage,
{
    fn archive_workspace(&mut self, input: crate::host::ArchiveWorkspace) -> Result<crate::host::ArchiveWorkspaceOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = WorkspaceStorage::get(&self.ports, &input.workspace_id) else {
            return Ok(crate::host::ArchiveWorkspaceOutcome::NotFound { error: crate::host::WorkspaceNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyWorkspace::Registered(instance) => crate::host::AnyWorkspace::Archived(instance.archive()),
            _ => return Ok(crate::host::ArchiveWorkspaceOutcome::WrongState { error: crate::host::WorkspaceStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::ArchiveWorkspaceOutcome::Applied { archive_workspace_applied: crate::host::ArchiveWorkspaceApplied { workspace_id: input.workspace_id.clone() } };
        WorkspaceStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.BlockAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::BlockAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn block_assignment(&mut self, input: crate::host::BlockAssignment) -> Result<crate::host::BlockAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::BlockAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Implementing(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Merging(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Queued(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::ReadyToMerge(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            _ => return Ok(crate::host::BlockAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.reason = input.reason.clone();
        let answer = crate::host::BlockAssignmentOutcome::Applied { block_assignment_applied: crate::host::BlockAssignmentApplied { assignment_id: input.assignment_id.clone(), reason: input.reason.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.CancelAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::CancelAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn cancel_assignment(&mut self, input: crate::host::CancelAssignment) -> Result<crate::host::CancelAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::CancelAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Cancelled(instance.cancel()),
            crate::host::AnyAssignment::Implementing(instance) => crate::host::AnyAssignment::Cancelled(instance.cancel()),
            crate::host::AnyAssignment::Queued(instance) => crate::host::AnyAssignment::Cancelled(instance.cancel()),
            crate::host::AnyAssignment::ReadyToMerge(instance) => crate::host::AnyAssignment::Cancelled(instance.cancel()),
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Cancelled(instance.cancel()),
            _ => return Ok(crate::host::CancelAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::CancelAssignmentOutcome::Applied { cancel_assignment_applied: crate::host::CancelAssignmentApplied { assignment_id: input.assignment_id.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.CancelGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::CancelGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn cancel_goal(&mut self, input: crate::host::CancelGoal) -> Result<crate::host::CancelGoalOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::CancelGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyGoal::Paused(instance) => crate::host::AnyGoal::Cancelled(instance.cancel()),
            crate::host::AnyGoal::Running(instance) => crate::host::AnyGoal::Cancelled(instance.cancel()),
            _ => return Ok(crate::host::CancelGoalOutcome::WrongState { error: crate::host::GoalStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::CancelGoalOutcome::Applied { cancel_goal_applied: crate::host::CancelGoalApplied { goal_id: input.goal_id.clone() } };
        GoalStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ClaimAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ClaimAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn claim_assignment(&mut self, input: crate::host::ClaimAssignment) -> Result<crate::host::ClaimAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ClaimAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `evidence-missing`: selected by the addressed row.
        if decided(any(&[equal(Some(&input.implementor_run).map(|value| value.clone()), Some("".to_owned())), equal(Some(&input.worktree_id).map(|value| value.clone()), Some("".to_owned())), equal(Some(&input.base_revision).map(|value| value.clone()), Some("".to_owned()))]), "controlplane.host.ClaimAssignment")? {
            return Ok(crate::host::ClaimAssignmentOutcome::EvidenceMissing { error: crate::host::EvidenceMissing });
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ClaimAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::host::AnyAssignment::Queued(instance) => crate::host::AnyAssignment::Implementing(instance.claim()),
            _ => return Ok(crate::host::ClaimAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.worktree_id = input.worktree_id.clone();
        next.data.attempt = before.attempt + 1;
        next.data.implementor_run = input.implementor_run.clone();
        next.data.base_revision = input.base_revision.clone();
        let answer = crate::host::ClaimAssignmentOutcome::Applied { claim_assignment_applied: crate::host::ClaimAssignmentApplied { assignment_id: input.assignment_id.clone(), worktree_id: input.worktree_id.clone(), implementor_run: input.implementor_run.clone(), base_revision: input.base_revision.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ClosePublication`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ClosePublicationBehavior for Generated<P>
where
    P: PublicationIntentStorage,
{
    fn close_publication(&mut self, input: crate::host::ClosePublication) -> Result<crate::host::ClosePublicationOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = PublicationIntentStorage::get(&self.ports, &input.publication_id) else {
            return Ok(crate::host::ClosePublicationOutcome::NotFound { error: crate::host::PublicationIntentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyPublicationIntent::Prepared(instance) => crate::host::AnyPublicationIntent::NotPublished(instance.close()),
            crate::host::AnyPublicationIntent::Uncertain(instance) => crate::host::AnyPublicationIntent::NotPublished(instance.close()),
            _ => return Ok(crate::host::ClosePublicationOutcome::WrongState { error: crate::host::PublicationIntentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.reason = Some(input.reason.clone());
        let answer = crate::host::ClosePublicationOutcome::Applied { close_publication_applied: crate::host::ClosePublicationApplied { publication_id: input.publication_id.clone(), reason: input.reason.clone() } };
        PublicationIntentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.CompleteAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::CompleteAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn complete_assignment(&mut self, input: crate::host::CompleteAssignment) -> Result<crate::host::CompleteAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::CompleteAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `receipt-missing`: selected by the addressed row.
        if decided(equal(Some(&input.merge_receipt).map(|value| value.clone()), Some("".to_owned())), "controlplane.host.CompleteAssignment")? {
            return Ok(crate::host::CompleteAssignmentOutcome::ReceiptMissing { error: crate::host::EvidenceMissing });
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::CompleteAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Merging(instance) => crate::host::AnyAssignment::Merged(instance.complete()),
            _ => return Ok(crate::host::CompleteAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.merge_receipt = input.merge_receipt.clone();
        let answer = crate::host::CompleteAssignmentOutcome::Applied { complete_assignment_applied: crate::host::CompleteAssignmentApplied { assignment_id: input.assignment_id.clone(), merge_receipt: input.merge_receipt.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ConfigureRepository`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ConfigureRepositoryBehavior for Generated<P>
where
    P: RepositoryRegistrationStorage,
{
    fn configure_repository(&mut self, input: crate::host::ConfigureRepository) -> Result<crate::host::ConfigureRepositoryOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = RepositoryRegistrationStorage::get(&self.ports, &input.repository_id) else {
            return Ok(crate::host::ConfigureRepositoryOutcome::NotFound { error: crate::host::RepositoryRegistrationNotFound });
        };
        let _ = &held;
        let mut next = held;
        next.data.base_branch = input.base_branch.clone();
        next.data.test_command = input.test_command.clone();
        next.data.publish_command = input.publish_command.clone();
        let answer = crate::host::ConfigureRepositoryOutcome::Applied { configure_repository_applied: crate::host::ConfigureRepositoryApplied { repository_id: input.repository_id.clone(), base_branch: input.base_branch.clone(), test_command: input.test_command.clone(), publish_command: input.publish_command.clone() } };
        RepositoryRegistrationStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ConfirmPublication`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ConfirmPublicationBehavior for Generated<P>
where
    P: PublicationIntentStorage,
{
    fn confirm_publication(&mut self, input: crate::host::ConfirmPublication) -> Result<crate::host::ConfirmPublicationOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = PublicationIntentStorage::get(&self.ports, &input.publication_id) else {
            return Ok(crate::host::ConfirmPublicationOutcome::NotFound { error: crate::host::PublicationIntentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyPublicationIntent::Prepared(instance) => crate::host::AnyPublicationIntent::Confirmed(instance.confirm()),
            crate::host::AnyPublicationIntent::Uncertain(instance) => crate::host::AnyPublicationIntent::Confirmed(instance.confirm()),
            _ => return Ok(crate::host::ConfirmPublicationOutcome::WrongState { error: crate::host::PublicationIntentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.receipt = input.receipt.clone();
        let answer = crate::host::ConfirmPublicationOutcome::Applied { confirm_publication_applied: crate::host::ConfirmPublicationApplied { publication_id: input.publication_id.clone(), receipt: input.receipt.clone() } };
        PublicationIntentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.CreateGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::CreateGoalBehavior for Generated<P>
where
    P: TryContext + GoalStorage,
{
    fn create_goal(&mut self, input: crate::host::CreateGoal) -> Result<crate::host::CreateGoalOutcome, UnmetObligation> {
        let _ = &input;
        // `workers-invalid`: an input-guarded refusal, before the addressed subject is loaded.
        if decided(compare_numbers(Some(&input.max_workers).map(|value| value.to_string()), Some("0".to_owned()), core::cmp::Ordering::is_le), "controlplane.host.CreateGoal")? {
            return Ok(crate::host::CreateGoalOutcome::WorkersInvalid { error: crate::host::GoalLimitInvalid });
        }
        // `attempts-invalid`: an input-guarded refusal, before the addressed subject is loaded.
        if decided(compare_numbers(Some(&input.max_attempts).map(|value| value.to_string()), Some("0".to_owned()), core::cmp::Ordering::is_le), "controlplane.host.CreateGoal")? {
            return Ok(crate::host::CreateGoalOutcome::AttemptsInvalid { error: crate::host::GoalLimitInvalid });
        }
        // `minutes-invalid`: an input-guarded refusal, before the addressed subject is loaded.
        if decided(compare_numbers(Some(&input.max_minutes).map(|value| value.to_string()), Some("0".to_owned()), core::cmp::Ordering::is_le), "controlplane.host.CreateGoal")? {
            return Ok(crate::host::CreateGoalOutcome::MinutesInvalid { error: crate::host::GoalLimitInvalid });
        }
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::GoalData {
            goal_id: identity.clone(),
            workspace_id: input.workspace_id.clone(),
            objective: input.objective.clone(),
            acceptance: input.acceptance.clone(),
            max_workers: input.max_workers.clone(),
            max_attempts: input.max_attempts.clone(),
            max_minutes: input.max_minutes.clone(),
            planner_model: input.planner_model.clone(),
            implementor_model: input.implementor_model.clone(),
            reviewer_model: input.reviewer_model.clone(),
            merge_authority: input.merge_authority.clone(),
            revision: 1,
            satisfaction_receipt: "".to_owned(),
            planning_revision: 0,
            planning_fingerprint: "".to_owned(),
            planning_repository: "".to_owned(),
            planning_worktree_id: "".to_owned(),
            planning_worktree_path: "".to_owned(),
            planning_reason: "".to_owned(),
            planning_receipt: "".to_owned(),
            planning_phase: crate::host::PlanningPhase::Idle,
        };
        let answer = crate::host::CreateGoalOutcome::Created { goal_created: crate::host::GoalCreated { goal_id: identity.clone(), workspace_id: input.workspace_id.clone(), objective: input.objective.clone(), acceptance: input.acceptance.clone(), max_workers: input.max_workers.clone(), max_attempts: input.max_attempts.clone(), max_minutes: input.max_minutes.clone(), planner_model: input.planner_model.clone(), implementor_model: input.implementor_model.clone(), reviewer_model: input.reviewer_model.clone(), merge_authority: input.merge_authority.clone(), planning_revision: 0, planning_fingerprint: "".to_owned(), planning_repository: "".to_owned(), planning_worktree_id: "".to_owned(), planning_worktree_path: "".to_owned(), planning_reason: "".to_owned(), planning_receipt: "".to_owned(), planning_phase: crate::host::PlanningPhase::Idle } };
        GoalStorage::put(&mut self.ports, crate::host::AnyGoal::Paused(crate::host::Goal::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.DeleteGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::DeleteGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn delete_goal(&mut self, input: crate::host::DeleteGoal) -> Result<crate::host::DeleteGoalOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::DeleteGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        // `applied`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Cancelled)), "controlplane.host.DeleteGoal")? {
            let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
                return Ok(crate::host::DeleteGoalOutcome::NotFound { error: crate::host::GoalNotFound });
            };
            let _ = &held;
            let answer = crate::host::DeleteGoalOutcome::Applied { goal_deleted: crate::host::GoalDeleted { goal_id: input.goal_id.clone() } };
            GoalStorage::delete(&mut self.ports, &input.goal_id);
            return Ok(answer);
        }
        // `paused`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Paused)), "controlplane.host.DeleteGoal")? {
            return Ok(crate::host::DeleteGoalOutcome::Paused { error: crate::host::GoalStateConflict { state: held.state } });
        }
        // `running`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Running)), "controlplane.host.DeleteGoal")? {
            return Ok(crate::host::DeleteGoalOutcome::Running { error: crate::host::GoalStateConflict { state: held.state } });
        }
        // `satisfied`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Satisfied)), "controlplane.host.DeleteGoal")? {
            return Ok(crate::host::DeleteGoalOutcome::Satisfied { error: crate::host::GoalStateConflict { state: held.state } });
        }
        // No declared branch answers this request.
        Err(undeclared("controlplane.host.DeleteGoal"))
    }
}

/// `controlplane.host.DisableRepositoryRegistration`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::DisableRepositoryRegistrationBehavior for Generated<P>
where
    P: RepositoryRegistrationStorage,
{
    fn disable_repository_registration(&mut self, input: crate::host::DisableRepositoryRegistration) -> Result<crate::host::DisableRepositoryRegistrationOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = RepositoryRegistrationStorage::get(&self.ports, &input.repository_id) else {
            return Ok(crate::host::DisableRepositoryRegistrationOutcome::NotFound { error: crate::host::RepositoryRegistrationNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyRepositoryRegistration::Registered(instance) => crate::host::AnyRepositoryRegistration::Disabled(instance.disable()),
            _ => return Ok(crate::host::DisableRepositoryRegistrationOutcome::WrongState { error: crate::host::RepositoryRegistrationStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::DisableRepositoryRegistrationOutcome::Applied { disable_repository_registration_applied: crate::host::DisableRepositoryRegistrationApplied { repository_id: input.repository_id.clone() } };
        RepositoryRegistrationStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.EnableRepositoryRegistration`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::EnableRepositoryRegistrationBehavior for Generated<P>
where
    P: RepositoryRegistrationStorage,
{
    fn enable_repository_registration(&mut self, input: crate::host::EnableRepositoryRegistration) -> Result<crate::host::EnableRepositoryRegistrationOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = RepositoryRegistrationStorage::get(&self.ports, &input.repository_id) else {
            return Ok(crate::host::EnableRepositoryRegistrationOutcome::NotFound { error: crate::host::RepositoryRegistrationNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyRepositoryRegistration::Disabled(instance) => crate::host::AnyRepositoryRegistration::Registered(instance.enable()),
            _ => return Ok(crate::host::EnableRepositoryRegistrationOutcome::WrongState { error: crate::host::RepositoryRegistrationStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::EnableRepositoryRegistrationOutcome::Applied { enable_repository_registration_applied: crate::host::EnableRepositoryRegistrationApplied { repository_id: input.repository_id.clone() } };
        RepositoryRegistrationStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.MarkPublicationUncertain`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::MarkPublicationUncertainBehavior for Generated<P>
where
    P: PublicationIntentStorage,
{
    fn mark_publication_uncertain(&mut self, input: crate::host::MarkPublicationUncertain) -> Result<crate::host::MarkPublicationUncertainOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = PublicationIntentStorage::get(&self.ports, &input.publication_id) else {
            return Ok(crate::host::MarkPublicationUncertainOutcome::NotFound { error: crate::host::PublicationIntentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyPublicationIntent::Prepared(instance) => crate::host::AnyPublicationIntent::Uncertain(instance.uncertain()),
            _ => return Ok(crate::host::MarkPublicationUncertainOutcome::WrongState { error: crate::host::PublicationIntentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::MarkPublicationUncertainOutcome::Applied { mark_publication_uncertain_applied: crate::host::MarkPublicationUncertainApplied { publication_id: input.publication_id.clone() } };
        PublicationIntentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.MergeAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::MergeAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn merge_assignment(&mut self, input: crate::host::MergeAssignment) -> Result<crate::host::MergeAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::MergeAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::ReadyToMerge(instance) => crate::host::AnyAssignment::Merging(instance.merge()),
            _ => return Ok(crate::host::MergeAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::MergeAssignmentOutcome::Applied { merge_assignment_applied: crate::host::MergeAssignmentApplied { assignment_id: input.assignment_id.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.PauseGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::PauseGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn pause_goal(&mut self, input: crate::host::PauseGoal) -> Result<crate::host::PauseGoalOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::PauseGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyGoal::Running(instance) => crate::host::AnyGoal::Paused(instance.pause()),
            _ => return Ok(crate::host::PauseGoalOutcome::WrongState { error: crate::host::GoalStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::PauseGoalOutcome::Applied { pause_goal_applied: crate::host::PauseGoalApplied { goal_id: input.goal_id.clone() } };
        GoalStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.PreparePublication`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::PreparePublicationBehavior for Generated<P>
where
    P: TryContext + PublicationIntentStorage,
{
    fn prepare_publication(&mut self, input: crate::host::PreparePublication) -> Result<crate::host::PreparePublicationOutcome, UnmetObligation> {
        let _ = &input;
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::PublicationIntentData {
            publication_id: identity.clone(),
            assignment_id: input.assignment_id.clone(),
            candidate: input.candidate.clone(),
            target: input.target.clone(),
            expected_base: input.expected_base.clone(),
            receipt: "".to_owned(),
            reason: None,
        };
        let answer = crate::host::PreparePublicationOutcome::Created { publication_intent_created: crate::host::PublicationIntentCreated { assignment_id: input.assignment_id.clone(), candidate: input.candidate.clone(), target: input.target.clone(), expected_base: input.expected_base.clone(), publication_id: identity.clone() } };
        PublicationIntentStorage::put(&mut self.ports, crate::host::AnyPublicationIntent::Prepared(crate::host::PublicationIntent::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.QueueAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::QueueAssignmentBehavior for Generated<P>
where
    P: TryContext + AssignmentStorage + GoalStorage,
{
    fn queue_assignment(&mut self, input: crate::host::QueueAssignment) -> Result<crate::host::QueueAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // `when_related:` reads the `controlplane.host.Goal` row `input.goal_id` names, through its storage port; an absent
        // reference reads no row and selects no related branch.
        let reference = Some(&input.goal_id);
        let related = reference.and_then(|identity| GoalStorage::get(&self.ports, identity));
        // `goal-not-found`: the reference names an identity no row carries.
        if reference.is_some() && related.is_none() {
            return Ok(crate::host::QueueAssignmentOutcome::GoalNotFound { error: crate::host::GoalNotFound });
        }
        let _ = &related;
        // `goal-not-current`: selected by the present related row, in declaration order.
        if let Some(related) = &related {
        if decided(any(&[equal(Some(&related.state).map(|value| match value { crate::host::GoalState::Cancelled => "Cancelled", crate::host::GoalState::Paused => "Paused", crate::host::GoalState::Running => "Running", crate::host::GoalState::Satisfied => "Satisfied" }.to_owned()), Some("Running".to_owned())).map(|value| !value), compare_numbers(Some(&related.data.revision).map(|value| value.to_string()), Some(&input.goal_revision).map(|value| value.to_string()), core::cmp::Ordering::is_ne)]), "controlplane.host.QueueAssignment")? {
            return Ok(crate::host::QueueAssignmentOutcome::GoalNotCurrent { error: crate::host::GoalNotCurrent });
        }
        }
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::AssignmentData {
            assignment_id: identity.clone(),
            goal_id: input.goal_id.clone(),
            repository_id: input.repository_id.clone(),
            story_id: input.story_id.clone(),
            case_id: input.case_id.clone(),
            worktree_id: input.worktree_id.clone(),
            candidate: input.candidate.clone(),
            attempt: input.attempt.clone(),
            reason: input.reason.clone(),
            implementor_run: input.implementor_run.clone(),
            reviewer_run: input.reviewer_run.clone(),
            goal_revision: input.goal_revision.clone(),
            base_revision: "".to_owned(),
            test_revision: "".to_owned(),
            review_revision: "".to_owned(),
            merge_receipt: "".to_owned(),
        };
        let answer = crate::host::QueueAssignmentOutcome::Created { assignment_created: crate::host::AssignmentCreated { assignment_id: identity.clone(), goal_id: input.goal_id.clone(), repository_id: input.repository_id.clone(), story_id: input.story_id.clone(), case_id: input.case_id.clone(), worktree_id: input.worktree_id.clone(), candidate: input.candidate.clone(), attempt: input.attempt.clone(), reason: input.reason.clone(), implementor_run: input.implementor_run.clone(), reviewer_run: input.reviewer_run.clone(), goal_revision: input.goal_revision.clone() } };
        AssignmentStorage::put(&mut self.ports, crate::host::AnyAssignment::Queued(crate::host::Assignment::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.ReadyAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ReadyAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn ready_assignment(&mut self, input: crate::host::ReadyAssignment) -> Result<crate::host::ReadyAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReadyAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `reviewer-missing`: selected by the addressed row.
        if decided(equal(Some(&input.reviewer_run).map(|value| value.clone()), Some("".to_owned())), "controlplane.host.ReadyAssignment")? {
            return Ok(crate::host::ReadyAssignmentOutcome::ReviewerMissing { error: crate::host::ReviewNotIndependent });
        }
        // `review-not-independent`: selected by the addressed row.
        if decided(equal(Some(&held.data.implementor_run).map(|value| value.clone()), Some(&input.reviewer_run).map(|value| value.clone())), "controlplane.host.ReadyAssignment")? {
            return Ok(crate::host::ReadyAssignmentOutcome::ReviewNotIndependent { error: crate::host::ReviewNotIndependent });
        }
        // `evidence-not-current`: selected by the addressed row.
        if decided(all(&[equal(Some(&held.state).map(|value| match value { crate::host::AssignmentState::Blocked => "Blocked", crate::host::AssignmentState::Cancelled => "Cancelled", crate::host::AssignmentState::Implementing => "Implementing", crate::host::AssignmentState::Merged => "Merged", crate::host::AssignmentState::Merging => "Merging", crate::host::AssignmentState::Queued => "Queued", crate::host::AssignmentState::ReadyToMerge => "ReadyToMerge", crate::host::AssignmentState::Reviewing => "Reviewing" }.to_owned()), Some("Reviewing".to_owned())), any(&[equal(Some(&held.data.candidate).map(|value| value.clone()), Some("".to_owned())), equal(Some(&held.data.test_revision).map(|value| value.clone()), Some(&held.data.candidate).map(|value| value.clone())).map(|value| !value), equal(Some(&held.data.candidate).map(|value| value.clone()), Some(&input.review_revision).map(|value| value.clone())).map(|value| !value)])]), "controlplane.host.ReadyAssignment")? {
            return Ok(crate::host::ReadyAssignmentOutcome::EvidenceNotCurrent { error: crate::host::EvidenceNotCurrent });
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReadyAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::ReadyToMerge(instance.ready()),
            _ => return Ok(crate::host::ReadyAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.reviewer_run = input.reviewer_run.clone();
        next.data.review_revision = input.review_revision.clone();
        let answer = crate::host::ReadyAssignmentOutcome::Applied { ready_assignment_applied: crate::host::ReadyAssignmentApplied { assignment_id: input.assignment_id.clone(), reviewer_run: input.reviewer_run.clone(), review_revision: input.review_revision.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ReconcileAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ReconcileAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn reconcile_assignment(&mut self, input: crate::host::ReconcileAssignment) -> Result<crate::host::ReconcileAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReconcileAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `receipt-missing`: selected by the addressed row.
        if decided(equal(Some(&input.merge_receipt).map(|value| value.clone()), Some("".to_owned())), "controlplane.host.ReconcileAssignment")? {
            return Ok(crate::host::ReconcileAssignmentOutcome::ReceiptMissing { error: crate::host::EvidenceMissing });
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReconcileAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Merged(instance.reconcile()),
            crate::host::AnyAssignment::Merging(instance) => crate::host::AnyAssignment::Merged(instance.reconcile()),
            _ => return Ok(crate::host::ReconcileAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.merge_receipt = input.merge_receipt.clone();
        let answer = crate::host::ReconcileAssignmentOutcome::Applied { reconcile_assignment_applied: crate::host::ReconcileAssignmentApplied { assignment_id: input.assignment_id.clone(), merge_receipt: input.merge_receipt.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.RecordPlanningProgress`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RecordPlanningProgressBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn record_planning_progress(&mut self, input: crate::host::RecordPlanningProgress) -> Result<crate::host::RecordPlanningProgressOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::RecordPlanningProgressOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        let mut next = held;
        next.data.planning_revision = input.planning_revision.clone();
        next.data.planning_fingerprint = input.planning_fingerprint.clone();
        next.data.planning_repository = input.planning_repository.clone();
        next.data.planning_worktree_id = input.planning_worktree_id.clone();
        next.data.planning_worktree_path = input.planning_worktree_path.clone();
        next.data.planning_reason = input.planning_reason.clone();
        next.data.planning_receipt = input.planning_receipt.clone();
        next.data.planning_phase = input.planning_phase.clone();
        let answer = crate::host::RecordPlanningProgressOutcome::Applied { planning_progress_recorded: crate::host::PlanningProgressRecorded { goal_id: input.goal_id.clone(), planning_revision: input.planning_revision.clone(), planning_fingerprint: input.planning_fingerprint.clone(), planning_repository: input.planning_repository.clone(), planning_worktree_id: input.planning_worktree_id.clone(), planning_worktree_path: input.planning_worktree_path.clone(), planning_reason: input.planning_reason.clone(), planning_receipt: input.planning_receipt.clone(), planning_phase: input.planning_phase.clone() } };
        GoalStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.RegisterRepository`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RegisterRepositoryBehavior for Generated<P>
where
    P: TryContext + RepositoryRegistrationStorage,
{
    fn register_repository(&mut self, input: crate::host::RegisterRepository) -> Result<crate::host::RegisterRepositoryOutcome, UnmetObligation> {
        let _ = &input;
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::RepositoryRegistrationData {
            repository_id: identity.clone(),
            workspace_id: input.workspace_id.clone(),
            name: input.name.clone(),
            path: input.path.clone(),
            common_dir: input.common_dir.clone(),
            base_branch: input.base_branch.clone(),
            test_command: input.test_command.clone(),
            publish_command: input.publish_command.clone(),
        };
        let answer = crate::host::RegisterRepositoryOutcome::Created { repository_registration_created: crate::host::RepositoryRegistrationCreated { repository_id: identity.clone(), workspace_id: input.workspace_id.clone(), name: input.name.clone(), path: input.path.clone(), common_dir: input.common_dir.clone(), base_branch: input.base_branch.clone(), test_command: input.test_command.clone(), publish_command: input.publish_command.clone() } };
        RepositoryRegistrationStorage::put(&mut self.ports, crate::host::AnyRepositoryRegistration::Registered(crate::host::RepositoryRegistration::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.RegisterWorkspace`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RegisterWorkspaceBehavior for Generated<P>
where
    P: TryContext + WorkspaceStorage,
{
    fn register_workspace(&mut self, input: crate::host::RegisterWorkspace) -> Result<crate::host::RegisterWorkspaceOutcome, UnmetObligation> {
        let _ = &input;
        // `created`: the default.
        let identity: crate::primitives::Uuid = self.ports.try_generate_uuid()?;
        let data = crate::host::WorkspaceData {
            workspace_id: identity.clone(),
            path: input.path.clone(),
            name: input.name.clone(),
        };
        let answer = crate::host::RegisterWorkspaceOutcome::Created { workspace_created: crate::host::WorkspaceCreated { workspace_id: identity.clone(), path: input.path.clone(), name: input.name.clone() } };
        WorkspaceStorage::put(&mut self.ports, crate::host::AnyWorkspace::Registered(crate::host::Workspace::new(data)).snapshot());
        return Ok(answer);
    }
}

/// `controlplane.host.RemoveWorkspaceDirectory`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RemoveWorkspaceDirectoryBehavior for Generated<P>
where
    P: WorkspaceDirectoryStorage,
{
    fn remove_workspace_directory(&mut self, input: crate::host::RemoveWorkspaceDirectory) -> Result<crate::host::RemoveWorkspaceDirectoryOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = WorkspaceDirectoryStorage::get(&self.ports, &input.directory_id) else {
            return Ok(crate::host::RemoveWorkspaceDirectoryOutcome::NotFound { error: crate::host::WorkspaceDirectoryNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyWorkspaceDirectory::Registered(instance) => crate::host::AnyWorkspaceDirectory::Removed(instance.remove()),
            _ => return Ok(crate::host::RemoveWorkspaceDirectoryOutcome::WrongState { error: crate::host::WorkspaceDirectoryStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::RemoveWorkspaceDirectoryOutcome::Applied { workspace_directory_removed: crate::host::WorkspaceDirectoryRemoved { directory_id: input.directory_id.clone() } };
        WorkspaceDirectoryStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.RepairAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RepairAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn repair_assignment(&mut self, input: crate::host::RepairAssignment) -> Result<crate::host::RepairAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::RepairAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `evidence-missing`: selected by the addressed row.
        if decided(equal(Some(&input.implementor_run).map(|value| value.clone()), Some("".to_owned())), "controlplane.host.RepairAssignment")? {
            return Ok(crate::host::RepairAssignmentOutcome::EvidenceMissing { error: crate::host::EvidenceMissing });
        }
        // `base-missing`: selected by the addressed row.
        if decided(all(&[Some(Some(&input.base_revision).and_then(|value| value.as_ref()).is_some()), equal(Some(&input.base_revision).and_then(|value| value.as_ref()).map(|value| value.clone()), Some("".to_owned()))]), "controlplane.host.RepairAssignment")? {
            return Ok(crate::host::RepairAssignmentOutcome::BaseMissing { error: crate::host::EvidenceMissing });
        }
        // `rebased`: an accepting branch, in declaration order.
        if decided(Some(Some(&input.base_revision).and_then(|value| value.as_ref()).is_some()), "controlplane.host.RepairAssignment")? {
            let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
                return Ok(crate::host::RepairAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
            };
            let _ = &held;
            let held_state = held.state;
            let before = held.data.clone();
            let moved = match held.refine() {
                crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
                crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
                _ => return Ok(crate::host::RepairAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
            };
            let mut next = moved.snapshot();
            next.data.attempt = before.attempt + 1;
            next.data.reason = input.reason.clone();
            next.data.implementor_run = input.implementor_run.clone();
            next.data.reviewer_run = "".to_owned();
            next.data.base_revision = match input.base_revision.clone() { Some(value) => value, None => "".to_owned() };
            next.data.test_revision = "".to_owned();
            next.data.review_revision = "".to_owned();
            let answer = crate::host::RepairAssignmentOutcome::Rebased { repair_assignment_applied: crate::host::RepairAssignmentApplied { assignment_id: input.assignment_id.clone(), reason: input.reason.clone(), implementor_run: input.implementor_run.clone() } };
            AssignmentStorage::put(&mut self.ports, next);
            return Ok(answer);
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::RepairAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
            _ => return Ok(crate::host::RepairAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.attempt = before.attempt + 1;
        next.data.reason = input.reason.clone();
        next.data.implementor_run = input.implementor_run.clone();
        next.data.reviewer_run = "".to_owned();
        next.data.test_revision = "".to_owned();
        next.data.review_revision = "".to_owned();
        let answer = crate::host::RepairAssignmentOutcome::Applied { repair_assignment_applied: crate::host::RepairAssignmentApplied { assignment_id: input.assignment_id.clone(), reason: input.reason.clone(), implementor_run: input.implementor_run.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.ReviewAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::ReviewAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn review_assignment(&mut self, input: crate::host::ReviewAssignment) -> Result<crate::host::ReviewAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReviewAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        // `tests-not-current`: selected by the addressed row.
        if decided(any(&[equal(Some(&input.candidate).map(|value| value.clone()), Some("".to_owned())), equal(Some(&input.candidate).map(|value| value.clone()), Some(&input.test_revision).map(|value| value.clone())).map(|value| !value)]), "controlplane.host.ReviewAssignment")? {
            return Ok(crate::host::ReviewAssignmentOutcome::TestsNotCurrent { error: crate::host::EvidenceNotCurrent });
        }
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ReviewAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Implementing(instance) => crate::host::AnyAssignment::Reviewing(instance.review()),
            _ => return Ok(crate::host::ReviewAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.candidate = input.candidate.clone();
        next.data.test_revision = input.test_revision.clone();
        let answer = crate::host::ReviewAssignmentOutcome::Applied { review_assignment_applied: crate::host::ReviewAssignmentApplied { assignment_id: input.assignment_id.clone(), candidate: input.candidate.clone(), test_revision: input.test_revision.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.SatisfyGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::SatisfyGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn satisfy_goal(&mut self, input: crate::host::SatisfyGoal) -> Result<crate::host::SatisfyGoalOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::SatisfyGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyGoal::Running(instance) => crate::host::AnyGoal::Satisfied(instance.satisfy()),
            _ => return Ok(crate::host::SatisfyGoalOutcome::WrongState { error: crate::host::GoalStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.satisfaction_receipt = input.satisfaction_receipt.clone();
        let answer = crate::host::SatisfyGoalOutcome::Applied { satisfy_goal_applied: crate::host::SatisfyGoalApplied { goal_id: input.goal_id.clone(), satisfaction_receipt: input.satisfaction_receipt.clone() } };
        GoalStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.StartGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::StartGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn start_goal(&mut self, input: crate::host::StartGoal) -> Result<crate::host::StartGoalOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::StartGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyGoal::Paused(instance) => crate::host::AnyGoal::Running(instance.start()),
            _ => return Ok(crate::host::StartGoalOutcome::WrongState { error: crate::host::GoalStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::StartGoalOutcome::Applied { start_goal_applied: crate::host::StartGoalApplied { goal_id: input.goal_id.clone() } };
        GoalStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `controlplane.host.UpdateGoal`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::UpdateGoalBehavior for Generated<P>
where
    P: GoalStorage,
{
    fn update_goal(&mut self, input: crate::host::UpdateGoal) -> Result<crate::host::UpdateGoalOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
            return Ok(crate::host::UpdateGoalOutcome::NotFound { error: crate::host::GoalNotFound });
        };
        let _ = &held;
        // `applied`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Paused | crate::host::GoalState::Running)), "controlplane.host.UpdateGoal")? {
            let Some(held) = GoalStorage::get(&self.ports, &input.goal_id) else {
                return Ok(crate::host::UpdateGoalOutcome::NotFound { error: crate::host::GoalNotFound });
            };
            let _ = &held;
            let before = held.data.clone();
            let mut next = held;
            next.data.objective = input.objective.clone();
            next.data.acceptance = input.acceptance.clone();
            next.data.max_workers = input.max_workers.clone();
            next.data.max_attempts = input.max_attempts.clone();
            next.data.max_minutes = input.max_minutes.clone();
            next.data.planner_model = input.planner_model.clone();
            next.data.implementor_model = input.implementor_model.clone();
            next.data.reviewer_model = input.reviewer_model.clone();
            next.data.merge_authority = input.merge_authority.clone();
            next.data.revision = before.revision + 1;
            next.data.satisfaction_receipt = "".to_owned();
            next.data.planning_fingerprint = "".to_owned();
            let answer = crate::host::UpdateGoalOutcome::Applied { update_goal_applied: crate::host::UpdateGoalApplied { goal_id: input.goal_id.clone(), objective: input.objective.clone(), acceptance: input.acceptance.clone(), max_workers: input.max_workers.clone(), max_attempts: input.max_attempts.clone(), max_minutes: input.max_minutes.clone(), planner_model: input.planner_model.clone(), implementor_model: input.implementor_model.clone(), reviewer_model: input.reviewer_model.clone(), merge_authority: input.merge_authority.clone() } };
            GoalStorage::put(&mut self.ports, next);
            return Ok(answer);
        }
        // `satisfied`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Satisfied)), "controlplane.host.UpdateGoal")? {
            return Ok(crate::host::UpdateGoalOutcome::Satisfied { error: crate::host::GoalStateConflict { state: held.state } });
        }
        // `cancelled`: selected by the addressed row.
        if decided(Some(matches!(held.state, crate::host::GoalState::Cancelled)), "controlplane.host.UpdateGoal")? {
            return Ok(crate::host::UpdateGoalOutcome::Cancelled { error: crate::host::GoalStateConflict { state: held.state } });
        }
        // No declared branch answers this request.
        Err(undeclared("controlplane.host.UpdateGoal"))
    }
}

/// `controlplane.host.AssignmentList`, generated: every row is one the specification fully determines from the stored `controlplane.host.Assignment`s.
impl<P> crate::host::obligations::AssignmentListQuery for Generated<P>
where
    P: AssignmentStorage,
{
    fn assignment_list(&self) -> Result<Vec<crate::host::AssignmentList>, UnmetObligation> {
        let admitted = AssignmentStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::AssignmentList {
                assignment_id: held.data.assignment_id,
                goal_id: held.data.goal_id,
                repository_id: held.data.repository_id,
                story_id: held.data.story_id,
                case_id: held.data.case_id,
                worktree_id: held.data.worktree_id,
                candidate: held.data.candidate,
                attempt: held.data.attempt,
                reason: held.data.reason,
                implementor_run: held.data.implementor_run,
                reviewer_run: held.data.reviewer_run,
                goal_revision: held.data.goal_revision,
                base_revision: held.data.base_revision,
                test_revision: held.data.test_revision,
                review_revision: held.data.review_revision,
                merge_receipt: held.data.merge_receipt,
                state: held.state,
            })
            .collect())
    }
}

/// `controlplane.host.GoalList`, generated: every row is one the specification fully determines from the stored `controlplane.host.Goal`s.
impl<P> crate::host::obligations::GoalListQuery for Generated<P>
where
    P: GoalStorage,
{
    fn goal_list(&self) -> Result<Vec<crate::host::GoalList>, UnmetObligation> {
        let admitted = GoalStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::GoalList {
                goal_id: held.data.goal_id,
                workspace_id: held.data.workspace_id,
                objective: held.data.objective,
                acceptance: held.data.acceptance,
                max_workers: held.data.max_workers,
                max_attempts: held.data.max_attempts,
                max_minutes: held.data.max_minutes,
                planner_model: held.data.planner_model,
                implementor_model: held.data.implementor_model,
                reviewer_model: held.data.reviewer_model,
                merge_authority: held.data.merge_authority,
                revision: held.data.revision,
                satisfaction_receipt: held.data.satisfaction_receipt,
                state: held.state,
                planning_revision: held.data.planning_revision,
                planning_fingerprint: held.data.planning_fingerprint,
                planning_repository: held.data.planning_repository,
                planning_worktree_id: held.data.planning_worktree_id,
                planning_worktree_path: held.data.planning_worktree_path,
                planning_reason: held.data.planning_reason,
                planning_receipt: held.data.planning_receipt,
                planning_phase: held.data.planning_phase,
            })
            .collect())
    }
}

/// `controlplane.host.PublicationIntentList`, generated: every row is one the specification fully determines from the stored `controlplane.host.PublicationIntent`s.
impl<P> crate::host::obligations::PublicationIntentListQuery for Generated<P>
where
    P: PublicationIntentStorage,
{
    fn publication_intent_list(&self) -> Result<Vec<crate::host::PublicationIntentList>, UnmetObligation> {
        let admitted = PublicationIntentStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::PublicationIntentList {
                publication_id: held.data.publication_id,
                assignment_id: held.data.assignment_id,
                candidate: held.data.candidate,
                target: held.data.target,
                expected_base: held.data.expected_base,
                receipt: held.data.receipt,
                reason: held.data.reason,
                state: held.state,
            })
            .collect())
    }
}

/// `controlplane.host.RepositoryRegistrationList`, generated: every row is one the specification fully determines from the stored `controlplane.host.RepositoryRegistration`s.
impl<P> crate::host::obligations::RepositoryRegistrationListQuery for Generated<P>
where
    P: RepositoryRegistrationStorage,
{
    fn repository_registration_list(&self) -> Result<Vec<crate::host::RepositoryRegistrationList>, UnmetObligation> {
        let admitted = RepositoryRegistrationStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::RepositoryRegistrationList {
                repository_id: held.data.repository_id,
                workspace_id: held.data.workspace_id,
                name: held.data.name,
                path: held.data.path,
                common_dir: held.data.common_dir,
                base_branch: held.data.base_branch,
                test_command: held.data.test_command,
                publish_command: held.data.publish_command,
                state: held.state,
            })
            .collect())
    }
}

/// `controlplane.host.WorkspaceDirectoryList`, generated: every row is one the specification fully determines from the stored `controlplane.host.WorkspaceDirectory`s.
impl<P> crate::host::obligations::WorkspaceDirectoryListQuery for Generated<P>
where
    P: WorkspaceDirectoryStorage,
{
    fn workspace_directory_list(&self) -> Result<Vec<crate::host::WorkspaceDirectoryList>, UnmetObligation> {
        let admitted = WorkspaceDirectoryStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::WorkspaceDirectoryList {
                directory_id: held.data.directory_id,
                workspace_id: held.data.workspace_id,
                path: held.data.path,
                repository_common_dirs: held.data.repository_common_dirs,
                managed_common_dirs: held.data.managed_common_dirs,
                state: held.state,
            })
            .collect())
    }
}

/// `controlplane.host.WorkspaceList`, generated: every row is one the specification fully determines from the stored `controlplane.host.Workspace`s.
impl<P> crate::host::obligations::WorkspaceListQuery for Generated<P>
where
    P: WorkspaceStorage,
{
    fn workspace_list(&self) -> Result<Vec<crate::host::WorkspaceList>, UnmetObligation> {
        let admitted = WorkspaceStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::host::WorkspaceList {
                workspace_id: held.data.workspace_id,
                path: held.data.path,
                name: held.data.name,
                state: held.state,
            })
            .collect())
    }
}

/// The typed refusal of a request the model declares no outcome for.
fn undeclared(source: &'static str) -> UnmetObligation {
    let capability = "command behaviour";
    UnmetObligation { capability, source }
}

/// A guard's truth, where it has one: Unknown selects no branch, so the model declares no outcome.
fn decided(truth: Option<bool>, command: &'static str) -> Result<bool, UnmetObligation> {
    truth.ok_or_else(|| undeclared(command))
}

/// Three-valued conjunction: false wins, then Unknown.
fn all(truths: &[Option<bool>]) -> Option<bool> {
    if truths.contains(&Some(false)) {
        Some(false)
    } else if truths.contains(&None) {
        None
    } else {
        Some(true)
    }
}

/// Three-valued disjunction: true wins, then Unknown.
fn any(truths: &[Option<bool>]) -> Option<bool> {
    if truths.contains(&Some(true)) {
        Some(true)
    } else if truths.contains(&None) {
        None
    } else {
        Some(false)
    }
}

/// Equality of two read values; an unread one is Unknown.
fn equal<T: PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {
    Some(left? == right?)
}

/// A decimal rendering as its sign, its whole digits and its fraction digits, without the zeros
/// that do not change its value; `None` where it is not a plain decimal.
fn number_parts(text: &str) -> Option<(bool, String, String)> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole.bytes().chain(fraction.bytes()).all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let whole = whole.trim_start_matches('0').to_owned();
    let fraction = fraction.trim_end_matches('0').to_owned();
    let zero = whole.is_empty() && fraction.is_empty();
    Some((negative && !zero, whole, fraction))
}

/// Compares two decimal renderings exactly; an unread or unparsable one is Unknown.
fn compare_numbers(
    left: Option<String>,
    right: Option<String>,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let (left, right) = (number_parts(&left?)?, number_parts(&right?)?);
    let magnitude = left
        .1
        .len()
        .cmp(&right.1.len())
        .then_with(|| left.1.cmp(&right.1))
        .then_with(|| left.2.cmp(&right.2));
    let ordering = match (left.0, right.0) {
        (false, false) => magnitude,
        (true, true) => magnitude.reverse(),
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
    };
    Some(accepts(ordering))
}
