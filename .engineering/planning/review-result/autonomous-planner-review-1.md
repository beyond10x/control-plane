---
format: aep.planning-md/3
id: review-result:autonomous-planner-review-1
kind: review-result
status: active
title: Independent planner isolation and revision review
relations:
- reviews: story:autonomous-planner
revision: 1
---
needs-revision

unit: autonomous-planner, bab2aa67738df161cbf920e4235669cac7591560
verdict: CONFIRMED
cases: executed 9→14, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route five failing regressions to implementation

```text
crates/control-plane-runtime/tests/planner.rs | 178 +++++++++++++++++++++++++++++
1 file changed, 178 insertions(+)
```

Tests committed as `5806e6e19294a0a548351d195722c931009da03a`; author and committer verified as the bot. The existing coordinator-owned `Cargo.lock` changes were untouched. Lease released and tree handed to `protocol_scope`.

Each new case was written before execution and run alone before the suite. All five returned exit 101:

| Case | Observed failure |
|---|---|
| `updated_goal_replans_past_stale_queued_assignments` | After `UpdateGoal`, tick reports `planned: 0, queued: 0, blockers: []`; only revision-1 work remains queued. |
| `unavailable_registered_repository_is_a_durable_blocker_not_a_dead_supervisor` | Tick returns `Err(start git … No such file or directory)`, without recording a blocker. |
| `linked_aep_store_cannot_be_mutated_outside_planning_checkout` | Actual AEP story creation writes through the tracked `.engineering` symlink into the external fixture store. Subsequent critique rejection does not undo it. |
| `another_workspaces_assignment_does_not_rewake_unchanged_idle_goal` | Model calls increase from two to four after an unrelated paused workspace acquires an assignment. |
| `planning_commit_is_retained_on_a_named_branch` | The newly committed planning checkout remains detached: `fatal: ref HEAD is not a symbolic ref`. |

First-run output is retained verbatim in `target/adversary-{goal-revision,unavailable-repository,linked-store,workspace-isolation,branch}.log`.

Then executed:

```console
CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p control-plane-runtime
```

```text
running 14 tests
test process_exit_timeout_and_cancellation_are_not_successful_observations ... ok
test unavailable_registered_repository_is_a_durable_blocker_not_a_dead_supervisor ... FAILED
test model_cannot_override_aep_store ... ok
test commit_refusal_is_a_blocker_and_never_queues ... ok
test model_cannot_write_workflows_when_ess_is_at_root ... ok
test rejected_critique_is_durable_and_idle_after_restart ... ok
test linked_aep_store_cannot_be_mutated_outside_planning_checkout ... FAILED
test existing_backlog_is_not_duplicated ... ok
test goal_completion_requires_evidence ... ok
test updated_goal_replans_past_stale_queued_assignments ... FAILED
test goal_drives_plan ... ok
test planning_commit_is_retained_on_a_named_branch ... FAILED
test pause_during_model_call_prevents_further_effects ... ok
test another_workspaces_assignment_does_not_rewake_unchanged_idle_goal ... FAILED

test result: FAILED. 9 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.32s
```

Exit 101; full output retained in `target/adversary-planner-suite.log`.

The findings cover reachable public workflows:

- `supervisor.rs:88`: ordinary operator goal revision after planning. Stale queued work suppresses revalidation and current-revision assignment creation.
- `supervisor.rs:70`: a previously registered repository becomes unavailable. `run()` propagates this tick error and stops processing subsequent work.
- `engine.rs:294`: a registered Git repository contains a tracked `.engineering` symlink. An admitted model action escapes the isolated planning checkout.
- `supervisor.rs:422`: another workspace’s assignment changes. Unfiltered assignments invalidate an otherwise unchanged goal’s fingerprint.
- `supervisor.rs:462`: planning creates and commits new artifacts. No branch is created before committing, contrary to the managed-worktree workflow. The commit remains referenced by the retained tree and assignment; this test does not claim it was lost.

The original nine tests remain green, including pause/resume, critique rejection, commit refusal, root-specification write restrictions, AEP argument restrictions, and process cancellation.

No implementation or AEP files were edited. Test fixtures and logs remain inside the assigned tree.

```findings
- file: crates/control-plane-runtime/src/supervisor.rs
  line: 88
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Stale queued assignments suppress replanning after an operator advances the goal revision.
- file: crates/control-plane-runtime/src/supervisor.rs
  line: 70
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: An unavailable registered repository propagates a fingerprint error that terminates the supervisor loop instead of recording a scoped blocker.
- file: crates/control-plane-runtime/src/engine.rs
  line: 294
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: A tracked engineering-store symlink permits model-directed AEP mutations outside the isolated planning checkout.
- file: crates/control-plane-runtime/src/supervisor.rs
  line: 422
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Fingerprinting assignments from unrelated workspaces causes additional model calls for an unchanged idle goal.
- file: crates/control-plane-runtime/src/supervisor.rs
  line: 462
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Planning commits are created on detached HEAD without the branch required by the managed-worktree workflow.
```
