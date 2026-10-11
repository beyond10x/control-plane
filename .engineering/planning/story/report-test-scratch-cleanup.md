---
format: aep.planning-md/3
id: story:report-test-scratch-cleanup
kind: story
status: implemented
title: The eval report tests leave no fixture repositories under .scratch
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-xtask/tests/unattended_run_report.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-11T22:05:08Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-11T22:05:09Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-11T22:54:04Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`cargo test -p control-plane-xtask --test unattended_run_report` leaves nothing under `.scratch/unattended-run-report/` once it ends, so a gate run neither grows the tree nor stops `worktree finish` from finishing it.

## Evidence

- `crates/control-plane-xtask/tests/unattended_run_report.rs:78` builds each recorded run under `tempfile::tempdir_in(<checkout>/.scratch/unattended-run-report).keep()`. `keep()` disowns the directory, so each of the three tests leaves a throwaway Git repository with its local origin and linked managed worktrees behind.
- Wave 8: two package-gate runs left 94M there, and `worktree finish` refused the integration tree until they were deleted by path (`.engineering/waves/2026-10-08-wave-8.md`, stage log 13:48:17Z).

## Acceptance

- `unattended_run_report_leaves_no_scratch`: after the report tests run, `.scratch/unattended-run-report/` holds no directory they created.
- The three existing report tests (`unattended_run_report_counts_operator_commands`, `unattended_run_report_requires_a_merged_commit_on_target` and `unattended_run_report_passes_a_clean_run`) still pass.
- The managed worktrees the fixture registers are retired through the `worktree` CLI, not removed by hand, or deleted with the fixture root that holds their records (`XDG_STATE_HOME` and `XDG_CONFIG_HOME` point inside it), so no stale record outlives the test.

## Out of scope

Any change to `control-plane-xtask eval report` itself.

## Scope

Inferred: crates/control-plane-xtask/tests/unattended_run_report.rs.
