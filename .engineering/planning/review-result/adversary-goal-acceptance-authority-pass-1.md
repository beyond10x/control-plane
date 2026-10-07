---
format: aep.planning-md/3
id: review-result:adversary-goal-acceptance-authority-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:goal-acceptance-authority at 95274aa'
relations:
- reviews: story:goal-acceptance-authority
revision: 1
---
unit: story:goal-acceptance-authority; findings cover 95274aa plus my two untracked test files
verdict: CONFIRMED
cases: executed 150→153, red 2
origin: introduced 1 / pre-existing 0 / undecided 2
wrote-outside-worktree: 3 paths (scratch logs dir, assigned build dir, scratch tmp now empty)
needs-coordinator: findings 1 and 2 are outside the story's UpdateGoal acceptance. Decide whether they hold this unit or become their own story. My two red runtime cases keep `cargo test -p control-plane-runtime` red until you decide.

**1. Diff stat** (worktree `$HOME/.local/state/worktree/trees/b10x/control-plane/cp-wave3-goal-acceptance-authority`)
```
$ git --no-pager diff --stat
(empty: no tracked file changed)
$ git status --short
?? crates/control-plane-core/tests/goal_acceptance_authority_attack.rs      (122 lines)
?? crates/control-plane-runtime/tests/goal_acceptance_authority_attack.rs   (435 lines)
```
Both paths are test files. No implementation file was touched.

**2. Cases added** (written before anything ran; each case run alone)

| File | Case | Asserts | Now |
|---|---|---|---|
| runtime attack | `configuration_change_after_final_check_is_not_satisfied` | An operator `ConfigureRepository` (test_command → `false`) lands after acceptance's last configuration comparison and before `SatisfyGoal`. The case first checks it landed while the goal was Running. Then it asserts the goal is not Satisfied. | red |
| runtime attack | `pause_during_goal_review_does_not_latch_acceptance_after_resume` | PauseGoal during the goal review, then StartGoal, then 2× tick + fleet_tick. Asserts the goal is Satisfied. | red |
| core attack | `only_the_exact_revision_is_admitted_and_other_goals_keep_declared_refusals` | Receipts naming a future revision (2), `1.0` or `null` are refused with the exact messages and record no decision. Cancelled → `wrong-state` and unknown → `not-found` for receipts `not json`, `[1]` and revision 99. | green |

Red output, each case alone (before the suite, before a whitespace-only rustfmt). Case A's goal row is elided because it contains absolute home paths:
```
test configuration_change_after_final_check_is_not_satisfied ... FAILED
thread '…' panicked at crates/control-plane-runtime/tests/goal_acceptance_authority_attack.rs:344:5:
assertion `left == right` failed: goal satisfied although its repository's check changed after the final comparison; report Ok(TickReport { planned: 0, queued: 0, blockers: [], merged: 1, satisfied: 1 }); goal {[elided: "revision":1,"state":"Satisfied","satisfaction_receipt":"{\"goal_revision\":1,\"kind\":\"goal_acceptance\",…"]}
  left: String("Satisfied")
 right: "Running"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 11.46s
EXIT=101

test pause_during_goal_review_does_not_latch_acceptance_after_resume ... FAILED
thread '…' panicked at crates/control-plane-runtime/tests/goal_acceptance_authority_attack.rs:421:5:
assertion `left == right` failed: resumed goal never re-reviewed (1 goal reviews); acceptance {"at":"2026-10-06T09:56:57Z","fingerprint":"d4202f…3e26","goal_revision":1,"reason":"Goal acceptance blocked: goal changed during final review","status":"failed"}
  left: String("Running")
 right: "Satisfied"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 7.40s
EXIT=101
```
After the rustfmt (whitespace only), I re-ran the runtime attack binary: both cases still red, now at :352 and :429, exit 101. The core case passed alone: `1 passed`, exit 0.

**3. Suite** (run after the cases existed)
`cargo test --locked --no-fail-fast -p control-plane-core -p control-plane-runtime` → EXIT=101
```
core lib 44 passed | core goal_acceptance_authority_attack 1 passed | recorded_history 2 | recorded_history_pass2_attack 1 | terminal_goal_edits_attack 1 | terminal_goal_edits_pass2_attack 4
runtime lib 38 passed | fleet 24 passed | runtime goal_acceptance_authority_attack 0 passed; 2 failed | planner 36 passed | doc-tests 0, 0
```
The 150 "before" comes from this same run with my two binaries' lines (1 + 2 cases) left out. I did not make a separate deselected run. My two files also pass `rustfmt --check` and `cargo clippy --tests -D warnings` (exit 0).

**4. Findings** (these cover 95274aa)

| file:line | Finding | Verdict | Origin | Severity | What was measured / what reaches it |
|---|---|---|---|---|---|
| `crates/control-plane-runtime/src/fleet.rs:2099` | The repository/directory comparison after the review has the same check-then-release gap this unit closed for the goal revision. A configuration change during the target observation (`git ls-remote`, :2103-2116) is not seen, and `SatisfyGoal` (:2122) goes through. The goal ends Satisfied under a test command that is no longer configured. | CONFIRMED | undecided (not run at base; the only fleet.rs change since base is a 3-line comment) | warning | **Measured:** runtime attack :352 red. **Reaches it:** operator ConfigureRepository, RegisterRepository or AddWorkspaceDirectory (Operator grants, host.yaml:320-333) via CLI or HTTP, which take the same store mutex. I made the window wider with a delayed upload-pack so the test is deterministic; the interleaving itself is real. **Fix:** observe targets before the store comparisons, then run the comparisons and `SatisfyGoal` under one lock. |
| `crates/control-plane-runtime/src/fleet.rs:2149` | A pause during acceptance is recorded as a `failed` acceptance for an unchanged fingerprint. The latch (`acceptance_is_unchanged`, :703-715; supervisor.rs:96-110) then blocks both acceptance and planning after resume. The goal stays Running with 1 goal review, and only an UpdateGoal gets it moving again. | CONFIRMED | undecided | warning | **Measured:** runtime attack :429 red. **Reaches it:** operator PauseGoal then StartGoal while acceptance runs, which takes minutes (checks plus model review). Read but not run: cancellation at service shutdown ("goal acceptance cancelled") goes through the same error arm. **Fix:** record interruptions (goal not Running, cancellation) without the `failed` latch status. |
| `crates/control-plane-core/tests/terminal_goal_edits_attack.rs:70` | The unit edited two existing wave-2 attack files (also `terminal_goal_edits_pass2_attack.rs:96`). The brief only assigns *new* files under `tests/`. The edit only swaps the receipt input to a JSON receipt at revision 1, and the assertions are unchanged. | CONFIRMED | introduced | note | From reading the diff. The edit is needed because the new guard refuses plain-string receipts. |

**5. Attacked and could not break**
- **UpdateGoal in the window:** guard and apply run under one `&mut Store` (lib.rs:283-284), and the fleet holds the mutex across that call (fleet.rs:142). Every applied edit increments the revision (host.yaml:1061), so an edit can never restore an earlier revision number.
- **The unit's fleet acceptance test:** passed 15 times in 15 sequential runs (5.4–7.1 s). By construction it fails at base: it asserts the admission message and the absence of "goal changed during final review".
- **Exact equality:** the unit's suite alone could not tell `==` from `>=` at guards.rs:225. My core case can. I did not run it against a mutated build (no second build dir allowed), and no caller can produce a future revision anyway.
- **Replay:** `Store::open` replays without guards (lib.rs:213-219). A temporary probe (since removed) opened the base fixture from 9691f41, which has plain-string receipts, under 95274aa; it matched all 6 base views.
- **Other SatisfyGoal senders:** only fleet.rs:2122 and xtask history.rs. HTTP refuses SatisfyGoal (403, app tests.rs:586). ContractStore is conformance-only.
- **After the UpdateGoal refusal:** the error arm skips the latch because the revision changed, and revision 2 is re-planned. The acceptance record stays `running` at revision 1; the latch only reacts to `failed` (fleet.rs:710).
- **Receipt parsing:** serde_json's recursion limit is out of reach, because the review schema is `{approved, reason}` (model.rs:63) and the observations are strings.

**6. Paths written outside the worktree**
- `/dev/shm/cp-wave3-goal-acceptance-authority-scratch/adversary/logs/`: 8 logs (core-alone, runtime-A-alone, runtime-B-alone, suite, flake, probe-base-history, clippy, runtime-after-fmt).
- `/dev/shm/cp-wave3-goal-acceptance-authority-target`: the assigned build dir (my two test binaries, clippy metadata).
- `/dev/shm/cp-wave3-goal-acceptance-authority-scratch/tmp`: used by the runs, now empty.
- Already deleted: `adversary/probe-uploadpack`, `adversary/base-history`, and the `.scratch/{fleet,planner,goal-acceptance-attack}` dirs my runs created inside the worktree. `.scratch` is empty.
- Worktree lease `adversary-goal-acceptance-authority-w3p1`: acquired and released.

**7. Findings block**
```findings
- file: crates/control-plane-runtime/src/fleet.rs
  line: 2099
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: The post-review repository and directory comparison leaves the store before SatisfyGoal, so a ConfigureRepository landing during the target observation is swallowed and the goal is Satisfied under a check that is no longer configured.
- file: crates/control-plane-runtime/src/fleet.rs
  line: 2149
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: A pause during goal acceptance is recorded as a failed acceptance for an unchanged fingerprint, so after resume the durable latch blocks both acceptance and planning and the goal stays Running until an edit.
- file: crates/control-plane-core/tests/terminal_goal_edits_attack.rs
  line: 70
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: The unit edited two existing wave-2 attack test files outside its file assignment; only the receipt input changed and no assertion was weakened.
```
