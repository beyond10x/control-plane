---
format: aep.planning-md/3
id: review-result:adversary-bounded-progress-records-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:bounded-progress-records (red, 5 introduced, 1 undecided)'
relations:
- reviews: story:bounded-progress-records
revision: 1
---
unit: story:bounded-progress-records
verdict: red
cases: executed 157→163, red 5
origin: introduced 5, pre-existing 0, undecided 1
wrote-outside-worktree: $HOME/.cache/cp-wave1/bounded-progress-records/adversary-p1-gate.log, /dev/shm/cp-wave1-bounded-progress-records-target (the assigned build dir, reused)
needs-coordinator: yes

The findings cover head bdafab2 plus my uncommitted test additions. I changed only test files:

```
 crates/control-plane-app/src/tests.rs       |  73 +++++++++++++++
 crates/control-plane-core/src/tests.rs      | 140 ++++++++++++++++++++++++++++
 crates/control-plane-runtime/tests/fleet.rs |  53 +++++++++++
 3 files changed, 266 insertions(+)
```

| # | Finding | file:line | Origin | Severity |
|---|---|---|---|---|
| F1 | Progress decisions can still go over 16 KiB. `record_activity` copies every `planning_*` field into each decision, and the core caps none of them. A blocked planning attempt stores `format!("{error:#}")` as the reason, and a failed command's error text includes its full stdout and stderr. With that reason on the goal, each fleet progress decision measured **40,476 bytes**. | crates/control-plane-core/src/lib.rs:349 | undecided | warning |
| F2 | GoalList shows a different `planning_receipt` than the one the command recorded and published. The view adds `activity`, `fleet` and `planner` from the in-memory journal, but ESS says GoalList shows the Goal's field as it was set. | crates/control-plane-core/src/lib.rs:404 | introduced | warning |
| F3 | The acceptance reason is cut to 2 KiB, and the cut adds a note meant for the model ("Request a relevant file page with read_range") to the operator's record. A 2.9 KiB goal-reviewer rejection loses its end. The full reason is now stored nowhere in the host, because the blocked activity keeps only a 180-character summary. | crates/control-plane-runtime/src/fleet.rs:118 | introduced | warning |
| F4 | Activity bounding runs before the event is written. For a structured detail over 1 KiB it keeps only `summary` and `omitted_bytes`, so a `tool.run` loses its args (and any worktree or target). The HTML dashboard and the evidence page now show `cargo` where they showed `cargo test --locked --package …`. The SSE projection never showed args, so it is unchanged. | crates/control-plane-core/src/memory.rs:259 | introduced | warning |
| F5 | `omitted_bytes` is copied unchanged into the bounded detail, whatever its type or size. One activity measured 73,868 bytes. Nothing in the runtime produces that key today. | crates/control-plane-core/src/memory.rs:281 | introduced | note |
| F6 | Two journal behaviours are not tested by the unit's own suite: the 64-entry cap and the skip of a repeated last activity. Its tests check at most the newest 24 entries and never re-record an unchanged last activity. My pin test passes against this head; I did not run it against a mutated copy because of disk space. | crates/control-plane-core/src/memory.rs:45 | introduced | note |

**What reaches each finding**
- **F1:** Planning errors become the stored reason at `supervisor.rs:152` and `:179`. The error text includes stdout and stderr (`process.rs:28-35`). The fleet does not check the planning phase, so its Loom progress events keep being written while the reason stays on the goal. Reading `git show a09cff6` shows the old `Host::progress` copied `planning_*` the same way, but I did not run the case on the base.
- **F2:** Every progress decision does this.
- **F3:** The goal reviewer is a model, and its schema puts no length limit on `reason` (`model.rs:63`).
- **F4:** `fleet.rs:1409` records the model-chosen args (up to 128 are allowed, `fleet.rs:1535`). The dashboard shows program plus 12 args (`dashboard.rs:84-95`).
- **F5:** Grep finds no producer of `omitted_bytes`, so this is a state I built myself.

**Cases added.** Each was run alone first, and the red output below was captured before the suite ran.
- `adversary_progress_after_a_long_planning_reason_stays_under_16_kib` (core): red, `progress decisions after a blocked planning attempt: [40476, 40476, 40476] bytes`
- `adversary_goal_list_shows_the_receipt_the_command_recorded` (core): red. The view's receipt ends `"receipt_format":2,"activity":[…]}`, while the recorded one ends `"receipt_format":2}`.
- `adversary_structured_detail_bounds_every_member_it_keeps` (core): red, `bounded activity is 73868 bytes`
- `adversary_journal_keeps_the_newest_64_once_each_across_restart` (core): green. It pins the 64-entry cap, the newest entry per assignment, the skip on re-record and the restart result for F6.
- `adversary_recorded_tool_run_keeps_the_command_the_dashboard_shows` (app): red, `recorded detail: {"omitted_bytes":1406,"summary":"cargo"}`. Its control check, the same event rendered from a hand-built view, passes.
- `adversary_rejected_goal_acceptance_keeps_the_reviewers_whole_reason` (fleet): red. The recorded reason ends `…Obligation 30 is not dem\n[Excerpt: 2048 of 2837 bytes shown. Request a relevant file page with read_range; …]`

**Suite run, after the cases existed:** `cargo test --locked -p control-plane-core -p control-plane-runtime -p control-plane-app --no-fail-fast` exited 101.
```
control_plane_app lib: test result: FAILED. 26 passed; 1 failed
operator_boundary:     test result: ok. 3 passed; 0 failed
control_plane_core lib: test result: FAILED. 35 passed; 3 failed
control_plane_runtime lib: test result: ok. 38 passed; 0 failed
tests/fleet.rs:        test result: FAILED. 21 passed; 1 failed
tests/planner.rs:      test result: ok. 35 passed; 0 failed
```
- **Before (157):** taken from the implementing state's own gate log, `gate-test.log` in the scratch dir.
- **After (163):** this run. Every failing test is one of mine, and the printed names exist in this tree.

**Decisions for you (needs-coordinator)**
- **F2:** Is an ESS view filled from a journal held in host memory the specification change, or new record type, that the story ruled out? That is your call.
- **F1:** Its origin needs a run on the base. Fixing it, and F3, would mean recording non-activity receipt fields once instead of copying them into every decision, or capping them in the core.

**Attacked and could not break**
- **Moving Memory instead of cloning:** every `stage` caller works on a clone and throws it away on error. `commit` only replaces memory after a successful append.
- **Replay outcome check:** `outcome == decision.outcome` is unchanged.
- **Live view vs replay:** the journal pin test shows they are identical over 100 events, 3 assignments and 10 re-records.
- **Authority checks:** RecordPlanningProgress has no state precondition in ESS. No progress path lost a goal check it had at base.
- **Console API:** RecordPlanningProgress is Supervisor-only and not in the operator command list.
- **Opening the store with the old binary:** reading the code, the old binary replays bounded receipts with the same outcome and only shows a shorter history. Not run.

**Paths written outside the worktree**
- $HOME/.cache/cp-wave1/bounded-progress-records/adversary-p1-gate.log
- /dev/shm/cp-wave1-bounded-progress-records-target, the assigned build dir, reused
- Temporary dirs created by the existing test fixtures under $HOME/.cache/control-plane-host/ and $HOME/.cache/control-plane-console/. They were removed when the tests finished, and both are empty now.

I took my own worktree session lease for this pass and released it at the end.

```findings
[
  {"file":"crates/control-plane-core/src/lib.rs","line":349,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"record_activity copies uncapped planning_* fields into every progress decision, so a blocked planning reason holding command output makes each decision 40,476 bytes, against the story's 16 KiB bound."},
  {"file":"crates/control-plane-core/src/lib.rs","line":404,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"GoalList planning_receipt is expanded from a host-memory journal and no longer equals the receipt RecordPlanningProgress set and published, as ESS declares."},
  {"file":"crates/control-plane-runtime/src/fleet.rs","line":118,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The goal acceptance reason is cut to 2 KiB, with a read_range note meant for the model, so a 2.9 KiB reviewer rejection is no longer recorded whole anywhere in the host."},
  {"file":"crates/control-plane-core/src/memory.rs","line":259,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Activity details over 1 KiB are reduced to summary and omitted_bytes before they are written, so a recorded tool.run loses its args on the dashboard, on the evidence page and in the durable record."},
  {"file":"crates/control-plane-core/src/memory.rs","line":281,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"bounded_detail copies omitted_bytes unchanged at any size (73,868 bytes measured), but nothing in the runtime produces that key."},
  {"file":"crates/control-plane-core/src/memory.rs","line":45,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"No test from the unit checks the 64-entry journal cap or the skip of a repeated last activity; the added pin test covers both and passes against this head."}
]
```
