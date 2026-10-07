---
format: aep.planning-md/3
id: story:unattended-goal-evidence
kind: story
status: draft
title: One goal on a real repository is accepted with no operator input, and the run's record shows it
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: story:external-evaluations
- depends_on: story:typed-satisfaction-receipt
- depends_on: story:blocked-reason-update
- depends_on: story:repair-on-moved-target
scope:
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/control-plane-xtask/src/eval.rs
- confidence: inferred
  path: crates/control-plane-xtask/tests
revision: 2
---
## Outcome

One goal on a real repository is planned, implemented, reviewed, merged and accepted by the control plane with no operator input after it starts, and the run leaves a record that shows it: the goal Satisfied with a receipt for the revision its acceptance checked, the merged commit on the repository's target branch, and no Operator command between StartGoal and SatisfyGoal. This is the evidence goal G1 asks for.

## Evidence

- epic:unattended-operation names unattended goals as its outcome; no story in it produces a run that shows one end to end.
- story:external-evaluations (active, under epic:bootstrap) built isolated real-model evaluation workspaces with external Go repositories, local origins and a trusted Rust verifier in crates/control-plane-xtask/src/eval.rs; this story reuses that harness instead of building a second one.
- story:acceptance-edit-ordering showed that a goal is satisfied only at the revision its acceptance checked; story:blocked-reason-update and story:repair-on-moved-target remove the two states in which an unattended goal could stall with a stale reason or a spent attempt.

## Acceptance

- One run against a real repository with the operator's configured model: the goal goes from StartGoal to Satisfied, and the run's report, produced by the xtask eval command, states the goal id, the merged commit on the target branch, the satisfaction receipt's revision, the elapsed time and the number of Operator commands between StartGoal and SatisfyGoal, which is zero.
- `unattended_run_report_counts_operator_commands`: given a recorded decision log, the report counts every Operator command after StartGoal and refuses to call a run unattended when the count is above zero.
- `unattended_run_report_requires_a_merged_commit_on_target`: the report refuses a Satisfied goal whose merged commit is not on the target branch.
- The run's report and the decision log it was read from are kept as evidence on this story (`aep plan artifact evidence` with kind verification), with the run's real instant.

## Out of scope

Several goals, several repositories, a benchmark, publication to a hosted service, and any change to how goals are planned or accepted.

## Scope

Inferred: crates/control-plane-xtask/src/eval.rs, crates/control-plane-xtask/tests/, README.md (the run procedure), .engineering/evidence/ (written by the coordinator).

## Planning

Planned for the wave after wave 6. The run spends the operator's model budget and needs the operator's configured model login, so the wave that holds it states the expected spend in its proposal.
