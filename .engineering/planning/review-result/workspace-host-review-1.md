---
format: aep.planning-md/3
id: review-result:workspace-host-review-1
kind: review-result
status: active
title: Durable host adversary pass 1
relations:
- reviews: story:workspace-host
revision: 1
---
needs-revision

Independent adversary pass 1 on workspace-host commit 11bb206; tests-only commit eb1d870. All four failures use real public Store/discover operations. The complete suite reports: test result: FAILED. 11 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out. Exit 101. Each added test first compiled and failed individually. This record retains the findings verbatim; machine-local build and cache paths from the worker report are omitted from public source.

The new cases are archived_workspace_cannot_restart_its_paused_goal, configured_target_change_invalidates_prepared_publication, blocked_assignment_does_not_consume_worker_for_another_repository, and detached_head_remains_a_discoverable_repository. Parent routed all four to the original implementor; assertions must remain intact.

```findings
- file: crates/control-plane-core/src/guards.rs
  line: 98
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: StartGoal restarts a paused goal after its workspace has been archived.
- file: crates/control-plane-core/src/guards.rs
  line: 317
  category: integrity
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: MergeAssignment admits a prepared publication after ConfigureRepository changes its target branch.
- file: crates/control-plane-core/src/guards.rs
  line: 248
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Blocked assignments consume worker capacity and prevent ready work in unrelated repositories from starting.
- file: crates/control-plane-core/src/discovery.rs
  line: 51
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Discovery silently treats a detached Git checkout as a nongit directory and returns no repository.
```
