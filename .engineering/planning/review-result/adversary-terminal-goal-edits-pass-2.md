---
format: aep.planning-md/3
id: review-result:adversary-terminal-goal-edits-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:terminal-goal-edits (1c58835): green, no findings'
relations:
- reviews: story:terminal-goal-edits
revision: 1
---
unit: story:terminal-goal-edits
verdict: green
cases: executed 79→83, red 0
origin: introduced 0, pre-existing 0, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-terminal-goal-edits-scratch/adversary-p2/ (4 small files), /dev/shm/cp-wave2-terminal-goal-edits-target (the assigned build dir; one new test binary)
needs-coordinator: no

I found nothing wrong with the fix or with the rest of the unit. These findings cover 1c58835 plus one untracked test file I added.

## 1. Worktree diff

`git --no-pager diff --stat` is empty, so no tracked file changed. `git status --short` shows one untracked file, a test:
```
?? crates/control-plane-core/tests/terminal_goal_edits_pass2_attack.rs
```

## 2. Cases added (I ran the file alone before the suite)

| Case | What it asserts | Now |
|---|---|---|
| `every_field_check_yields_to_the_declared_refusal_on_a_finished_goal` | Covers Satisfied and Cancelled goals, each with 6 edits: each of the 5 field rules broken alone, then all 5 at once. Each edit gets the declared `satisfied` or `cancelled` answer with `GoalStateConflict`, and the committed version moves by exactly 1. The goal row is unchanged, and after reopening the version and the row are the same. | green |
| `open_goals_keep_every_field_check` | Paused and Running goals: each of the 5 field rules, plus a missing `objective`, is refused by the host with its exact message. Nothing is recorded and the row is unchanged. | green |
| `edit_of_unknown_or_deleted_goal_behaves_as_at_base` | An unknown id and a deleted goal: invalid fields get the host error and nothing is recorded. A valid edit gets `not-found` with `GoalNotFound`, and 1 decision is recorded. | green |
| `malformed_edit_of_finished_goal_records_nothing` | A Cancelled goal sent a missing field, a limit as text, and `1.5` as a limit. Each one is an error, nothing is recorded, and the row is unchanged. | green |

Run alone (`cargo test --locked -p control-plane-core --test terminal_goal_edits_pass2_attack`):
```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
EXIT=0
```

## 3. Suite run, after the cases existed

`cargo test --locked -p control-plane-core -p control-plane-app --no-fail-fast` exited 0. Per test binary: 30, 0, 3, 42, 2, 1, 1, and 4 (mine) passed, plus 0 and 0 doc-tests. That is 83 executed.

- Before figure: 79 comes from the implementor's gate log at `/dev/shm/cp-wave2-terminal-goal-edits-scratch/r1/gate/6-test.log`. It has the same binaries without mine.
- Same tree: `control_plane_core-3e55050ce32d4a58 --list` includes `open_goal_edit_keeps_field_checks`, so the binary that ran was built from this tree.

## 4. Findings

None.

## 5. Attacked and could not break

- Invalid value stored: the generated `applied` branch requires Paused or Running on the same row (`behaviour.rs:922`). So a skipped field check can never reach the goal row. Case 1 confirms this for all 5 rules in both terminal states.
- Same snapshot: `guard` reads `self.memory`, and `apply` clones that same `self.memory` straight afterwards under `&mut self` with nothing awaited in between (`lib.rs:283-290`). `Uuid` is the raw string (`wire.rs`, `uuid_at`), so both sides look the goal up by the same key.
- Replay of recorded refusals: replay never runs the guard (`lib.rs:212`). Refusals of invalid edits are recorded once each and replay to the same version and row (case 1).
- Paused and Running: they keep all 5 checks plus the type check (case 2). Reading the code, a mutant that extends the skip to Paused or Running fails `open_goal_edit_keeps_field_checks`.
- Unknown or deleted goal: it takes the same path as at base, because the skip needs the goal to exist (case 3).
- Other guards that could still pre-empt the declared refusal: UpdateGoal has no other branch in `guards.rs:84-212`.
- Stored refusal bodies can now hold invalid limits: only replay reads stored decision bodies (searched for `read_stream` and `HostDecision`), so nothing consumes them.
- The unit's tests: they assert literal values. Reading the code, they would miss a mutant that keeps only the acceptance check for finished goals; case 1 catches that mutant.
- Live store: not opened or copied, as instructed. The implementor's figure for it (3602 decisions, 8 UpdateGoal, none on a finished goal) is from 279e420. 1c58835 changes only the guard, which replay does not run.

## 6. Paths written outside the worktree

- `/dev/shm/cp-wave2-terminal-goal-edits-scratch/adversary-p2/`: `alone.log`, `suite.log`, `scratch-before.txt`, `scratch-after.txt`.
- `/dev/shm/cp-wave2-terminal-goal-edits-target`: the assigned build dir, plus the `terminal_goal_edits_pass2_attack-*` test binaries.
- Test tempdirs in `/dev/shm/cp-wave2-terminal-goal-edits-scratch/tmp/` and `$HOME/.cache/control-plane-host/` were removed by the tests.
- The worktree's `.scratch/` is unchanged; I compared listings before and after the suite.
- I acquired and released the worktree lease `adversary-terminal-goal-edits-w2p2`.

```findings
[]
```
