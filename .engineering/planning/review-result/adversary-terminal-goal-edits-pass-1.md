---
format: aep.planning-md/3
id: review-result:adversary-terminal-goal-edits-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:terminal-goal-edits (279e420): red, 1 introduced, 2 pre-existing'
relations:
- reviews: story:terminal-goal-edits
revision: 1
---
unit: story:terminal-goal-edits
verdict: red
cases: executed 77→85, red 2
origin: introduced 1, pre-existing 2, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-terminal-goal-edits-scratch/adversary/ (logs, spec copies, two green probe files), /dev/shm/cp-wave2-terminal-goal-edits-scratch/tmp/ (test tempdirs, now empty), /dev/shm/cp-wave2-terminal-goal-edits-target (the assigned build dir)
needs-coordinator: no

Two cases are red, but neither is a blocker: one is a note this unit introduced, the other a note that was already there before it. The one warning is also pre-existing and has no test. The change itself held under every other attack. Findings cover the worktree at 279e420 plus my two untracked test files.

## 1. Worktree diff

`git --no-pager diff --stat` is empty: no tracked file changed. There are two untracked files, both tests:
```
?? crates/control-plane-core/tests/terminal_goal_edits_attack.rs
?? crates/control-plane-xtask/tests/terminal_goal_edits_ack_attack.rs
```

## 2. Cases added (each run alone first, before any suite run)

| File | What it asserts | Now |
|---|---|---|
| `crates/control-plane-core/tests/terminal_goal_edits_attack.rs` `invalid_edit_of_satisfied_goal_answers_the_declared_refusal` | An edit of a Satisfied goal with `max_workers: 0` leaves the goal row unchanged and returns the declared `satisfied` / `GoalStateConflict` outcome | red |
| `crates/control-plane-xtask/tests/terminal_goal_edits_ack_attack.rs` `unit_acknowledgements_admit_only_the_refusals_they_reviewed` | The test rebuilds the reviewed baseline, then puts the unit's 3 acknowledgements back against it. The reviewed change passes. A different, valid change must be refused: `satisfied` and `cancelled` swapped between the two states and answering `GoalNotFound` | red |

Red output, verbatim:
```
thread 'invalid_edit_of_satisfied_goal_answers_the_declared_refusal' (3726853) panicked at crates/control-plane-core/tests/terminal_goal_edits_attack.rs:79:9:
an edit of a Satisfied goal was refused by an undeclared host error instead of the declared `satisfied` outcome: max_workers must be positive
test result: FAILED. 0 passed; 1 failed; ...
```
```
thread 'unit_acknowledgements_admit_only_the_refusals_they_reviewed' (3751949) panicked at crates/control-plane-xtask/tests/terminal_goal_edits_ack_attack.rs:131:24:
the unit's acknowledgements admitted a change they never reviewed (satisfied and cancelled swapped between Satisfied and Cancelled, answering GoalNotFound): specification history gate passed: 3 change(s) since baseline d45926529b5ed2787c972b59cfd6d544741cbe1a (merge base with main), 3 could break replay, 3 acknowledged
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out
```

## 3. Suite run (after the cases existed)

- **Core + app:** `cargo test --locked -p control-plane-core -p control-plane-app --no-fail-fast` gave EXIT=101.
  - Passed: 30, 0, 3, 41, 2 (`recorded_history`), 1, plus 0 and 0 doc-tests.
  - `terminal_goal_edits_attack`: 0 passed, 1 failed.
  - That is 78 executed. The before figure, 77, comes from the implementor's log `/dev/shm/…-scratch/gate/6-test.log`.
- **xtask:** `cargo test --locked -p control-plane-xtask --test terminal_goal_edits_ack_attack` gave EXIT=101 with "6 passed; 1 failed". The 6 are the gate's own unit tests, which run again because my file compiles the gate module in. So 77 + 1 + 7 = 85 executed.
- **Generated output:** `generated-check` reported "generated contracts match emitter bytes", EXIT=0.
- **Spec history:** `spec-history-check` reported "passed: 3 change(s) … 3 acknowledged", EXIT=0.
- **Conformance:** "171 scenarios passed" (167 at base, so 4 new for UpdateGoal), EXIT=0.

## 4. Findings

| file:line | Finding | Verdict | Origin | Severity | What reaches it |
|---|---|---|---|---|---|
| `crates/control-plane-core/src/guards.rs:104` | The field guard on UpdateGoal runs before the generated state check. An invalid edit of a Satisfied or Cancelled goal therefore gets an undeclared host error, not the declared refusal, and no refusal decision is recorded. The goal itself stays unchanged. | CONFIRMED | introduced (the guard is unchanged since base; the declared refusal it pre-empts is new) | note | `control-plane goal edit <id> --max-workers 0` (`cli.rs:111`, `Option<i64>`). The HTTP status is 409 either way. Fix: skip the field checks when the addressed goal is terminal. |
| `ess/spec-acknowledgements.json` | The `applied` acknowledgement is precise, because its change object carries the before and after text. The two `outcome-added` acknowledgements are not: ESS's change object holds only the outcome's name, so they admit any later outcome with that name, whatever its state or error. | INFEASIBLE | pre-existing (`spec_history.rs` is unchanged since base) | note | No real change of this kind has been made; I built one. `recorded_history_replays` would still catch it, because the re-recorded fixture stores both refusals. |
| `crates/control-plane-runtime/src/fleet.rs:2079` | The fleet's goal-acceptance step checks state and revision at :2079, releases the store lock, runs `git ls-remote` for each repository (:2098), then sends `SatisfyGoal` at :2117. An operator edit landing in that window is applied. The goal then ends Satisfied with a receipt for the old `goal_revision`, and after this unit nobody can edit it. | CONFIRMED (from reading the code; no run observed) | pre-existing (runtime unchanged in the diff) | warning | The HTTP `UpdateGoal` handler takes the same mutex as the fleet. Fix: hold the lock from the check through `SatisfyGoal`, or give `SatisfyGoal` a `goal_revision` input that it refuses when stale. |

The commit also edits `crates/control-plane-xtask/src/history.rs` and `crates/control-plane-xtask/README.md`, which the brief lists as not the implementor's files. Your message already names this, so it is not in the findings.

## 5. Attacked and could not break

- **The four lifecycle states:** the lifecycle has exactly Paused, Running, Satisfied and Cancelled. ESS refuses outcome conditions that overlap. The `applied` conformance scenario covers both Paused and Running.
- **Other commands that touch a Goal:** only `RecordPlanningProgress` does, and it writes only the `planning_*` fields. `SatisfyGoal` and `CancelGoal` on a terminal goal answer `wrong-state`. The runtime never sends `UpdateGoal`.
- **Wiping the receipt:** `satisfaction_receipt: ''` in `applied` cannot wipe one. Only `SatisfyGoal` sets the receipt, and it moves the goal to a terminal state at the same time.
- **HTTP answer:** a probe test passed. JSON API edit of a Satisfied goal: 409, `error` names `"outcome":"satisfied"` and `GoalStateConflict`. HTML form edit of a Cancelled goal: 409. Edit of a Paused goal: 303 and applied. Neither terminal goal changed.
- **The ack's claim that the old fixture still replays:** I replayed the base fixture (bc1efdf) under the new code and compared it with the base views. They match.
- **The re-recorded fixture:** it replays, covers both refusals (409 in OpenAPI), and its views keep the original objective and receipt.
- **The unit's tests:** they assert literal values and fail at base. None would pass if the calls returned defaults.
- **Unacknowledged changes:** the gate refuses any change with no matching entry (`spec_history.rs:490-521`). `unacknowledged_spec_change_fails_gate` passed in my binary.
- **The Vue console and the server-rendered page:** both show the edit form only for Paused and Running goals.

## 6. Paths written outside the worktree

- `/dev/shm/cp-wave2-terminal-goal-edits-scratch/adversary/`: run logs, spec copies (`base/`, `mut/`, `mutC/`), and the two green probe files (`terminal_goal_edits_base_history_probe.rs`, `terminal_goal_edits_http_probe.rs`).
- `/dev/shm/cp-wave2-terminal-goal-edits-scratch/tmp/`: test tempdirs, now empty.
- `/dev/shm/cp-wave2-terminal-goal-edits-target`: the assigned build dir.
- The harness wrote 2 task output files under `$HOME/.cache/claude-tmp/claude-1000/…/tasks/`.
- My run added `.scratch/spec-history-tests` inside the worktree (4 KB); I deleted it.
- I acquired and released the worktree session lease `adversary-terminal-goal-edits-w2p1`.

```findings
- file: crates/control-plane-core/src/guards.rs
  line: 104
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: The UpdateGoal field guard runs before the state check, so an invalid edit of a Satisfied or Cancelled goal gets an undeclared host error instead of the declared satisfied or cancelled refusal, and no refusal decision is recorded.
- file: ess/spec-acknowledgements.json
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: The two outcome-added acknowledgements match on the outcome name alone, so they also admit a later satisfied or cancelled outcome with a different state or error; recorded_history_replays would still catch that.
- file: crates/control-plane-runtime/src/fleet.rs
  line: 2079
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: Goal acceptance checks revision, releases the store lock across git ls-remote, then sends SatisfyGoal, so an operator edit in that window yields a Satisfied goal whose receipt names the old revision and which can no longer be edited (inferred from code, no run observed).
```
