// generated from controlplane v1
// model digest 897a3414c77f9f4e1cf2364f3618ac52b1f26e3c84b3ff542878e9e78509dbd7
// contract digest b6fee6bf66c237bc1382d9b6b607570b4f096d13d3d91c101d1efb3fde6bb9b7
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `controlplane` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
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

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ArchiveWorkspaceApplied(_) => "controlplane.host.ArchiveWorkspaceApplied",
            Self::AssignmentCreated(_) => "controlplane.host.AssignmentCreated",
            Self::BlockAssignmentApplied(_) => "controlplane.host.BlockAssignmentApplied",
            Self::CancelAssignmentApplied(_) => "controlplane.host.CancelAssignmentApplied",
            Self::CancelGoalApplied(_) => "controlplane.host.CancelGoalApplied",
            Self::ClaimAssignmentApplied(_) => "controlplane.host.ClaimAssignmentApplied",
            Self::ClosePublicationApplied(_) => "controlplane.host.ClosePublicationApplied",
            Self::CompleteAssignmentApplied(_) => "controlplane.host.CompleteAssignmentApplied",
            Self::ConfigureRepositoryApplied(_) => "controlplane.host.ConfigureRepositoryApplied",
            Self::ConfirmPublicationApplied(_) => "controlplane.host.ConfirmPublicationApplied",
            Self::DisableRepositoryRegistrationApplied(_) => "controlplane.host.DisableRepositoryRegistrationApplied",
            Self::EnableRepositoryRegistrationApplied(_) => "controlplane.host.EnableRepositoryRegistrationApplied",
            Self::GoalCreated(_) => "controlplane.host.GoalCreated",
            Self::GoalDeleted(_) => "controlplane.host.GoalDeleted",
            Self::MarkPublicationUncertainApplied(_) => "controlplane.host.MarkPublicationUncertainApplied",
            Self::MergeAssignmentApplied(_) => "controlplane.host.MergeAssignmentApplied",
            Self::PauseGoalApplied(_) => "controlplane.host.PauseGoalApplied",
            Self::PlanningProgressRecorded(_) => "controlplane.host.PlanningProgressRecorded",
            Self::PublicationIntentCreated(_) => "controlplane.host.PublicationIntentCreated",
            Self::ReadyAssignmentApplied(_) => "controlplane.host.ReadyAssignmentApplied",
            Self::ReconcileAssignmentApplied(_) => "controlplane.host.ReconcileAssignmentApplied",
            Self::RepairAssignmentApplied(_) => "controlplane.host.RepairAssignmentApplied",
            Self::RepositoryRegistrationCreated(_) => "controlplane.host.RepositoryRegistrationCreated",
            Self::ReviewAssignmentApplied(_) => "controlplane.host.ReviewAssignmentApplied",
            Self::SatisfyGoalApplied(_) => "controlplane.host.SatisfyGoalApplied",
            Self::StartGoalApplied(_) => "controlplane.host.StartGoalApplied",
            Self::UpdateGoalApplied(_) => "controlplane.host.UpdateGoalApplied",
            Self::WorkspaceCreated(_) => "controlplane.host.WorkspaceCreated",
            Self::WorkspaceDirectoryCreated(_) => "controlplane.host.WorkspaceDirectoryCreated",
            Self::WorkspaceDirectoryRemoved(_) => "controlplane.host.WorkspaceDirectoryRemoved",
        }
    }
}

impl From<crate::ports::control_plane::PublishedEvent> for SystemEvent {
    fn from(event: crate::ports::control_plane::PublishedEvent) -> Self {
        match event {
            crate::ports::control_plane::PublishedEvent::ArchiveWorkspaceApplied(event) => Self::ArchiveWorkspaceApplied(event),
            crate::ports::control_plane::PublishedEvent::AssignmentCreated(event) => Self::AssignmentCreated(event),
            crate::ports::control_plane::PublishedEvent::BlockAssignmentApplied(event) => Self::BlockAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::CancelAssignmentApplied(event) => Self::CancelAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::CancelGoalApplied(event) => Self::CancelGoalApplied(event),
            crate::ports::control_plane::PublishedEvent::ClaimAssignmentApplied(event) => Self::ClaimAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::ClosePublicationApplied(event) => Self::ClosePublicationApplied(event),
            crate::ports::control_plane::PublishedEvent::CompleteAssignmentApplied(event) => Self::CompleteAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::ConfigureRepositoryApplied(event) => Self::ConfigureRepositoryApplied(event),
            crate::ports::control_plane::PublishedEvent::ConfirmPublicationApplied(event) => Self::ConfirmPublicationApplied(event),
            crate::ports::control_plane::PublishedEvent::DisableRepositoryRegistrationApplied(event) => Self::DisableRepositoryRegistrationApplied(event),
            crate::ports::control_plane::PublishedEvent::EnableRepositoryRegistrationApplied(event) => Self::EnableRepositoryRegistrationApplied(event),
            crate::ports::control_plane::PublishedEvent::GoalCreated(event) => Self::GoalCreated(event),
            crate::ports::control_plane::PublishedEvent::GoalDeleted(event) => Self::GoalDeleted(event),
            crate::ports::control_plane::PublishedEvent::MarkPublicationUncertainApplied(event) => Self::MarkPublicationUncertainApplied(event),
            crate::ports::control_plane::PublishedEvent::MergeAssignmentApplied(event) => Self::MergeAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::PauseGoalApplied(event) => Self::PauseGoalApplied(event),
            crate::ports::control_plane::PublishedEvent::PlanningProgressRecorded(event) => Self::PlanningProgressRecorded(event),
            crate::ports::control_plane::PublishedEvent::PublicationIntentCreated(event) => Self::PublicationIntentCreated(event),
            crate::ports::control_plane::PublishedEvent::ReadyAssignmentApplied(event) => Self::ReadyAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::ReconcileAssignmentApplied(event) => Self::ReconcileAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::RepairAssignmentApplied(event) => Self::RepairAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::RepositoryRegistrationCreated(event) => Self::RepositoryRegistrationCreated(event),
            crate::ports::control_plane::PublishedEvent::ReviewAssignmentApplied(event) => Self::ReviewAssignmentApplied(event),
            crate::ports::control_plane::PublishedEvent::SatisfyGoalApplied(event) => Self::SatisfyGoalApplied(event),
            crate::ports::control_plane::PublishedEvent::StartGoalApplied(event) => Self::StartGoalApplied(event),
            crate::ports::control_plane::PublishedEvent::UpdateGoalApplied(event) => Self::UpdateGoalApplied(event),
            crate::ports::control_plane::PublishedEvent::WorkspaceCreated(event) => Self::WorkspaceCreated(event),
            crate::ports::control_plane::PublishedEvent::WorkspaceDirectoryCreated(event) => Self::WorkspaceDirectoryCreated(event),
            crate::ports::control_plane::PublishedEvent::WorkspaceDirectoryRemoved(event) => Self::WorkspaceDirectoryRemoved(event),
        }
    }
}

/// The `controlplane` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<ControlPlaneBehaviors> {
    /// The `control-plane` component.
    pub control_plane: crate::ports::control_plane::ControlPlane<ControlPlaneBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<ControlPlaneBehaviors> System<ControlPlaneBehaviors> {
    /// Assembles the system from its components.
    pub fn new(control_plane: crate::ports::control_plane::ControlPlane<ControlPlaneBehaviors>) -> Self {
        Self {
            control_plane,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<ControlPlaneBehaviors> System<ControlPlaneBehaviors>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::ClosePublicationBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DeleteGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), crate::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.control_plane.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
