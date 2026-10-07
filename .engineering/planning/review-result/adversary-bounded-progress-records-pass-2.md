---
format: aep.planning-md/3
id: review-result:adversary-bounded-progress-records-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:bounded-progress-records (red, 3 introduced, 1 undecided)'
relations:
- reviews: story:bounded-progress-records
revision: 1
---
unit: story:bounded-progress-records
verdict: red
cases: executed 164→169, red 3
origin: introduced 3, pre-existing 0, undecided 1
wrote-outside-worktree: $HOME/.cache/cp-wave1/bounded-progress-records/pass2/ (6 logs), /dev/shm/cp-wave1-bounded-progress-records-target (the assigned build dir, reused)
needs-coordinator: yes

The findings cover head 982d88d plus my uncommitted test additions. I changed only test files:

```
 crates/control-plane-app/src/tests.rs         | 154 ++++++++++++++++++++++++++
 crates/control-plane-core/src/tests.rs        |  69 ++++++++++++
 crates/control-plane-runtime/tests/fleet.rs   |  90 +++++++++++++++
 crates/control-plane-runtime/tests/planner.rs | 101 +++++++++++++++++
 4 files changed, 414 insertions(+)
```

The correction holds on the paths you named: history readers, restart, the old-format latch and per-member bounding. What fails is the 16 KiB outcome, on two record types that are recorded once rather than per event.

| # | Finding | file:line | Origin | Severity |
|---|---|---|---|---|
| P1 | A validated plan is one progress decision of **108,904 bytes**. Its receipt is the engine's output: the whole transcript (up to 96 KiB) plus the action history. `bound_progress` keeps changed planner evidence whole, so the decision stores it in the body and again in the outcome. | crates/control-plane-runtime/src/supervisor.rs:435 | undecided | warning |
| P2 | A rejected acceptance is one decision of **19,738 bytes**. The new 8 KiB reason cap is stored twice, which alone reaches 16 KiB. Any reviewer reason over about 6.7 KiB breaks the outcome. | crates/control-plane-runtime/src/fleet.rs:32 | introduced | warning |
| P3 | The evidence endpoint takes the store lock twice: once in `snapshot()` at :14, again at :31. A progress decision that commits between the two shows up in `history` but not in the goal's receipt or its attached `activity_history`. The response also carries the history twice, so the planner evidence is doubled. | crates/control-plane-app/src/dashboard.rs:31 | introduced | note |
| P4 | No case from the unit measured a decision recorded after an acceptance record. My pin covers it and passes: later decisions are under 4 KiB, the stored receipt has no `acceptance`, and history keeps it across reopen. I did not run a mutated copy. | crates/control-plane-core/src/lib.rs:404 | introduced | note |

**What reaches each finding**
- **P1:** Every successful planning attempt (supervisor.rs:435, engine.rs:199). It is once per attempt, not per event, so replay stays fast. Reading the base with `git show a09cff6` shows the same record unbounded, but I did not run the case there.
- **P2:** The goal reviewer is a model and its reason has no length limit. The stored reason wraps the whole review JSON (fleet.rs:2153), and each escape level adds bytes.
- **P3:** Any evidence request that overlaps a progress commit. Progress commits happen on every Loom event during a run. I forced the overlap with the store lock, which is FIFO.
- **P4:** Every progress decision after a goal review copies the receipt. Without the strip, each would carry the acceptance record, up to 8 KiB twice.

**Cases added.** Each was run alone first. The red output below was captured before the suite ran.
- `adversary_validated_plan_decision_stays_under_16_kib` (tests/planner.rs:1647): red, 4 of 4 runs.
  - `the validated-plan progress decision (version 86) is 108904 bytes of event data`
  - A 1 ms observer samples version and bytes on both sides of that one decision. A version-gap check guards the measurement.
- `adversary_rejected_acceptance_record_decision_stays_under_16_kib` (tests/fleet.rs:652): red.
  - `the rejected acceptance record is one decision of 19738 bytes of event data (reviewer reason 8243 bytes; blocked activity 2682 bytes)`
  - It runs the real fleet path. The blocked activity decision is measured by recording a twin of it.
- `tests::adversary_evidence_shows_one_state_of_the_goal` (app tests.rs:325): red.
  - `history ends with "step 01", the goal's attached history with "step 00", its recorded receipt with "step 00"`
- `tests::adversary_state_api_and_dashboard_keep_history_after_restart` (app tests.rs:396): green. After a reopen, `/api/state` returns all 30 activities and the fleet entry, and the dashboard shows steps 10, 20 and 29.
- `tests::adversary_rejected_acceptance_is_recorded_once_and_survives_restart` (core tests.rs:1440): green. This is the P4 pin.

**Suite run, after the cases existed:** `cargo test --locked -p control-plane-core -p control-plane-runtime -p control-plane-app --no-fail-fast` exited 101.
```
control_plane_app lib:     test result: FAILED. 29 passed; 1 failed
operator_boundary:         test result: ok. 3 passed; 0 failed
control_plane_core lib:    test result: ok. 39 passed; 0 failed
control_plane_runtime lib: test result: ok. 38 passed; 0 failed
tests/fleet.rs:            test result: FAILED. 22 passed; 1 failed
tests/planner.rs:          test result: FAILED. 35 passed; 1 failed
```
- **Before (164):** from the implementing state's own gate log, `c1-gate-test.log` in the scratch dir.
- **After (169):** this run. Only my three cases fail. Their names exist only in this tree, so these binaries are this tree's.

**Decisions for you**
- **P1:** Either limit the 16 KiB outcome to per-event progress, or move the transcript out of the validated receipt (Loom files each session). Moving it costs the evidence page its observations, and planner tests such as `missing_reads_are_observations_and_planning_can_continue` read them from there.
- **P2:** Your F3 decision set 8 KiB without counting the second copy in the outcome. A cap of about 6 KiB fits; otherwise the outcome needs an exception for acceptance records.
- **P3:** A small fix would be to read the history from the snapshot's `goal["activity_history"]`.

**Attacked and could not break**
- **Every history reader:**
  - SSE, `/api/console`, `/api/state` and the dashboard read `activity_history` through `snapshot()`.
  - Fleet and supervisor latches read it through `activity_history`.
  - The restart probe is green.
- **History vs views:** apart from the P3 race, I found no gap. Only `RecordPlanningProgress` sets the receipt (ESS), and every applied one advances the journal. The journal check therefore cannot go stale.
- **Old-format latch:** the legacy fixture's acceptance is `failed` at revision 1. The unit's upgrade test pins it through the first bounded decision and a reopen.
- **Latch across restart:** both suite tests pass, the unit's own `rejected_goal_acceptance_remains_durable…` and my P4 pin.
- **Bounding idempotence:** a second pass returns the same value for every detail shape I traced, so the journal does not double-record a copied `last_activity`.
- **Maximum bounded activity:** about 3.5 KB from real producers. Only `args` is an array, and its atoms keep 12 × 240 bytes. The code allows about 26 KB raw (9 kept keys × 12 × 240), but nothing produces that.
- **Shown keys:** the dashboard and console fields are all in `SHOWN`.
- **4 KiB planning caps:** they never alter a path, fingerprint or phase that is compared later. With a quote-heavy error text, the blocked planning decision estimates at about 13–15 KB.
- **Pass-1 cases:** all six pass, with no regressions found.

**Paths written outside the worktree**
- $HOME/.cache/cp-wave1/bounded-progress-records/pass2/: `red-planner.log`, `red-fleet.log`, `red-app-evidence.log`, `app-restart.log`, `core-acceptance-pin.log`, `gate.log`
- /dev/shm/cp-wave1-bounded-progress-records-target, the assigned build dir, reused
- Temporary dirs from the existing test fixtures under `$HOME/.cache/control-plane-console`. Both that directory and `control-plane-host` are empty now.
- The planner and fleet fixtures kept their dirs under the worktree's gitignored `.scratch/`, as the existing tests do.

I took my own worktree session lease for this pass and released it at the end.

```findings
[
  {"file":"crates/control-plane-runtime/src/supervisor.rs","line":435,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"The validated-plan RecordPlanningProgress decision keeps the engine's whole transcript as changed planner evidence and measured 108,904 bytes, against the story's 16 KiB per progress decision."},
  {"file":"crates/control-plane-runtime/src/fleet.rs","line":32,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The 8 KiB acceptance-reason cap is stored in both body and outcome, so a rejected goal review with a long reason is one 19,738-byte decision over the 16 KiB bound."},
  {"file":"crates/control-plane-app/src/dashboard.rs","line":31,"category":"concurrency","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The evidence endpoint reads history under a second store lock, so one response can show a history newer than the goal and attached history it carries, and it carries the history twice."},
  {"file":"crates/control-plane-core/src/lib.rs","line":404,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"No unit case observed that an unchanged acceptance record is left out of later decisions; the added pin covers it and passes, but no mutated copy was run."}
]
```
