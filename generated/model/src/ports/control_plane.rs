// generated from controlplane v1
// model digest 81c7f5e0230fee0d706b0ec2528b81407c999295df8491e6ebc040ba5bdb6bdf
// contract digest 003b16f133632f8fde01042faa8c7c30286cba0b098ee1c71da99eb0004dbf1a
// do not edit: regenerate with `ess synthesize --layout crate`

//! control-plane — the `control-plane` component of `controlplane` v1.
//!
//! Local autonomous engineering service.
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `controlplane.host.ArchiveWorkspaceApplied`.
    ArchiveWorkspaceApplied(crate::host::ArchiveWorkspaceApplied),
    /// `controlplane.host.AssignmentCreated`.
    AssignmentCreated(crate::host::AssignmentCreated),
    /// `controlplane.host.BlockAssignmentApplied`.
    BlockAssignmentApplied(crate::host::BlockAssignmentApplied),
    /// `controlplane.host.CancelAssignmentApplied`.
    CancelAssignmentApplied(crate::host::CancelAssignmentApplied),
    /// `controlplane.host.CancelGoalApplied`.
    CancelGoalApplied(crate::host::CancelGoalApplied),
    /// `controlplane.host.ClaimAssignmentApplied`.
    ClaimAssignmentApplied(crate::host::ClaimAssignmentApplied),
    /// `controlplane.host.ClosePublicationApplied`.
    ClosePublicationApplied(crate::host::ClosePublicationApplied),
    /// `controlplane.host.CompleteAssignmentApplied`.
    CompleteAssignmentApplied(crate::host::CompleteAssignmentApplied),
    /// `controlplane.host.ConfigureRepositoryApplied`.
    ConfigureRepositoryApplied(crate::host::ConfigureRepositoryApplied),
    /// `controlplane.host.ConfirmPublicationApplied`.
    ConfirmPublicationApplied(crate::host::ConfirmPublicationApplied),
    /// `controlplane.host.DisableRepositoryRegistrationApplied`.
    DisableRepositoryRegistrationApplied(crate::host::DisableRepositoryRegistrationApplied),
    /// `controlplane.host.EnableRepositoryRegistrationApplied`.
    EnableRepositoryRegistrationApplied(crate::host::EnableRepositoryRegistrationApplied),
    /// `controlplane.host.GoalCreated`.
    GoalCreated(crate::host::GoalCreated),
    /// `controlplane.host.GoalDeleted`.
    GoalDeleted(crate::host::GoalDeleted),
    /// `controlplane.host.MarkPublicationUncertainApplied`.
    MarkPublicationUncertainApplied(crate::host::MarkPublicationUncertainApplied),
    /// `controlplane.host.MergeAssignmentApplied`.
    MergeAssignmentApplied(crate::host::MergeAssignmentApplied),
    /// `controlplane.host.PauseGoalApplied`.
    PauseGoalApplied(crate::host::PauseGoalApplied),
    /// `controlplane.host.PlanningProgressRecorded`.
    PlanningProgressRecorded(crate::host::PlanningProgressRecorded),
    /// `controlplane.host.PublicationIntentCreated`.
    PublicationIntentCreated(crate::host::PublicationIntentCreated),
    /// `controlplane.host.ReadyAssignmentApplied`.
    ReadyAssignmentApplied(crate::host::ReadyAssignmentApplied),
    /// `controlplane.host.ReconcileAssignmentApplied`.
    ReconcileAssignmentApplied(crate::host::ReconcileAssignmentApplied),
    /// `controlplane.host.RepairAssignmentApplied`.
    RepairAssignmentApplied(crate::host::RepairAssignmentApplied),
    /// `controlplane.host.RepositoryRegistrationCreated`.
    RepositoryRegistrationCreated(crate::host::RepositoryRegistrationCreated),
    /// `controlplane.host.ReviewAssignmentApplied`.
    ReviewAssignmentApplied(crate::host::ReviewAssignmentApplied),
    /// `controlplane.host.SatisfyGoalApplied`.
    SatisfyGoalApplied(crate::host::SatisfyGoalApplied),
    /// `controlplane.host.StartGoalApplied`.
    StartGoalApplied(crate::host::StartGoalApplied),
    /// `controlplane.host.UpdateGoalApplied`.
    UpdateGoalApplied(crate::host::UpdateGoalApplied),
    /// `controlplane.host.WorkspaceCreated`.
    WorkspaceCreated(crate::host::WorkspaceCreated),
    /// `controlplane.host.WorkspaceDirectoryCreated`.
    WorkspaceDirectoryCreated(crate::host::WorkspaceDirectoryCreated),
    /// `controlplane.host.WorkspaceDirectoryRemoved`.
    WorkspaceDirectoryRemoved(crate::host::WorkspaceDirectoryRemoved),
}

/// control-plane — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct ControlPlane<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> ControlPlane<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> ControlPlane<B>
where
    B: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::ClosePublicationBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DeleteGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    /// Accepts `controlplane.host.AddWorkspaceDirectory`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn add_workspace_directory(&mut self, input: crate::host::AddWorkspaceDirectory) -> Result<crate::host::AddWorkspaceDirectoryOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.add_workspace_directory(input)?;
        match &outcome {
            crate::host::AddWorkspaceDirectoryOutcome::Created { workspace_directory_created, .. } => {
                self.outbox.push(PublishedEvent::WorkspaceDirectoryCreated(workspace_directory_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ArchiveWorkspace`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn archive_workspace(&mut self, input: crate::host::ArchiveWorkspace) -> Result<crate::host::ArchiveWorkspaceOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.archive_workspace(input)?;
        match &outcome {
            crate::host::ArchiveWorkspaceOutcome::Applied { archive_workspace_applied, .. } => {
                self.outbox.push(PublishedEvent::ArchiveWorkspaceApplied(archive_workspace_applied.clone()));
            }
            crate::host::ArchiveWorkspaceOutcome::WrongState { .. } => {}
            crate::host::ArchiveWorkspaceOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.BlockAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn block_assignment(&mut self, input: crate::host::BlockAssignment) -> Result<crate::host::BlockAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.block_assignment(input)?;
        match &outcome {
            crate::host::BlockAssignmentOutcome::Applied { block_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::BlockAssignmentApplied(block_assignment_applied.clone()));
            }
            crate::host::BlockAssignmentOutcome::WrongState { .. } => {}
            crate::host::BlockAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.CancelAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn cancel_assignment(&mut self, input: crate::host::CancelAssignment) -> Result<crate::host::CancelAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.cancel_assignment(input)?;
        match &outcome {
            crate::host::CancelAssignmentOutcome::Applied { cancel_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::CancelAssignmentApplied(cancel_assignment_applied.clone()));
            }
            crate::host::CancelAssignmentOutcome::WrongState { .. } => {}
            crate::host::CancelAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.CancelGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn cancel_goal(&mut self, input: crate::host::CancelGoal) -> Result<crate::host::CancelGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.cancel_goal(input)?;
        match &outcome {
            crate::host::CancelGoalOutcome::Applied { cancel_goal_applied, .. } => {
                self.outbox.push(PublishedEvent::CancelGoalApplied(cancel_goal_applied.clone()));
            }
            crate::host::CancelGoalOutcome::WrongState { .. } => {}
            crate::host::CancelGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ClaimAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn claim_assignment(&mut self, input: crate::host::ClaimAssignment) -> Result<crate::host::ClaimAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.claim_assignment(input)?;
        match &outcome {
            crate::host::ClaimAssignmentOutcome::Applied { claim_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::ClaimAssignmentApplied(claim_assignment_applied.clone()));
            }
            crate::host::ClaimAssignmentOutcome::WrongState { .. } => {}
            crate::host::ClaimAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ClosePublication`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn close_publication(&mut self, input: crate::host::ClosePublication) -> Result<crate::host::ClosePublicationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.close_publication(input)?;
        match &outcome {
            crate::host::ClosePublicationOutcome::Applied { close_publication_applied, .. } => {
                self.outbox.push(PublishedEvent::ClosePublicationApplied(close_publication_applied.clone()));
            }
            crate::host::ClosePublicationOutcome::NotFound { .. } => {}
            crate::host::ClosePublicationOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.CompleteAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn complete_assignment(&mut self, input: crate::host::CompleteAssignment) -> Result<crate::host::CompleteAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.complete_assignment(input)?;
        match &outcome {
            crate::host::CompleteAssignmentOutcome::Applied { complete_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::CompleteAssignmentApplied(complete_assignment_applied.clone()));
            }
            crate::host::CompleteAssignmentOutcome::WrongState { .. } => {}
            crate::host::CompleteAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ConfigureRepository`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn configure_repository(&mut self, input: crate::host::ConfigureRepository) -> Result<crate::host::ConfigureRepositoryOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.configure_repository(input)?;
        match &outcome {
            crate::host::ConfigureRepositoryOutcome::Applied { configure_repository_applied, .. } => {
                self.outbox.push(PublishedEvent::ConfigureRepositoryApplied(configure_repository_applied.clone()));
            }
            crate::host::ConfigureRepositoryOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ConfirmPublication`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn confirm_publication(&mut self, input: crate::host::ConfirmPublication) -> Result<crate::host::ConfirmPublicationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.confirm_publication(input)?;
        match &outcome {
            crate::host::ConfirmPublicationOutcome::Applied { confirm_publication_applied, .. } => {
                self.outbox.push(PublishedEvent::ConfirmPublicationApplied(confirm_publication_applied.clone()));
            }
            crate::host::ConfirmPublicationOutcome::NotFound { .. } => {}
            crate::host::ConfirmPublicationOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.CreateGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn create_goal(&mut self, input: crate::host::CreateGoal) -> Result<crate::host::CreateGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.create_goal(input)?;
        match &outcome {
            crate::host::CreateGoalOutcome::Created { goal_created, .. } => {
                self.outbox.push(PublishedEvent::GoalCreated(goal_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.DeleteGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn delete_goal(&mut self, input: crate::host::DeleteGoal) -> Result<crate::host::DeleteGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.delete_goal(input)?;
        match &outcome {
            crate::host::DeleteGoalOutcome::Applied { goal_deleted, .. } => {
                self.outbox.push(PublishedEvent::GoalDeleted(goal_deleted.clone()));
            }
            crate::host::DeleteGoalOutcome::Paused { .. } => {}
            crate::host::DeleteGoalOutcome::Running { .. } => {}
            crate::host::DeleteGoalOutcome::Satisfied { .. } => {}
            crate::host::DeleteGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.DisableRepositoryRegistration`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn disable_repository_registration(&mut self, input: crate::host::DisableRepositoryRegistration) -> Result<crate::host::DisableRepositoryRegistrationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.disable_repository_registration(input)?;
        match &outcome {
            crate::host::DisableRepositoryRegistrationOutcome::Applied { disable_repository_registration_applied, .. } => {
                self.outbox.push(PublishedEvent::DisableRepositoryRegistrationApplied(disable_repository_registration_applied.clone()));
            }
            crate::host::DisableRepositoryRegistrationOutcome::WrongState { .. } => {}
            crate::host::DisableRepositoryRegistrationOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.EnableRepositoryRegistration`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn enable_repository_registration(&mut self, input: crate::host::EnableRepositoryRegistration) -> Result<crate::host::EnableRepositoryRegistrationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.enable_repository_registration(input)?;
        match &outcome {
            crate::host::EnableRepositoryRegistrationOutcome::Applied { enable_repository_registration_applied, .. } => {
                self.outbox.push(PublishedEvent::EnableRepositoryRegistrationApplied(enable_repository_registration_applied.clone()));
            }
            crate::host::EnableRepositoryRegistrationOutcome::WrongState { .. } => {}
            crate::host::EnableRepositoryRegistrationOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.MarkPublicationUncertain`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn mark_publication_uncertain(&mut self, input: crate::host::MarkPublicationUncertain) -> Result<crate::host::MarkPublicationUncertainOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.mark_publication_uncertain(input)?;
        match &outcome {
            crate::host::MarkPublicationUncertainOutcome::Applied { mark_publication_uncertain_applied, .. } => {
                self.outbox.push(PublishedEvent::MarkPublicationUncertainApplied(mark_publication_uncertain_applied.clone()));
            }
            crate::host::MarkPublicationUncertainOutcome::NotFound { .. } => {}
            crate::host::MarkPublicationUncertainOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.MergeAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn merge_assignment(&mut self, input: crate::host::MergeAssignment) -> Result<crate::host::MergeAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.merge_assignment(input)?;
        match &outcome {
            crate::host::MergeAssignmentOutcome::Applied { merge_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::MergeAssignmentApplied(merge_assignment_applied.clone()));
            }
            crate::host::MergeAssignmentOutcome::WrongState { .. } => {}
            crate::host::MergeAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.PauseGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn pause_goal(&mut self, input: crate::host::PauseGoal) -> Result<crate::host::PauseGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.pause_goal(input)?;
        match &outcome {
            crate::host::PauseGoalOutcome::Applied { pause_goal_applied, .. } => {
                self.outbox.push(PublishedEvent::PauseGoalApplied(pause_goal_applied.clone()));
            }
            crate::host::PauseGoalOutcome::WrongState { .. } => {}
            crate::host::PauseGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.PreparePublication`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn prepare_publication(&mut self, input: crate::host::PreparePublication) -> Result<crate::host::PreparePublicationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.prepare_publication(input)?;
        match &outcome {
            crate::host::PreparePublicationOutcome::Created { publication_intent_created, .. } => {
                self.outbox.push(PublishedEvent::PublicationIntentCreated(publication_intent_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.QueueAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn queue_assignment(&mut self, input: crate::host::QueueAssignment) -> Result<crate::host::QueueAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.queue_assignment(input)?;
        match &outcome {
            crate::host::QueueAssignmentOutcome::Created { assignment_created, .. } => {
                self.outbox.push(PublishedEvent::AssignmentCreated(assignment_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ReadyAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn ready_assignment(&mut self, input: crate::host::ReadyAssignment) -> Result<crate::host::ReadyAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.ready_assignment(input)?;
        match &outcome {
            crate::host::ReadyAssignmentOutcome::Applied { ready_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::ReadyAssignmentApplied(ready_assignment_applied.clone()));
            }
            crate::host::ReadyAssignmentOutcome::WrongState { .. } => {}
            crate::host::ReadyAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ReconcileAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn reconcile_assignment(&mut self, input: crate::host::ReconcileAssignment) -> Result<crate::host::ReconcileAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.reconcile_assignment(input)?;
        match &outcome {
            crate::host::ReconcileAssignmentOutcome::Applied { reconcile_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::ReconcileAssignmentApplied(reconcile_assignment_applied.clone()));
            }
            crate::host::ReconcileAssignmentOutcome::NotFound { .. } => {}
            crate::host::ReconcileAssignmentOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.RecordPlanningProgress`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_planning_progress(&mut self, input: crate::host::RecordPlanningProgress) -> Result<crate::host::RecordPlanningProgressOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_planning_progress(input)?;
        match &outcome {
            crate::host::RecordPlanningProgressOutcome::Applied { planning_progress_recorded, .. } => {
                self.outbox.push(PublishedEvent::PlanningProgressRecorded(planning_progress_recorded.clone()));
            }
            crate::host::RecordPlanningProgressOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.RegisterRepository`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn register_repository(&mut self, input: crate::host::RegisterRepository) -> Result<crate::host::RegisterRepositoryOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.register_repository(input)?;
        match &outcome {
            crate::host::RegisterRepositoryOutcome::Created { repository_registration_created, .. } => {
                self.outbox.push(PublishedEvent::RepositoryRegistrationCreated(repository_registration_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.RegisterWorkspace`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn register_workspace(&mut self, input: crate::host::RegisterWorkspace) -> Result<crate::host::RegisterWorkspaceOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.register_workspace(input)?;
        match &outcome {
            crate::host::RegisterWorkspaceOutcome::Created { workspace_created, .. } => {
                self.outbox.push(PublishedEvent::WorkspaceCreated(workspace_created.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.RemoveWorkspaceDirectory`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn remove_workspace_directory(&mut self, input: crate::host::RemoveWorkspaceDirectory) -> Result<crate::host::RemoveWorkspaceDirectoryOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.remove_workspace_directory(input)?;
        match &outcome {
            crate::host::RemoveWorkspaceDirectoryOutcome::Applied { workspace_directory_removed, .. } => {
                self.outbox.push(PublishedEvent::WorkspaceDirectoryRemoved(workspace_directory_removed.clone()));
            }
            crate::host::RemoveWorkspaceDirectoryOutcome::WrongState { .. } => {}
            crate::host::RemoveWorkspaceDirectoryOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.RepairAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn repair_assignment(&mut self, input: crate::host::RepairAssignment) -> Result<crate::host::RepairAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.repair_assignment(input)?;
        match &outcome {
            crate::host::RepairAssignmentOutcome::Applied { repair_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::RepairAssignmentApplied(repair_assignment_applied.clone()));
            }
            crate::host::RepairAssignmentOutcome::WrongState { .. } => {}
            crate::host::RepairAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.ReviewAssignment`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn review_assignment(&mut self, input: crate::host::ReviewAssignment) -> Result<crate::host::ReviewAssignmentOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.review_assignment(input)?;
        match &outcome {
            crate::host::ReviewAssignmentOutcome::Applied { review_assignment_applied, .. } => {
                self.outbox.push(PublishedEvent::ReviewAssignmentApplied(review_assignment_applied.clone()));
            }
            crate::host::ReviewAssignmentOutcome::WrongState { .. } => {}
            crate::host::ReviewAssignmentOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.SatisfyGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn satisfy_goal(&mut self, input: crate::host::SatisfyGoal) -> Result<crate::host::SatisfyGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.satisfy_goal(input)?;
        match &outcome {
            crate::host::SatisfyGoalOutcome::Applied { satisfy_goal_applied, .. } => {
                self.outbox.push(PublishedEvent::SatisfyGoalApplied(satisfy_goal_applied.clone()));
            }
            crate::host::SatisfyGoalOutcome::WrongState { .. } => {}
            crate::host::SatisfyGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.StartGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn start_goal(&mut self, input: crate::host::StartGoal) -> Result<crate::host::StartGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.start_goal(input)?;
        match &outcome {
            crate::host::StartGoalOutcome::Applied { start_goal_applied, .. } => {
                self.outbox.push(PublishedEvent::StartGoalApplied(start_goal_applied.clone()));
            }
            crate::host::StartGoalOutcome::WrongState { .. } => {}
            crate::host::StartGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `controlplane.host.UpdateGoal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn update_goal(&mut self, input: crate::host::UpdateGoal) -> Result<crate::host::UpdateGoalOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.update_goal(input)?;
        match &outcome {
            crate::host::UpdateGoalOutcome::Applied { update_goal_applied, .. } => {
                self.outbox.push(PublishedEvent::UpdateGoalApplied(update_goal_applied.clone()));
            }
            crate::host::UpdateGoalOutcome::Satisfied { .. } => {}
            crate::host::UpdateGoalOutcome::Cancelled { .. } => {}
            crate::host::UpdateGoalOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Serves `controlplane.host.AssignmentList` at `read_your_writes` consistency, from the owed projection.
    pub fn assignment_list(&self) -> Result<Vec<crate::host::AssignmentList>, crate::obligation::UnmetObligation> {
        self.behaviors.assignment_list()
    }

    /// Serves `controlplane.host.GoalList` at `read_your_writes` consistency, from the owed projection.
    pub fn goal_list(&self) -> Result<Vec<crate::host::GoalList>, crate::obligation::UnmetObligation> {
        self.behaviors.goal_list()
    }

    /// Serves `controlplane.host.PublicationIntentList` at `read_your_writes` consistency, from the owed projection.
    pub fn publication_intent_list(&self) -> Result<Vec<crate::host::PublicationIntentList>, crate::obligation::UnmetObligation> {
        self.behaviors.publication_intent_list()
    }

    /// Serves `controlplane.host.RepositoryRegistrationList` at `read_your_writes` consistency, from the owed projection.
    pub fn repository_registration_list(&self) -> Result<Vec<crate::host::RepositoryRegistrationList>, crate::obligation::UnmetObligation> {
        self.behaviors.repository_registration_list()
    }

    /// Serves `controlplane.host.WorkspaceDirectoryList` at `read_your_writes` consistency, from the owed projection.
    pub fn workspace_directory_list(&self) -> Result<Vec<crate::host::WorkspaceDirectoryList>, crate::obligation::UnmetObligation> {
        self.behaviors.workspace_directory_list()
    }

    /// Serves `controlplane.host.WorkspaceList` at `read_your_writes` consistency, from the owed projection.
    pub fn workspace_list(&self) -> Result<Vec<crate::host::WorkspaceList>, crate::obligation::UnmetObligation> {
        self.behaviors.workspace_list()
    }
}
