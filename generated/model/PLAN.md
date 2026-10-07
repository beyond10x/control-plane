<!--
  generated from controlplane v1
  model digest 897a3414c77f9f4e1cf2364f3618ac52b1f26e3c84b3ff542878e9e78509dbd7
  contract digest b6fee6bf66c237bc1382d9b6b607570b4f096d13d3d91c101d1efb3fde6bb9b7
  do not edit: regenerate with `ess synthesize --layout crate`
-->
# Synthesis plan — controlplane v1

Scope: `component-skeletons`, laid out as `crate`, planned by `ess-synth`. Regenerate with `ess synthesize --layout crate`.

131 capabilities: **131 generated**, **0 obligations**, **0 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `controlplane.host.Assignment.State` |
| domain type | `controlplane.host.Goal.State` |
| domain type | `controlplane.host.PlanningPhase` |
| domain type | `controlplane.host.PublicationIntent.State` |
| domain type | `controlplane.host.RepositoryRegistration.State` |
| domain type | `controlplane.host.Workspace.State` |
| domain type | `controlplane.host.WorkspaceDirectory.State` |
| entity lifecycle | `controlplane.host.Assignment` |
| entity lifecycle | `controlplane.host.Goal` |
| entity lifecycle | `controlplane.host.PublicationIntent` |
| entity lifecycle | `controlplane.host.RepositoryRegistration` |
| entity lifecycle | `controlplane.host.Workspace` |
| entity lifecycle | `controlplane.host.WorkspaceDirectory` |
| command contract | `controlplane.host.AddWorkspaceDirectory` |
| command behaviour | `controlplane.host.AddWorkspaceDirectory` |
| command contract | `controlplane.host.ArchiveWorkspace` |
| command behaviour | `controlplane.host.ArchiveWorkspace` |
| command contract | `controlplane.host.BlockAssignment` |
| command behaviour | `controlplane.host.BlockAssignment` |
| command contract | `controlplane.host.CancelAssignment` |
| command behaviour | `controlplane.host.CancelAssignment` |
| command contract | `controlplane.host.CancelGoal` |
| command behaviour | `controlplane.host.CancelGoal` |
| command contract | `controlplane.host.ClaimAssignment` |
| command behaviour | `controlplane.host.ClaimAssignment` |
| command contract | `controlplane.host.ClosePublication` |
| command behaviour | `controlplane.host.ClosePublication` |
| command contract | `controlplane.host.CompleteAssignment` |
| command behaviour | `controlplane.host.CompleteAssignment` |
| command contract | `controlplane.host.ConfigureRepository` |
| command behaviour | `controlplane.host.ConfigureRepository` |
| command contract | `controlplane.host.ConfirmPublication` |
| command behaviour | `controlplane.host.ConfirmPublication` |
| command contract | `controlplane.host.CreateGoal` |
| command behaviour | `controlplane.host.CreateGoal` |
| command contract | `controlplane.host.DeleteGoal` |
| command behaviour | `controlplane.host.DeleteGoal` |
| command contract | `controlplane.host.DisableRepositoryRegistration` |
| command behaviour | `controlplane.host.DisableRepositoryRegistration` |
| command contract | `controlplane.host.EnableRepositoryRegistration` |
| command behaviour | `controlplane.host.EnableRepositoryRegistration` |
| command contract | `controlplane.host.MarkPublicationUncertain` |
| command behaviour | `controlplane.host.MarkPublicationUncertain` |
| command contract | `controlplane.host.MergeAssignment` |
| command behaviour | `controlplane.host.MergeAssignment` |
| command contract | `controlplane.host.PauseGoal` |
| command behaviour | `controlplane.host.PauseGoal` |
| command contract | `controlplane.host.PreparePublication` |
| command behaviour | `controlplane.host.PreparePublication` |
| command contract | `controlplane.host.QueueAssignment` |
| command behaviour | `controlplane.host.QueueAssignment` |
| command contract | `controlplane.host.ReadyAssignment` |
| command behaviour | `controlplane.host.ReadyAssignment` |
| command contract | `controlplane.host.ReconcileAssignment` |
| command behaviour | `controlplane.host.ReconcileAssignment` |
| command contract | `controlplane.host.RecordPlanningProgress` |
| command behaviour | `controlplane.host.RecordPlanningProgress` |
| command contract | `controlplane.host.RegisterRepository` |
| command behaviour | `controlplane.host.RegisterRepository` |
| command contract | `controlplane.host.RegisterWorkspace` |
| command behaviour | `controlplane.host.RegisterWorkspace` |
| command contract | `controlplane.host.RemoveWorkspaceDirectory` |
| command behaviour | `controlplane.host.RemoveWorkspaceDirectory` |
| command contract | `controlplane.host.RepairAssignment` |
| command behaviour | `controlplane.host.RepairAssignment` |
| command contract | `controlplane.host.ReviewAssignment` |
| command behaviour | `controlplane.host.ReviewAssignment` |
| command contract | `controlplane.host.SatisfyGoal` |
| command behaviour | `controlplane.host.SatisfyGoal` |
| command contract | `controlplane.host.StartGoal` |
| command behaviour | `controlplane.host.StartGoal` |
| command contract | `controlplane.host.UpdateGoal` |
| command behaviour | `controlplane.host.UpdateGoal` |
| event type | `controlplane.host.ArchiveWorkspaceApplied` |
| event type | `controlplane.host.AssignmentCreated` |
| event type | `controlplane.host.BlockAssignmentApplied` |
| event type | `controlplane.host.CancelAssignmentApplied` |
| event type | `controlplane.host.CancelGoalApplied` |
| event type | `controlplane.host.ClaimAssignmentApplied` |
| event type | `controlplane.host.ClosePublicationApplied` |
| event type | `controlplane.host.CompleteAssignmentApplied` |
| event type | `controlplane.host.ConfigureRepositoryApplied` |
| event type | `controlplane.host.ConfirmPublicationApplied` |
| event type | `controlplane.host.DisableRepositoryRegistrationApplied` |
| event type | `controlplane.host.EnableRepositoryRegistrationApplied` |
| event type | `controlplane.host.GoalCreated` |
| event type | `controlplane.host.GoalDeleted` |
| event type | `controlplane.host.MarkPublicationUncertainApplied` |
| event type | `controlplane.host.MergeAssignmentApplied` |
| event type | `controlplane.host.PauseGoalApplied` |
| event type | `controlplane.host.PlanningProgressRecorded` |
| event type | `controlplane.host.PublicationIntentCreated` |
| event type | `controlplane.host.ReadyAssignmentApplied` |
| event type | `controlplane.host.ReconcileAssignmentApplied` |
| event type | `controlplane.host.RepairAssignmentApplied` |
| event type | `controlplane.host.RepositoryRegistrationCreated` |
| event type | `controlplane.host.ReviewAssignmentApplied` |
| event type | `controlplane.host.SatisfyGoalApplied` |
| event type | `controlplane.host.StartGoalApplied` |
| event type | `controlplane.host.UpdateGoalApplied` |
| event type | `controlplane.host.WorkspaceCreated` |
| event type | `controlplane.host.WorkspaceDirectoryCreated` |
| event type | `controlplane.host.WorkspaceDirectoryRemoved` |
| error type | `controlplane.host.AssignmentNotFound` |
| error type | `controlplane.host.AssignmentStateConflict` |
| error type | `controlplane.host.GoalNotFound` |
| error type | `controlplane.host.GoalStateConflict` |
| error type | `controlplane.host.PublicationIntentNotFound` |
| error type | `controlplane.host.PublicationIntentStateConflict` |
| error type | `controlplane.host.RepositoryRegistrationNotFound` |
| error type | `controlplane.host.RepositoryRegistrationStateConflict` |
| error type | `controlplane.host.WorkspaceDirectoryNotFound` |
| error type | `controlplane.host.WorkspaceDirectoryStateConflict` |
| error type | `controlplane.host.WorkspaceNotFound` |
| error type | `controlplane.host.WorkspaceStateConflict` |
| view type | `controlplane.host.AssignmentList` |
| view query | `controlplane.host.AssignmentList` |
| view type | `controlplane.host.GoalList` |
| view query | `controlplane.host.GoalList` |
| view type | `controlplane.host.PublicationIntentList` |
| view query | `controlplane.host.PublicationIntentList` |
| view type | `controlplane.host.RepositoryRegistrationList` |
| view query | `controlplane.host.RepositoryRegistrationList` |
| view type | `controlplane.host.WorkspaceDirectoryList` |
| view query | `controlplane.host.WorkspaceDirectoryList` |
| view type | `controlplane.host.WorkspaceList` |
| view query | `controlplane.host.WorkspaceList` |
| actor grants | `controlplane.host.Operator` |
| actor grants | `controlplane.host.Supervisor` |
| component port | `control-plane` |
| component transport | `control-plane` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
