---
format: aep.planning-md/3
id: review-result:adversary-console-component-tests-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:console-component-tests (956ab67): red, 3 introduced notes'
relations:
- reviews: story:console-component-tests
revision: 1
---
unit: story:console-component-tests
verdict: red
cases: executed 56→62, red 1
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-console-component-tests-scratch/adv2/, /dev/shm/cp-wave2-console-component-tests-scratch/npm-cache, /dev/shm/cp-wave2-console-component-tests-scratch/npm-logs, /dev/shm/cp-wave2-console-component-tests-scratch/tmp/node-compile-cache, /dev/shm/cp-wave2-console-component-tests-target, worktree lease record for session adv-console-component-tests-pass-2
needs-coordinator: yes

The fixes for pass-1 findings 1, 2, 3, 4 and 6 hold. One new red case remains: the gate passes when the acceptance test is marked `test.fails` and the console is broken. All three findings are notes, none is a blocker, and nothing in the repository reaches any of them today. They cover head 956ab67 plus one new test file that is not committed.

**1. What I changed in the worktree**

`git --no-pager diff --stat` is empty. `git status --short` shows one line:
```
?? crates/control-plane-xtask/tests/frontend_check_pass2_attack.rs
```
That is a test file. I changed no implementation file, wrote nothing under `.engineering/` and made no commit.

**2. The cases I added** (in `frontend_check_pass2_attack.rs`, using the same copy-and-link setup as `frontend_check.rs`)

| Case | What it plants in the copy | Now |
|---|---|---|
| `inverted_acceptance_test_fails_the_gate` | Removes the goal from the `.sse` payload. A control step checks the gate refuses that. Then marks the acceptance test `test.fails` | **red** |
| `passing_with_no_tests_fails_the_gate` | `passWithNoTests: true` and no test file | green; red against mutant M1 |
| `report_is_read_back_under_a_root_with_spaces` | Root path contains spaces; then a `test.skip` | green |
| `passing_suite_plus_skipped_spec_file_fails_the_gate` | Passing `App.test.js` plus `src/Later.spec.ts` whose only suite is `describe.skip` | green |
| `tests_outside_the_frontend_are_not_collected` | Failing tests in the xtask fixture directory and in `.scratch/` | green |

I ran the exact CI install (`npm ci --prefix frontend --ignore-scripts --no-audit --no-fund`, "added 83 packages"), then the cases alone with `cargo test --locked -p control-plane-xtask --test frontend_check_pass2_attack` (exit 101):
```
 Test Files  1 passed (1)
      Tests  1 expected fail (1)
JSON report written to .../.scratch/frontend-build-Q8TW9L/vitest.json
Vue component tests passed
Error: frontend-check passed although app_renders_a_snapshot_payload is marked test.fails, so a console that no longer renders the recorded goal passes it
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.43s
```

**3. The suite run, after the cases existed** (the brief's gate, script `adv2/gate.sh`)
```
### EXIT 0 : npm ci --prefix frontend --ignore-scripts --no-audit --no-fund
embedded Vue assets match the source build
 Test Files  1 passed (1)
      Tests  1 passed (1)
### EXIT 0 : cargo run --locked -p control-plane-xtask -- frontend-check
### EXIT 0 : cargo fmt -p control-plane-xtask --check
### EXIT 0 : cargo clippy --locked -p control-plane-xtask --all-targets -- -D warnings
### EXIT 101 : cargo test --locked -p control-plane-xtask --no-fail-fast
```
- **Per test binary:** main 14, frontend_check 6, frontend_check_attack 9, frontend_check_pass2_attack 5 passed and 1 failed, generation_ownership 4, record_history_attack 2, spec_history_attack 12, spec_history_pass2_attack 8.
- **Count:** before is 55 Rust cases plus 1 component test, from the implementor's `r1-gate.log`. After is 61 plus 1.
- **No writes:** `find frontend -newer marker` listed nothing after the suite, and `git status --ignored` was the same before and after.
- **`frontend/dist`:** `sha256sum -c` gives OK for app.js, index.css and index.html.

**Mutant probes**

These ran in a standalone crate in scratch that compiles copies of the source; the worktree was only read.
- **M1:** with the line at `frontend.rs:66` changed from `passed > 0,` to `true,`, frontend_check (6) and frontend_check_attack (9) stayed green. Only my `passing_with_no_tests_fails_the_gate` turned red.
- **M2:** I changed the test helper back to linking the whole `node_modules` directory, as in pass 1. The implementor's `component_tests_leave_the_real_frontend_untouched` went red, listing `node_modules/.vite/vitest/da39a3ee…/results.json` among others, so that test does catch the pass-1 problem.

**4. Findings**

| file:line | What was measured | Verdict / origin | What reaches it |
|---|---|---|---|
| `crates/control-plane-xtask/src/frontend.rs:94` | Vitest's JSON report records a `test.fails` test as `"passed"` and carries no mark of the inversion. So a broken console plus a `test.fails` acceptance test passes the gate. Vitest's own summary line does say "1 expected fail". | INFEASIBLE / introduced | Nothing: no test in the repository uses `.fails`. Possible fixes: refuse when the summary reports an expected fail, or refuse `.fails(` in test sources. |
| `crates/control-plane-xtask/src/frontend.rs:66` | The "no test passed" check is never exercised by the unit's suite (mutant M1 survives). | CONFIRMED / introduced | The check can only fire if `passWithNoTests` is set, which nothing sets. Fix: adopt `passing_with_no_tests_fails_the_gate`. |
| `README.md:101` | The README names `npm run test --prefix frontend` as the gate's test step. Run on its own with the acceptance test skipped, that command exits 0 ("Tests 1 skipped (1)"). The skip check exists only inside `frontend-check`. | CONFIRMED / introduced | A developer reproducing a gate refusal by hand. Fix: name `frontend-check` as the command that refuses skipped tests. |

**5. What I attacked and could not break**

- **Missing or stale report:** each run writes its report into a fresh temporary directory, so a stale report is impossible. A missing report or invalid JSON is refused.
- **Unknown or missing status:** any status other than `"passed"` is refused, so these fail safe. Vitest's own fallback for an unknown state is "skipped".
- **Reporter crash:** Vitest either exits non-zero, which is refused, or writes no report, which is also refused. I did not trigger an actual crash; this rests on reading the code.
- **Removing `include`:** Vitest's default exclude in 5.0.3 is `**/node_modules/**` and `**/.git/**`, and nothing under `node_modules` was collected. The xtask fixtures and `.scratch/` are not collected (my case is green). `dist/` **is** collected (`vitest list` showed `dist/planted.test.js`), but the drift check refuses any extra file in `dist` before the tests run, so nothing reaches this through the gate.
- **Linking after the exact CI `npm ci`:** `.bin` resolves and every copy run passes. The real `node_modules` received no `.vite` writes from copy runs. I tested only npm 12.0.2; CI's bundled npm 10 is untested.
- **Root path with spaces:** the report path reaches Vitest as one argument and is read back.
- **`frontend/dist`:** byte-identical after the gate.
- **README's Node, npm and `npm ci` sentence for `cargo test`:** accurate; `task check` and CI both run `npm ci` before `cargo test`.

**6. Paths I wrote outside the worktree**

- `/dev/shm/cp-wave2-console-component-tests-scratch/adv2/`: `env.sh`, `cases.sh`, `gate.sh`, `mutants.sh`, `probes.sh`, `cases.log`, `gate.log`, `mutants.log`, `probes.log`, `status-before.txt`, `status-after.txt`, `dist-before.sha256` and `marker-suite`. I already deleted `mutant/` and `mutant-target/` (101 MB).
- `/dev/shm/cp-wave2-console-component-tests-scratch/npm-cache`, `npm-logs` and `tmp/node-compile-cache`: shared with the earlier runs. The temporary test directories were removed when the tests finished.
- `/dev/shm/cp-wave2-console-component-tests-target`: my test binary was built here.
- Inside the worktree, ignored by git: `npm ci` reinstalled `frontend/node_modules` twice, and `frontend-check` wrote `node_modules/.vite` and `.vite-temp`.
- Worktree lease: I took a lease as session `adv-console-component-tests-pass-2` and released it.

**Needs a decision:** whether to keep the red `test.fails` case and route it back for a gate fix, or accept the gap and park the case.

```findings
[{"file":"crates/control-plane-xtask/src/frontend.rs","line":94,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"Vitest's JSON report marks a test.fails test whose body throws as passed with no trace of the inversion, so a broken console with app_renders_a_snapshot_payload marked test.fails passes frontend-check; no test in the repository uses test.fails"},
{"file":"crates/control-plane-xtask/src/frontend.rs","line":66,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"replacing the passed > 0 refusal with true leaves frontend_check.rs and frontend_check_attack.rs green; only passing_with_no_tests_fails_the_gate observes it, and only passWithNoTests reaches it"},
{"file":"README.md","line":101,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the README names npm run test --prefix frontend as the step that fails on skipped or todo tests, but that command exits 0 with the acceptance test skipped; only frontend-check refuses"}]
```
