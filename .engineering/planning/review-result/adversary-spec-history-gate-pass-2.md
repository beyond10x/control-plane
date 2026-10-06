---
format: aep.planning-md/3
id: review-result:adversary-spec-history-gate-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:spec-history-gate (red, 4 introduced)'
relations:
- reviews: story:spec-history-gate
revision: 1
---
unit: story:spec-history-gate
verdict: red
cases: executed 60→68, red 3
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/cp-wave1/spec-history-gate/pass2/, /dev/shm/cp-wave1-spec-history-gate-target/adversary-e2e/
needs-coordinator: yes

The unit is still red, and it is red in the parts the correction added. Findings cover worktree HEAD `600c126` plus my two untracked test files.

- **F6:** a branch can make an unreviewed breaking change pass by pointing the recorded baseline at its own commit. "The descendant wins" then picks that commit, and the gate passes with no acknowledgement.
- **F7:** an acknowledgement published with an earlier baseline is never reported as stale. It stays in the file indefinitely and admits a later repeat of the same change without a new review.
- **Decision for you:** the fix for F6 goes against your F4 rule, so it needs your call.

The exemption list and the restatement rule both held. I tested them on regenerated code, not just argued them.

**1. What I touched.** `git --no-pager diff --stat` prints nothing because both files are new and untracked. `git status --short` shows only test paths:
```
?? crates/control-plane-core/tests/recorded_history_pass2_attack.rs
?? crates/control-plane-xtask/tests/spec_history_pass2_attack.rs
```
`spec_history_pass2_attack.rs` pulls in `src/spec_history.rs` with `#[path]`, the same way pass 1 did. As a result, 5 of the 8 added executions are the module's own unit tests running a second time. rustfmt and `cargo clippy --all-targets -D warnings` are clean on both files (checked after a `touch` to force the recheck).

**2. Cases, each run alone first (exit 101 each)**

| case | red output (verbatim) |
|---|---|
| `recorded_baseline_moved_onto_the_branch_does_not_hide_its_change` | `a branch that removes Blocked from repair.from passed with no acknowledgement after pointing the recorded baseline at its own commit 781591a… (main is a76b69d…): specification history gate passed: 0 change(s) since baseline 781591a… (recorded in ess/spec-acknowledgements.json), 0 could break replay, 0 acknowledged` |
| `published_acknowledgement_does_not_admit_a_later_repeat_of_its_change` | `an acknowledgement reviewed for the first removal of wrong-state admitted a second removal, after stores had recorded wrong-state again: specification history gate passed: 1 change(s) since baseline 4d512d8… (merge base with main), 1 could break replay, 1 acknowledged` |
| `recorded_history_answers_every_declared_refusal_outcome` | `the recorded history answers 43 of 45 declared refusal outcomes; never answered: [("DeleteGoal", "running"), ("DeleteGoal", "satisfied")]` |

Before its final assertion, each gate case checks that its setup is one the gate accepts. The first case also checks that the committed removal is refused before the baseline moves. An independent count of the spec (`awk` over `host.yaml`) also gives 45 declared refusal outcomes.

**3. Gate, run after the cases**
```
control_plane_core lib            ok. 30 passed
recorded_history                  ok. 2 passed
recorded_history_pass2_attack     FAILED. 0 passed; 1 failed
control_plane_xtask main          ok. 12 passed
generation_ownership              ok. 4 passed
record_history_attack             ok. 1 passed
spec_history_attack               ok. 11 passed
spec_history_pass2_attack         FAILED. 5 passed; 2 failed
error: 2 targets failed:          TEST_EXIT=101
spec-history-check: specification history gate passed: 0 change(s) since baseline eaa37d4057b4c6b90c6cebccefa28dca4abd411e (recorded in ess/spec-acknowledgements.json), 0 could break replay, 0 acknowledged   GATE_EXIT=0
```
The before count of 60 comes from a second run with my two targets deselected (exit 0, same per-target counts).

**4. Findings** (tree `600c126`)

| # | finding | file:line | origin | severity |
|---|---|---|---|---|
| F6 | A recorded baseline that descends from the merge base wins, and nothing checks that the mainline contains it. A branch can point it at its own commit and pass with no acknowledgement. | crates/control-plane-xtask/src/spec_history.rs:200 | introduced | warning |
| F7 | A published acknowledgement is never stale, so it is never removed. A later change with the same id and the same `change` object is admitted on the old review. | crates/control-plane-xtask/src/spec_history.rs:479 | introduced | warning |
| F8 | Refusal coverage is counted per HTTP status, not per outcome. DeleteGoal's 409 carries `paused`, `running` and `satisfied`, and only `paused` is ever recorded. The commit message says "answers every declared refusal of every command". | crates/control-plane-core/tests/recorded_history.rs:153 | introduced | note |
| F9 | The README says a new view field changes the recorded views, so you should delete both fixture files. I measured an `Optional` view field, the only kind the gate exempts: the replayed views still match, so following the README throws away good evidence. | crates/control-plane-xtask/README.md:142 | introduced | note |

**What reaches each:**
- **F6:** README:120 documents moving the recorded baseline, and the refusal at spec_history.rs:212 says "move the recorded baseline". The live baseline today is `eaa37d4`, which is not on `origin/main`. `origin/main` is `268e6a1` "Initial commit" and has no `ess/`, so the recorded baseline is the only one in use, and any commit id put there is accepted.
  - Fix (needs your call): accept a recorded baseline over the merge base only when the mainline contains it (`merge-base --is-ancestor <recorded> origin/main`), and refuse otherwise.
- **F7:** the gate never asks anyone to remove a published acknowledgement. The duplicate-id rule doesn't help either, because restoring an outcome (`outcome-added`) has a different id from removing it (`outcome-removed`). The `change` object for an outcome removal is just `{kind, outcome}`, so two removals at different times look identical. I built this scenario; pass 1 measured `Store::open` refusing stores after exactly this removal.
  - Fix: require removing an acknowledgement once its change is published, as `stale_acknowledgement_fails_gate` literally states. Alternatively, tie each acknowledgement to the baseline commit it was reviewed against. The unit's own test `acknowledgement_published_with_the_baseline_is_not_stale` asserts the opposite of that acceptance text.
- **F8:** changes to DeleteGoal's `running` and `satisfied` outcomes are invisible to the fixture. The gate does catch outcome changes now, so this only weakens a second line of defence.
- **F9:** see the exemption-list row in part 5.

**5. Attacked and could not break**
- **Exemption list, measured:** I regenerated the model in a scratch copy of HEAD. The control run, with no edits, is byte-identical (0 differing files). With the four exempt changes from `additive_changes_pass_gate`, 23 model files change, and the committed fixture still opens with every view matching (2/2 tests pass).
- **Other changes ESS might pass:** probing with ESS showed every other candidate I tried is not compatible, so the gate still holds it:
  - an `Optional` input mapped into a payload is reported as `outcome-payload-changed` (unknown)
  - an unmapped event field is refused by ESS itself
  - an `Optional` error field is rated readers unknown
  - changes to `sets`, relations, view consistency and the initial state are all unknown

  New types, errors, views and grants can't change any recorded answer, because a 403 is never recorded.
- **Stored bodies:** every Input schema has `additionalProperties: false`, so a new `Optional` input can never pick up a value already sitting in a stored body.
- **Restatement rule:** it requires the outcome text to equal the original exactly after one route replacement. A route that now ends in `Queued` is still refused through the bound `change` (the pass-1 case stays green).
- **Baseline order:** `origin/main` is tried before `main`. A merge base newer than the recorded baseline wins. A recorded commit missing from the clone produces an error rather than a pass.
- **`record-history`:** it compares the views before it overwrites the fixture (the pass-1 case stays green).

**6. Paths written outside the worktree**
- `$HOME/.cache/cp-wave1/spec-history-gate/pass2/` (12M): `probe/` (ESS copies, `diff.sh`, `s*.pl`, a fixture copy, JSON output), `e2e/` (HEAD tree copy, `head-model/`, `gen/`, logs), `regen.sh`, `additive.pl`, and the logs `case-{a,b,c}.log`, `suite.log`, `suite-deselected.log`, `gate.log`.
- `/dev/shm/cp-wave1-spec-history-gate-target/adversary-e2e/` (250M): pass 1's separate target dir, reused for my regeneration builds. Safe to delete.
- Inside the worktree but git-ignored: an empty `.scratch/spec-history-pass2-attack-tests/`.
- I didn't acquire a session lease on this tree, so there is none to release. That departs from the procedure.

```findings
[
  {"file": "crates/control-plane-xtask/src/spec_history.rs", "line": 200, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A recorded baseline that descends from the merge base wins without any check that the mainline contains it, so a branch that points it at its own commit passes an unacknowledged removal of Blocked from repair.from."},
  {"file": "crates/control-plane-xtask/src/spec_history.rs", "line": 479, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Acknowledgements published with the baseline are exempt from staleness and never removed, so an old review of an outcome removal admits a later identical removal after stores have recorded that outcome again."},
  {"file": "crates/control-plane-core/tests/recorded_history.rs", "line": 153, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Refusal coverage counts one outcome per response status, so DeleteGoal running and satisfied (43 of 45 declared refusal outcomes recorded) are never replayed, contrary to the commit's every-declared-refusal claim."},
  {"file": "crates/control-plane-xtask/README.md", "line": 142, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The README tells you to delete both fixture files after a new view field, but a regenerated model with an Optional view field still replays the committed fixture to identical views, so following it throws away good evidence."}
]
```
