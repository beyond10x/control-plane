---
format: aep.planning-md/3
id: story:test-scratch-cleanup-runtime
kind: story
status: active
title: The runtime and xtask tests leave no fixture directories under .scratch
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
scope:
- confidence: cited
  path: crates/control-plane-runtime/tests
- confidence: cited
  path: crates/control-plane-xtask/src/target.rs
- confidence: cited
  path: crates/control-plane-xtask/tests
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-12T00:57:22Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-12T00:57:22Z", actor: "human:timo", revision: 7}
---
## Outcome

`cargo test -p control-plane-runtime` and `cargo test -p control-plane-xtask` leave nothing under the tree's `.scratch/` once they end, apart from the conformance reports the gate keeps on purpose, so a gate run neither grows the tree nor leaves fixture repositories for `worktree finish` to refuse.

## Evidence

- Wave 10 (2026-10-11): after one implementor's package gates of control-plane-xtask and control-plane-runtime, the unit tree's `.scratch/` held 960M in 21 directories, `fleet/` alone 620M, then `goal-acceptance-pass2/` 78M, `publication-exit-pass2-attack/` 62M, `publication-exit-attack/` 48M, `mutation-<random>/` 46M (a `--keep` run), `planner/` 34M, `goal-acceptance-attack/` 32M, `blocked-reason-update-attack/` 31M (`du -sh`). With `/` at 18G free the leftovers held the wave's gates under the 20G build floor until they were deleted by path.
- story:report-test-scratch-cleanup fixed the same defect in `crates/control-plane-xtask/tests/unattended_run_report.rs` by holding the `TempDir` guard instead of keeping the directory.

## Acceptance

- `runtime_tests_leave_no_scratch` (or one such check per test file that writes under `.scratch/`): after the runtime tests run, no directory they created remains under `.scratch/`.
- Every test that writes under `.scratch/` holds its directory guard for its own duration; the tests that pass today still pass.
- Directories the gate keeps on purpose stay: `.scratch/conformance/` (`report.json` and `diagnostics.json`, written by `crates/control-plane-xtask/src/target.rs` and read by the evidence import), and a `mutation-<random>/` directory only when `xtask mutate --keep` asks for it.

## Out of scope

Changing what any test asserts.

## Scope

Cited (read at 81d84a1, `grep -n 'keep()\|tempdir_in'`):

- `crates/control-plane-runtime/tests/`: `fleet.rs:57`, `planner.rs:180`, `blocked_reason_update_attack.rs:51`, `goal_acceptance_authority_attack.rs:54`, `goal_acceptance_authority_pass2_attack.rs:160`, `publication_exit_attack.rs:65`, `publication_exit_pass2_attack.rs:60`, `repair_on_moved_target_attack.rs:102` each call `tempdir_in(.scratch/<name>).keep()`, so every fixture root stays.
- `crates/control-plane-xtask/tests/`: `generation_ownership.rs`, `spec_history_attack.rs`, `spec_history_pass2_attack.rs`, `precise_outcome_acknowledgements_attack.rs`, `precise_outcome_acknowledgements_pass2_attack.rs`, `record_history_attack.rs` hold a `TempDir` guard but leave the parent directory they create; `precise_outcome_acknowledgements_pass2_attack.rs:218` writes `.scratch/generated`.
- `crates/control-plane-xtask/src/target.rs`: the in-crate tests write `.scratch/conformance-queue-tests` and `.scratch/conformance-tests`.

The planner and fleet fixtures register linked Git worktrees; `planner.rs:167` says they were kept so a `TempDir` would not delete linked worktrees behind the registry's back. The cleanup removes those worktrees (or the whole fixture root together with the registry state it owns) in the order that leaves no dangling record, and the check proves it by listing `.scratch/` after the run.
