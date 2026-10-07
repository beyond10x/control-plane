// generated from controlplane v1
// model digest 5ee011354cbdde7e1cd5aaec6e606c149d9e94ef7ac62c9bf1fc21621ff9b8ba
// contract digest c0aa7ecbbdf9304cda7cddf46352bf7a404ef748f1fd5a39222ff2628fadf6ca
// do not edit: regenerate with `ess synthesize --layout crate`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `controlplane.host.Operator`.
    Operator,
    /// `controlplane.host.Supervisor`.
    Supervisor,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::Operator,
        Actor::Supervisor,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::Operator => "controlplane.host.Operator",
            Actor::Supervisor => "controlplane.host.Supervisor",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::Operator => &[
            "controlplane.host.AddWorkspaceDirectory",
            "controlplane.host.ArchiveWorkspace",
            "controlplane.host.CancelGoal",
            "controlplane.host.ConfigureRepository",
            "controlplane.host.CreateGoal",
            "controlplane.host.DeleteGoal",
            "controlplane.host.DisableRepositoryRegistration",
            "controlplane.host.EnableRepositoryRegistration",
            "controlplane.host.PauseGoal",
            "controlplane.host.RegisterRepository",
            "controlplane.host.RegisterWorkspace",
            "controlplane.host.RemoveWorkspaceDirectory",
            "controlplane.host.StartGoal",
            "controlplane.host.UpdateGoal",
        ],
        Actor::Supervisor => &[
            "controlplane.host.ArchiveWorkspace",
            "controlplane.host.BlockAssignment",
            "controlplane.host.CancelAssignment",
            "controlplane.host.CancelGoal",
            "controlplane.host.ClaimAssignment",
            "controlplane.host.CompleteAssignment",
            "controlplane.host.ConfirmPublication",
            "controlplane.host.CreateGoal",
            "controlplane.host.DisableRepositoryRegistration",
            "controlplane.host.EnableRepositoryRegistration",
            "controlplane.host.MarkPublicationUncertain",
            "controlplane.host.MergeAssignment",
            "controlplane.host.PauseGoal",
            "controlplane.host.PreparePublication",
            "controlplane.host.QueueAssignment",
            "controlplane.host.ReadyAssignment",
            "controlplane.host.ReconcileAssignment",
            "controlplane.host.RecordPlanningProgress",
            "controlplane.host.RegisterRepository",
            "controlplane.host.RegisterWorkspace",
            "controlplane.host.RepairAssignment",
            "controlplane.host.ReviewAssignment",
            "controlplane.host.SatisfyGoal",
            "controlplane.host.StartGoal",
        ],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}
