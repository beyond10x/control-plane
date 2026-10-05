// generated from controlplane v1
// model digest 8e307f3ce0541f736b4688846bf3bc3617af6ba4bd43e0b156673e614f1f8a57
// contract digest c4a296ae41814f3a2a24c5f55da9b458369ad96cbca829869fd81211af1fd1ed
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
            crate::host::AnyAssignment::Implementing(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Merging(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Queued(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::ReadyToMerge(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Blocked(instance.block()),
            _ => return Ok(crate::host::BlockAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::BlockAssignmentOutcome::Applied { block_assignment_applied: crate::host::BlockAssignmentApplied { assignment_id: input.assignment_id.clone() } };
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
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::ClaimAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Queued(instance) => crate::host::AnyAssignment::Implementing(instance.claim()),
            _ => return Ok(crate::host::ClaimAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::ClaimAssignmentOutcome::Applied { claim_assignment_applied: crate::host::ClaimAssignmentApplied { assignment_id: input.assignment_id.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
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
        let next = moved.snapshot();
        let answer = crate::host::CompleteAssignmentOutcome::Applied { complete_assignment_applied: crate::host::CompleteAssignmentApplied { assignment_id: input.assignment_id.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
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
        };
        let answer = crate::host::CreateGoalOutcome::Created { goal_created: crate::host::GoalCreated { goal_id: identity.clone(), workspace_id: input.workspace_id.clone(), objective: input.objective.clone(), acceptance: input.acceptance.clone(), max_workers: input.max_workers.clone(), max_attempts: input.max_attempts.clone(), max_minutes: input.max_minutes.clone(), planner_model: input.planner_model.clone(), implementor_model: input.implementor_model.clone(), reviewer_model: input.reviewer_model.clone(), merge_authority: input.merge_authority.clone() } };
        GoalStorage::put(&mut self.ports, crate::host::AnyGoal::Paused(crate::host::Goal::new(data)).snapshot());
        return Ok(answer);
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

/// `controlplane.host.QueueAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::QueueAssignmentBehavior for Generated<P>
where
    P: TryContext + AssignmentStorage,
{
    fn queue_assignment(&mut self, input: crate::host::QueueAssignment) -> Result<crate::host::QueueAssignmentOutcome, UnmetObligation> {
        let _ = &input;
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
        };
        let answer = crate::host::QueueAssignmentOutcome::Created { assignment_created: crate::host::AssignmentCreated { assignment_id: identity.clone(), goal_id: input.goal_id.clone(), repository_id: input.repository_id.clone(), story_id: input.story_id.clone(), case_id: input.case_id.clone(), worktree_id: input.worktree_id.clone(), candidate: input.candidate.clone(), attempt: input.attempt.clone(), reason: input.reason.clone(), implementor_run: input.implementor_run.clone(), reviewer_run: input.reviewer_run.clone() } };
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
        let next = moved.snapshot();
        let answer = crate::host::ReadyAssignmentOutcome::Applied { ready_assignment_applied: crate::host::ReadyAssignmentApplied { assignment_id: input.assignment_id.clone() } };
        AssignmentStorage::put(&mut self.ports, next);
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

/// `controlplane.host.RepairAssignment`, generated: every outcome is one the specification fully determines.
impl<P> crate::host::obligations::RepairAssignmentBehavior for Generated<P>
where
    P: AssignmentStorage,
{
    fn repair_assignment(&mut self, input: crate::host::RepairAssignment) -> Result<crate::host::RepairAssignmentOutcome, UnmetObligation> {
        let _ = &input;
        // `applied`: the default.
        let Some(held) = AssignmentStorage::get(&self.ports, &input.assignment_id) else {
            return Ok(crate::host::RepairAssignmentOutcome::NotFound { error: crate::host::AssignmentNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::host::AnyAssignment::Blocked(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
            crate::host::AnyAssignment::Reviewing(instance) => crate::host::AnyAssignment::Implementing(instance.repair()),
            _ => return Ok(crate::host::RepairAssignmentOutcome::WrongState { error: crate::host::AssignmentStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        let answer = crate::host::RepairAssignmentOutcome::Applied { repair_assignment_applied: crate::host::RepairAssignmentApplied { assignment_id: input.assignment_id.clone() } };
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
        let next = moved.snapshot();
        let answer = crate::host::ReviewAssignmentOutcome::Applied { review_assignment_applied: crate::host::ReviewAssignmentApplied { assignment_id: input.assignment_id.clone() } };
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
        let next = moved.snapshot();
        let answer = crate::host::SatisfyGoalOutcome::Applied { satisfy_goal_applied: crate::host::SatisfyGoalApplied { goal_id: input.goal_id.clone() } };
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
