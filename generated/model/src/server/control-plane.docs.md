<!--
generated from controlplane v1
model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
contract digest slice-sha256/2:e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
do not edit: regenerate with `ess generate`
-->

# host

Local workspace goals and assignments; AEP owns stories, Loom owns agent executions.

`controlplane.host` is one of controlplane's bounded contexts. [Back to the index](../index.md).

## Types

### `PlanningPhase`

`controlplane.host.PlanningPhase` is one of `Idle`, `Provisioning`, `Planning`, `Validated`, `Queued` and `Blocked`.

## Entities

An entity is what this context is about: something with an identity that outlives any one request, a shape, and a lifecycle. The lifecycle is exhaustive — a move that is not drawn below is a move this specification does not permit, and that is the only way it says so. Every move is labelled with the command that takes it, because a move nothing can trigger is refused rather than drawn.

### `Assignment`

`controlplane.host.Assignment`.

An instance is identified by `assignment_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `goal_id` — `Uuid`
- `repository_id` — `Uuid`
- `story_id` — `String`
- `case_id` — `String`
- `worktree_id` — `String`
- `candidate` — `String`
- `attempt` — `Integer`
- `reason` — `String`
- `implementor_run` — `String`
- `reviewer_run` — `String`
- `goal_revision` — `Integer`
- `base_revision` — `String`
- `test_revision` — `String`
- `review_revision` — `String`
- `merge_receipt` — `String`

It references at most one [`RepositoryRegistration`](#repositoryregistration), as `repository`, carried by `Assignment.repository_id`. It owns any number of [`PublicationIntent`](#publicationintent), as `publications`, carried by `PublicationIntent.assignment_id`. Its `goal_id` is what [`Goal`](#goal) owns it by, as `assignments`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.Assignment.State`, one of `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued`, `ReadyToMerge` and `Reviewing`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Queued`. `Cancelled` and `Merged` are terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.

```mermaid
stateDiagram-v2
    [*] --> Queued
    Queued --> Implementing: claim (ClaimAssignment)
    Implementing --> Reviewing: review (ReviewAssignment)
    Blocked --> Implementing: repair (RepairAssignment)
    Reviewing --> Implementing: repair (RepairAssignment)
    Reviewing --> ReadyToMerge: ready (ReadyAssignment)
    ReadyToMerge --> Merging: merge (MergeAssignment)
    Merging --> Merged: complete (CompleteAssignment)
    Blocked --> Blocked: block (BlockAssignment)
    Implementing --> Blocked: block (BlockAssignment)
    Merging --> Blocked: block (BlockAssignment)
    Queued --> Blocked: block (BlockAssignment)
    ReadyToMerge --> Blocked: block (BlockAssignment)
    Reviewing --> Blocked: block (BlockAssignment)
    Blocked --> Cancelled: cancel (CancelAssignment)
    Implementing --> Cancelled: cancel (CancelAssignment)
    Queued --> Cancelled: cancel (CancelAssignment)
    ReadyToMerge --> Cancelled: cancel (CancelAssignment)
    Reviewing --> Cancelled: cancel (CancelAssignment)
    Blocked --> Merged: reconcile (ReconcileAssignment)
    Merging --> Merged: reconcile (ReconcileAssignment)
    Cancelled --> [*]
    Merged --> [*]
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `claim` — taken by `controlplane.host.ClaimAssignment` on its `applied` outcome
- `review` — taken by `controlplane.host.ReviewAssignment` on its `applied` outcome
- `repair` — taken by `controlplane.host.RepairAssignment` on its `rebased` outcome and `controlplane.host.RepairAssignment` on its `applied` outcome
- `ready` — taken by `controlplane.host.ReadyAssignment` on its `applied` outcome
- `merge` — taken by `controlplane.host.MergeAssignment` on its `applied` outcome
- `complete` — taken by `controlplane.host.CompleteAssignment` on its `applied` outcome
- `block` — taken by `controlplane.host.BlockAssignment` on its `applied` outcome
- `cancel` — taken by `controlplane.host.CancelAssignment` on its `applied` outcome
- `reconcile` — taken by `controlplane.host.ReconcileAssignment` on its `applied` outcome

An instance is brought into existence by `controlplane.host.QueueAssignment` on its `created` outcome.

Illegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.

- `Blocked` may not become `Merging`
- `Blocked` may not become `Queued`
- `Blocked` may not become `ReadyToMerge`
- `Blocked` may not become `Reviewing`
- `Cancelled` may not become `Blocked`
- `Cancelled` may not become `Implementing`
- `Cancelled` may not become `Merged`
- `Cancelled` may not become `Merging`
- `Cancelled` may not become `Queued`
- `Cancelled` may not become `ReadyToMerge`
- `Cancelled` may not become `Reviewing`
- `Implementing` may not become `Merged`
- `Implementing` may not become `Merging`
- `Implementing` may not become `Queued`
- `Implementing` may not become `ReadyToMerge`
- `Merged` may not become `Blocked`
- `Merged` may not become `Cancelled`
- `Merged` may not become `Implementing`
- `Merged` may not become `Merging`
- `Merged` may not become `Queued`
- `Merged` may not become `ReadyToMerge`
- `Merged` may not become `Reviewing`
- `Merging` may not become `Cancelled`
- `Merging` may not become `Implementing`
- `Merging` may not become `Queued`
- `Merging` may not become `ReadyToMerge`
- `Merging` may not become `Reviewing`
- `Queued` may not become `Merged`
- `Queued` may not become `Merging`
- `Queued` may not become `ReadyToMerge`
- `Queued` may not become `Reviewing`
- `ReadyToMerge` may not become `Implementing`
- `ReadyToMerge` may not become `Merged`
- `ReadyToMerge` may not become `Queued`
- `ReadyToMerge` may not become `Reviewing`
- `Reviewing` may not become `Merged`
- `Reviewing` may not become `Merging`
- `Reviewing` may not become `Queued`

One view projects it: [`AssignmentList`](#assignmentlist).

### `Goal`

`controlplane.host.Goal`.

An instance is identified by `goal_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `workspace_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`
- `revision` — `Integer`
- `satisfaction_receipt` — `String`
- `planning_revision` — `Integer`
- `planning_fingerprint` — `String`
- `planning_repository` — `String`
- `planning_worktree_id` — `String`
- `planning_worktree_path` — `String`
- `planning_reason` — `String`
- `planning_receipt` — `String`
- `planning_phase` — `controlplane.host.PlanningPhase`

It owns any number of [`Assignment`](#assignment), as `assignments`, carried by `Assignment.goal_id`. Its `workspace_id` is what [`Workspace`](#workspace) owns it by, as `goals`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.Goal.State`, one of `Cancelled`, `Paused`, `Running` and `Satisfied`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Paused`. `Cancelled` and `Satisfied` are terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.

```mermaid
stateDiagram-v2
    [*] --> Paused
    Paused --> Running: start (StartGoal)
    Running --> Paused: pause (PauseGoal)
    Running --> Satisfied: satisfy (SatisfyGoal)
    Paused --> Cancelled: cancel (CancelGoal)
    Running --> Cancelled: cancel (CancelGoal)
    Cancelled --> [*]
    Satisfied --> [*]
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `start` — taken by `controlplane.host.StartGoal` on its `applied` outcome
- `pause` — taken by `controlplane.host.PauseGoal` on its `applied` outcome
- `satisfy` — taken by `controlplane.host.SatisfyGoal` on its `applied` outcome
- `cancel` — taken by `controlplane.host.CancelGoal` on its `applied` outcome

An instance is brought into existence by `controlplane.host.CreateGoal` on its `created` outcome.

Illegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.

- `Cancelled` may not become `Paused`
- `Cancelled` may not become `Running`
- `Cancelled` may not become `Satisfied`
- `Paused` may not become `Satisfied`
- `Satisfied` may not become `Cancelled`
- `Satisfied` may not become `Paused`
- `Satisfied` may not become `Running`

One view projects it: [`GoalList`](#goallist).

### `PublicationIntent`

`controlplane.host.PublicationIntent`.

An instance is identified by `publication_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `assignment_id` — `Uuid`
- `candidate` — `String`
- `target` — `String`
- `expected_base` — `String`
- `receipt` — `String`
- `reason` — `Optional<String>`, which may be absent

Its `assignment_id` is what [`Assignment`](#assignment) owns it by, as `publications`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.PublicationIntent.State`, one of `Confirmed`, `NotPublished`, `Prepared` and `Uncertain`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Prepared`. `Confirmed` and `NotPublished` are terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.

```mermaid
stateDiagram-v2
    [*] --> Prepared
    Prepared --> Uncertain: uncertain (MarkPublicationUncertain)
    Prepared --> Confirmed: confirm (ConfirmPublication)
    Uncertain --> Confirmed: confirm (ConfirmPublication)
    Prepared --> NotPublished: close (ClosePublication)
    Uncertain --> NotPublished: close (ClosePublication)
    Confirmed --> [*]
    NotPublished --> [*]
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `uncertain` — taken by `controlplane.host.MarkPublicationUncertain` on its `applied` outcome
- `confirm` — taken by `controlplane.host.ConfirmPublication` on its `applied` outcome
- `close` — taken by `controlplane.host.ClosePublication` on its `applied` outcome

An instance is brought into existence by `controlplane.host.PreparePublication` on its `created` outcome.

Illegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.

- `Confirmed` may not become `NotPublished`
- `Confirmed` may not become `Prepared`
- `Confirmed` may not become `Uncertain`
- `NotPublished` may not become `Confirmed`
- `NotPublished` may not become `Prepared`
- `NotPublished` may not become `Uncertain`
- `Uncertain` may not become `Prepared`

One view projects it: [`PublicationIntentList`](#publicationintentlist).

### `RepositoryRegistration`

`controlplane.host.RepositoryRegistration`.

An instance is identified by `repository_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `workspace_id` — `Uuid`
- `name` — `String`
- `path` — `String`
- `common_dir` — `String`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`

Its `workspace_id` is what [`Workspace`](#workspace) owns it by, as `repositories`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.RepositoryRegistration.State`, one of `Disabled` and `Registered`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Registered`. No state is terminal: nothing in this lifecycle says an instance may stop moving.

```mermaid
stateDiagram-v2
    [*] --> Registered
    Registered --> Disabled: disable (DisableRepositoryRegistration)
    Disabled --> Registered: enable (EnableRepositoryRegistration)
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `disable` — taken by `controlplane.host.DisableRepositoryRegistration` on its `applied` outcome
- `enable` — taken by `controlplane.host.EnableRepositoryRegistration` on its `applied` outcome

An instance is brought into existence by `controlplane.host.RegisterRepository` on its `created` outcome.

Every ordered pair of these states is connected by some move, so this lifecycle forbids nothing.

One view projects it: [`RepositoryRegistrationList`](#repositoryregistrationlist).

### `Workspace`

`controlplane.host.Workspace`.

An instance is identified by `workspace_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `path` — `String`
- `name` — `String`

It owns any number of [`WorkspaceDirectory`](#workspacedirectory), as `directories`, carried by `WorkspaceDirectory.workspace_id`. It owns any number of [`RepositoryRegistration`](#repositoryregistration), as `repositories`, carried by `RepositoryRegistration.workspace_id`. It owns any number of [`Goal`](#goal), as `goals`, carried by `Goal.workspace_id`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.Workspace.State`, one of `Archived` and `Registered`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Registered`. `Archived` is terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.

```mermaid
stateDiagram-v2
    [*] --> Registered
    Registered --> Archived: archive (ArchiveWorkspace)
    Archived --> [*]
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `archive` — taken by `controlplane.host.ArchiveWorkspace` on its `applied` outcome

An instance is brought into existence by `controlplane.host.RegisterWorkspace` on its `created` outcome.

Illegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.

- `Archived` may not become `Registered`

One view projects it: [`WorkspaceList`](#workspacelist).

### `WorkspaceDirectory`

`controlplane.host.WorkspaceDirectory`.

An instance is identified by `directory_id`, a `Uuid`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.

It holds:

- `workspace_id` — `Uuid`
- `path` — `String`
- `repository_common_dirs` — `List<String>`
- `managed_common_dirs` — `List<String>`

Its `workspace_id` is what [`Workspace`](#workspace) owns it by, as `directories`.

No invariant is declared, so nothing here constrains an instance at rest.

Its state is a `controlplane.host.WorkspaceDirectory.State`, one of `Registered` and `Removed`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.

An instance is created in `Registered`. `Removed` is terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.

```mermaid
stateDiagram-v2
    [*] --> Registered
    Registered --> Removed: remove (RemoveWorkspaceDirectory)
    Removed --> [*]
```

Each move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:

- `remove` — taken by `controlplane.host.RemoveWorkspaceDirectory` on its `applied` outcome

An instance is brought into existence by `controlplane.host.AddWorkspaceDirectory` on its `created` outcome.

Illegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.

- `Removed` may not become `Registered`

One view projects it: [`WorkspaceDirectoryList`](#workspacedirectorylist).

## Views

A view is what the outside world is promised it can observe. Each one says which instances it contains and how soon it reflects a command that has already returned, because "you can read this" without "how soon" is the promise every flaky suite is built on.

### `AssignmentList`

`controlplane.host.AssignmentList`.

It reads [`Assignment`](#assignment).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `assignment_id` — `Uuid`
- `goal_id` — `Uuid`
- `repository_id` — `Uuid`
- `story_id` — `String`
- `case_id` — `String`
- `worktree_id` — `String`
- `candidate` — `String`
- `attempt` — `Integer`
- `reason` — `String`
- `implementor_run` — `String`
- `reviewer_run` — `String`
- `goal_revision` — `Integer`
- `base_revision` — `String`
- `test_revision` — `String`
- `review_revision` — `String`
- `merge_receipt` — `String`
- `state` — `controlplane.host.Assignment.State`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

### `GoalList`

`controlplane.host.GoalList`.

It reads [`Goal`](#goal).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `goal_id` — `Uuid`
- `workspace_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`
- `revision` — `Integer`
- `satisfaction_receipt` — `String`
- `state` — `controlplane.host.Goal.State`
- `planning_revision` — `Integer`
- `planning_fingerprint` — `String`
- `planning_repository` — `String`
- `planning_worktree_id` — `String`
- `planning_worktree_path` — `String`
- `planning_reason` — `String`
- `planning_receipt` — `String`
- `planning_phase` — `controlplane.host.PlanningPhase`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

### `PublicationIntentList`

`controlplane.host.PublicationIntentList`.

It reads [`PublicationIntent`](#publicationintent).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `publication_id` — `Uuid`
- `assignment_id` — `Uuid`
- `candidate` — `String`
- `target` — `String`
- `expected_base` — `String`
- `receipt` — `String`
- `reason` — `Optional<String>`, which may be absent
- `state` — `controlplane.host.PublicationIntent.State`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

### `RepositoryRegistrationList`

`controlplane.host.RepositoryRegistrationList`.

It reads [`RepositoryRegistration`](#repositoryregistration).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `repository_id` — `Uuid`
- `workspace_id` — `Uuid`
- `name` — `String`
- `path` — `String`
- `common_dir` — `String`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`
- `state` — `controlplane.host.RepositoryRegistration.State`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

### `WorkspaceDirectoryList`

`controlplane.host.WorkspaceDirectoryList`.

It reads [`WorkspaceDirectory`](#workspacedirectory).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `directory_id` — `Uuid`
- `workspace_id` — `Uuid`
- `path` — `String`
- `repository_common_dirs` — `List<String>`
- `managed_common_dirs` — `List<String>`
- `state` — `controlplane.host.WorkspaceDirectory.State`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

### `WorkspaceList`

`controlplane.host.WorkspaceList`.

It reads [`Workspace`](#workspace).

It contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.

It exposes:

- `workspace_id` — `Uuid`
- `path` — `String`
- `name` — `String`
- `state` — `controlplane.host.Workspace.State`

It declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.

**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.

A generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.

## Commands

### `AddWorkspaceDirectory`

`controlplane.host.AddWorkspaceDirectory`.

It takes:

- `workspace_id` — `Uuid`
- `path` — `String`
- `repository_common_dirs` — `List<String>`
- `managed_common_dirs` — `List<String>`

It has one outcome.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.WorkspaceDirectory`, which starts in `Registered`. The new instance's identity is published as `directory_id` on `controlplane.host.WorkspaceDirectoryCreated`. It emits `controlplane.host.WorkspaceDirectoryCreated`. It sets `workspace_id` from `input.workspace_id`, `path` from `input.path`, `repository_common_dirs` from `input.repository_common_dirs` and `managed_common_dirs` from `input.managed_common_dirs`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `ArchiveWorkspace`

`controlplane.host.ArchiveWorkspace`.

It takes:

- `workspace_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Workspace` from `Registered` to `Archived`, along the declared move `archive`. The instance is the one named by the input field `workspace_id`. It emits `controlplane.host.ArchiveWorkspaceApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Workspace` in `Archived`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.WorkspaceStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.WorkspaceNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `BlockAssignment`

`controlplane.host.BlockAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `reason` — `String`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Blocked`, `Implementing`, `Merging`, `Queued`, `ReadyToMerge` and `Reviewing` to `Blocked`, along the declared move `block`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.BlockAssignmentApplied`. It sets `reason` from `input.reason`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Cancelled` and `Merged`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `CancelAssignment`

`controlplane.host.CancelAssignment`.

It takes:

- `assignment_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Blocked`, `Implementing`, `Queued`, `ReadyToMerge` and `Reviewing` to `Cancelled`, along the declared move `cancel`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.CancelAssignmentApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Cancelled`, `Merged` and `Merging`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `CancelGoal`

`controlplane.host.CancelGoal`.

It takes:

- `goal_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Goal` from `Paused` and `Running` to `Cancelled`, along the declared move `cancel`. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.CancelGoalApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Goal` in `Cancelled` and `Satisfied`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ClaimAssignment`

`controlplane.host.ClaimAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `worktree_id` — `String`
- `implementor_run` — `String`
- `base_revision` — `String`

It has four outcomes.

**`evidence-missing`** — Taken when the existing subject's stored fields satisfy `(input.implementor_run == "" or input.worktree_id == "" or input.base_revision == "")`. No entity in this specification changes. It reports `controlplane.host.EvidenceMissing`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Queued` to `Implementing`, along the declared move `claim`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.ClaimAssignmentApplied`. It sets `worktree_id` from `input.worktree_id`, `attempt` from `its previous value plus 1`, `implementor_run` from `input.implementor_run` and `base_revision` from `input.base_revision`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `ReadyToMerge` and `Reviewing`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ClosePublication`

`controlplane.host.ClosePublication`.

It takes:

- `publication_id` — `Uuid`
- `reason` — `String`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.PublicationIntent` from `Prepared` and `Uncertain` to `NotPublished`, along the declared move `close`. The instance is the one named by the input field `publication_id`. It emits `controlplane.host.ClosePublicationApplied`. It sets `reason` from `input.reason`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.PublicationIntent` in `Confirmed` and `NotPublished`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

### `CompleteAssignment`

`controlplane.host.CompleteAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `merge_receipt` — `String`

It has four outcomes.

**`receipt-missing`** — Taken when the existing subject's stored fields satisfy `input.merge_receipt == ""`. No entity in this specification changes. It reports `controlplane.host.EvidenceMissing`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Merging` to `Merged`, along the declared move `complete`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.CompleteAssignmentApplied`. It sets `merge_receipt` from `input.merge_receipt`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Queued`, `ReadyToMerge` and `Reviewing`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ConfigureRepository`

`controlplane.host.ConfigureRepository`.

It takes:

- `repository_id` — `Uuid`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`

It has two outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It changes a `controlplane.host.RepositoryRegistration` without moving it along its lifecycle. The instance is the one named by the input field `repository_id`. It emits `controlplane.host.ConfigureRepositoryApplied`. It sets `base_branch` from `input.base_branch`, `test_command` from `input.test_command` and `publish_command` from `input.publish_command`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.RepositoryRegistrationNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ConfirmPublication`

`controlplane.host.ConfirmPublication`.

It takes:

- `publication_id` — `Uuid`
- `receipt` — `String`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.PublicationIntent` from `Prepared` and `Uncertain` to `Confirmed`, along the declared move `confirm`. The instance is the one named by the input field `publication_id`. It emits `controlplane.host.ConfirmPublicationApplied`. It sets `receipt` from `input.receipt`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.PublicationIntent` in `Confirmed` and `NotPublished`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

### `CreateGoal`

`controlplane.host.CreateGoal`.

It takes:

- `workspace_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`

It has four outcomes.

**`workers-invalid`** — Taken when `max_workers <= 0` holds of the input. No entity in this specification changes. It reports `controlplane.host.GoalLimitInvalid`. It emits nothing. A test reaches it by constructing an input that satisfies that condition.

**`attempts-invalid`** — Taken when `max_attempts <= 0` holds of the input. No entity in this specification changes. It reports `controlplane.host.GoalLimitInvalid`. It emits nothing. A test reaches it by constructing an input that satisfies that condition.

**`minutes-invalid`** — Taken when `max_minutes <= 0` holds of the input. No entity in this specification changes. It reports `controlplane.host.GoalLimitInvalid`. It emits nothing. A test reaches it by constructing an input that satisfies that condition.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.Goal`, which starts in `Paused`. The new instance's identity is published as `goal_id` on `controlplane.host.GoalCreated`. It emits `controlplane.host.GoalCreated`. It sets `workspace_id` from `input.workspace_id`, `objective` from `input.objective`, `acceptance` from `input.acceptance`, `max_workers` from `input.max_workers`, `max_attempts` from `input.max_attempts`, `max_minutes` from `input.max_minutes`, `planner_model` from `input.planner_model`, `implementor_model` from `input.implementor_model`, `reviewer_model` from `input.reviewer_model`, `merge_authority` from `input.merge_authority`, `revision` from `"1"`, `satisfaction_receipt` from `""`, `planning_revision` from `"0"`, `planning_fingerprint` from `""`, `planning_repository` from `""`, `planning_worktree_id` from `""`, `planning_worktree_path` from `""`, `planning_reason` from `""`, `planning_receipt` from `""` and `planning_phase` from `"Idle"`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `DeleteGoal`

`controlplane.host.DeleteGoal`.

It takes:

- `goal_id` — `Uuid`

It has five outcomes.

**`applied`** — Taken when the existing subject is in Cancelled. It removes the `controlplane.host.Goal` its input names; no view shows it afterwards. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.GoalDeleted`. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`paused`** — Taken when the existing subject is in Paused. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`running`** — Taken when the existing subject is in Running. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`satisfied`** — Taken when the existing subject is in Satisfied. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `DisableRepositoryRegistration`

`controlplane.host.DisableRepositoryRegistration`.

It takes:

- `repository_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.RepositoryRegistration` from `Registered` to `Disabled`, along the declared move `disable`. The instance is the one named by the input field `repository_id`. It emits `controlplane.host.DisableRepositoryRegistrationApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.RepositoryRegistration` in `Disabled`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.RepositoryRegistrationStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.RepositoryRegistrationNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `EnableRepositoryRegistration`

`controlplane.host.EnableRepositoryRegistration`.

It takes:

- `repository_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.RepositoryRegistration` from `Disabled` to `Registered`, along the declared move `enable`. The instance is the one named by the input field `repository_id`. It emits `controlplane.host.EnableRepositoryRegistrationApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.RepositoryRegistration` in `Registered`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.RepositoryRegistrationStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.RepositoryRegistrationNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `MarkPublicationUncertain`

`controlplane.host.MarkPublicationUncertain`.

It takes:

- `publication_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.PublicationIntent` from `Prepared` to `Uncertain`, along the declared move `uncertain`. The instance is the one named by the input field `publication_id`. It emits `controlplane.host.MarkPublicationUncertainApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.PublicationIntent` in `Confirmed`, `NotPublished` and `Uncertain`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.PublicationIntentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

### `MergeAssignment`

`controlplane.host.MergeAssignment`.

It takes:

- `assignment_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `ReadyToMerge` to `Merging`, along the declared move `merge`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.MergeAssignmentApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued` and `Reviewing`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `PauseGoal`

`controlplane.host.PauseGoal`.

It takes:

- `goal_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Goal` from `Running` to `Paused`, along the declared move `pause`. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.PauseGoalApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Goal` in `Cancelled`, `Paused` and `Satisfied`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `PreparePublication`

`controlplane.host.PreparePublication`.

It takes:

- `assignment_id` — `Uuid`
- `candidate` — `String`
- `target` — `String`
- `expected_base` — `String`

It has one outcome.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.PublicationIntent`, which starts in `Prepared`. The new instance's identity is published as `publication_id` on `controlplane.host.PublicationIntentCreated`. It emits `controlplane.host.PublicationIntentCreated`. It sets `assignment_id` from `input.assignment_id`, `candidate` from `input.candidate`, `target` from `input.target`, `expected_base` from `input.expected_base` and `receipt` from `""`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `QueueAssignment`

`controlplane.host.QueueAssignment`.

It takes:

- `goal_id` — `Uuid`
- `repository_id` — `Uuid`
- `story_id` — `String`
- `case_id` — `String`
- `worktree_id` — `String`
- `candidate` — `String`
- `attempt` — `Integer`
- `reason` — `String`
- `implementor_run` — `String`
- `reviewer_run` — `String`
- `goal_revision` — `Integer`

It has three outcomes.

**`goal-not-found`** — Taken when no `controlplane.host.Goal` carries the identity `input.goal_id` names. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by arranging the row of the other entity the input names, or its absence, and sending the command for it.

**`goal-not-current`** — Taken when the `controlplane.host.Goal` that `input.goal_id` names exists and its stored fields satisfy `(state != Running or revision != input.goal_revision)`. No entity in this specification changes. It reports `controlplane.host.GoalNotCurrent`. It emits nothing. A test reaches it by arranging the row of the other entity the input names, or its absence, and sending the command for it.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.Assignment`, which starts in `Queued`. The new instance's identity is published as `assignment_id` on `controlplane.host.AssignmentCreated`. It emits `controlplane.host.AssignmentCreated`. It sets `goal_id` from `input.goal_id`, `repository_id` from `input.repository_id`, `story_id` from `input.story_id`, `case_id` from `input.case_id`, `worktree_id` from `input.worktree_id`, `candidate` from `input.candidate`, `attempt` from `input.attempt`, `reason` from `input.reason`, `implementor_run` from `input.implementor_run`, `reviewer_run` from `input.reviewer_run`, `goal_revision` from `input.goal_revision`, `base_revision` from `""`, `test_revision` from `""`, `review_revision` from `""` and `merge_receipt` from `""`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `ReadyAssignment`

`controlplane.host.ReadyAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `reviewer_run` — `String`
- `review_revision` — `String`

It has six outcomes.

**`reviewer-missing`** — Taken when the existing subject's stored fields satisfy `input.reviewer_run == ""`. No entity in this specification changes. It reports `controlplane.host.ReviewNotIndependent`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`review-not-independent`** — Taken when the existing subject's stored fields satisfy `implementor_run == input.reviewer_run`. No entity in this specification changes. It reports `controlplane.host.ReviewNotIndependent`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`evidence-not-current`** — Taken when the existing subject's stored fields satisfy `(state == Reviewing and (candidate == "" or test_revision != {fact: candidate} or candidate != input.review_revision))`. No entity in this specification changes. It reports `controlplane.host.EvidenceNotCurrent`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Reviewing` to `ReadyToMerge`, along the declared move `ready`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.ReadyAssignmentApplied`. It sets `reviewer_run` from `input.reviewer_run` and `review_revision` from `input.review_revision`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued` and `ReadyToMerge`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ReconcileAssignment`

`controlplane.host.ReconcileAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `merge_receipt` — `String`

It has four outcomes.

**`receipt-missing`** — Taken when the existing subject's stored fields satisfy `input.merge_receipt == ""`. No entity in this specification changes. It reports `controlplane.host.EvidenceMissing`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Blocked` and `Merging` to `Merged`, along the declared move `reconcile`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.ReconcileAssignmentApplied`. It sets `merge_receipt` from `input.merge_receipt`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Cancelled`, `Implementing`, `Merged`, `Queued`, `ReadyToMerge` and `Reviewing`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

### `RecordPlanningProgress`

`controlplane.host.RecordPlanningProgress`.

It takes:

- `goal_id` — `Uuid`
- `planning_revision` — `Integer`
- `planning_fingerprint` — `String`
- `planning_repository` — `String`
- `planning_worktree_id` — `String`
- `planning_worktree_path` — `String`
- `planning_reason` — `String`
- `planning_receipt` — `String`
- `planning_phase` — `controlplane.host.PlanningPhase`

It has two outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It changes a `controlplane.host.Goal` without moving it along its lifecycle. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.PlanningProgressRecorded`. It sets `planning_revision` from `input.planning_revision`, `planning_fingerprint` from `input.planning_fingerprint`, `planning_repository` from `input.planning_repository`, `planning_worktree_id` from `input.planning_worktree_id`, `planning_worktree_path` from `input.planning_worktree_path`, `planning_reason` from `input.planning_reason`, `planning_receipt` from `input.planning_receipt` and `planning_phase` from `input.planning_phase`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `RegisterRepository`

`controlplane.host.RegisterRepository`.

It takes:

- `workspace_id` — `Uuid`
- `name` — `String`
- `path` — `String`
- `common_dir` — `String`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`

It has one outcome.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.RepositoryRegistration`, which starts in `Registered`. The new instance's identity is published as `repository_id` on `controlplane.host.RepositoryRegistrationCreated`. It emits `controlplane.host.RepositoryRegistrationCreated`. It sets `workspace_id` from `input.workspace_id`, `name` from `input.name`, `path` from `input.path`, `common_dir` from `input.common_dir`, `base_branch` from `input.base_branch`, `test_command` from `input.test_command` and `publish_command` from `input.publish_command`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `RegisterWorkspace`

`controlplane.host.RegisterWorkspace`.

It takes:

- `path` — `String`
- `name` — `String`

It has one outcome.

**`created`** — The default branch, taken when no other outcome's condition matched. It creates a `controlplane.host.Workspace`, which starts in `Registered`. The new instance's identity is published as `workspace_id` on `controlplane.host.WorkspaceCreated`. It emits `controlplane.host.WorkspaceCreated`. It sets `path` from `input.path` and `name` from `input.name`. A test reaches it by constructing an input that satisfies no other outcome's condition.

### `RemoveWorkspaceDirectory`

`controlplane.host.RemoveWorkspaceDirectory`.

It takes:

- `directory_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.WorkspaceDirectory` from `Registered` to `Removed`, along the declared move `remove`. The instance is the one named by the input field `directory_id`. It emits `controlplane.host.WorkspaceDirectoryRemoved`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.WorkspaceDirectory` in `Removed`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.WorkspaceDirectoryStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.WorkspaceDirectoryNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `RepairAssignment`

`controlplane.host.RepairAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `reason` — `String`
- `implementor_run` — `String`
- `base_revision` — `Optional<String>`, which may be absent

It has six outcomes.

**`evidence-missing`** — Taken when the existing subject's stored fields satisfy `input.implementor_run == ""`. No entity in this specification changes. It reports `controlplane.host.EvidenceMissing`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`base-missing`** — Taken when the existing subject's stored fields satisfy `(defined(input.base_revision) and input.base_revision == "")`. No entity in this specification changes. It reports `controlplane.host.EvidenceMissing`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`rebased`** — Taken when `defined(base_revision)` holds of the input. It moves a `controlplane.host.Assignment` from `Blocked` and `Reviewing` to `Implementing`, along the declared move `repair`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.RepairAssignmentApplied`. It sets `attempt` from `its previous value plus 1`, `reason` from `input.reason`, `implementor_run` from `input.implementor_run`, `reviewer_run` from `""`, `base_revision` from `input.base_revision, else ""`, `test_revision` from `""` and `review_revision` from `""`. A test reaches it by constructing an input that satisfies that condition.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Blocked` and `Reviewing` to `Implementing`, along the declared move `repair`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.RepairAssignmentApplied`. It sets `attempt` from `its previous value plus 1`, `reason` from `input.reason`, `implementor_run` from `input.implementor_run`, `reviewer_run` from `""`, `test_revision` from `""` and `review_revision` from `""`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued` and `ReadyToMerge`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `ReviewAssignment`

`controlplane.host.ReviewAssignment`.

It takes:

- `assignment_id` — `Uuid`
- `candidate` — `String`
- `test_revision` — `String`

It has four outcomes.

**`tests-not-current`** — Taken when the existing subject's stored fields satisfy `(input.candidate == "" or input.candidate != input.test_revision)`. No entity in this specification changes. It reports `controlplane.host.EvidenceNotCurrent`. It emits nothing. A test establishes and independently observes the subject enum fact before selecting this branch.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Assignment` from `Implementing` to `Reviewing`, along the declared move `review`. The instance is the one named by the input field `assignment_id`. It emits `controlplane.host.ReviewAssignmentApplied`. It sets `candidate` from `input.candidate` and `test_revision` from `input.test_revision`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Assignment` in `Blocked`, `Cancelled`, `Merged`, `Merging`, `Queued`, `ReadyToMerge` and `Reviewing`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.AssignmentStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.AssignmentNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `SatisfyGoal`

`controlplane.host.SatisfyGoal`.

It takes:

- `goal_id` — `Uuid`
- `satisfaction_receipt` — `String`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Goal` from `Running` to `Satisfied`, along the declared move `satisfy`. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.SatisfyGoalApplied`. It sets `satisfaction_receipt` from `input.satisfaction_receipt`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Goal` in `Cancelled`, `Paused` and `Satisfied`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `StartGoal`

`controlplane.host.StartGoal`.

It takes:

- `goal_id` — `Uuid`

It has three outcomes.

**`applied`** — The default branch, taken when no other outcome's condition matched. It moves a `controlplane.host.Goal` from `Paused` to `Running`, along the declared move `start`. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.StartGoalApplied`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`wrong-state`** — Taken when the subject is resting in a state none of this command's moves start from — a `controlplane.host.Goal` in `Cancelled`, `Running` and `Satisfied`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

### `UpdateGoal`

`controlplane.host.UpdateGoal`.

It takes:

- `goal_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`

It has four outcomes.

**`applied`** — Taken when the existing subject is in Paused or Running. It changes a `controlplane.host.Goal` without moving it along its lifecycle. The instance is the one named by the input field `goal_id`. It emits `controlplane.host.UpdateGoalApplied`. It sets `objective` from `input.objective`, `acceptance` from `input.acceptance`, `max_workers` from `input.max_workers`, `max_attempts` from `input.max_attempts`, `max_minutes` from `input.max_minutes`, `planner_model` from `input.planner_model`, `implementor_model` from `input.implementor_model`, `reviewer_model` from `input.reviewer_model`, `merge_authority` from `input.merge_authority`, `revision` from `its previous value plus 1`, `satisfaction_receipt` from `""` and `planning_fingerprint` from `""`. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`satisfied`** — Taken when the existing subject is in Satisfied. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`cancelled`** — Taken when the existing subject is in Cancelled. No entity in this specification changes. It reports `controlplane.host.GoalStateConflict`, carrying `state`. It emits nothing. A test establishes the declared subject state and constructs input selecting this branch in that state.

**`not-found`** — Taken when the identity the command names is one no record carries, before any other answer for it. No entity in this specification changes. It reports `controlplane.host.GoalNotFound`. It emits nothing. A test reaches it by sending an identity no record carries, arranging nothing.

## Events

### `ArchiveWorkspaceApplied`

`controlplane.host.ArchiveWorkspaceApplied`.

It carries:

- `workspace_id` — `Uuid`

Emitted by `controlplane.host.ArchiveWorkspace` on its `applied` outcome.

Nothing in this system reacts to it.

### `AssignmentCreated`

`controlplane.host.AssignmentCreated`.

It carries:

- `assignment_id` — `Uuid`
- `goal_id` — `Uuid`
- `repository_id` — `Uuid`
- `story_id` — `String`
- `case_id` — `String`
- `worktree_id` — `String`
- `candidate` — `String`
- `attempt` — `Integer`
- `reason` — `String`
- `implementor_run` — `String`
- `reviewer_run` — `String`
- `goal_revision` — `Integer`

Emitted by `controlplane.host.QueueAssignment` on its `created` outcome.

Nothing in this system reacts to it.

### `BlockAssignmentApplied`

`controlplane.host.BlockAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `reason` — `String`

Emitted by `controlplane.host.BlockAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `CancelAssignmentApplied`

`controlplane.host.CancelAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`

Emitted by `controlplane.host.CancelAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `CancelGoalApplied`

`controlplane.host.CancelGoalApplied`.

It carries:

- `goal_id` — `Uuid`

Emitted by `controlplane.host.CancelGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `ClaimAssignmentApplied`

`controlplane.host.ClaimAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `worktree_id` — `String`
- `implementor_run` — `String`
- `base_revision` — `String`

Emitted by `controlplane.host.ClaimAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `ClosePublicationApplied`

`controlplane.host.ClosePublicationApplied`.

It carries:

- `publication_id` — `Uuid`
- `reason` — `String`

Emitted by `controlplane.host.ClosePublication` on its `applied` outcome.

Nothing in this system reacts to it.

### `CompleteAssignmentApplied`

`controlplane.host.CompleteAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `merge_receipt` — `String`

Emitted by `controlplane.host.CompleteAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `ConfigureRepositoryApplied`

`controlplane.host.ConfigureRepositoryApplied`.

It carries:

- `repository_id` — `Uuid`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`

Emitted by `controlplane.host.ConfigureRepository` on its `applied` outcome.

Nothing in this system reacts to it.

### `ConfirmPublicationApplied`

`controlplane.host.ConfirmPublicationApplied`.

It carries:

- `publication_id` — `Uuid`
- `receipt` — `String`

Emitted by `controlplane.host.ConfirmPublication` on its `applied` outcome.

Nothing in this system reacts to it.

### `DisableRepositoryRegistrationApplied`

`controlplane.host.DisableRepositoryRegistrationApplied`.

It carries:

- `repository_id` — `Uuid`

Emitted by `controlplane.host.DisableRepositoryRegistration` on its `applied` outcome.

Nothing in this system reacts to it.

### `EnableRepositoryRegistrationApplied`

`controlplane.host.EnableRepositoryRegistrationApplied`.

It carries:

- `repository_id` — `Uuid`

Emitted by `controlplane.host.EnableRepositoryRegistration` on its `applied` outcome.

Nothing in this system reacts to it.

### `GoalCreated`

`controlplane.host.GoalCreated`.

It carries:

- `goal_id` — `Uuid`
- `workspace_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`
- `planning_revision` — `Integer`
- `planning_fingerprint` — `String`
- `planning_repository` — `String`
- `planning_worktree_id` — `String`
- `planning_worktree_path` — `String`
- `planning_reason` — `String`
- `planning_receipt` — `String`
- `planning_phase` — `controlplane.host.PlanningPhase`

Emitted by `controlplane.host.CreateGoal` on its `created` outcome.

Nothing in this system reacts to it.

### `GoalDeleted`

`controlplane.host.GoalDeleted`.

It carries:

- `goal_id` — `Uuid`

Emitted by `controlplane.host.DeleteGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `MarkPublicationUncertainApplied`

`controlplane.host.MarkPublicationUncertainApplied`.

It carries:

- `publication_id` — `Uuid`

Emitted by `controlplane.host.MarkPublicationUncertain` on its `applied` outcome.

Nothing in this system reacts to it.

### `MergeAssignmentApplied`

`controlplane.host.MergeAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`

Emitted by `controlplane.host.MergeAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `PauseGoalApplied`

`controlplane.host.PauseGoalApplied`.

It carries:

- `goal_id` — `Uuid`

Emitted by `controlplane.host.PauseGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `PlanningProgressRecorded`

`controlplane.host.PlanningProgressRecorded`.

It carries:

- `goal_id` — `Uuid`
- `planning_revision` — `Integer`
- `planning_fingerprint` — `String`
- `planning_repository` — `String`
- `planning_worktree_id` — `String`
- `planning_worktree_path` — `String`
- `planning_reason` — `String`
- `planning_receipt` — `String`
- `planning_phase` — `controlplane.host.PlanningPhase`

Emitted by `controlplane.host.RecordPlanningProgress` on its `applied` outcome.

Nothing in this system reacts to it.

### `PublicationIntentCreated`

`controlplane.host.PublicationIntentCreated`.

It carries:

- `assignment_id` — `Uuid`
- `candidate` — `String`
- `target` — `String`
- `expected_base` — `String`
- `publication_id` — `Uuid`

Emitted by `controlplane.host.PreparePublication` on its `created` outcome.

Nothing in this system reacts to it.

### `ReadyAssignmentApplied`

`controlplane.host.ReadyAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `reviewer_run` — `String`
- `review_revision` — `String`

Emitted by `controlplane.host.ReadyAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `ReconcileAssignmentApplied`

`controlplane.host.ReconcileAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `merge_receipt` — `String`

Emitted by `controlplane.host.ReconcileAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `RepairAssignmentApplied`

`controlplane.host.RepairAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `reason` — `String`
- `implementor_run` — `String`

Emitted by `controlplane.host.RepairAssignment` on its `rebased` and `applied` outcomes.

Nothing in this system reacts to it.

### `RepositoryRegistrationCreated`

`controlplane.host.RepositoryRegistrationCreated`.

It carries:

- `repository_id` — `Uuid`
- `workspace_id` — `Uuid`
- `name` — `String`
- `path` — `String`
- `common_dir` — `String`
- `base_branch` — `String`
- `test_command` — `String`
- `publish_command` — `String`

Emitted by `controlplane.host.RegisterRepository` on its `created` outcome.

Nothing in this system reacts to it.

### `ReviewAssignmentApplied`

`controlplane.host.ReviewAssignmentApplied`.

It carries:

- `assignment_id` — `Uuid`
- `candidate` — `String`
- `test_revision` — `String`

Emitted by `controlplane.host.ReviewAssignment` on its `applied` outcome.

Nothing in this system reacts to it.

### `SatisfyGoalApplied`

`controlplane.host.SatisfyGoalApplied`.

It carries:

- `goal_id` — `Uuid`
- `satisfaction_receipt` — `String`

Emitted by `controlplane.host.SatisfyGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `StartGoalApplied`

`controlplane.host.StartGoalApplied`.

It carries:

- `goal_id` — `Uuid`

Emitted by `controlplane.host.StartGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `UpdateGoalApplied`

`controlplane.host.UpdateGoalApplied`.

It carries:

- `goal_id` — `Uuid`
- `objective` — `String`
- `acceptance` — `String`
- `max_workers` — `Integer`
- `max_attempts` — `Integer`
- `max_minutes` — `Integer`
- `planner_model` — `String`
- `implementor_model` — `String`
- `reviewer_model` — `String`
- `merge_authority` — `Boolean`

Emitted by `controlplane.host.UpdateGoal` on its `applied` outcome.

Nothing in this system reacts to it.

### `WorkspaceCreated`

`controlplane.host.WorkspaceCreated`.

It carries:

- `workspace_id` — `Uuid`
- `path` — `String`
- `name` — `String`

Emitted by `controlplane.host.RegisterWorkspace` on its `created` outcome.

Nothing in this system reacts to it.

### `WorkspaceDirectoryCreated`

`controlplane.host.WorkspaceDirectoryCreated`.

It carries:

- `directory_id` — `Uuid`
- `workspace_id` — `Uuid`
- `path` — `String`
- `repository_common_dirs` — `List<String>`
- `managed_common_dirs` — `List<String>`

Emitted by `controlplane.host.AddWorkspaceDirectory` on its `created` outcome.

Nothing in this system reacts to it.

### `WorkspaceDirectoryRemoved`

`controlplane.host.WorkspaceDirectoryRemoved`.

It carries:

- `directory_id` — `Uuid`

Emitted by `controlplane.host.RemoveWorkspaceDirectory` on its `applied` outcome.

Nothing in this system reacts to it.

## Errors

### `AssignmentNotFound`

The requested identity is not held.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.BlockAssignment` on its `not-found` outcome.

Reported by `controlplane.host.CancelAssignment` on its `not-found` outcome.

Reported by `controlplane.host.ClaimAssignment` on its `not-found` outcome.

Reported by `controlplane.host.CompleteAssignment` on its `not-found` outcome.

Reported by `controlplane.host.MergeAssignment` on its `not-found` outcome.

Reported by `controlplane.host.ReadyAssignment` on its `not-found` outcome.

Reported by `controlplane.host.ReconcileAssignment` on its `not-found` outcome.

Reported by `controlplane.host.RepairAssignment` on its `not-found` outcome.

Reported by `controlplane.host.ReviewAssignment` on its `not-found` outcome.

### `AssignmentStateConflict`

The command cannot act in the current state.

It carries:

- `state` — `controlplane.host.Assignment.State`

Reported by `controlplane.host.BlockAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.CancelAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.ClaimAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.CompleteAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.MergeAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.ReadyAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.ReconcileAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.RepairAssignment` on its `wrong-state` outcome.

Reported by `controlplane.host.ReviewAssignment` on its `wrong-state` outcome.

### `EvidenceMissing`

A required run, worktree, revision or receipt is empty.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ClaimAssignment` on its `evidence-missing` outcome.

Reported by `controlplane.host.CompleteAssignment` on its `receipt-missing` outcome.

Reported by `controlplane.host.ReconcileAssignment` on its `receipt-missing` outcome.

Reported by `controlplane.host.RepairAssignment` on its `evidence-missing` and `base-missing` outcomes.

### `EvidenceNotCurrent`

Tests or review do not cover the assignment's current candidate.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ReadyAssignment` on its `evidence-not-current` outcome.

Reported by `controlplane.host.ReviewAssignment` on its `tests-not-current` outcome.

### `GoalLimitInvalid`

Worker, attempt and minute limits are positive counts.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.CreateGoal` on its `workers-invalid`, `attempts-invalid` and `minutes-invalid` outcomes.

### `GoalNotCurrent`

The goal is not running, or changed since the assignment was planned.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.QueueAssignment` on its `goal-not-current` outcome.

### `GoalNotFound`

The requested identity is not held.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.CancelGoal` on its `not-found` outcome.

Reported by `controlplane.host.DeleteGoal` on its `not-found` outcome.

Reported by `controlplane.host.PauseGoal` on its `not-found` outcome.

Reported by `controlplane.host.QueueAssignment` on its `goal-not-found` outcome.

Reported by `controlplane.host.RecordPlanningProgress` on its `not-found` outcome.

Reported by `controlplane.host.SatisfyGoal` on its `not-found` outcome.

Reported by `controlplane.host.StartGoal` on its `not-found` outcome.

Reported by `controlplane.host.UpdateGoal` on its `not-found` outcome.

### `GoalStateConflict`

The command cannot act in the current state.

It carries:

- `state` — `controlplane.host.Goal.State`

Reported by `controlplane.host.CancelGoal` on its `wrong-state` outcome.

Reported by `controlplane.host.DeleteGoal` on its `paused`, `running` and `satisfied` outcomes.

Reported by `controlplane.host.PauseGoal` on its `wrong-state` outcome.

Reported by `controlplane.host.SatisfyGoal` on its `wrong-state` outcome.

Reported by `controlplane.host.StartGoal` on its `wrong-state` outcome.

Reported by `controlplane.host.UpdateGoal` on its `satisfied` and `cancelled` outcomes.

### `PublicationIntentNotFound`

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ClosePublication` on its `not-found` outcome.

Reported by `controlplane.host.ConfirmPublication` on its `not-found` outcome.

Reported by `controlplane.host.MarkPublicationUncertain` on its `not-found` outcome.

### `PublicationIntentStateConflict`

It carries:

- `state` — `controlplane.host.PublicationIntent.State`

Reported by `controlplane.host.ClosePublication` on its `wrong-state` outcome.

Reported by `controlplane.host.ConfirmPublication` on its `wrong-state` outcome.

Reported by `controlplane.host.MarkPublicationUncertain` on its `wrong-state` outcome.

### `RepositoryRegistrationNotFound`

The requested identity is not held.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ConfigureRepository` on its `not-found` outcome.

Reported by `controlplane.host.DisableRepositoryRegistration` on its `not-found` outcome.

Reported by `controlplane.host.EnableRepositoryRegistration` on its `not-found` outcome.

### `RepositoryRegistrationStateConflict`

The command cannot act in the current state.

It carries:

- `state` — `controlplane.host.RepositoryRegistration.State`

Reported by `controlplane.host.DisableRepositoryRegistration` on its `wrong-state` outcome.

Reported by `controlplane.host.EnableRepositoryRegistration` on its `wrong-state` outcome.

### `ReviewNotIndependent`

Review must use an execution context other than the implementor's.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ReadyAssignment` on its `reviewer-missing` and `review-not-independent` outcomes.

### `WorkspaceDirectoryNotFound`

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.RemoveWorkspaceDirectory` on its `not-found` outcome.

### `WorkspaceDirectoryStateConflict`

It carries:

- `state` — `controlplane.host.WorkspaceDirectory.State`

Reported by `controlplane.host.RemoveWorkspaceDirectory` on its `wrong-state` outcome.

### `WorkspaceNotFound`

The requested identity is not held.

It carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.

Reported by `controlplane.host.ArchiveWorkspace` on its `not-found` outcome.

### `WorkspaceStateConflict`

The command cannot act in the current state.

It carries:

- `state` — `controlplane.host.Workspace.State`

Reported by `controlplane.host.ArchiveWorkspace` on its `wrong-state` outcome.

## Actors

An actor is who may ask this context for something. Every grant below points at a command this specification declares — a grant is a resolved reference, so "may invoke" something nobody wrote is not a permission this model can express, and an authorisation that authorises nothing cannot ship quietly.

### `Operator`

`controlplane.host.Operator`.

It may invoke [`AddWorkspaceDirectory`](#addworkspacedirectory), [`ArchiveWorkspace`](#archiveworkspace), [`CancelGoal`](#cancelgoal), [`ConfigureRepository`](#configurerepository), [`CreateGoal`](#creategoal), [`DeleteGoal`](#deletegoal), [`DisableRepositoryRegistration`](#disablerepositoryregistration), [`EnableRepositoryRegistration`](#enablerepositoryregistration), [`PauseGoal`](#pausegoal), [`RegisterRepository`](#registerrepository), [`RegisterWorkspace`](#registerworkspace), [`RemoveWorkspaceDirectory`](#removeworkspacedirectory), [`StartGoal`](#startgoal) and [`UpdateGoal`](#updategoal).

### `Supervisor`

`controlplane.host.Supervisor`.

It may invoke [`BlockAssignment`](#blockassignment), [`CancelAssignment`](#cancelassignment), [`ClaimAssignment`](#claimassignment), [`ClosePublication`](#closepublication), [`CompleteAssignment`](#completeassignment), [`ConfirmPublication`](#confirmpublication), [`MarkPublicationUncertain`](#markpublicationuncertain), [`MergeAssignment`](#mergeassignment), [`PreparePublication`](#preparepublication), [`QueueAssignment`](#queueassignment), [`ReadyAssignment`](#readyassignment), [`ReconcileAssignment`](#reconcileassignment), [`RecordPlanningProgress`](#recordplanningprogress), [`RepairAssignment`](#repairassignment), [`ReviewAssignment`](#reviewassignment) and [`SatisfyGoal`](#satisfygoal).


---

Generated from controlplane v1 · model digest `c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548` · contract digest `slice-sha256/2:e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10`. Do not edit this file; change the specification and regenerate it with `ess generate`.
