---
format: aep.planning-md/3
id: story:test-scratch-cleanup-runtime
kind: story
status: draft
title: The runtime and xtask tests leave no fixture directories under .scratch
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-runtime/tests
- confidence: inferred
  path: crates/control-plane-xtask/tests
revision: 2
---
## Outcome

`cargo test -p control-plane-runtime` and `cargo test -p control-plane-xtask` leave nothing under the tree's `.scratch/` once they end, apart from the conformance reports the gate keeps on purpose, so a gate run neither grows the tree nor leaves fixture repositories for `worktree finish` to refuse.

## Evidence

- Wave 10 (2026-10-11): after one implementor's package gates of control-plane-xtask and control-plane-runtime, the unit tree's `.scratch/` held 960M in 21 directories, `fleet/` alone 620M, then `goal-acceptance-pass2/` 78M, `publication-exit-pass2-attack/` 62M, `publication-exit-attack/` 48M, `mutation-<random>/` 46M (a `--keep` run), `planner/` 34M, `goal-acceptance-attack/` 32M, `blocked-reason-update-attack/` 31M (`du -sh`). With `/` at 18G free the leftovers held the wave's gates under the 20G build floor until they were deleted by path.
- story:report-test-scratch-cleanup fixed the same defect in `crates/control-plane-xtask/tests/unattended_run_report.rs` by holding the `TempDir` guard instead of keeping the directory.

## Acceptance

- `runtime_tests_leave_no_scratch` (or one such check per test file that writes under `.scratch/`): after the runtime tests run, no directory they created remains under `.scratch/`.
- Every test that writes under `.scratch/` holds its directory guard for its own duration; the tests that pass today still pass.
- Directories the gate keeps on purpose (the conformance report the evidence import reads) are named in the story when it is scoped, and stay.

## Out of scope

Changing what any test asserts.

## Scope

Inferred: crates/control-plane-runtime/tests/, crates/control-plane-xtask/tests/.
