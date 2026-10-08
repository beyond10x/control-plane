---
format: aep.planning-md/3
id: story:unattended-run-report
kind: story
status: active
title: The eval report states whether a recorded goal ran unattended, without a model
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- depends_on: story:typed-satisfaction-receipt
scope:
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/control-plane-xtask/Cargo.toml
- confidence: inferred
  path: crates/control-plane-xtask/src/eval.rs
- confidence: inferred
  path: crates/control-plane-xtask/src/eval_report.rs
- confidence: inferred
  path: crates/control-plane-xtask/tests/unattended_run_report.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 4}
---
## Outcome

`control-plane-xtask eval report --state <state.sqlite> --goal <goal-id> --repo <repo>` reads a recorded state store and prints the unattended-run report for one goal: the goal id, its final state, the satisfaction receipt's revision, the merged commit and whether it is on the repository's target branch, the elapsed time from StartGoal to SatisfyGoal, and the number of Operator commands recorded between them. It refuses to call a run unattended when that count is above zero or when the merged commit is not on the target branch. It needs no model: it reads a store any run (real or scripted) leaves behind, so story:unattended-goal-evidence only has to run the goal and keep the report.

## Evidence

- story:unattended-goal-evidence lists both checks as acceptance lines that need no model spend; this story takes them so they ship before the spending decision.
- Actors are generated: `generated/model/src/actor.rs:15` (`Actor::Operator` is `controlplane.host.Operator`, declared at `ess/domains/host.yaml:328`). The report reads the actor from the recorded decision, through the generated type, not from a string match.
- The eval harness lives in `crates/control-plane-xtask/src/eval.rs` (`Action::Init`, `Action::Verify`); the report is a third action beside them.

## Acceptance

- `unattended_run_report_counts_operator_commands`: given a recorded decision log, the report counts every Operator command after StartGoal and refuses to call a run unattended when the count is above zero.
- `unattended_run_report_requires_a_merged_commit_on_target`: the report refuses a Satisfied goal whose merged commit is not on the target branch.
- `unattended_run_report_passes_a_clean_run`: for a store recorded by a scripted-model run with no Operator command after StartGoal and a merged commit on the target, the report exits 0 and prints the goal id, the receipt revision, the commit and the count 0.

The three tests use a scripted model and disposable repositories, as the repository's working rules require.

## Spec first

The report reads recorded decisions and generated actor and command types; it adds no domain noun. If the report needs a fact the specification does not declare (for example which decision carries the merged commit), model it in `ess/` first and regenerate.

## Out of scope

Running a real model, choosing the repository for the real run, any change to planning or acceptance.

## Scope

Inferred: crates/control-plane-xtask/src/eval.rs, a new crates/control-plane-xtask/src/eval_report.rs, crates/control-plane-xtask/tests/unattended_run_report.rs, crates/control-plane-xtask/Cargo.toml (if it needs control-plane-core), README.md (the eval report command).
