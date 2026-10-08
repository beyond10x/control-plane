// generated from controlplane v1
// model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
// contract digest e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
// do not edit: regenerate with `ess synthesize --layout crate`

//! host — `controlplane.host`.
//!
//! Local workspace goals and assignments; AEP owns stories, Loom owns agent executions.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// The states of `controlplane.host.Assignment`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Assignment<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentState {
    /// `Blocked`.
    Blocked,
    /// `Cancelled`.
    Cancelled,
    /// `Implementing`.
    Implementing,
    /// `Merged`.
    Merged,
    /// `Merging`.
    Merging,
    /// `Queued`.
    Queued,
    /// `ReadyToMerge`.
    ReadyToMerge,
    /// `Reviewing`.
    Reviewing,
}

/// The states of `controlplane.host.Goal`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Goal<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalState {
    /// `Cancelled`.
    Cancelled,
    /// `Paused`.
    Paused,
    /// `Running`.
    Running,
    /// `Satisfied`.
    Satisfied,
}

/// PlanningPhase — `controlplane.host.PlanningPhase`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningPhase {
    /// `Idle`.
    Idle,
    /// `Provisioning`.
    Provisioning,
    /// `Planning`.
    Planning,
    /// `Validated`.
    Validated,
    /// `Queued`.
    Queued,
    /// `Blocked`.
    Blocked,
}

/// The states of `controlplane.host.PublicationIntent`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `PublicationIntent<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationIntentState {
    /// `Confirmed`.
    Confirmed,
    /// `NotPublished`.
    NotPublished,
    /// `Prepared`.
    Prepared,
    /// `Uncertain`.
    Uncertain,
}

/// The states of `controlplane.host.RepositoryRegistration`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `RepositoryRegistration<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryRegistrationState {
    /// `Disabled`.
    Disabled,
    /// `Registered`.
    Registered,
}

/// The states of `controlplane.host.Workspace`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Workspace<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceState {
    /// `Archived`.
    Archived,
    /// `Registered`.
    Registered,
}

/// The states of `controlplane.host.WorkspaceDirectory`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `WorkspaceDirectory<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceDirectoryState {
    /// `Registered`.
    Registered,
    /// `Removed`.
    Removed,
}

/// What Assignment — `controlplane.host.Assignment` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Assignment<S>`], and at a boundary by [`AssignmentSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentData {
    /// The identity: `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `goal_id` — `Uuid`.
    ///
    /// Carries `assignments`: `controlplane.host.Goal` owns many `controlplane.host.Assignment`.
    pub goal_id: crate::primitives::Uuid,
    /// `repository_id` — `Uuid`.
    ///
    /// Carries `repository`: `controlplane.host.Assignment` references one `controlplane.host.RepositoryRegistration`.
    pub repository_id: crate::primitives::Uuid,
    /// `story_id` — `String`.
    pub story_id: String,
    /// `case_id` — `String`.
    pub case_id: String,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `attempt` — `Integer`.
    pub attempt: i64,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `goal_revision` — `Integer`.
    pub goal_revision: i64,
    /// `base_revision` — `String`.
    pub base_revision: String,
    /// `test_revision` — `String`.
    pub test_revision: String,
    /// `review_revision` — `String`.
    pub review_revision: String,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
}

/// The states of `controlplane.host.Assignment`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](assignment_state::Marker), so [`Assignment<S>`](Assignment) can only ever rest in a real state.
pub mod assignment_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Blocked {}
        impl Sealed for super::Cancelled {}
        impl Sealed for super::Implementing {}
        impl Sealed for super::Merged {}
        impl Sealed for super::Merging {}
        impl Sealed for super::Queued {}
        impl Sealed for super::ReadyToMerge {}
        impl Sealed for super::Reviewing {}
    }

    /// A declared state of `Assignment`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::AssignmentState;
    }

    /// `Blocked`.
    pub struct Blocked;

    impl Marker for Blocked {
        const STATE: super::AssignmentState = super::AssignmentState::Blocked;
    }

    /// `Cancelled`. Terminal: an instance may rest here forever.
    pub struct Cancelled;

    impl Marker for Cancelled {
        const STATE: super::AssignmentState = super::AssignmentState::Cancelled;
    }

    /// `Implementing`.
    pub struct Implementing;

    impl Marker for Implementing {
        const STATE: super::AssignmentState = super::AssignmentState::Implementing;
    }

    /// `Merged`. Terminal: an instance may rest here forever.
    pub struct Merged;

    impl Marker for Merged {
        const STATE: super::AssignmentState = super::AssignmentState::Merged;
    }

    /// `Merging`.
    pub struct Merging;

    impl Marker for Merging {
        const STATE: super::AssignmentState = super::AssignmentState::Merging;
    }

    /// `Queued`. Where a new instance starts.
    pub struct Queued;

    impl Marker for Queued {
        const STATE: super::AssignmentState = super::AssignmentState::Queued;
    }

    /// `ReadyToMerge`.
    pub struct ReadyToMerge;

    impl Marker for ReadyToMerge {
        const STATE: super::AssignmentState = super::AssignmentState::ReadyToMerge;
    }

    /// `Reviewing`.
    pub struct Reviewing;

    impl Marker for Reviewing {
        const STATE: super::AssignmentState = super::AssignmentState::Reviewing;
    }
}

/// Assignment — `controlplane.host.Assignment` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Queued`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`AssignmentSnapshot`]
/// and [`AssignmentSnapshot::refine`].
pub struct Assignment<S: assignment_state::Marker> {
    data: AssignmentData,
    state: core::marker::PhantomData<S>,
}

impl<S: assignment_state::Marker> Assignment<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> AssignmentState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &AssignmentData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> AssignmentData {
        self.data
    }
}

impl Assignment<assignment_state::Queued> {
    /// A new instance, resting in `Queued` — the only state the lifecycle starts one in.
    pub fn new(data: AssignmentData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::Blocked> {
    /// `repair` — `Blocked` → `Implementing`. Taken by the `rebased` outcome of `controlplane.host.RepairAssignment`, the `applied` outcome of `controlplane.host.RepairAssignment`.
    pub fn repair(self) -> Assignment<assignment_state::Implementing> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `Blocked` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Blocked` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelAssignment`.
    pub fn cancel(self) -> Assignment<assignment_state::Cancelled> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reconcile` — `Blocked` → `Merged`. Taken by the `applied` outcome of `controlplane.host.ReconcileAssignment`.
    pub fn reconcile(self) -> Assignment<assignment_state::Merged> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::Implementing> {
    /// `review` — `Implementing` → `Reviewing`. Taken by the `applied` outcome of `controlplane.host.ReviewAssignment`.
    pub fn review(self) -> Assignment<assignment_state::Reviewing> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `Implementing` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Implementing` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelAssignment`.
    pub fn cancel(self) -> Assignment<assignment_state::Cancelled> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::Merging> {
    /// `complete` — `Merging` → `Merged`. Taken by the `applied` outcome of `controlplane.host.CompleteAssignment`.
    pub fn complete(self) -> Assignment<assignment_state::Merged> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `Merging` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reconcile` — `Merging` → `Merged`. Taken by the `applied` outcome of `controlplane.host.ReconcileAssignment`.
    pub fn reconcile(self) -> Assignment<assignment_state::Merged> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::Queued> {
    /// `claim` — `Queued` → `Implementing`. Taken by the `applied` outcome of `controlplane.host.ClaimAssignment`.
    pub fn claim(self) -> Assignment<assignment_state::Implementing> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `Queued` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Queued` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelAssignment`.
    pub fn cancel(self) -> Assignment<assignment_state::Cancelled> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::ReadyToMerge> {
    /// `merge` — `ReadyToMerge` → `Merging`. Taken by the `applied` outcome of `controlplane.host.MergeAssignment`.
    pub fn merge(self) -> Assignment<assignment_state::Merging> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `ReadyToMerge` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `ReadyToMerge` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelAssignment`.
    pub fn cancel(self) -> Assignment<assignment_state::Cancelled> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assignment<assignment_state::Reviewing> {
    /// `repair` — `Reviewing` → `Implementing`. Taken by the `rebased` outcome of `controlplane.host.RepairAssignment`, the `applied` outcome of `controlplane.host.RepairAssignment`.
    pub fn repair(self) -> Assignment<assignment_state::Implementing> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `ready` — `Reviewing` → `ReadyToMerge`. Taken by the `applied` outcome of `controlplane.host.ReadyAssignment`.
    pub fn ready(self) -> Assignment<assignment_state::ReadyToMerge> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `block` — `Reviewing` → `Blocked`. Taken by the `applied` outcome of `controlplane.host.BlockAssignment`.
    pub fn block(self) -> Assignment<assignment_state::Blocked> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Reviewing` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelAssignment`.
    pub fn cancel(self) -> Assignment<assignment_state::Cancelled> {
        Assignment {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.Assignment` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`AssignmentSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: AssignmentState,
    /// What it holds.
    pub data: AssignmentData,
}

/// An `Assignment` in whichever declared state it was found.
pub enum AnyAssignment {
    /// Resting in `Blocked`.
    Blocked(Assignment<assignment_state::Blocked>),
    /// Resting in `Cancelled`.
    Cancelled(Assignment<assignment_state::Cancelled>),
    /// Resting in `Implementing`.
    Implementing(Assignment<assignment_state::Implementing>),
    /// Resting in `Merged`.
    Merged(Assignment<assignment_state::Merged>),
    /// Resting in `Merging`.
    Merging(Assignment<assignment_state::Merging>),
    /// Resting in `Queued`.
    Queued(Assignment<assignment_state::Queued>),
    /// Resting in `ReadyToMerge`.
    ReadyToMerge(Assignment<assignment_state::ReadyToMerge>),
    /// Resting in `Reviewing`.
    Reviewing(Assignment<assignment_state::Reviewing>),
}

impl AssignmentSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `AssignmentState` cannot spell one.
    pub fn refine(self) -> AnyAssignment {
        match self.state {
            AssignmentState::Blocked => AnyAssignment::Blocked(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Cancelled => AnyAssignment::Cancelled(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Implementing => AnyAssignment::Implementing(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Merged => AnyAssignment::Merged(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Merging => AnyAssignment::Merging(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Queued => AnyAssignment::Queued(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::ReadyToMerge => AnyAssignment::ReadyToMerge(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssignmentState::Reviewing => AnyAssignment::Reviewing(Assignment {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyAssignment {
    /// The state, as the runtime value.
    pub fn state(&self) -> AssignmentState {
        match self {
            Self::Blocked(_) => AssignmentState::Blocked,
            Self::Cancelled(_) => AssignmentState::Cancelled,
            Self::Implementing(_) => AssignmentState::Implementing,
            Self::Merged(_) => AssignmentState::Merged,
            Self::Merging(_) => AssignmentState::Merging,
            Self::Queued(_) => AssignmentState::Queued,
            Self::ReadyToMerge(_) => AssignmentState::ReadyToMerge,
            Self::Reviewing(_) => AssignmentState::Reviewing,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> AssignmentSnapshot {
        match self {
            Self::Blocked(instance) => AssignmentSnapshot {
                state: AssignmentState::Blocked,
                data: instance.into_data(),
            },
            Self::Cancelled(instance) => AssignmentSnapshot {
                state: AssignmentState::Cancelled,
                data: instance.into_data(),
            },
            Self::Implementing(instance) => AssignmentSnapshot {
                state: AssignmentState::Implementing,
                data: instance.into_data(),
            },
            Self::Merged(instance) => AssignmentSnapshot {
                state: AssignmentState::Merged,
                data: instance.into_data(),
            },
            Self::Merging(instance) => AssignmentSnapshot {
                state: AssignmentState::Merging,
                data: instance.into_data(),
            },
            Self::Queued(instance) => AssignmentSnapshot {
                state: AssignmentState::Queued,
                data: instance.into_data(),
            },
            Self::ReadyToMerge(instance) => AssignmentSnapshot {
                state: AssignmentState::ReadyToMerge,
                data: instance.into_data(),
            },
            Self::Reviewing(instance) => AssignmentSnapshot {
                state: AssignmentState::Reviewing,
                data: instance.into_data(),
            },
        }
    }
}

/// What Goal — `controlplane.host.Goal` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Goal<S>`], and at a boundary by [`GoalSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalData {
    /// The identity: `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    ///
    /// Carries `goals`: `controlplane.host.Workspace` owns many `controlplane.host.Goal`.
    pub workspace_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
    /// `revision` — `Integer`.
    pub revision: i64,
    /// `satisfaction_receipt` — `String`.
    pub satisfaction_receipt: String,
    /// `planning_revision` — `Integer`.
    pub planning_revision: i64,
    /// `planning_fingerprint` — `String`.
    pub planning_fingerprint: String,
    /// `planning_repository` — `String`.
    pub planning_repository: String,
    /// `planning_worktree_id` — `String`.
    pub planning_worktree_id: String,
    /// `planning_worktree_path` — `String`.
    pub planning_worktree_path: String,
    /// `planning_reason` — `String`.
    pub planning_reason: String,
    /// `planning_receipt` — `String`.
    pub planning_receipt: String,
    /// `planning_phase` — `controlplane.host.PlanningPhase`.
    pub planning_phase: PlanningPhase,
}

/// The states of `controlplane.host.Goal`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](goal_state::Marker), so [`Goal<S>`](Goal) can only ever rest in a real state.
pub mod goal_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Cancelled {}
        impl Sealed for super::Paused {}
        impl Sealed for super::Running {}
        impl Sealed for super::Satisfied {}
    }

    /// A declared state of `Goal`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::GoalState;
    }

    /// `Cancelled`. Terminal: an instance may rest here forever.
    pub struct Cancelled;

    impl Marker for Cancelled {
        const STATE: super::GoalState = super::GoalState::Cancelled;
    }

    /// `Paused`. Where a new instance starts.
    pub struct Paused;

    impl Marker for Paused {
        const STATE: super::GoalState = super::GoalState::Paused;
    }

    /// `Running`.
    pub struct Running;

    impl Marker for Running {
        const STATE: super::GoalState = super::GoalState::Running;
    }

    /// `Satisfied`. Terminal: an instance may rest here forever.
    pub struct Satisfied;

    impl Marker for Satisfied {
        const STATE: super::GoalState = super::GoalState::Satisfied;
    }
}

/// Goal — `controlplane.host.Goal` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Paused`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`GoalSnapshot`]
/// and [`GoalSnapshot::refine`].
pub struct Goal<S: goal_state::Marker> {
    data: GoalData,
    state: core::marker::PhantomData<S>,
}

impl<S: goal_state::Marker> Goal<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> GoalState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &GoalData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> GoalData {
        self.data
    }
}

impl Goal<goal_state::Paused> {
    /// A new instance, resting in `Paused` — the only state the lifecycle starts one in.
    pub fn new(data: GoalData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Goal<goal_state::Paused> {
    /// `start` — `Paused` → `Running`. Taken by the `applied` outcome of `controlplane.host.StartGoal`.
    pub fn start(self) -> Goal<goal_state::Running> {
        Goal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Paused` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelGoal`.
    pub fn cancel(self) -> Goal<goal_state::Cancelled> {
        Goal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Goal<goal_state::Running> {
    /// `pause` — `Running` → `Paused`. Taken by the `applied` outcome of `controlplane.host.PauseGoal`.
    pub fn pause(self) -> Goal<goal_state::Paused> {
        Goal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `satisfy` — `Running` → `Satisfied`. Taken by the `applied` outcome of `controlplane.host.SatisfyGoal`.
    pub fn satisfy(self) -> Goal<goal_state::Satisfied> {
        Goal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Running` → `Cancelled`. Taken by the `applied` outcome of `controlplane.host.CancelGoal`.
    pub fn cancel(self) -> Goal<goal_state::Cancelled> {
        Goal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.Goal` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`GoalSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: GoalState,
    /// What it holds.
    pub data: GoalData,
}

/// An `Goal` in whichever declared state it was found.
pub enum AnyGoal {
    /// Resting in `Cancelled`.
    Cancelled(Goal<goal_state::Cancelled>),
    /// Resting in `Paused`.
    Paused(Goal<goal_state::Paused>),
    /// Resting in `Running`.
    Running(Goal<goal_state::Running>),
    /// Resting in `Satisfied`.
    Satisfied(Goal<goal_state::Satisfied>),
}

impl GoalSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `GoalState` cannot spell one.
    pub fn refine(self) -> AnyGoal {
        match self.state {
            GoalState::Cancelled => AnyGoal::Cancelled(Goal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            GoalState::Paused => AnyGoal::Paused(Goal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            GoalState::Running => AnyGoal::Running(Goal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            GoalState::Satisfied => AnyGoal::Satisfied(Goal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyGoal {
    /// The state, as the runtime value.
    pub fn state(&self) -> GoalState {
        match self {
            Self::Cancelled(_) => GoalState::Cancelled,
            Self::Paused(_) => GoalState::Paused,
            Self::Running(_) => GoalState::Running,
            Self::Satisfied(_) => GoalState::Satisfied,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> GoalSnapshot {
        match self {
            Self::Cancelled(instance) => GoalSnapshot {
                state: GoalState::Cancelled,
                data: instance.into_data(),
            },
            Self::Paused(instance) => GoalSnapshot {
                state: GoalState::Paused,
                data: instance.into_data(),
            },
            Self::Running(instance) => GoalSnapshot {
                state: GoalState::Running,
                data: instance.into_data(),
            },
            Self::Satisfied(instance) => GoalSnapshot {
                state: GoalState::Satisfied,
                data: instance.into_data(),
            },
        }
    }
}

/// What PublicationIntent — `controlplane.host.PublicationIntent` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`PublicationIntent<S>`], and at a boundary by [`PublicationIntentSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentData {
    /// The identity: `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `assignment_id` — `Uuid`.
    ///
    /// Carries `publications`: `controlplane.host.Assignment` owns many `controlplane.host.PublicationIntent`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `target` — `String`.
    pub target: String,
    /// `expected_base` — `String`.
    pub expected_base: String,
    /// `receipt` — `String`.
    pub receipt: String,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

/// The states of `controlplane.host.PublicationIntent`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](publication_intent_state::Marker), so [`PublicationIntent<S>`](PublicationIntent) can only ever rest in a real state.
pub mod publication_intent_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Confirmed {}
        impl Sealed for super::NotPublished {}
        impl Sealed for super::Prepared {}
        impl Sealed for super::Uncertain {}
    }

    /// A declared state of `PublicationIntent`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::PublicationIntentState;
    }

    /// `Confirmed`. Terminal: an instance may rest here forever.
    pub struct Confirmed;

    impl Marker for Confirmed {
        const STATE: super::PublicationIntentState = super::PublicationIntentState::Confirmed;
    }

    /// `NotPublished`. Terminal: an instance may rest here forever.
    pub struct NotPublished;

    impl Marker for NotPublished {
        const STATE: super::PublicationIntentState = super::PublicationIntentState::NotPublished;
    }

    /// `Prepared`. Where a new instance starts.
    pub struct Prepared;

    impl Marker for Prepared {
        const STATE: super::PublicationIntentState = super::PublicationIntentState::Prepared;
    }

    /// `Uncertain`.
    pub struct Uncertain;

    impl Marker for Uncertain {
        const STATE: super::PublicationIntentState = super::PublicationIntentState::Uncertain;
    }
}

/// PublicationIntent — `controlplane.host.PublicationIntent` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Prepared`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`PublicationIntentSnapshot`]
/// and [`PublicationIntentSnapshot::refine`].
pub struct PublicationIntent<S: publication_intent_state::Marker> {
    data: PublicationIntentData,
    state: core::marker::PhantomData<S>,
}

impl<S: publication_intent_state::Marker> PublicationIntent<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> PublicationIntentState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &PublicationIntentData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> PublicationIntentData {
        self.data
    }
}

impl PublicationIntent<publication_intent_state::Prepared> {
    /// A new instance, resting in `Prepared` — the only state the lifecycle starts one in.
    pub fn new(data: PublicationIntentData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl PublicationIntent<publication_intent_state::Prepared> {
    /// `uncertain` — `Prepared` → `Uncertain`. Taken by the `applied` outcome of `controlplane.host.MarkPublicationUncertain`.
    pub fn uncertain(self) -> PublicationIntent<publication_intent_state::Uncertain> {
        PublicationIntent {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `confirm` — `Prepared` → `Confirmed`. Taken by the `applied` outcome of `controlplane.host.ConfirmPublication`.
    pub fn confirm(self) -> PublicationIntent<publication_intent_state::Confirmed> {
        PublicationIntent {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `close` — `Prepared` → `NotPublished`. Taken by the `applied` outcome of `controlplane.host.ClosePublication`.
    pub fn close(self) -> PublicationIntent<publication_intent_state::NotPublished> {
        PublicationIntent {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl PublicationIntent<publication_intent_state::Uncertain> {
    /// `confirm` — `Uncertain` → `Confirmed`. Taken by the `applied` outcome of `controlplane.host.ConfirmPublication`.
    pub fn confirm(self) -> PublicationIntent<publication_intent_state::Confirmed> {
        PublicationIntent {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `close` — `Uncertain` → `NotPublished`. Taken by the `applied` outcome of `controlplane.host.ClosePublication`.
    pub fn close(self) -> PublicationIntent<publication_intent_state::NotPublished> {
        PublicationIntent {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.PublicationIntent` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`PublicationIntentSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: PublicationIntentState,
    /// What it holds.
    pub data: PublicationIntentData,
}

/// An `PublicationIntent` in whichever declared state it was found.
pub enum AnyPublicationIntent {
    /// Resting in `Confirmed`.
    Confirmed(PublicationIntent<publication_intent_state::Confirmed>),
    /// Resting in `NotPublished`.
    NotPublished(PublicationIntent<publication_intent_state::NotPublished>),
    /// Resting in `Prepared`.
    Prepared(PublicationIntent<publication_intent_state::Prepared>),
    /// Resting in `Uncertain`.
    Uncertain(PublicationIntent<publication_intent_state::Uncertain>),
}

impl PublicationIntentSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `PublicationIntentState` cannot spell one.
    pub fn refine(self) -> AnyPublicationIntent {
        match self.state {
            PublicationIntentState::Confirmed => AnyPublicationIntent::Confirmed(PublicationIntent {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            PublicationIntentState::NotPublished => AnyPublicationIntent::NotPublished(PublicationIntent {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            PublicationIntentState::Prepared => AnyPublicationIntent::Prepared(PublicationIntent {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            PublicationIntentState::Uncertain => AnyPublicationIntent::Uncertain(PublicationIntent {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyPublicationIntent {
    /// The state, as the runtime value.
    pub fn state(&self) -> PublicationIntentState {
        match self {
            Self::Confirmed(_) => PublicationIntentState::Confirmed,
            Self::NotPublished(_) => PublicationIntentState::NotPublished,
            Self::Prepared(_) => PublicationIntentState::Prepared,
            Self::Uncertain(_) => PublicationIntentState::Uncertain,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> PublicationIntentSnapshot {
        match self {
            Self::Confirmed(instance) => PublicationIntentSnapshot {
                state: PublicationIntentState::Confirmed,
                data: instance.into_data(),
            },
            Self::NotPublished(instance) => PublicationIntentSnapshot {
                state: PublicationIntentState::NotPublished,
                data: instance.into_data(),
            },
            Self::Prepared(instance) => PublicationIntentSnapshot {
                state: PublicationIntentState::Prepared,
                data: instance.into_data(),
            },
            Self::Uncertain(instance) => PublicationIntentSnapshot {
                state: PublicationIntentState::Uncertain,
                data: instance.into_data(),
            },
        }
    }
}

/// What RepositoryRegistration — `controlplane.host.RepositoryRegistration` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`RepositoryRegistration<S>`], and at a boundary by [`RepositoryRegistrationSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationData {
    /// The identity: `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    ///
    /// Carries `repositories`: `controlplane.host.Workspace` owns many `controlplane.host.RepositoryRegistration`.
    pub workspace_id: crate::primitives::Uuid,
    /// `name` — `String`.
    pub name: String,
    /// `path` — `String`.
    pub path: String,
    /// `common_dir` — `String`.
    pub common_dir: String,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
}

/// The states of `controlplane.host.RepositoryRegistration`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](repository_registration_state::Marker), so [`RepositoryRegistration<S>`](RepositoryRegistration) can only ever rest in a real state.
pub mod repository_registration_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Disabled {}
        impl Sealed for super::Registered {}
    }

    /// A declared state of `RepositoryRegistration`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::RepositoryRegistrationState;
    }

    /// `Disabled`.
    pub struct Disabled;

    impl Marker for Disabled {
        const STATE: super::RepositoryRegistrationState = super::RepositoryRegistrationState::Disabled;
    }

    /// `Registered`. Where a new instance starts.
    pub struct Registered;

    impl Marker for Registered {
        const STATE: super::RepositoryRegistrationState = super::RepositoryRegistrationState::Registered;
    }
}

/// RepositoryRegistration — `controlplane.host.RepositoryRegistration` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Registered`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`RepositoryRegistrationSnapshot`]
/// and [`RepositoryRegistrationSnapshot::refine`].
pub struct RepositoryRegistration<S: repository_registration_state::Marker> {
    data: RepositoryRegistrationData,
    state: core::marker::PhantomData<S>,
}

impl<S: repository_registration_state::Marker> RepositoryRegistration<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> RepositoryRegistrationState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &RepositoryRegistrationData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> RepositoryRegistrationData {
        self.data
    }
}

impl RepositoryRegistration<repository_registration_state::Registered> {
    /// A new instance, resting in `Registered` — the only state the lifecycle starts one in.
    pub fn new(data: RepositoryRegistrationData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl RepositoryRegistration<repository_registration_state::Disabled> {
    /// `enable` — `Disabled` → `Registered`. Taken by the `applied` outcome of `controlplane.host.EnableRepositoryRegistration`.
    pub fn enable(self) -> RepositoryRegistration<repository_registration_state::Registered> {
        RepositoryRegistration {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl RepositoryRegistration<repository_registration_state::Registered> {
    /// `disable` — `Registered` → `Disabled`. Taken by the `applied` outcome of `controlplane.host.DisableRepositoryRegistration`.
    pub fn disable(self) -> RepositoryRegistration<repository_registration_state::Disabled> {
        RepositoryRegistration {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.RepositoryRegistration` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`RepositoryRegistrationSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: RepositoryRegistrationState,
    /// What it holds.
    pub data: RepositoryRegistrationData,
}

/// An `RepositoryRegistration` in whichever declared state it was found.
pub enum AnyRepositoryRegistration {
    /// Resting in `Disabled`.
    Disabled(RepositoryRegistration<repository_registration_state::Disabled>),
    /// Resting in `Registered`.
    Registered(RepositoryRegistration<repository_registration_state::Registered>),
}

impl RepositoryRegistrationSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `RepositoryRegistrationState` cannot spell one.
    pub fn refine(self) -> AnyRepositoryRegistration {
        match self.state {
            RepositoryRegistrationState::Disabled => AnyRepositoryRegistration::Disabled(RepositoryRegistration {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            RepositoryRegistrationState::Registered => AnyRepositoryRegistration::Registered(RepositoryRegistration {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyRepositoryRegistration {
    /// The state, as the runtime value.
    pub fn state(&self) -> RepositoryRegistrationState {
        match self {
            Self::Disabled(_) => RepositoryRegistrationState::Disabled,
            Self::Registered(_) => RepositoryRegistrationState::Registered,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> RepositoryRegistrationSnapshot {
        match self {
            Self::Disabled(instance) => RepositoryRegistrationSnapshot {
                state: RepositoryRegistrationState::Disabled,
                data: instance.into_data(),
            },
            Self::Registered(instance) => RepositoryRegistrationSnapshot {
                state: RepositoryRegistrationState::Registered,
                data: instance.into_data(),
            },
        }
    }
}

/// What Workspace — `controlplane.host.Workspace` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Workspace<S>`], and at a boundary by [`WorkspaceSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceData {
    /// The identity: `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `name` — `String`.
    pub name: String,
}

/// The states of `controlplane.host.Workspace`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](workspace_state::Marker), so [`Workspace<S>`](Workspace) can only ever rest in a real state.
pub mod workspace_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Archived {}
        impl Sealed for super::Registered {}
    }

    /// A declared state of `Workspace`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::WorkspaceState;
    }

    /// `Archived`. Terminal: an instance may rest here forever.
    pub struct Archived;

    impl Marker for Archived {
        const STATE: super::WorkspaceState = super::WorkspaceState::Archived;
    }

    /// `Registered`. Where a new instance starts.
    pub struct Registered;

    impl Marker for Registered {
        const STATE: super::WorkspaceState = super::WorkspaceState::Registered;
    }
}

/// Workspace — `controlplane.host.Workspace` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Registered`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`WorkspaceSnapshot`]
/// and [`WorkspaceSnapshot::refine`].
pub struct Workspace<S: workspace_state::Marker> {
    data: WorkspaceData,
    state: core::marker::PhantomData<S>,
}

impl<S: workspace_state::Marker> Workspace<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> WorkspaceState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &WorkspaceData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> WorkspaceData {
        self.data
    }
}

impl Workspace<workspace_state::Registered> {
    /// A new instance, resting in `Registered` — the only state the lifecycle starts one in.
    pub fn new(data: WorkspaceData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Workspace<workspace_state::Registered> {
    /// `archive` — `Registered` → `Archived`. Taken by the `applied` outcome of `controlplane.host.ArchiveWorkspace`.
    pub fn archive(self) -> Workspace<workspace_state::Archived> {
        Workspace {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.Workspace` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`WorkspaceSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: WorkspaceState,
    /// What it holds.
    pub data: WorkspaceData,
}

/// An `Workspace` in whichever declared state it was found.
pub enum AnyWorkspace {
    /// Resting in `Archived`.
    Archived(Workspace<workspace_state::Archived>),
    /// Resting in `Registered`.
    Registered(Workspace<workspace_state::Registered>),
}

impl WorkspaceSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `WorkspaceState` cannot spell one.
    pub fn refine(self) -> AnyWorkspace {
        match self.state {
            WorkspaceState::Archived => AnyWorkspace::Archived(Workspace {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            WorkspaceState::Registered => AnyWorkspace::Registered(Workspace {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyWorkspace {
    /// The state, as the runtime value.
    pub fn state(&self) -> WorkspaceState {
        match self {
            Self::Archived(_) => WorkspaceState::Archived,
            Self::Registered(_) => WorkspaceState::Registered,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> WorkspaceSnapshot {
        match self {
            Self::Archived(instance) => WorkspaceSnapshot {
                state: WorkspaceState::Archived,
                data: instance.into_data(),
            },
            Self::Registered(instance) => WorkspaceSnapshot {
                state: WorkspaceState::Registered,
                data: instance.into_data(),
            },
        }
    }
}

/// What WorkspaceDirectory — `controlplane.host.WorkspaceDirectory` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`WorkspaceDirectory<S>`], and at a boundary by [`WorkspaceDirectorySnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryData {
    /// The identity: `directory_id` — `Uuid`.
    pub directory_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    ///
    /// Carries `directories`: `controlplane.host.Workspace` owns many `controlplane.host.WorkspaceDirectory`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `repository_common_dirs` — `List<String>`.
    pub repository_common_dirs: Vec<String>,
    /// `managed_common_dirs` — `List<String>`.
    pub managed_common_dirs: Vec<String>,
}

/// The states of `controlplane.host.WorkspaceDirectory`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](workspace_directory_state::Marker), so [`WorkspaceDirectory<S>`](WorkspaceDirectory) can only ever rest in a real state.
pub mod workspace_directory_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Registered {}
        impl Sealed for super::Removed {}
    }

    /// A declared state of `WorkspaceDirectory`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::WorkspaceDirectoryState;
    }

    /// `Registered`. Where a new instance starts.
    pub struct Registered;

    impl Marker for Registered {
        const STATE: super::WorkspaceDirectoryState = super::WorkspaceDirectoryState::Registered;
    }

    /// `Removed`. Terminal: an instance may rest here forever.
    pub struct Removed;

    impl Marker for Removed {
        const STATE: super::WorkspaceDirectoryState = super::WorkspaceDirectoryState::Removed;
    }
}

/// WorkspaceDirectory — `controlplane.host.WorkspaceDirectory` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Registered`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`WorkspaceDirectorySnapshot`]
/// and [`WorkspaceDirectorySnapshot::refine`].
pub struct WorkspaceDirectory<S: workspace_directory_state::Marker> {
    data: WorkspaceDirectoryData,
    state: core::marker::PhantomData<S>,
}

impl<S: workspace_directory_state::Marker> WorkspaceDirectory<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> WorkspaceDirectoryState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &WorkspaceDirectoryData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> WorkspaceDirectoryData {
        self.data
    }
}

impl WorkspaceDirectory<workspace_directory_state::Registered> {
    /// A new instance, resting in `Registered` — the only state the lifecycle starts one in.
    pub fn new(data: WorkspaceDirectoryData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl WorkspaceDirectory<workspace_directory_state::Registered> {
    /// `remove` — `Registered` → `Removed`. Taken by the `applied` outcome of `controlplane.host.RemoveWorkspaceDirectory`.
    pub fn remove(self) -> WorkspaceDirectory<workspace_directory_state::Removed> {
        WorkspaceDirectory {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `controlplane.host.WorkspaceDirectory` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`WorkspaceDirectorySnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectorySnapshot {
    /// Where the instance is in its lifecycle.
    pub state: WorkspaceDirectoryState,
    /// What it holds.
    pub data: WorkspaceDirectoryData,
}

/// An `WorkspaceDirectory` in whichever declared state it was found.
pub enum AnyWorkspaceDirectory {
    /// Resting in `Registered`.
    Registered(WorkspaceDirectory<workspace_directory_state::Registered>),
    /// Resting in `Removed`.
    Removed(WorkspaceDirectory<workspace_directory_state::Removed>),
}

impl WorkspaceDirectorySnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `WorkspaceDirectoryState` cannot spell one.
    pub fn refine(self) -> AnyWorkspaceDirectory {
        match self.state {
            WorkspaceDirectoryState::Registered => AnyWorkspaceDirectory::Registered(WorkspaceDirectory {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            WorkspaceDirectoryState::Removed => AnyWorkspaceDirectory::Removed(WorkspaceDirectory {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyWorkspaceDirectory {
    /// The state, as the runtime value.
    pub fn state(&self) -> WorkspaceDirectoryState {
        match self {
            Self::Registered(_) => WorkspaceDirectoryState::Registered,
            Self::Removed(_) => WorkspaceDirectoryState::Removed,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> WorkspaceDirectorySnapshot {
        match self {
            Self::Registered(instance) => WorkspaceDirectorySnapshot {
                state: WorkspaceDirectoryState::Registered,
                data: instance.into_data(),
            },
            Self::Removed(instance) => WorkspaceDirectorySnapshot {
                state: WorkspaceDirectoryState::Removed,
                data: instance.into_data(),
            },
        }
    }
}

/// AddWorkspaceDirectory — the input of `controlplane.host.AddWorkspaceDirectory`.
///
/// Everything it can result in is [`AddWorkspaceDirectoryOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddWorkspaceDirectory {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `repository_common_dirs` — `List<String>`.
    pub repository_common_dirs: Vec<String>,
    /// `managed_common_dirs` — `List<String>`.
    pub managed_common_dirs: Vec<String>,
}

/// Everything `controlplane.host.AddWorkspaceDirectory` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddWorkspaceDirectoryOutcome {
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.WorkspaceDirectoryCreated` this outcome publishes.
        workspace_directory_created: WorkspaceDirectoryCreated,
    },
}

/// ArchiveWorkspace — the input of `controlplane.host.ArchiveWorkspace`.
///
/// Everything it can result in is [`ArchiveWorkspaceOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveWorkspace {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.ArchiveWorkspace` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveWorkspaceOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ArchiveWorkspaceApplied` this outcome publishes.
        archive_workspace_applied: ArchiveWorkspaceApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.WorkspaceStateConflict`.
        error: WorkspaceStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.WorkspaceNotFound`.
        error: WorkspaceNotFound,
    },
}

/// BlockAssignment — the input of `controlplane.host.BlockAssignment`.
///
/// Everything it can result in is [`BlockAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
}

/// Everything `controlplane.host.BlockAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockAssignmentOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.BlockAssignmentApplied` this outcome publishes.
        block_assignment_applied: BlockAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// CancelAssignment — the input of `controlplane.host.CancelAssignment`.
///
/// Everything it can result in is [`CancelAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.CancelAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelAssignmentOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.CancelAssignmentApplied` this outcome publishes.
        cancel_assignment_applied: CancelAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// CancelGoal — the input of `controlplane.host.CancelGoal`.
///
/// Everything it can result in is [`CancelGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.CancelGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelGoalOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.CancelGoalApplied` this outcome publishes.
        cancel_goal_applied: CancelGoalApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// ClaimAssignment — the input of `controlplane.host.ClaimAssignment`.
///
/// Everything it can result in is [`ClaimAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `base_revision` — `String`.
    pub base_revision: String,
}

/// Everything `controlplane.host.ClaimAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimAssignmentOutcome {
    /// `evidence-missing` — when the existing subject's stored fields satisfy `(input.implementor_run == "" or input.worktree_id == "" or input.base_revision == "")`.
    EvidenceMissing {
        /// Why it was refused: `controlplane.host.EvidenceMissing`.
        error: EvidenceMissing,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ClaimAssignmentApplied` this outcome publishes.
        claim_assignment_applied: ClaimAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// ClosePublication — the input of `controlplane.host.ClosePublication`.
///
/// Everything it can result in is [`ClosePublicationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosePublication {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
}

/// Everything `controlplane.host.ClosePublication` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosePublicationOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ClosePublicationApplied` this outcome publishes.
        close_publication_applied: ClosePublicationApplied,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.PublicationIntentNotFound`.
        error: PublicationIntentNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.PublicationIntentStateConflict`.
        error: PublicationIntentStateConflict,
    },
}

/// CompleteAssignment — the input of `controlplane.host.CompleteAssignment`.
///
/// Everything it can result in is [`CompleteAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
}

/// Everything `controlplane.host.CompleteAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteAssignmentOutcome {
    /// `receipt-missing` — when the existing subject's stored fields satisfy `input.merge_receipt == ""`.
    ReceiptMissing {
        /// Why it was refused: `controlplane.host.EvidenceMissing`.
        error: EvidenceMissing,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.CompleteAssignmentApplied` this outcome publishes.
        complete_assignment_applied: CompleteAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// ConfigureRepository — the input of `controlplane.host.ConfigureRepository`.
///
/// Everything it can result in is [`ConfigureRepositoryOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureRepository {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
}

/// Everything `controlplane.host.ConfigureRepository` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigureRepositoryOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ConfigureRepositoryApplied` this outcome publishes.
        configure_repository_applied: ConfigureRepositoryApplied,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.RepositoryRegistrationNotFound`.
        error: RepositoryRegistrationNotFound,
    },
}

/// ConfirmPublication — the input of `controlplane.host.ConfirmPublication`.
///
/// Everything it can result in is [`ConfirmPublicationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmPublication {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `receipt` — `String`.
    pub receipt: String,
}

/// Everything `controlplane.host.ConfirmPublication` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmPublicationOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ConfirmPublicationApplied` this outcome publishes.
        confirm_publication_applied: ConfirmPublicationApplied,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.PublicationIntentNotFound`.
        error: PublicationIntentNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.PublicationIntentStateConflict`.
        error: PublicationIntentStateConflict,
    },
}

/// CreateGoal — the input of `controlplane.host.CreateGoal`.
///
/// Everything it can result in is [`CreateGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateGoal {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
}

/// Everything `controlplane.host.CreateGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateGoalOutcome {
    /// `workers-invalid` — when `max_workers <= 0`.
    WorkersInvalid {
        /// Why it was refused: `controlplane.host.GoalLimitInvalid`.
        error: GoalLimitInvalid,
    },
    /// `attempts-invalid` — when `max_attempts <= 0`.
    AttemptsInvalid {
        /// Why it was refused: `controlplane.host.GoalLimitInvalid`.
        error: GoalLimitInvalid,
    },
    /// `minutes-invalid` — when `max_minutes <= 0`.
    MinutesInvalid {
        /// Why it was refused: `controlplane.host.GoalLimitInvalid`.
        error: GoalLimitInvalid,
    },
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.GoalCreated` this outcome publishes.
        goal_created: GoalCreated,
    },
}

/// DeleteGoal — the input of `controlplane.host.DeleteGoal`.
///
/// Everything it can result in is [`DeleteGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.DeleteGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteGoalOutcome {
    /// `applied` — when the existing subject is in Cancelled.
    Applied {
        /// The `controlplane.host.GoalDeleted` this outcome publishes.
        goal_deleted: GoalDeleted,
    },
    /// `paused` — when the existing subject is in Paused.
    Paused {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `running` — when the existing subject is in Running.
    Running {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `satisfied` — when the existing subject is in Satisfied.
    Satisfied {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// DisableRepositoryRegistration — the input of `controlplane.host.DisableRepositoryRegistration`.
///
/// Everything it can result in is [`DisableRepositoryRegistrationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableRepositoryRegistration {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.DisableRepositoryRegistration` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisableRepositoryRegistrationOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.DisableRepositoryRegistrationApplied` this outcome publishes.
        disable_repository_registration_applied: DisableRepositoryRegistrationApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.RepositoryRegistrationStateConflict`.
        error: RepositoryRegistrationStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.RepositoryRegistrationNotFound`.
        error: RepositoryRegistrationNotFound,
    },
}

/// EnableRepositoryRegistration — the input of `controlplane.host.EnableRepositoryRegistration`.
///
/// Everything it can result in is [`EnableRepositoryRegistrationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnableRepositoryRegistration {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.EnableRepositoryRegistration` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnableRepositoryRegistrationOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.EnableRepositoryRegistrationApplied` this outcome publishes.
        enable_repository_registration_applied: EnableRepositoryRegistrationApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.RepositoryRegistrationStateConflict`.
        error: RepositoryRegistrationStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.RepositoryRegistrationNotFound`.
        error: RepositoryRegistrationNotFound,
    },
}

/// MarkPublicationUncertain — the input of `controlplane.host.MarkPublicationUncertain`.
///
/// Everything it can result in is [`MarkPublicationUncertainOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkPublicationUncertain {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.MarkPublicationUncertain` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkPublicationUncertainOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.MarkPublicationUncertainApplied` this outcome publishes.
        mark_publication_uncertain_applied: MarkPublicationUncertainApplied,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.PublicationIntentNotFound`.
        error: PublicationIntentNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.PublicationIntentStateConflict`.
        error: PublicationIntentStateConflict,
    },
}

/// MergeAssignment — the input of `controlplane.host.MergeAssignment`.
///
/// Everything it can result in is [`MergeAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.MergeAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeAssignmentOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.MergeAssignmentApplied` this outcome publishes.
        merge_assignment_applied: MergeAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// PauseGoal — the input of `controlplane.host.PauseGoal`.
///
/// Everything it can result in is [`PauseGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.PauseGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PauseGoalOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.PauseGoalApplied` this outcome publishes.
        pause_goal_applied: PauseGoalApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// PreparePublication — the input of `controlplane.host.PreparePublication`.
///
/// Everything it can result in is [`PreparePublicationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparePublication {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `target` — `String`.
    pub target: String,
    /// `expected_base` — `String`.
    pub expected_base: String,
}

/// Everything `controlplane.host.PreparePublication` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparePublicationOutcome {
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.PublicationIntentCreated` this outcome publishes.
        publication_intent_created: PublicationIntentCreated,
    },
}

/// QueueAssignment — the input of `controlplane.host.QueueAssignment`.
///
/// Everything it can result in is [`QueueAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueAssignment {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `story_id` — `String`.
    pub story_id: String,
    /// `case_id` — `String`.
    pub case_id: String,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `attempt` — `Integer`.
    pub attempt: i64,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `goal_revision` — `Integer`.
    pub goal_revision: i64,
}

/// Everything `controlplane.host.QueueAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueAssignmentOutcome {
    /// `goal-not-found` — when no `controlplane.host.Goal` carries the identity `input.goal_id` names.
    GoalNotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
    /// `goal-not-current` — when the `controlplane.host.Goal` that `input.goal_id` names satisfies `(state != Running or revision != input.goal_revision)`.
    GoalNotCurrent {
        /// Why it was refused: `controlplane.host.GoalNotCurrent`.
        error: GoalNotCurrent,
    },
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.AssignmentCreated` this outcome publishes.
        assignment_created: AssignmentCreated,
    },
}

/// ReadyAssignment — the input of `controlplane.host.ReadyAssignment`.
///
/// Everything it can result in is [`ReadyAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `review_revision` — `String`.
    pub review_revision: String,
}

/// Everything `controlplane.host.ReadyAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadyAssignmentOutcome {
    /// `reviewer-missing` — when the existing subject's stored fields satisfy `input.reviewer_run == ""`.
    ReviewerMissing {
        /// Why it was refused: `controlplane.host.ReviewNotIndependent`.
        error: ReviewNotIndependent,
    },
    /// `review-not-independent` — when the existing subject's stored fields satisfy `implementor_run == input.reviewer_run`.
    ReviewNotIndependent {
        /// Why it was refused: `controlplane.host.ReviewNotIndependent`.
        error: ReviewNotIndependent,
    },
    /// `evidence-not-current` — when the existing subject's stored fields satisfy `(state == Reviewing and (candidate == "" or test_revision != {fact: candidate} or candidate != input.review_revision))`.
    EvidenceNotCurrent {
        /// Why it was refused: `controlplane.host.EvidenceNotCurrent`.
        error: EvidenceNotCurrent,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ReadyAssignmentApplied` this outcome publishes.
        ready_assignment_applied: ReadyAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// ReconcileAssignment — the input of `controlplane.host.ReconcileAssignment`.
///
/// Everything it can result in is [`ReconcileAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
}

/// Everything `controlplane.host.ReconcileAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileAssignmentOutcome {
    /// `receipt-missing` — when the existing subject's stored fields satisfy `input.merge_receipt == ""`.
    ReceiptMissing {
        /// Why it was refused: `controlplane.host.EvidenceMissing`.
        error: EvidenceMissing,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ReconcileAssignmentApplied` this outcome publishes.
        reconcile_assignment_applied: ReconcileAssignmentApplied,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
}

/// RecordPlanningProgress — the input of `controlplane.host.RecordPlanningProgress`.
///
/// Everything it can result in is [`RecordPlanningProgressOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPlanningProgress {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `planning_revision` — `Integer`.
    pub planning_revision: i64,
    /// `planning_fingerprint` — `String`.
    pub planning_fingerprint: String,
    /// `planning_repository` — `String`.
    pub planning_repository: String,
    /// `planning_worktree_id` — `String`.
    pub planning_worktree_id: String,
    /// `planning_worktree_path` — `String`.
    pub planning_worktree_path: String,
    /// `planning_reason` — `String`.
    pub planning_reason: String,
    /// `planning_receipt` — `String`.
    pub planning_receipt: String,
    /// `planning_phase` — `controlplane.host.PlanningPhase`.
    pub planning_phase: PlanningPhase,
}

/// Everything `controlplane.host.RecordPlanningProgress` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordPlanningProgressOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.PlanningProgressRecorded` this outcome publishes.
        planning_progress_recorded: PlanningProgressRecorded,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// RegisterRepository — the input of `controlplane.host.RegisterRepository`.
///
/// Everything it can result in is [`RegisterRepositoryOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterRepository {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `name` — `String`.
    pub name: String,
    /// `path` — `String`.
    pub path: String,
    /// `common_dir` — `String`.
    pub common_dir: String,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
}

/// Everything `controlplane.host.RegisterRepository` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterRepositoryOutcome {
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.RepositoryRegistrationCreated` this outcome publishes.
        repository_registration_created: RepositoryRegistrationCreated,
    },
}

/// RegisterWorkspace — the input of `controlplane.host.RegisterWorkspace`.
///
/// Everything it can result in is [`RegisterWorkspaceOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterWorkspace {
    /// `path` — `String`.
    pub path: String,
    /// `name` — `String`.
    pub name: String,
}

/// Everything `controlplane.host.RegisterWorkspace` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterWorkspaceOutcome {
    /// `created` — otherwise.
    Created {
        /// The `controlplane.host.WorkspaceCreated` this outcome publishes.
        workspace_created: WorkspaceCreated,
    },
}

/// RemoveWorkspaceDirectory — the input of `controlplane.host.RemoveWorkspaceDirectory`.
///
/// Everything it can result in is [`RemoveWorkspaceDirectoryOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveWorkspaceDirectory {
    /// `directory_id` — `Uuid`.
    pub directory_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.RemoveWorkspaceDirectory` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveWorkspaceDirectoryOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.WorkspaceDirectoryRemoved` this outcome publishes.
        workspace_directory_removed: WorkspaceDirectoryRemoved,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.WorkspaceDirectoryStateConflict`.
        error: WorkspaceDirectoryStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.WorkspaceDirectoryNotFound`.
        error: WorkspaceDirectoryNotFound,
    },
}

/// RepairAssignment — the input of `controlplane.host.RepairAssignment`.
///
/// Everything it can result in is [`RepairAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `base_revision` — `Optional<String>`.
    pub base_revision: Option<String>,
}

/// Everything `controlplane.host.RepairAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAssignmentOutcome {
    /// `evidence-missing` — when the existing subject's stored fields satisfy `input.implementor_run == ""`.
    EvidenceMissing {
        /// Why it was refused: `controlplane.host.EvidenceMissing`.
        error: EvidenceMissing,
    },
    /// `base-missing` — when the existing subject's stored fields satisfy `(defined(input.base_revision) and input.base_revision == "")`.
    BaseMissing {
        /// Why it was refused: `controlplane.host.EvidenceMissing`.
        error: EvidenceMissing,
    },
    /// `rebased` — when `defined(base_revision)`.
    Rebased {
        /// The `controlplane.host.RepairAssignmentApplied` this outcome publishes.
        repair_assignment_applied: RepairAssignmentApplied,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.RepairAssignmentApplied` this outcome publishes.
        repair_assignment_applied: RepairAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// ReviewAssignment — the input of `controlplane.host.ReviewAssignment`.
///
/// Everything it can result in is [`ReviewAssignmentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAssignment {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `test_revision` — `String`.
    pub test_revision: String,
}

/// Everything `controlplane.host.ReviewAssignment` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewAssignmentOutcome {
    /// `tests-not-current` — when the existing subject's stored fields satisfy `(input.candidate == "" or input.candidate != input.test_revision)`.
    TestsNotCurrent {
        /// Why it was refused: `controlplane.host.EvidenceNotCurrent`.
        error: EvidenceNotCurrent,
    },
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.ReviewAssignmentApplied` this outcome publishes.
        review_assignment_applied: ReviewAssignmentApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.AssignmentStateConflict`.
        error: AssignmentStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.AssignmentNotFound`.
        error: AssignmentNotFound,
    },
}

/// SatisfyGoal — the input of `controlplane.host.SatisfyGoal`.
///
/// Everything it can result in is [`SatisfyGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SatisfyGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `satisfaction_receipt` — `String`.
    pub satisfaction_receipt: String,
}

/// Everything `controlplane.host.SatisfyGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatisfyGoalOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.SatisfyGoalApplied` this outcome publishes.
        satisfy_goal_applied: SatisfyGoalApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// StartGoal — the input of `controlplane.host.StartGoal`.
///
/// Everything it can result in is [`StartGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// Everything `controlplane.host.StartGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartGoalOutcome {
    /// `applied` — otherwise.
    Applied {
        /// The `controlplane.host.StartGoalApplied` this outcome publishes.
        start_goal_applied: StartGoalApplied,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// UpdateGoal — the input of `controlplane.host.UpdateGoal`.
///
/// Everything it can result in is [`UpdateGoalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateGoal {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
}

/// Everything `controlplane.host.UpdateGoal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateGoalOutcome {
    /// `applied` — when the existing subject is in Paused or Running.
    Applied {
        /// The `controlplane.host.UpdateGoalApplied` this outcome publishes.
        update_goal_applied: UpdateGoalApplied,
    },
    /// `satisfied` — when the existing subject is in Satisfied.
    Satisfied {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `cancelled` — when the existing subject is in Cancelled.
    Cancelled {
        /// Why it was refused: `controlplane.host.GoalStateConflict`.
        error: GoalStateConflict,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `controlplane.host.GoalNotFound`.
        error: GoalNotFound,
    },
}

/// ArchiveWorkspaceApplied — the event `controlplane.host.ArchiveWorkspaceApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveWorkspaceApplied {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
}

/// AssignmentCreated — the event `controlplane.host.AssignmentCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentCreated {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `story_id` — `String`.
    pub story_id: String,
    /// `case_id` — `String`.
    pub case_id: String,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `attempt` — `Integer`.
    pub attempt: i64,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `goal_revision` — `Integer`.
    pub goal_revision: i64,
}

/// BlockAssignmentApplied — the event `controlplane.host.BlockAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
}

/// CancelAssignmentApplied — the event `controlplane.host.CancelAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
}

/// CancelGoalApplied — the event `controlplane.host.CancelGoalApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelGoalApplied {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// ClaimAssignmentApplied — the event `controlplane.host.ClaimAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `base_revision` — `String`.
    pub base_revision: String,
}

/// ClosePublicationApplied — the event `controlplane.host.ClosePublicationApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosePublicationApplied {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
}

/// CompleteAssignmentApplied — the event `controlplane.host.CompleteAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
}

/// ConfigureRepositoryApplied — the event `controlplane.host.ConfigureRepositoryApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureRepositoryApplied {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
}

/// ConfirmPublicationApplied — the event `controlplane.host.ConfirmPublicationApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmPublicationApplied {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `receipt` — `String`.
    pub receipt: String,
}

/// DisableRepositoryRegistrationApplied — the event `controlplane.host.DisableRepositoryRegistrationApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableRepositoryRegistrationApplied {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
}

/// EnableRepositoryRegistrationApplied — the event `controlplane.host.EnableRepositoryRegistrationApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnableRepositoryRegistrationApplied {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
}

/// GoalCreated — the event `controlplane.host.GoalCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalCreated {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
    /// `planning_revision` — `Integer`.
    pub planning_revision: i64,
    /// `planning_fingerprint` — `String`.
    pub planning_fingerprint: String,
    /// `planning_repository` — `String`.
    pub planning_repository: String,
    /// `planning_worktree_id` — `String`.
    pub planning_worktree_id: String,
    /// `planning_worktree_path` — `String`.
    pub planning_worktree_path: String,
    /// `planning_reason` — `String`.
    pub planning_reason: String,
    /// `planning_receipt` — `String`.
    pub planning_receipt: String,
    /// `planning_phase` — `controlplane.host.PlanningPhase`.
    pub planning_phase: PlanningPhase,
}

/// GoalDeleted — the event `controlplane.host.GoalDeleted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalDeleted {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// MarkPublicationUncertainApplied — the event `controlplane.host.MarkPublicationUncertainApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkPublicationUncertainApplied {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
}

/// MergeAssignmentApplied — the event `controlplane.host.MergeAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
}

/// PauseGoalApplied — the event `controlplane.host.PauseGoalApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseGoalApplied {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// PlanningProgressRecorded — the event `controlplane.host.PlanningProgressRecorded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningProgressRecorded {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `planning_revision` — `Integer`.
    pub planning_revision: i64,
    /// `planning_fingerprint` — `String`.
    pub planning_fingerprint: String,
    /// `planning_repository` — `String`.
    pub planning_repository: String,
    /// `planning_worktree_id` — `String`.
    pub planning_worktree_id: String,
    /// `planning_worktree_path` — `String`.
    pub planning_worktree_path: String,
    /// `planning_reason` — `String`.
    pub planning_reason: String,
    /// `planning_receipt` — `String`.
    pub planning_receipt: String,
    /// `planning_phase` — `controlplane.host.PlanningPhase`.
    pub planning_phase: PlanningPhase,
}

/// PublicationIntentCreated — the event `controlplane.host.PublicationIntentCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentCreated {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `target` — `String`.
    pub target: String,
    /// `expected_base` — `String`.
    pub expected_base: String,
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
}

/// ReadyAssignmentApplied — the event `controlplane.host.ReadyAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `review_revision` — `String`.
    pub review_revision: String,
}

/// ReconcileAssignmentApplied — the event `controlplane.host.ReconcileAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
}

/// RepairAssignmentApplied — the event `controlplane.host.RepairAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
}

/// RepositoryRegistrationCreated — the event `controlplane.host.RepositoryRegistrationCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationCreated {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `name` — `String`.
    pub name: String,
    /// `path` — `String`.
    pub path: String,
    /// `common_dir` — `String`.
    pub common_dir: String,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
}

/// ReviewAssignmentApplied — the event `controlplane.host.ReviewAssignmentApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAssignmentApplied {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `test_revision` — `String`.
    pub test_revision: String,
}

/// SatisfyGoalApplied — the event `controlplane.host.SatisfyGoalApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SatisfyGoalApplied {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `satisfaction_receipt` — `String`.
    pub satisfaction_receipt: String,
}

/// StartGoalApplied — the event `controlplane.host.StartGoalApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartGoalApplied {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
}

/// UpdateGoalApplied — the event `controlplane.host.UpdateGoalApplied`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateGoalApplied {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
}

/// WorkspaceCreated — the event `controlplane.host.WorkspaceCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceCreated {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `name` — `String`.
    pub name: String,
}

/// WorkspaceDirectoryCreated — the event `controlplane.host.WorkspaceDirectoryCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryCreated {
    /// `directory_id` — `Uuid`.
    pub directory_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `repository_common_dirs` — `List<String>`.
    pub repository_common_dirs: Vec<String>,
    /// `managed_common_dirs` — `List<String>`.
    pub managed_common_dirs: Vec<String>,
}

/// WorkspaceDirectoryRemoved — the event `controlplane.host.WorkspaceDirectoryRemoved`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryRemoved {
    /// `directory_id` — `Uuid`.
    pub directory_id: crate::primitives::Uuid,
}

/// The declared error `controlplane.host.AssignmentNotFound`.
///
/// The requested identity is not held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentNotFound;

/// The declared error `controlplane.host.AssignmentStateConflict`.
///
/// The command cannot act in the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentStateConflict {
    /// `state` — `controlplane.host.Assignment.State`.
    pub state: AssignmentState,
}

/// The declared error `controlplane.host.EvidenceMissing`.
///
/// A required run, worktree, revision or receipt is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceMissing;

/// The declared error `controlplane.host.EvidenceNotCurrent`.
///
/// Tests or review do not cover the assignment's current candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceNotCurrent;

/// The declared error `controlplane.host.GoalLimitInvalid`.
///
/// Worker, attempt and minute limits are positive counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalLimitInvalid;

/// The declared error `controlplane.host.GoalNotCurrent`.
///
/// The goal is not running, or changed since the assignment was planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalNotCurrent;

/// The declared error `controlplane.host.GoalNotFound`.
///
/// The requested identity is not held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalNotFound;

/// The declared error `controlplane.host.GoalStateConflict`.
///
/// The command cannot act in the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalStateConflict {
    /// `state` — `controlplane.host.Goal.State`.
    pub state: GoalState,
}

/// The declared error `controlplane.host.PublicationIntentNotFound`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentNotFound;

/// The declared error `controlplane.host.PublicationIntentStateConflict`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentStateConflict {
    /// `state` — `controlplane.host.PublicationIntent.State`.
    pub state: PublicationIntentState,
}

/// The declared error `controlplane.host.RepositoryRegistrationNotFound`.
///
/// The requested identity is not held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationNotFound;

/// The declared error `controlplane.host.RepositoryRegistrationStateConflict`.
///
/// The command cannot act in the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationStateConflict {
    /// `state` — `controlplane.host.RepositoryRegistration.State`.
    pub state: RepositoryRegistrationState,
}

/// The declared error `controlplane.host.ReviewNotIndependent`.
///
/// Review must use an execution context other than the implementor's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewNotIndependent;

/// The declared error `controlplane.host.WorkspaceDirectoryNotFound`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryNotFound;

/// The declared error `controlplane.host.WorkspaceDirectoryStateConflict`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryStateConflict {
    /// `state` — `controlplane.host.WorkspaceDirectory.State`.
    pub state: WorkspaceDirectoryState,
}

/// The declared error `controlplane.host.WorkspaceNotFound`.
///
/// The requested identity is not held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceNotFound;

/// The declared error `controlplane.host.WorkspaceStateConflict`.
///
/// The command cannot act in the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceStateConflict {
    /// `state` — `controlplane.host.Workspace.State`.
    pub state: WorkspaceState,
}

/// AssignmentList — one row of the view `controlplane.host.AssignmentList`.
///
/// Projects `controlplane.host.Assignment` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentList {
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `story_id` — `String`.
    pub story_id: String,
    /// `case_id` — `String`.
    pub case_id: String,
    /// `worktree_id` — `String`.
    pub worktree_id: String,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `attempt` — `Integer`.
    pub attempt: i64,
    /// `reason` — `String`.
    pub reason: String,
    /// `implementor_run` — `String`.
    pub implementor_run: String,
    /// `reviewer_run` — `String`.
    pub reviewer_run: String,
    /// `goal_revision` — `Integer`.
    pub goal_revision: i64,
    /// `base_revision` — `String`.
    pub base_revision: String,
    /// `test_revision` — `String`.
    pub test_revision: String,
    /// `review_revision` — `String`.
    pub review_revision: String,
    /// `merge_receipt` — `String`.
    pub merge_receipt: String,
    /// `state` — `controlplane.host.Assignment.State`.
    pub state: AssignmentState,
}

/// GoalList — one row of the view `controlplane.host.GoalList`.
///
/// Projects `controlplane.host.Goal` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalList {
    /// `goal_id` — `Uuid`.
    pub goal_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `objective` — `String`.
    pub objective: String,
    /// `acceptance` — `String`.
    pub acceptance: String,
    /// `max_workers` — `Integer`.
    pub max_workers: i64,
    /// `max_attempts` — `Integer`.
    pub max_attempts: i64,
    /// `max_minutes` — `Integer`.
    pub max_minutes: i64,
    /// `planner_model` — `String`.
    pub planner_model: String,
    /// `implementor_model` — `String`.
    pub implementor_model: String,
    /// `reviewer_model` — `String`.
    pub reviewer_model: String,
    /// `merge_authority` — `Boolean`.
    pub merge_authority: bool,
    /// `revision` — `Integer`.
    pub revision: i64,
    /// `satisfaction_receipt` — `String`.
    pub satisfaction_receipt: String,
    /// `state` — `controlplane.host.Goal.State`.
    pub state: GoalState,
    /// `planning_revision` — `Integer`.
    pub planning_revision: i64,
    /// `planning_fingerprint` — `String`.
    pub planning_fingerprint: String,
    /// `planning_repository` — `String`.
    pub planning_repository: String,
    /// `planning_worktree_id` — `String`.
    pub planning_worktree_id: String,
    /// `planning_worktree_path` — `String`.
    pub planning_worktree_path: String,
    /// `planning_reason` — `String`.
    pub planning_reason: String,
    /// `planning_receipt` — `String`.
    pub planning_receipt: String,
    /// `planning_phase` — `controlplane.host.PlanningPhase`.
    pub planning_phase: PlanningPhase,
}

/// PublicationIntentList — one row of the view `controlplane.host.PublicationIntentList`.
///
/// Projects `controlplane.host.PublicationIntent` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationIntentList {
    /// `publication_id` — `Uuid`.
    pub publication_id: crate::primitives::Uuid,
    /// `assignment_id` — `Uuid`.
    pub assignment_id: crate::primitives::Uuid,
    /// `candidate` — `String`.
    pub candidate: String,
    /// `target` — `String`.
    pub target: String,
    /// `expected_base` — `String`.
    pub expected_base: String,
    /// `receipt` — `String`.
    pub receipt: String,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
    /// `state` — `controlplane.host.PublicationIntent.State`.
    pub state: PublicationIntentState,
}

/// RepositoryRegistrationList — one row of the view `controlplane.host.RepositoryRegistrationList`.
///
/// Projects `controlplane.host.RepositoryRegistration` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRegistrationList {
    /// `repository_id` — `Uuid`.
    pub repository_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `name` — `String`.
    pub name: String,
    /// `path` — `String`.
    pub path: String,
    /// `common_dir` — `String`.
    pub common_dir: String,
    /// `base_branch` — `String`.
    pub base_branch: String,
    /// `test_command` — `String`.
    pub test_command: String,
    /// `publish_command` — `String`.
    pub publish_command: String,
    /// `state` — `controlplane.host.RepositoryRegistration.State`.
    pub state: RepositoryRegistrationState,
}

/// WorkspaceDirectoryList — one row of the view `controlplane.host.WorkspaceDirectoryList`.
///
/// Projects `controlplane.host.WorkspaceDirectory` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryList {
    /// `directory_id` — `Uuid`.
    pub directory_id: crate::primitives::Uuid,
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `repository_common_dirs` — `List<String>`.
    pub repository_common_dirs: Vec<String>,
    /// `managed_common_dirs` — `List<String>`.
    pub managed_common_dirs: Vec<String>,
    /// `state` — `controlplane.host.WorkspaceDirectory.State`.
    pub state: WorkspaceDirectoryState,
}

/// WorkspaceList — one row of the view `controlplane.host.WorkspaceList`.
///
/// Projects `controlplane.host.Workspace` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceList {
    /// `workspace_id` — `Uuid`.
    pub workspace_id: crate::primitives::Uuid,
    /// `path` — `String`.
    pub path: String,
    /// `name` — `String`.
    pub name: String,
    /// `state` — `controlplane.host.Workspace.State`.
    pub state: WorkspaceState,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
pub mod obligations {
    /// The behaviour `controlplane.host.AddWorkspaceDirectory` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait AddWorkspaceDirectoryBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.AddWorkspaceDirectory`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn add_workspace_directory(&mut self, input: super::AddWorkspaceDirectory) -> Result<super::AddWorkspaceDirectoryOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ArchiveWorkspace` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ArchiveWorkspaceBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ArchiveWorkspace`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn archive_workspace(&mut self, input: super::ArchiveWorkspace) -> Result<super::ArchiveWorkspaceOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.BlockAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait BlockAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.BlockAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn block_assignment(&mut self, input: super::BlockAssignment) -> Result<super::BlockAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.CancelAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait CancelAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.CancelAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn cancel_assignment(&mut self, input: super::CancelAssignment) -> Result<super::CancelAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.CancelGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait CancelGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.CancelGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn cancel_goal(&mut self, input: super::CancelGoal) -> Result<super::CancelGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ClaimAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ClaimAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ClaimAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn claim_assignment(&mut self, input: super::ClaimAssignment) -> Result<super::ClaimAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ClosePublication` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ClosePublicationBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ClosePublication`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn close_publication(&mut self, input: super::ClosePublication) -> Result<super::ClosePublicationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.CompleteAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait CompleteAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.CompleteAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn complete_assignment(&mut self, input: super::CompleteAssignment) -> Result<super::CompleteAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ConfigureRepository` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ConfigureRepositoryBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ConfigureRepository`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn configure_repository(&mut self, input: super::ConfigureRepository) -> Result<super::ConfigureRepositoryOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ConfirmPublication` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ConfirmPublicationBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ConfirmPublication`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn confirm_publication(&mut self, input: super::ConfirmPublication) -> Result<super::ConfirmPublicationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.CreateGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait CreateGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.CreateGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn create_goal(&mut self, input: super::CreateGoal) -> Result<super::CreateGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.DeleteGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait DeleteGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.DeleteGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn delete_goal(&mut self, input: super::DeleteGoal) -> Result<super::DeleteGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.DisableRepositoryRegistration` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait DisableRepositoryRegistrationBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.DisableRepositoryRegistration`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn disable_repository_registration(&mut self, input: super::DisableRepositoryRegistration) -> Result<super::DisableRepositoryRegistrationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.EnableRepositoryRegistration` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait EnableRepositoryRegistrationBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.EnableRepositoryRegistration`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn enable_repository_registration(&mut self, input: super::EnableRepositoryRegistration) -> Result<super::EnableRepositoryRegistrationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.MarkPublicationUncertain` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait MarkPublicationUncertainBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.MarkPublicationUncertain`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn mark_publication_uncertain(&mut self, input: super::MarkPublicationUncertain) -> Result<super::MarkPublicationUncertainOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.MergeAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait MergeAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.MergeAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn merge_assignment(&mut self, input: super::MergeAssignment) -> Result<super::MergeAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.PauseGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait PauseGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.PauseGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn pause_goal(&mut self, input: super::PauseGoal) -> Result<super::PauseGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.PreparePublication` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait PreparePublicationBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.PreparePublication`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn prepare_publication(&mut self, input: super::PreparePublication) -> Result<super::PreparePublicationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.QueueAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait QueueAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.QueueAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn queue_assignment(&mut self, input: super::QueueAssignment) -> Result<super::QueueAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ReadyAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ReadyAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ReadyAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn ready_assignment(&mut self, input: super::ReadyAssignment) -> Result<super::ReadyAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ReconcileAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ReconcileAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ReconcileAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn reconcile_assignment(&mut self, input: super::ReconcileAssignment) -> Result<super::ReconcileAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.RecordPlanningProgress` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RecordPlanningProgressBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.RecordPlanningProgress`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn record_planning_progress(&mut self, input: super::RecordPlanningProgress) -> Result<super::RecordPlanningProgressOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.RegisterRepository` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RegisterRepositoryBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.RegisterRepository`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn register_repository(&mut self, input: super::RegisterRepository) -> Result<super::RegisterRepositoryOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.RegisterWorkspace` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RegisterWorkspaceBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.RegisterWorkspace`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn register_workspace(&mut self, input: super::RegisterWorkspace) -> Result<super::RegisterWorkspaceOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.RemoveWorkspaceDirectory` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RemoveWorkspaceDirectoryBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.RemoveWorkspaceDirectory`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn remove_workspace_directory(&mut self, input: super::RemoveWorkspaceDirectory) -> Result<super::RemoveWorkspaceDirectoryOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.RepairAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RepairAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.RepairAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn repair_assignment(&mut self, input: super::RepairAssignment) -> Result<super::RepairAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.ReviewAssignment` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ReviewAssignmentBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.ReviewAssignment`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn review_assignment(&mut self, input: super::ReviewAssignment) -> Result<super::ReviewAssignmentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.SatisfyGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait SatisfyGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.SatisfyGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn satisfy_goal(&mut self, input: super::SatisfyGoal) -> Result<super::SatisfyGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.StartGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait StartGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.StartGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn start_goal(&mut self, input: super::StartGoal) -> Result<super::StartGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `controlplane.host.UpdateGoal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait UpdateGoalBehavior {
        /// Decides and enacts exactly one declared outcome of `controlplane.host.UpdateGoal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn update_goal(&mut self, input: super::UpdateGoal) -> Result<super::UpdateGoalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.AssignmentList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait AssignmentListQuery {
        /// Serves `controlplane.host.AssignmentList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn assignment_list(&self) -> Result<Vec<super::AssignmentList>, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.GoalList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait GoalListQuery {
        /// Serves `controlplane.host.GoalList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn goal_list(&self) -> Result<Vec<super::GoalList>, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.PublicationIntentList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait PublicationIntentListQuery {
        /// Serves `controlplane.host.PublicationIntentList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn publication_intent_list(&self) -> Result<Vec<super::PublicationIntentList>, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.RepositoryRegistrationList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait RepositoryRegistrationListQuery {
        /// Serves `controlplane.host.RepositoryRegistrationList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn repository_registration_list(&self) -> Result<Vec<super::RepositoryRegistrationList>, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.WorkspaceDirectoryList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait WorkspaceDirectoryListQuery {
        /// Serves `controlplane.host.WorkspaceDirectoryList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn workspace_directory_list(&self) -> Result<Vec<super::WorkspaceDirectoryList>, crate::obligation::UnmetObligation>;
    }

    /// The query `controlplane.host.WorkspaceList` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait WorkspaceListQuery {
        /// Serves `controlplane.host.WorkspaceList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn workspace_list(&self) -> Result<Vec<super::WorkspaceList>, crate::obligation::UnmetObligation>;
    }

}
