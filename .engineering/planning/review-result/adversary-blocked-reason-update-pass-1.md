---
format: aep.planning-md/3
id: review-result:adversary-blocked-reason-update-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:blocked-reason-update on 8ab9839 plus working tree'
relations:
- reviews: story:blocked-reason-update
revision: 1
---
unit: story:blocked-reason-update, worktree cp-wave5-blocked-reason-update, uncommitted on base 8ab9839
verdict: NEEDS-CHANGE
cases: executed 300→302, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: the harness log of its background gate call; its worktree lease record (released)
needs-coordinator: the fix for finding 1 belongs in crates/control-plane-runtime/src/supervisor.rs (the planner fingerprint), which the unit brief does not assign

## Cases (crates/control-plane-runtime/tests/blocked_reason_update_attack.rs, untracked in the unit tree, both red)

- `replaced_blocked_reason_starts_no_planner_run`: after the planner settled, one changed blocker of a Blocked assignment started 1 planner run; the assignment row differed only in its reason.
- `alternating_blocker_starts_no_planner_run_per_cycle`: 6 service cycles with a remote failing every other tick, inside the 600 s grace, ran the planner model 6 more times and the critic 6 more times, committed 126 decisions and left 6 new planner worktrees.

## Gate after the cases

validate 0, generated-check 0, spec-history-check 0 (2 changes since 6341398, 2 could break replay, 1 acknowledged), fmt 0, clippy 0, test 101 (only the attack binary: 0 passed, 2 failed; core 65, runtime 121, xtask 114 passed), conformance 0 (178 scenarios passed).

## Findings

1. crates/control-plane-runtime/src/fleet.rs:265, NEEDS-CHANGE, blocker, introduced. Each reason replacement changes the planner fingerprint (supervisor.rs:752 and 760 digest whole AssignmentList rows, reason included), so every changed cause starts a planner and critic run and a new planner worktree. The planner never reads assignments (engine.rs:39-46). Reached by Supervisor::run every 15 s poll through the publication close, a flapping remote within the grace window, and a deliver path alternating fetch failure and moved target. Fix: drop reason from the rows the fingerprint digests; each Running goal re-plans once after deploy.
2. crates/control-plane-runtime/src/fleet.rs:264, CONFIRMED by reading, warning, introduced. progress commits before BlockAssignment, and ClosePublication (:2288) before block (:2291); a crash between the two commits leaves a stale reason that the same_activity early return never replaces. Fix: send BlockAssignment before progress, and block before ClosePublication.
3. crates/control-plane-runtime/tests/fleet.rs:1743, CONFIRMED by reading, note, introduced. The ten-tick half of closed_publication_replaces_the_blocked_reason reaches no block_with call (goal paused, intent NotPublished), and no added case reaches the equal-reason clause at fleet.rs:265.

## Held

The replay claim (no stored BlockAssignment on a Blocked assignment; recorded-history.db holds 4, the one refusal on a Merged assignment), Operator access refused before append, a failing replacement cannot abort a tick, observation errors decided by failure class, console and attention strip read assignment.reason, the long_unchanged_blocker_appends_once change strengthens the test, and spec-history-check folds the second change into the acknowledged one.

```findings
[
{"file":"crates/control-plane-runtime/src/fleet.rs","line":265,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"each Blocked-to-Blocked reason replacement changes the planner fingerprint (supervisor.rs:760 digests assignment reason), so every changed cause starts a planner and critic run plus a new planner worktree, once per service cycle for an alternating cause"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":264,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"progress commits before BlockAssignment (and ClosePublication before block at :2288-2291), so a crash between the two commits leaves a stale reason that the same_activity early return never replaces"},
{"file":"crates/control-plane-runtime/tests/fleet.rs","line":1743,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the ten-tick half of closed_publication_replaces_the_blocked_reason reaches no block_with call (paused goal, NotPublished intent), and no added case exercises the equal-reason clause at fleet.rs:265"}
]
```
