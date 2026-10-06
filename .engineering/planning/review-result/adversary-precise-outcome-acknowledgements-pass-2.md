---
format: aep.planning-md/3
id: review-result:adversary-precise-outcome-acknowledgements-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:precise-outcome-acknowledgements at 6a2482c'
relations:
- reviews: story:precise-outcome-acknowledgements
revision: 1
---
unit: story:precise-outcome-acknowledgements, control-plane worktree at 6a2482c plus one untracked test file
verdict: CONFIRMED
cases: executed 102→114, red 1
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: decide whether F1 (left over from pass-1 F1) gets fixed in this unit or recorded as a known limit, and whether to keep the red case

**1. Diff proof**

```
$ git status --short --untracked-files=all
?? crates/control-plane-xtask/tests/precise_outcome_acknowledgements_pass2_attack.rs
$ git --no-pager diff --stat
(empty: no tracked file changed)
```

The only path is one new test file (393 lines, no `/home/` path in it). `rustfmt` ran on that file only; `spec_history.rs`, which it reaches through `#[path]`, came out unchanged. I did not acquire a worktree session lease.

**2. Cases added**

All in `crates/control-plane-xtask/tests/precise_outcome_acknowledgements_pass2_attack.rs`.

| case | what it asserts | now |
|---|---|---|
| `dropping_a_later_declared_refusal_that_answers_first_is_refused` (:361) | The baseline is the tree before terminal-goal-edits. In the reviewed state, `too-many-workers` (`when: max_workers > 100`, `GoalWorkerLimit`) is declared after `cancelled`, and all 4 changes are acknowledged from compiler output. The generated `update_goal` is byte-identical to the control's and answers `TooManyWorkers` before `Satisfied`. Then `too-many-workers` is dropped and its entry removed, as the gate's "stale … remove it" asks. `satisfied`'s `outcome` and `preceded_by` are unchanged, but it now answers updates of a Satisfied goal that `too-many-workers` used to answer. The gate must refuse `satisfied`. | **red** |
| `dropping_an_earlier_declared_refusal_that_answers_first_is_refused` (:322) | Control: the same behaviour, with `too-many-workers` declared before `satisfied`. The same drop is refused for `satisfied` and `cancelled` (`preceded_by: reviewed ["applied","too-many-workers"], the model now says ["applied"]`). The gate's printed ready entries, used exactly as printed, are then accepted (3 acknowledged). | green |

Every assertion before the final one passed: `ess specify validate --strict-requires` accepted both states, all capabilities were generated, the two `update_goal` bodies are equal, and the stale entry was named. Red output, from running this case alone before the suite:

```
running 1 test
test dropping_a_later_declared_refusal_that_answers_first_is_refused ... FAILED
thread 'dropping_a_later_declared_refusal_that_answers_first_is_refused' (2554955) panicked at crates/control-plane-xtask/tests/precise_outcome_acknowledgements_pass2_attack.rs:379:13:
the acknowledgement of satisfied, reviewed while too-many-workers answered every update asking for more than 100 workers first, admitted satisfied answering those updates of a Satisfied goal itself after too-many-workers was dropped; the same drop is refused when too-many-workers is declared before satisfied, with the same generated update_goal: specification history gate passed: 3 change(s) since baseline 9722e5df8d1a33ca2e45cf7424865245db41700b (merge base with main), 3 could break replay, 3 acknowledged
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 7.49s
EXIT=101
```

The control, run alone: `test result: ok. 1 passed; … 11 filtered out`, EXIT=0.

**3. Suite run** (after both cases existed; `npm ci --prefix frontend …` exited 0 first)

```
$ cargo test --locked --no-fail-fast -p control-plane-xtask
unittests src/main.rs                       ok. 18 passed
acceptance_traceability                     ok. 3 passed
acceptance_traceability_attack              ok. 7 passed
acceptance_traceability_pass2_attack        ok. 4 passed
frontend_check                              ok. 7 passed
frontend_check_attack                       ok. 9 passed
frontend_check_pass2_attack                 ok. 6 passed
generation_ownership                        ok. 4 passed
precise_outcome_acknowledgements_attack     ok. 14 passed
precise_outcome_acknowledgements_pass2_attack  FAILED. 11 passed; 1 failed
record_history_attack                       ok. 2 passed
spec_history_attack                         ok. 16 passed
spec_history_pass2_attack                   ok. 12 passed
error: 1 target failed:   EXIT=101
```

- 114 cases ran. The 102 "before" is this same run without my test binary, which holds 12 cases: my 2 plus the 10 module tests it compiles in by path. 102 also equals pass 1's 98 plus 4 copies of the new `added_outcome_acknowledgement_binds_its_position`.
- `cargo fmt -p control-plane-xtask --check` exited 0, and so did `cargo clippy --locked -p control-plane-xtask --all-targets -- -D warnings`.
- `spec-history-check` on this tree: `specification history gate passed: 3 change(s) since baseline eaa37d4… (recorded in ess/spec-acknowledgements.json, on origin/control-plane/bootstrap), 3 could break replay, 3 acknowledged`, EXIT=0.

**4. Findings** (all against 6a2482c)

- **F1, `crates/control-plane-xtask/src/spec_history.rs:565` — INFEASIBLE, pre-existing, warning.**
  - **What fails:** `preceded_by` lists only the outcomes declared before the added one. ESS 0.53.0 answers input-guarded refusals first, wherever they are declared (`ess-synth/src/rust/behaviour.rs`, "The order of evaluation"). So an outcome declared after the reviewed one can still answer first.
    - If such an outcome is itself an added outcome and is later dropped, its change leaves the diff. The gate only asks to remove its entry.
    - The reviewed outcome then answers more calls than it did when reviewed, and the gate passes.
    - The same drop of an outcome declared earlier is refused, even though the generated behaviour is identical (the control case).
  - **Measured:** :361 is red and :322 is green, as quoted in part 2.
  - **Origin:** I ran the same case against base 9691f41's gate (format /1, entries hold the change only) in a scratch harness. It is red there too, with the same panic and "3 acknowledged". The correction narrowed this gap but did not close it.
  - **What reaches it:** nothing today. `ess/domains/host.yaml` declares 0 `when:` outcomes. It needs one unpublished branch that adds an input-guarded refusal after an added subject-guarded outcome, acknowledges both, and then drops the refusal.
  - **Fix to consider:** record the command's full ordered outcome-name list next to `preceded_by`, so that adding, dropping or moving any outcome of the command re-opens every added-outcome entry of that command. The alternative is to bind the outcomes ESS evaluates before it, which ties the gate to ESS's ordering rules.
- **F2, `crates/control-plane-xtask/README.md:131` (also `spec_history.rs:52` and the test comment at `:1187`) — CONFIRMED, introduced, note.**
  - **What fails:** these lines say "A command answers with the first outcome whose condition holds" / "in declaration order". That is false for ESS 0.53.0. My case asserts that the generated `update_goal` is byte-identical for both declaration orders and checks `TooManyWorkers` before `Satisfied`. In the live tree, `not-found` is declared last and answered before any row-guarded branch.
  - **Consequence:** `preceded_by` misses some outcomes (F1) and is also stricter than needed. The unit's own position test refuses a swap of `satisfied` and `cancelled`. Their states are disjoint, so the swap cannot change any answer (I inferred this from the conditions; I did not generate it).
  - **Fix:** correct the stated reason at all three places.

**5. Attacked and could not break**

- Pass 1's red case is now green (14 passed in its binary). The coordinator-accepted fixture edit only adds `preceded_by` read from the compiler; the assertion is unchanged.
- The one-line fixture in `spec_history_pass2_attack.rs` (`preceded_by: ["applied"]` for `wrong-state`) matches the compiler, because that case's `check(root)?` passes on it. No assertion changed.
- Ready-entry round trip after a position change: the gate accepts its own printed entries (control case).
- Reordering outcomes among the ones before an added outcome is caught either way: ESS reports `outcome-order-changed` when both outcomes are declared on both sides, and an added one's own `preceded_by` changes otherwise. This is from reading, not run.
- Outcome groups: ESS appends group outcomes in group-name order (`docs/design/outcome-groups.md`). A group rename therefore moves an added group outcome, which `preceded_by` catches. This is from reading, not run.
- Reader: a missing or malformed `preceded_by`, or `preceded_by` on a non-added entry, is refused (unit tests). Mutants that drop `preceded_by` from the comparison or from the ready entry are caught by `binds_its_position` (from reading, not run).

**6. Paths written outside the worktree**

- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-scratch/adversary-pass2/logs/` (36K, kept): `red-alone.log`, `control-alone.log`, `base-red.log`, `suite.log`, `gates.log`, `npm.log`.
- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-scratch/adversary-pass2/harness/` (56K, kept): `spec_history_base.rs` (9691f41 with its tests module removed), `base_pass2.rs` (my case for format /1; it contains absolute worktree paths), `Cargo.toml`, `Cargo.lock`.
- `/dev/shm/cp-wave3-precise-outcome-acknowledgements-target` (the assigned build directory): my test binary is there. `cargo clean -p adversary-harness` removed the harness artifacts (6 files, 9.3 MiB).
- Deleted: `adversary-pass2/probe`, `adversary-pass2/fixtures`, `adversary-pass2/compiled.json`, and `tmp/node-compile-cache`. `tmp/` is empty.
- In the worktree, git-ignored: `frontend/node_modules` was removed. I also removed the 4 `.scratch/` subdirectories my runs created (`conformance-adversary`, `conformance-tests`, `integer-codec`, `precise-outcome-acknowledgements-pass2-attack-tests`). The 4 `.scratch/` subdirectories that existed before I started were left in place, and they are empty.

**7. Findings block**

```findings
- file: crates/control-plane-xtask/src/spec_history.rs
  line: 565
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: preceded_by binds only outcomes declared before the added one, so dropping an added input-guarded refusal declared after it (which ESS answers first) widens the reviewed outcome and the gate admits it, while the same drop declared earlier is refused for identical generated behaviour
- file: crates/control-plane-xtask/README.md
  line: 131
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: README, the AddedOutcome doc and the position test state that outcomes are decided in declaration order, but ESS 0.53.0 answers input-guarded refusals first, so the generated update_goal is byte-identical for both declaration orders
```
