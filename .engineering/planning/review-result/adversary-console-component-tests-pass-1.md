---
format: aep.planning-md/3
id: review-result:adversary-console-component-tests-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:console-component-tests (15973c6): red, 6 introduced'
relations:
- reviews: story:console-component-tests
revision: 1
---
unit: story:console-component-tests
verdict: red
cases: executed 44→53, red 4
origin: introduced 6, pre-existing 0, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-console-component-tests-scratch/adv/ (scripts, logs, a database copy), /dev/shm/cp-wave2-console-component-tests-scratch/npm-cache and npm-logs, /dev/shm/cp-wave2-console-component-tests-scratch/tmp/node-compile-cache, /dev/shm/cp-wave2-console-component-tests-target
needs-coordinator: yes

Findings cover head 15973c6 plus one untracked test file. The gate fails on a real failing test, but it passes when no test runs at all, and one acceptance name has no test.

**1. What I changed in the worktree**

`git --no-pager diff --stat` is empty because the only change is a new, uncommitted file. `git status --short`:
```
?? crates/control-plane-xtask/tests/frontend_check_attack.rs
```
It is a test file. I changed no implementation file and wrote nothing under `.engineering/`.

**2. The cases I added**

All are in `crates/control-plane-xtask/tests/frontend_check_attack.rs`. Each case copies `frontend/` to a temporary directory, links the installed `node_modules`, changes the copy only, and runs `frontend::run(root, false)`. A case passes only if the gate fails inside the component tests.

| Case | What it changes in the copy | Now |
|---|---|---|
| `skipped_acceptance_test_fails_the_gate` | marks `app_renders_a_snapshot_payload` as `test.skip` | red |
| `todo_only_suite_fails_the_gate` | the only test left is a `test.todo` | red |
| `failing_spec_file_fails_the_gate` | adds `src/GoalForm.spec.js` with a failing test | red |
| `every_acceptance_scenario_names_a_test` | none; checks that each of the story's three names is a Rust test function or a `*.test.js` title | red |
| `removed_suite_fails_the_gate` | deletes `App.test.js` | green |
| `test_file_without_tests_fails_the_gate` | adds a test file with no tests | green |
| `test_file_failing_at_import_fails_the_gate` | adds a test file that throws when imported | green |
| `focused_test_fails_the_gate` | marks the test `test.only` | green |

Output of the cases run alone, before the suite (`cargo test --locked -p control-plane-xtask --test frontend_check_attack`, exit 101):
```
Error: story:console-component-tests names scenarios that no test carries: ["component_tests_run_in_the_gate"]
 Test Files  1 skipped (1)
      Tests  1 todo (1)
Vue component tests passed
Error: frontend-check passed although the only component test left is test.todo('app_renders_a_snapshot_payload'), so no test ran
 Test Files  1 skipped (1)
      Tests  1 skipped (1)
Vue component tests passed
Error: frontend-check passed although the acceptance test app_renders_a_snapshot_payload is marked test.skip, so nothing observes it
Error: frontend-check passed although frontend/src/GoalForm.spec.js holds a failing test (vitest.config.js include is src/**/*.test.js only)
test result: FAILED. 5 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s
```

**3. The suite run, after the cases existed**

I ran the brief's package gate (`adv/gate.sh`):
```
### EXIT 0 : npm ci --prefix frontend --ignore-scripts --no-audit --no-fund
embedded Vue assets match the source build
 Test Files  1 passed (1)
      Tests  1 passed (1)
### EXIT 0 : cargo run --locked -p control-plane-xtask -- frontend-check
### EXIT 0 : cargo fmt -p control-plane-xtask --check
### EXIT 0 : cargo clippy --locked -p control-plane-xtask --all-targets -- -D warnings
### EXIT 101 : cargo test --locked -p control-plane-xtask
```
`cargo test` stops at the first failing test binary, so four later binaries never ran. To get a full count I re-ran with `--no-fail-fast` (exit 101):

| Test binary | Result |
|---|---|
| main.rs unit tests | 14 passed |
| `frontend_check.rs` | 3 passed |
| `frontend_check_attack.rs` | 5 passed, 4 failed |
| `generation_ownership.rs` | 4 passed |
| `record_history_attack.rs` | 2 passed |
| `spec_history_attack.rs` | 12 passed |
| `spec_history_pass2_attack.rs` | 8 passed |

The before count of 44 is 43 Rust cases plus 1 component test, from the implementor's `gate-summary.txt`. The after count of 53 adds my 8 cases plus one test from `generation.rs`, which my test file includes the same way the implementor's does.

**4. Findings**

| file:line | What breaks | Verdict / origin | Who reaches it |
|---|---|---|---|
| `crates/control-plane-xtask/src/frontend.rs:47` | The gate only checks the exit status. A suite where every test is `.skip`, `.todo` or `skipIf(true)` prints "Vue component tests passed" with 0 tests run. The planned name check (story:acceptance-traceability) would still count the skipped title. | NEEDS-CHANGE / introduced | Anyone who skips a test that broke. Suggested fix: run Vitest with its JSON reporter as well and refuse when any test is skipped or todo, or when none passed. `serde_json` is already a dependency of the xtask crate. |
| `.engineering/planning/story/console-component-tests.md:35` | The scenario name `component_tests_run_in_the_gate` matches no test. The test that covers it is called `component_tests_run_after_the_drift_check` (`frontend_check.rs:82`). | NEEDS-CHANGE / introduced | `task check`, once the acceptance-traceability check is switched on. Fix: rename the test, or the coordinator rewrites the story name. |
| `frontend/vitest.config.js:3` | The test file pattern is `src/**/*.test.js` only, so a failing `*.spec.js` or `*.test.ts` file never runs and the gate stays green. | CONFIRMED / introduced | A later console story using Vitest's default naming. Fix: drop the `include` override. |
| `crates/control-plane-xtask/tests/frontend_check.rs:56` | The linked `node_modules` makes the test write into the real `frontend/node_modules/.vite/vitest/<hash>/results.json`. After the suite it lists `:src/failing.test.js` as failed, a file that exists only in the temporary copy. It affects only the order tests run in, and Vitest ignores read errors on that file. | CONFIRMED / introduced | Every `cargo test` run |
| `crates/control-plane-xtask/tests/frontend_check.rs:77` | `assert_eq!(exit_status(result), ExitCode::FAILURE)` holds for any error, so it cannot fail. The test never runs the `frontend-check` command itself, so it would stay green if `main.rs:62` called the build instead. | INFEASIBLE / introduced | The binary fixes its repository root at build time, so it cannot be pointed at a copy. Only a mutant reaches this. |
| `README.md:101` | `cargo test -p control-plane-xtask` (and `--workspace`) now needs Node, npm and `npm ci --prefix frontend`; it didn't at the base. The implementor's patch (`readme-component-tests.patch` in scratch) still doesn't say so. | CONFIRMED / introduced | Any wave unit whose gate runs `cargo test` without `npm ci` first. `task check` and CI install first, so they are fine. |

On origin: the base had no component gate at all, so the first three cases would also be red there, but only because nothing ran. The gaps belong to the gate this unit added.

**5. What I attacked and could not break**

- **Fixture is real:** I re-recorded the first frame by serving a copy of `recorded-history.db`. It matches the committed fixture byte for byte, except for the startup workspace's random ids and path.
- **Fixture content:** it contains no `/home/`, `/Users/` or user names, only `/dev/shm` paths and scripted names.
- **The test depends on the payload:** I ran 9 changed copies (empty frame, empty view, renamed goal, no assignments, `unavailable` event, and 4 changes to `App.vue`). All 9 failed `app_renders_a_snapshot_payload`, and the unchanged copy passed.
- **Fresh install with the exact CI command:** `npm ci --ignore-scripts` added 83 packages (76,126,958 bytes). The tests then pass.
- **Built assets unchanged:** `vite build` output is identical to `frontend/dist`.
- **Lockfile:** no existing package changed version. The only install script is `fsevents`, which is macOS-only. Vitest 5.0.3 supports the Node version CI uses (22.23.2).
- **Exit codes:** npm passes Vitest's exit 1 through, and the gate catches it.
- **No tests, empty test file, import error, `.only`:** each one fails the gate for its own reason.
- **Order:** the drift check runs before the component tests, and an existing test pins that.
- **Temporary directories:** the test's `frontend-check-*` directories are all removed afterwards.
- **Not tested:** `npm ci` with the npm version bundled in CI (I ran npm 12.0.2). I didn't run mutants of the implementor's xtask test; I only read it.

**6. Paths I wrote outside the worktree**

- `/dev/shm/cp-wave2-console-component-tests-scratch/adv/`, 4.7 MB: `record.sh`, `compare.sh`, `mutants.sh`, `guards.sh`, `fresh.sh`, `gate.sh`, `*.log`, `*.sse`, `*.json`, `serve.log`, `rec/` (database copy, lock file, empty `console/`), `home/` and `tmp/` (both empty). `fresh/` and the `mutant-*` and `guard-*` copies are already deleted.
- `/dev/shm/cp-wave2-console-component-tests-scratch/npm-cache` and `npm-logs`, shared with the implementor.
- `/dev/shm/cp-wave2-console-component-tests-scratch/tmp/node-compile-cache`
- `/dev/shm/cp-wave2-console-component-tests-target`: my test binary was built here.
- Inside the worktree: the gate's `npm ci` reinstalled `frontend/node_modules`. My runs also wrote to `node_modules/.vite/vitest/*/results.json` and to `.scratch/`. Both are ignored by git.

I released my session lease on the worktree.

```findings
[{"file":"crates/control-plane-xtask/src/frontend.rs","line":47,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"frontend-check passes when every component test is skipped or todo: 0 tests run and it prints Vue component tests passed, so app_renders_a_snapshot_payload can be skipped without the gate noticing"},
{"file":".engineering/planning/story/console-component-tests.md","line":35,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the scenario name component_tests_run_in_the_gate matches no test; the covering test is component_tests_run_after_the_drift_check, so the planned acceptance-traceability check will report it"},
{"file":"frontend/vitest.config.js","line":3,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"include is src/**/*.test.js only, so a failing *.spec.js or *.test.ts file never runs and frontend-check stays green"},
{"file":"crates/control-plane-xtask/tests/frontend_check.rs","line":56,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the linked node_modules makes the test write the real frontend/node_modules/.vite/vitest results.json, which then lists src/failing.test.js as failed"},
{"file":"crates/control-plane-xtask/tests/frontend_check.rs","line":77,"category":"mutant","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"the ExitCode::FAILURE assertion holds for any error and the frontend-check command itself is never run, so changing the main.rs:62 wiring leaves the suite green"},
{"file":"README.md","line":101,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"cargo test -p control-plane-xtask now needs Node, npm and npm ci --prefix frontend, which the README and the implementor's README patch do not say"}]
```
