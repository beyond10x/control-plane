---
format: aep.planning-md/3
id: review-result:adversary-spec-history-gate-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:spec-history-gate (red, 5 introduced)'
relations:
- reviews: story:spec-history-gate
revision: 1
---
unit: story:spec-history-gate
verdict: red
cases: executed 46→56, red 7
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/cp-wave1/spec-history-gate/{probe/, e2e/, adv-cases/, adv-suite.log, adv-suite-nff.log, adv-suite-final.log}, /dev/shm/cp-wave1-spec-history-gate-target/adversary-e2e/
needs-coordinator: yes

The unit is red. The gate reads only ESS's `history` verdict. But `Store::open` re-runs each recorded command with its recorded input and actor, and requires the same answer, byte for byte. ESS calls a removed outcome, a revoked grant, a changed payload or a changed error shape `history: compatible`, so the gate passes all of them. In a scratch copy I proved one end to end: the gate passes, the committed fixture still replays, and a store a normal operator action can produce fails to open. Findings cover worktree tree `ab6f1c7` plus my two untracked test files.

You need to decide one thing: the fix for F4 (diff against the merge base with `origin/main`) goes against the recorded-baseline design your brief suggested.

**1. What I touched.** `git diff --stat` is empty because both files are new and untracked. `git status --short` shows only test paths, and no implementation file was changed:
```
?? crates/control-plane-xtask/tests/record_history_attack.rs   (64 lines)
?? crates/control-plane-xtask/tests/spec_history_attack.rs     (262 lines)
```
`spec_history_attack.rs` pulls in `src/spec_history.rs` with `#[path]` (it isn't edited). As a result, 3 of the 10 added executions are the unit's own three tests running a second time.

**2. Cases, each run alone first (exit 101 each)**

| case | red output (verbatim, trimmed) |
|---|---|
| `removed_recorded_outcome_fails_gate` | `the gate passed a change that breaks replay of stored decisions (command/controlplane.host.DisableRepositoryRegistration/outcome-removed/wrong-state): specification history gate passed: 1 change(s) … 0 could affect stored history` |
| `changed_recorded_error_shape_fails_gate` | `… (error/controlplane.host.RepositoryRegistrationStateConflict/field-added/hint): specification history gate passed: 1 change(s) …` |
| `revoked_grant_that_replay_needs_fails_gate` | `… (actor/controlplane.host.Operator/grant-removed/controlplane.host.PauseGoal): specification history gate passed …` |
| `changed_recorded_payload_fails_gate` | `… (command/controlplane.host.RepairAssignment/outcome-payload-changed/applied): specification history gate passed …` |
| `acknowledgement_does_not_admit_a_different_change_with_the_same_id` | ``an acknowledgement reviewed for `Blocked, Reviewing -> Implementing` => `Reviewing -> Implementing` admitted `=> Reviewing -> Queued`: specification history gate passed: 2 change(s) …, 1 acknowledged`` |
| `route_published_after_baseline_then_removed_fails_gate` | `removing a route published after the baseline passed the gate: specification history gate passed: 0 change(s) …` |
| `record_history_refuses_a_fixture_whose_replay_no_longer_matches_its_views` | `recorded 45 host commands into …` then `record-history replaced a committed history whose replay no longer matches its recorded views` |

**3. Gate run after the cases** (`--no-fail-fast`; without that flag cargo stops at the first red binary)
```
control_plane_core lib:      test result: ok. 30 passed; 0 failed
recorded_history:            test result: ok. 2 passed; 0 failed
control_plane_xtask main:    test result: ok. 10 passed; 0 failed
generation_ownership:        test result: ok. 4 passed; 0 failed
record_history_attack:       test result: FAILED. 0 passed; 1 failed
spec_history_attack:         test result: FAILED. 3 passed; 6 failed
error: 2 targets failed:     TEST_EXIT=101
spec-history-check:          specification history gate passed: 0 change(s) since baseline eaa37d4057b4c6b90c6cebccefa28dca4abd411e, 0 could affect stored history, 0 acknowledged   GATE_EXIT=0
```
The before count of 46 comes from the implementor's `gate.log` (30+2+10+4+0). `rustfmt --check` and `cargo clippy -p control-plane-xtask --all-targets -D warnings` are clean on both added files.

**4. Findings**

| # | finding | file:line | origin | severity |
|---|---|---|---|---|
| F1 | The gate only checks the `history` verdict. Replay re-runs each recorded call (input and actor, which ESS rates under `callers`) and compares its full answer (rated under `readers`). So outcome, grant, payload, error-shape and component-accepts changes all pass. README:68 says caller and reader verdicts don't matter; that is wrong for this store. | crates/control-plane-xtask/src/spec_history.rs:228 | introduced | blocker |
| F2 | `record-history` only checks that the old fixture opens; it never compares the views. A fixture that opens but replays into different state (exactly what a `history: compatible` `sets` change produces) gets overwritten. That contradicts README:104, which says it refuses to replace a history that no longer replays. | crates/control-plane-xtask/src/history.rs:35 | introduced | warning |
| F3 | An acknowledgement names only a change id. A later, different change with the same id goes through on the old review. | crates/control-plane-xtask/src/spec_history.rs:234 | introduced | warning |
| F4 | The baseline is fixed until someone moves it. A route that `main` publishes after the baseline and a later branch removes doesn't show up in the diff at all, and the gate then insists the old acknowledgement be deleted as stale. | crates/control-plane-xtask/src/spec_history.rs:216 | introduced | warning |
| F5 | The fixture covers every command, but only one refusal (`PauseGoal` wrong-state). Changes to any other recorded refusal are invisible to it, and with F1 the gate can't catch them either. | crates/control-plane-core/tests/recorded_history.rs:81 | introduced | note |

**F1 end to end** (scratch copy of `ab6f1c7`; regenerating from the unchanged spec reproduces `generated/model` byte for byte):
- **What I changed:** I removed the `wrong-state` outcome from `DisableRepositoryRegistration`.
- **What still passes:** `ess specify validate` passes and generation is complete (128/128). ESS rates the change `history=compatible`. `recorded_history` passes 2/2 and the core library tests pass 30/30.
- **What breaks:** a store written by HEAD code fails to open. It holds the committed fixture plus an operator disabling the same repository twice, which records refusal decision 51. Opening it gives `OPEN: refused: generated command refused (501): … unmet obligation: command behaviour controlplane.host.DisableRepositoryRegistration`.
- **How a real store gets there:** the operator console route accepts `DisableRepositoryRegistration` (crates/control-plane-app/src/lib.rs:126-133). No guard stops it, and declared refusals are committed (fixture decision 13 is one).

**F2 end to end:** changing `RepairAssignment` `attempt` increment from 1 to 2 is rated `history=compatible`. With it, `Store::open` opens the fixture, but `recorded_history_replays` fails with `AssignmentList after replay`. That is exactly the state where `record-history` overwrites.

**Fixes (named, not applied):**
- **F1:** treat a change as gating when any of `history`, `callers` or `readers` is not `compatible`, and correct README:68.
- **F2:** compare the old fixture's replayed views with the committed views file before overwriting.
- **F3:** tie an acknowledgement to the change's before/after values (`detail`, spec_history.rs:194), or use ESS's own digest-bound `--acknowledgements`.
- **F4:** diff against the merge base with `origin/main`, as the story's Evidence says. This is the decision for you.

**5. Attacked and could not break**
- **`ess` missing from PATH:** the gate exits 1 (`start released ESS CLI`), so it doesn't pass by default.
- **Baseline missing (shallow clone):** `cat-file` exits 128, so the gate errors rather than passing. CI uses `fetch-depth: 0`.
- **Files in `ess/` not listed in `ess-inputs.yaml`:** both ESS and generation ignore them, so there's nothing to gate.
- **Replay and edited fixture data:** a changed outcome payload (`disagrees`), a changed actor (`403 not granted`) and a changed recorded id (`disagrees`) are all refused. I probed these with a temporary test file and deleted it.
- **Changing a transition's `from` states:** always rated `history=unknown`, so it is gated.
- **`record-history` with a fixture that fails `Store::open`:** refuses, as designed.

**6. Paths written outside the worktree**
- `$HOME/.cache/cp-wave1/spec-history-gate/probe/` (1.6M): ESS diff probes and fixture copies
- `$HOME/.cache/cp-wave1/spec-history-gate/e2e/` (12M): copied tree, `regen.sh`, `double-disable.sqlite`, logs
- `$HOME/.cache/cp-wave1/spec-history-gate/adv-cases/` plus `adv-suite.log`, `adv-suite-nff.log`, `adv-suite-final.log`
- `/dev/shm/cp-wave1-spec-history-gate-target/adversary-e2e/` (250M): a separate cargo target dir inside the assigned build dir, used only by the end-to-end copy. Safe to delete.
- Inside the worktree but git-ignored: an empty `.scratch/spec-history-attack-tests/`.

The session lease has been released.

```findings
[
  {"file": "crates/control-plane-xtask/src/spec_history.rs", "line": 228, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Gating on the history verdict alone admits outcome, grant, payload and error-shape changes ESS rates history-compatible, although Store::open re-runs recorded calls and compares their answers; one such change left the fixture green while a store from a normal operator action could not open."},
  {"file": "crates/control-plane-xtask/src/history.rs", "line": 35, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "record-history checks only that the old fixture opens, not that its views still match, so it overwrites a fixture a history-compatible sets change has made replay differently, contrary to README:104."},
  {"file": "crates/control-plane-xtask/src/spec_history.rs", "line": 234, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "An acknowledgement bound only to a change id lets a later, different change with the same id through without review."},
  {"file": "crates/control-plane-xtask/src/spec_history.rs", "line": 216, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "With a fixed baseline, a route published on main after the baseline and removed later never appears in the diff, and the gate demands deleting its acknowledgement as stale."},
  {"file": "crates/control-plane-core/tests/recorded_history.rs", "line": 81, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Fixture coverage counts commands but records only one refusal, so changes to every other recorded refusal are invisible to the replay test."}
]
```
