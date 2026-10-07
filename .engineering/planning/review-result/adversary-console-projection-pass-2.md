---
format: aep.planning-md/3
id: review-result:adversary-console-projection-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:console-projection at 3971b76'
relations:
- reviews: story:console-projection
revision: 1
---
unit: story:console-projection, worktree `cp-wave3-console-projection` at 3971b76 (base 9691f41), plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 47→50, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (one log directory holding 3 logs), plus new artefacts in the assigned build dir; all listed in part 6
needs-coordinator: the fix for finding 1 is a journal slot in control-plane-core `memory.rs`, the file this correction already changed. Finding 3 needs a decision: fix it now (that needs a record from the runtime or core), or accept it as the limit the implementor already named.

### 1. `git --no-pager diff --stat`

Empty. The only change is one untracked file, and it is a test file. No non-test path was touched. `git status --short`:
```
?? crates/control-plane-app/tests/console_projection_pass2_attack.rs
```

### 2. Cases added (`crates/control-plane-app/tests/console_projection_pass2_attack.rs`)

| Case | What it asserts | Now |
|---|---|---|
| `worker_wait_keeps_its_start_while_two_workers_stream` | Two Implementing assignments on one goal, each in its own repository. A records `model.request`, then B does, then each records 32 `loom.event` entries through `Supervisor::record_progress`. A's `waiting` must still be `{implementor, implementor-model, since: <at of A's request>}`. | red |
| `no_model_wait_once_the_runtime_has_stopped` | One Implementing assignment with an open `model.request`. Then `serve_with_runtime` runs until `runtime_error` is set: a second goal's receipt is `"not-json"`, and the supervisor's `record` refuses it. After that, the assignment's `waiting` must be null. | red |
| `a_wait_recorded_before_a_restart_is_not_open_after_it` | An open `model.request`, then the Store is dropped and reopened. The assignment's `waiting` must be null. | red |

This is the first run of these cases, alone and before any suite run (`cargo test --locked -p control-plane-app --test console_projection_pass2_attack`). Output verbatim:
```
---- a_wait_recorded_before_a_restart_is_not_open_after_it stdout ----

thread 'a_wait_recorded_before_a_restart_is_not_open_after_it' (2503948) panicked at crates/control-plane-app/tests/console_projection_pass2_attack.rs:359:5:
assertion `left == right` failed
  left: Object {"model": String("implementor-model"), "role": String("implementor"), "since": String("2026-10-06T10:17:26Z")}
 right: Null

---- no_model_wait_once_the_runtime_has_stopped stdout ----
Autonomous processing stopped: invalid planning receipt: expected ident at line 1 column 2. Restart the service after resolving this problem.

thread 'no_model_wait_once_the_runtime_has_stopped' (2503949) panicked at crates/control-plane-app/tests/console_projection_pass2_attack.rs:319:5:
assertion `left == right` failed: runtime_error: "Autonomous processing stopped: invalid planning receipt: expected ident at line 1 column 2. Restart the service after resolving this problem."
  left: Object {"model": String("implementor-model"), "role": String("implementor"), "since": String("2026-10-06T10:17:26Z")}
 right: Null

---- worker_wait_keeps_its_start_while_two_workers_stream stdout ----

thread 'worker_wait_keeps_its_start_while_two_workers_stream' (2503950) panicked at crates/control-plane-app/tests/console_projection_pass2_attack.rs:238:5:
assertion `left == right` failed: worker A after 32 streamed events per worker
  left: Object {"model": String("implementor-model"), "role": String("implementor"), "since": Null}
 right: Object {"model": String("implementor-model"), "role": String("implementor"), "since": String("2026-10-06T10:17:26Z")}

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
EXIT=101
```

### 3. Suite run (after the cases existed)

Command: `cargo test --locked -p control-plane-app --no-fail-fast`. Output, filtered to the summary lines:
```
Running unittests src/lib.rs                   test result: ok. 42 passed; 0 failed
Running unittests src/main.rs                  test result: ok. 0 passed; 0 failed
Running tests/console_projection_attack.rs     test result: ok. 2 passed; 0 failed
Running tests/console_projection_pass2_attack.rs
test a_wait_recorded_before_a_restart_is_not_open_after_it ... FAILED
test no_model_wait_once_the_runtime_has_stopped ... FAILED
test worker_wait_keeps_its_start_while_two_workers_stream ... FAILED
                                               test result: FAILED. 0 passed; 3 failed
Running tests/operator_boundary.rs             test result: ok. 3 passed; 0 failed
(doc-tests)                                    test result: ok. 0 passed; 0 failed
error: 1 target failed:
EXIT=101
```
- **Case counts:** 50 cases ran. The before-count of 47 is every other binary in this same run, with my file excluded. It matches the implementor's own log `r1-test-app.log` (42+0+2+3+0).
- **Formatting:** `rustfmt --edition 2024 --check` passes on the new file.
- **Clippy:** `cargo clippy --locked -p control-plane-app --all-targets -- -D warnings` exits 0.

### 4. Findings (they cover 3971b76 plus the untracked test file)

**F1: a worker's wait loses its start time after about 30 seconds of concurrent streaming.** warning · NEEDS-CHANGE · introduced
- **What was measured:** `console_projection_pass2_attack.rs:238` is red with `since: Null`. The cause is the `Call::Streaming` branch at `live.rs:365` (and `:337` for the planner).
- **What reaches it:**
  - The fleet runs up to `max_workers` assignments of one goal in parallel (`fleet.rs:436`).
  - Each worker records about one `loom.event` per second while its model streams (`loom_model.rs:372`).
  - All of a goal's workers share one 64-entry history (`memory.rs:47`). With 2 workers, a request leaves that history after about 32 s; with 1 worker, after about 64 s.
  - The `fleet` slot keeps only the newest entry, which by then is a `loom.event`.
- **Acceptance:** `model_wait_is_projected` says the wait carries "role, model and start time" until completion. The start time is lost exactly on the long waits where elapsed time matters. The unit's own test pins `since: null` as intended (`live.rs` `assignment_model_waits_are_projected`, `model_wait_outlasts_…`).
- **Fix:** add a journal slot per lane (each assignment, plus the planner) for the newest entry that is not a `loom.event`, the same way the correction added `planner_activity`. Then `since` is always derivable.

**F2: `waiting` stays after the runtime stops, until the service is restarted.** warning · NEEDS-CHANGE · introduced
- **What was measured:** `console_projection_pass2_attack.rs:319` is red. The projection carries `runtime_error` "Autonomous processing stopped…" and also `waiting: implementor`. `compact` (`live.rs:141-142`) never reads `view["runtime_error"]`, although every snapshot carries it.
- **What reaches it:** a store write that fails during a model call.
  - When an append fails, the in-memory state is left unchanged (`lib.rs:331`).
  - In the fleet, the worker's progress write fails, then its `worker.block(..)?` fails (`fleet.rs:442`), and the error propagates through `fleet.rs:450` → `supervisor.rs:46` until `run` returns.
  - In the planner, the same happens through `supervisor.rs:190`.
  - `serve_with_runtime` keeps the console up "for inspection", with the last open request still recorded.
  - My test reaches that same state through a different fatal error (a non-JSON receipt). I constructed that trigger; the fleet and planner paths above are what real users would hit.
- **Fix:** in `compact`, derive no waits while `runtime_error` is a string.
- **Caveat:** after `fleet_tick` returns early, its worker threads (started with `spawn_blocking`) keep running. A sibling worker's call can therefore still be genuinely open for a while.

**F3: a wait recorded before a restart is projected as open after it.** note · CONFIRMED · introduced
- **What was measured:** `console_projection_pass2_attack.rs:359` is red.
- **What reaches it:** a crash or kill during any model call. The implementor named this limit.
- **How long it lasts:** it ends at the first fleet tick after the restart, either through `retire_stale` (`fleet.rs:334`) or through the recovery `block` in `deliver`. That tick runs only after the planning tick over every Running goal (`supervisor.rs:46`), and that planning tick can include planner model calls for other goals.
- **Fix:** a `since` vs process-start comparison would break the unit's hard-coded test timestamps. A real fix needs a record at runtime start, or a version-based guard.

### 5. Attacked, could not break

- **Journal `record`/`seed`/`history`:**
  - The `assignment_id` rule is the same in `memory.rs` and `live.rs`.
  - Only `CreateGoal` and `RecordPlanningProgress` write `planning_receipt`, and every runtime writer stores a JSON object, so the journal is never reseeded empty.
  - Replay runs the same `advance_journal` path.
- **Revision guard:**
  - `UpdateGoal` changes only the goal (`host.yaml:1044-1065`).
  - So `model_wait_ends_when_the_goal_revision_changes` really pins the guard, for both the planner and the assignment. The implementor's mutant log `r1-f3-mutant.log` is red.
- **Roles and models:** the five roles match all model request sites. `goal.review` is recorded on `current[0]`, which is filtered to the goal's revision (`fleet.rs:1949`).
- **Implementor's limit 1 (governance entries during a call):** I found no path that reaches it (read only, not run). The implementor call runs synchronously inside `select` (`fleet.rs:1172-1173`), the governor hooks (`governance.rs:111`, `:158`) fire around selection and effects, and the reviewer and goal-review calls run outside any governor.
- **Implementor's limit 2 (read-only steps):** this holds, but the window is short. Every call ends with a "Provider completed this turn" `loom.event` (`loom_model.rs:391`). The wait then lasts through a Read (32 files or fewer) until the next `model.request`, or after Finish through `commit()` until `checks.run` (`fleet.rs:871`). That is milliseconds to seconds.
- **Privacy and ids:**
  - The new slot is projected through `observation()` only.
  - The `planner_activity` id is identical to the listed entry's id while that entry is retained.
  - For legacy id-less entries, renumbering stays within the exception the doc already states.

### 6. Paths written outside the worktree

- `/dev/shm/cp-wave3-console-projection-scratch/adversary-2/` (directory), containing:
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-2/attack-alone.log`
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-2/suite-nff.log`
  - `/dev/shm/cp-wave3-console-projection-scratch/adversary-2/clippy.log`
- **Build output:** the assigned dir `/dev/shm/cp-wave3-console-projection-target` gained `debug/deps/console_projection_pass2_attack-56b4623980a89a1f` plus clippy metadata (`…-08e12e444f1c38cc`). The dir is now 1.7G.
- **Scratch cleanup:** the suite again created an empty `dotscratch/app-runtime-review`, which I removed with `rmdir`. `tmp/` and `dotscratch/` are empty.
- **Lease:** session `cp-wave3-console-projection-adversary-2-1791280875`, acquired and released with `worktree hook`.

### 7. Findings block

```findings
- file: crates/control-plane-app/src/live.rs
  line: 365
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a worker's waiting.since becomes null once its request leaves the goal's 64 shared entries, about 32 s with two workers streaming, because the journal keeps no per-lane request; the acceptance requires the start time until completion.
- file: crates/control-plane-app/src/live.rs
  line: 142
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: compact derives waits without reading runtime_error, so after a fatal runtime error the console projects an open model call that nothing holds until the service restarts.
- file: crates/control-plane-app/src/live.rs
  line: 349
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a model call recorded before a restart is projected as open until the first fleet tick, which runs after the planning tick over every running goal.
```
