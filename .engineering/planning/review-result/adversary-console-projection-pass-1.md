---
format: aep.planning-md/3
id: review-result:adversary-console-projection-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:console-projection at 7fe631e'
relations:
- reviews: story:console-projection
revision: 1
---
unit: story:console-projection, control-plane worktree `cp-wave3-console-projection` at 7fe631e (base 9691f41), plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 42→44, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths left (one scratch log directory holding 3 logs), plus the assigned build dir; all listed in part 6
needs-coordinator: the fix for finding 1 is in control-plane-core (`memory.rs` Journal), outside this unit's files; finding 2 needs a shape decision: one `waiting` per goal, or one per worker (`max_workers` can be 2)

### 1. `git --no-pager diff --stat`

Empty: the only change is an untracked file. `git status --short`:
```
?? crates/control-plane-app/tests/console_projection_attack.rs
```
That one path is a test file, so no non-test path was touched. After the red run I reformatted that file with rustfmt; it moved one assertion from line 290 to 296.

### 2. Cases added (`crates/control-plane-app/tests/console_projection_attack.rs`)

| Case | What it asserts | Now |
|---|---|---|
| `planner_activity_survives_a_minute_of_worker_streaming` | Setup: a planner entry, then 64 worker `loom.event` entries recorded through `Supervisor::record_progress`. Asserts `planner_activity.id` is still the planner entry's id. | red |
| `final_goal_review_wait_is_projected` | Setup: a Running goal with a Merged assignment, then `goal.review` (role `goal_reviewer`) and a `loom.event` through the fleet path. Asserts `waiting == {role: goal_reviewer, model: reviewer-model, since: <at of goal.review>}`. | red |

This was the first run of these cases, each case alone, before any suite run (`cargo test --locked -p control-plane-app --test console_projection_attack`). Output verbatim:
```
---- final_goal_review_wait_is_projected stdout ----
thread 'final_goal_review_wait_is_projected' (1752351) panicked at crates/control-plane-app/tests/console_projection_attack.rs:290:5:
assertion `left == right` failed: while the goal reviewer's model call is open
  left: Null
 right: Object {"model": String("reviewer-model"), "role": String("goal_reviewer"), "since": String("2026-10-06T09:45:57Z")}

---- planner_activity_survives_a_minute_of_worker_streaming stdout ----
thread 'planner_activity_survives_a_minute_of_worker_streaming' (1752352) panicked at crates/control-plane-app/tests/console_projection_attack.rs:197:5:
assertion `left == right` failed: planner_activity after 64 worker entries: null
  left: Null
 right: String("75db3805-6dc0-4440-b64c-61b6225b4842")

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

### 3. Suite run (after the cases existed)

The plain `cargo test --locked -p control-plane-app` stopped at the first red binary, so `operator_boundary.rs` never ran. I reran it with `--no-fail-fast`:
```
Running unittests src/lib.rs        test result: ok. 39 passed; 0 failed; ...
Running unittests src/main.rs       test result: ok. 0 passed; 0 failed; ...
Running tests/console_projection_attack.rs
test final_goal_review_wait_is_projected ... FAILED
test planner_activity_survives_a_minute_of_worker_streaming ... FAILED
                                    test result: FAILED. 0 passed; 2 failed; ...
Running tests/operator_boundary.rs  test result: ok. 3 passed; 0 failed; ...
(doc-tests)                         test result: ok. 0 passed; 0 failed; ...
error: 1 target failed:
EXIT=101
```
- **Case counts:** 44 executed in this run. The 42 before my additions are the other binaries in the same run; my binary is the one excluded.
- **Clippy:** `cargo clippy --locked -p control-plane-app --all-targets -- -D warnings` finished with no warnings.
- **Formatting:** `rustfmt --edition 2024 --check` passes on the new file.

### 4. Findings (cover 7fe631e plus the untracked test file)

**F1: `planner_activity` goes null after about a minute of worker activity.** warning · INFEASIBLE · introduced
- **What was measured:** `live.rs:116-127` derive the field only from the 64 retained entries. `console_projection_attack.rs:197` is red: `left: Null`.
- **What reaches it:** almost every goal that has workers.
  - A worker records one `loom.event` per second of streaming (`loom_model.rs:372`, `fleet.rs:70`), and the journal keeps 64 entries (`memory.rs:47`).
  - The planner records nothing while any of the goal's assignments is active (`supervisor.rs:120-127`).
  - So for the rest of the implementation phase, the consumer checks `planner_status_uses_planner_activity` and the goal cards' planner card get null.
- **Why it can't be fixed here:** live.rs cannot fix this. The journal keeps a newest entry per assignment under `fleet`, but has no slot for the planner. The fix is a journal slot for the newest entry without an `assignment_id` (`memory.rs` `Journal::record`/`history`), which is outside this unit's files.
- **Origin:** introduced. The field does not exist at the base (`git show 9691f41:…/live.rs`, 0 matches).

**F2: no `waiting` during the final goal review, or any other worker model call.** warning · NEEDS-CHANGE · introduced
- **What was measured:** `waiting` at `live.rs:237` reads only entries without an assignment. `console_projection_attack.rs:296` is red: `left: Null`.
- **What reaches it:** the final goal review.
  - `fleet.rs:2064-2071` records `goal.review` and then waits on `reviewer_model`. Only afterwards does it record `goal.acceptance.completed` or `blocked`.
  - Meanwhile the goal is Running, all assignments are Merged and no planner call is open, so the consumer header can only say "idle".
- **Why the unit's reason doesn't hold:** the doc at `live.rs:235` says "no completion" is recorded. But the next non-`loom.event` entry of the same assignment ends every worker call, and `fleet{}` keeps that entry even after it leaves the 64-entry window. So this is fixable in live.rs.
- **Contract mismatch with a consumer:** story:console-status-and-attention's `working_is_shown_with_its_activity` ("an assignment Implementing and no wait") assumes a wait can exist while an assignment is Implementing. This projection never produces one, because implementor `model.request` and reviewer `review.request` are not derived either.

**F3: no case exercises the revision guard in `waiting`.** note · CONFIRMED · introduced
- **What was measured:** the guard is `live.rs:260`. `grep -c UpdateGoal live.rs` = 0, so no case changes the revision while a request is open.
- **Not shown by running:** deleting the guard leaving the suite green is my inference. I did not build a mutated copy, to stay within the near-full /dev/shm quota.
- **What reaches it:** saving the goal form (`UpdateGoal`) during a planner call.

### 5. Attacked and could not break (I read the code for these; I ran nothing beyond the suite)

- **Privacy:**
  - No new field name contains `receipt`.
  - `merged_at` reads only `observed_at`.
  - The `merge.completed` activity's `{"receipt":…}` detail is projected as an empty detail.
- **`merged_at` shape:** matches `observe_merge` (`fleet.rs:1874`), and the reconcile path uses the same function.
- **Planner and critic waits:**
  - The role strings match `engine.rs:606` and `engine.rs:751`.
  - Only `loom.event` entries are recorded between request and completion (`engine.rs:221-260`).
- **Ids:**
  - Every current progress path assigns a uuid (`supervisor.rs:648`, `fleet.rs:240`); the hash ids only matter for old entries.
  - The `.rev()` in `observe` is pinned: removing it fails the unit's stable-ids assertion on `activity[1]`.
- **`server_time`:** it is added after the comparison with the previous frame, and the existing idle-stream test is still green.
- **Frontend:**
  - `Activity.vue` renders only named fields.
  - `App.vue:16` overrides `id` with its own key, so the new field changes nothing on screen.
- **Not raised:** a stale `waiting` after Pause during a planner call followed by Start. Neither command changes the revision (`host.yaml:644-680`). It lasts only until the next re-plan records an entry.

### 6. Paths written outside the worktree

- Left in place:
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-1/` (directory)
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-1/attack-alone.log`
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-1/suite.log`
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-1/suite-nff.log`
- Build output: `/dev/shm/cp-wave3-console-projection-target`, the existing build dir. It now also holds the new test binary `console_projection_attack-902e8f1345f8a640` and clippy metadata.
- Test temp dirs went under `/dev/shm/cp-wave3-console-projection-scratch/tmp` and were removed automatically; `tmp/` is empty.
- The suite left an empty `/dev/shm/cp-wave3-console-projection-scratch/dotscratch/app-runtime-review` (11:47:06). I removed it with `rmdir`; `dotscratch/` is empty.
- Worktree lease: session `cp-wave3-console-projection-adversary-1-1791279803`, acquired and released with `worktree hook`.

### 7. Findings block

```findings
- file: crates/control-plane-app/src/live.rs
  line: 116
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: planner_activity is taken only from the 64 retained entries, so about a minute of worker streaming makes it null for the rest of implementation; keeping it needs a journal slot for the newest planner entry in control-plane-core.
- file: crates/control-plane-app/src/live.rs
  line: 237
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: waiting is derived for planner and critic calls only, so the final goal review's reviewer-model call (and every worker call) projects null although the next entry of the same assignment marks its end.
- file: crates/control-plane-app/src/live.rs
  line: 260
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no case changes the goal revision while a planner request is open, so the revision guard in waiting is untested.
```
