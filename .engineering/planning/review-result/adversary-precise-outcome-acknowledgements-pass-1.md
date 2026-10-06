---
format: aep.planning-md/3
id: review-result:adversary-precise-outcome-acknowledgements-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:precise-outcome-acknowledgements at f1b3d30'
relations:
- reviews: story:precise-outcome-acknowledgements
revision: 1
---
The added-outcome acknowledgement does what the story's acceptance line asks. It still admits one later change I could build: an added outcome moved ahead of an overlapping existing one. My case for that is red, and the same case is red on the base commit too.

```
unit: story:precise-outcome-acknowledgements, control-plane worktree at f1b3d30 plus one untracked test file
verdict: CONFIRMED
cases: executed 85→98, red 1
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: whether F1 (the order of an added outcome is not bound) belongs to this story or a new one; whether to keep the red case
```

**1. Diff proof**

```
$ git status --short --untracked-files=all
?? crates/control-plane-xtask/tests/precise_outcome_acknowledgements_attack.rs
$ git --no-pager diff --stat
(empty: no tracked file changed)
```

The only path is a new test file (405 lines). `frontend/node_modules` was installed for the suite and then removed.

**2. Cases added** (in `crates/control-plane-xtask/tests/precise_outcome_acknowledgements_attack.rs`)

| case | what it asserts | now |
|---|---|---|
| `moving_an_added_outcome_ahead_of_an_overlapping_one_is_not_admitted` (:256) | The baseline declares the input-guarded refusal `too-many-workers` (`max_workers > 100`). The branch adds `too-many-attempts` (`max_attempts > 100`) after it, and the entry records the compiled outcome. The reviewed placement passes. Moving `too-many-attempts` first must be refused. | **red** |
| `the_same_swap_of_two_declared_outcomes_is_refused` (:313) | Control: when both outcomes are already in the baseline, the same swap is refused as `outcome-order-changed`. | green |
| `migrated_acknowledgements_admit_only_the_reviewed_refusals` (:335) | The unit's three migrated live entries, moved onto a reconstructed pre-change baseline, admit the reviewed `UpdateGoal` change and refuse the refusals swapped and answering `GoalNotFound`. | green |
| `added_outcome_acknowledgement_binds_its_field_updates` (:378) | `applied` is added (the baseline calls it `edited`). A later `revision: increment: 2` must be refused, with `sets: reviewed` in the message. It kills the mutant in F2. | green |

Inside the red case, these assertions all passed before the gate call:
- the compiled `too-many-attempts` object is equal before and after the move;
- `ess specify validate --strict-requires` accepts both states;
- `ess generate synthesize` generates everything (no obligations), and the generated `update_goal` checks `TooManyWorkers` first before the move and `TooManyAttempts` first after it.

Red output, run alone before the suite. This is the second run: the first fixture used `GoalStateConflict`/`GoalNotFound` and was also red, but its refusal could not be fully generated, so I switched to two field-less errors and re-ran.

```
running 1 test
test moving_an_added_outcome_ahead_of_an_overlapping_one_is_not_admitted ... FAILED
thread 'moving_an_added_outcome_ahead_of_an_overlapping_one_is_not_admitted' (1818045) panicked at crates/control-plane-xtask/tests/precise_outcome_acknowledgements_attack.rs:266:24:
the acknowledgement reviewed for too-many-attempts declared after too-many-workers admitted it moved ahead of too-many-workers; an UpdateGoal recorded as too-many-workers (GoalWorkerLimit) now replays as too-many-attempts (GoalAttemptLimit): specification history gate passed: 1 change(s) since baseline ce12b5c92ef6115683b1a5c7b0ad1d61a031a6b9 (merge base with main), 1 could break replay, 1 acknowledged
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 7.63s
EXIT=101
```

After `rustfmt` the same panic is at :292.

**3. Suite run** (after the cases existed; summary lines quoted as captured)

`npm ci --prefix frontend ...` exited 0, then:

```
$ cargo test --locked --no-fail-fast -p control-plane-xtask
unittests src/main.rs                 ok. 17 passed
acceptance_traceability               ok. 3 passed
acceptance_traceability_attack        ok. 7 passed
acceptance_traceability_pass2_attack  ok. 4 passed
frontend_check                        ok. 7 passed
frontend_check_attack                 ok. 9 passed
frontend_check_pass2_attack           ok. 6 passed
generation_ownership                  ok. 4 passed
precise_outcome_acknowledgements_attack  FAILED. 12 passed; 1 failed
record_history_attack                 ok. 2 passed
spec_history_attack                   ok. 15 passed
spec_history_pass2_attack             ok. 11 passed
error: 1 target failed:   EXIT=101
```

- 98 cases ran. The "before" count of 85 is this same run with my test binary deselected (13 cases: my 4 plus the 9 module tests it compiles in by path).
- `cargo fmt -p control-plane-xtask --check` exited 0, and so did `cargo clippy --locked -p control-plane-xtask --all-targets -- -D warnings`.
- `spec-history-check` on this tree passes: 3 changes, 3 acknowledged.

**4. Findings** (all against f1b3d30)

- **F1, `crates/control-plane-xtask/src/spec_history.rs:608` — INFEASIBLE, pre-existing, warning.**
  - **What fails:** the entry binds the compiled outcome, which records no position. ESS 0.53.0 compares outcome order only over the outcomes both revisions declare (`ess-diff/src/diff.rs:1677`, "inserting one necessarily moves the rest"). So an added outcome can move ahead of an overlapping existing one with no change reported, and the gate admits it.
  - **Why it matters:** input-guarded refusals are answered in declaration order. After the move, a recorded call answered `too-many-workers` replays as `too-many-attempts`.
  - **Origin:** my harness ran the same case against base 9691f41 (format /1, no `outcome` field) and it is red there too.
  - **What reaches it:** nothing today. `ess/domains/host.yaml` declares 0 `when:` outcomes. It needs a branch that adds a `when:` refusal next to an overlapping one and then reorders them.
  - **Fix to consider:** record the outcome's position (the names declared before it in `.commands[cmd].outcomes`) and compare it. Alternatively, an ESS issue asking for added outcomes to be included in `outcome-order-changed`.
- **F2, `crates/control-plane-xtask/src/spec_history.rs:1041` — CONFIRMED, introduced, note.**
  - **What fails:** the README (lines 128–129) says the gate refuses a changed "error, subject, events, payload or field updates", but the unit's tests only change the condition and the error.
  - **Measured:** I made a mutant of :608 in a scratch copy that compares only `condition` and `error`. All 26 existing spec-history cases stayed green against it. My case at :378 goes red against it and is green on f1b3d30.
  - **Fix:** keep that case.
- **F3, `crates/control-plane-xtask/tests/spec_history_pass2_attack.rs:82` — CONFIRMED, introduced, note.** The unit edited two existing adversary case files, which were outside its assignment ("new test files"). The format bump forced the change and no case was weakened: the edits are the format string plus a hard-coded `wrong_state()` copy of ESS 0.53.0 compile output.

**5. Attacked and could not break**

- Changed condition or error under the same name: refused, including against the live migrated entries.
- `/1` format, missing `outcome`, `outcome` with the wrong name, and `outcome` on a non-outcome-added entry: all refused.
- An unacknowledged added outcome gets a ready entry that carries `outcome`.
- JSON key order does not matter; CI pins ESS 0.53.0, so the compiled output cannot drift between local and CI.
- Edits that only touch `summary`, or a state written as a list versus a single value, compile differently and are refused. That is stricter than needed but documented as intended.
- Overlapping subject-state branches cannot be written: ESS rejects them ("select 2 branches").

**6. Paths written outside the worktree**

- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-scratch/adversary-pass1/` (464K, kept): run logs in `logs/`, the harness in `harness/`, and `compiled.json`. The harness holds the mutant and base copies of `spec_history.rs`, test copies with absolute paths, `Cargo.toml` and `Cargo.lock`. The probe specs were deleted.
- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-target` (the assigned build directory): my test binary is there. I removed the harness's own artifacts with `cargo clean -p adversary-harness`; builds of anyhow, serde_json and tempfile for the harness remain.
- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-scratch/tmp/node-compile-cache`: deleted, `tmp/` is empty.
- Inside the worktree (git-ignored): the repository's own unit tests wrote `.scratch/conformance-tests/` (1.2 MB) during my suite run. I left it.

**7. Findings block**

```findings
- file: crates/control-plane-xtask/src/spec_history.rs
  line: 608
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: an outcome-added acknowledgement binds no position, so moving the added outcome ahead of an overlapping existing input-guarded outcome changes which outcome answers and the gate still admits it
- file: crates/control-plane-xtask/src/spec_history.rs
  line: 1041
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a mutant comparing only condition and error survives all 26 existing spec-history cases although the README promises field updates are bound; added_outcome_acknowledgement_binds_its_field_updates kills it
- file: crates/control-plane-xtask/tests/spec_history_pass2_attack.rs
  line: 82
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the unit edited two existing adversary case files outside its file assignment, forced by the format bump and without weakening any case
```
