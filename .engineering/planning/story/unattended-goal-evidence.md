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
- depends_on: story:unattended-run-report
scope:
- confidence: inferred
  path: README.md
revision: 4
---
## Outcome

One goal on a real repository is planned, implemented, reviewed, merged and accepted by the control plane with no operator input after it starts, and the run leaves a record that shows it: the goal Satisfied with a receipt for the revision its acceptance checked, the merged commit on the repository's target branch, and no Operator command between StartGoal and SatisfyGoal. This is the evidence goal G1 asks for.

## Evidence

- epic:unattended-operation names unattended goals as its outcome; no story in it produces a run that shows one end to end.
- story:external-evaluations (active, under epic:bootstrap) built isolated real-model evaluation workspaces with external Go repositories, local origins and a trusted Rust verifier in crates/control-plane-xtask/src/eval.rs; this story reuses that harness instead of building a second one.
- story:acceptance-edit-ordering showed that a goal is satisfied only at the revision its acceptance checked; story:blocked-reason-update and story:repair-on-moved-target remove the two states in which an unattended goal could stall with a stale reason or a spent attempt.

## Acceptance

- One run against a real repository with the operator's configured model: the goal goes from StartGoal to Satisfied, and the run's report, produced by the xtask eval command, states the goal id, the merged commit on the target branch, the satisfaction receipt's revision, the elapsed time and the number of Operator commands between StartGoal and SatisfyGoal, which is zero.
- The report is the one story:unattended-run-report builds (`control-plane-xtask eval report`); its checks `unattended_run_report_counts_operator_commands` and `unattended_run_report_requires_a_merged_commit_on_target` are that story's acceptance and need no model.
- The run's report and the decision log it was read from are kept as evidence on this story (`aep plan artifact evidence` with kind verification), with the run's real instant.

## Out of scope

Several goals, several repositories, a benchmark, publication to a hosted service, and any change to how goals are planned or accepted.

## Scope

Inferred: README.md (the run procedure), .engineering/evidence/ (written by the coordinator). The report code is story:unattended-run-report's.

## Planning

The run spends the operator's model budget and needs the operator's configured model login. decision-blocker:unattended-run-spending holds the spending question; no real-model run starts until it is answered. Wave 8 delivers story:unattended-run-report and leaves this story in draft.

## Findings

- Load sensitivity (wave 7 integration gate, 2026-10-08, 12:49Z): `editing_another_goal_during_acceptance_does_not_fail_the_fleet_tick` (crates/control-plane-runtime/tests/goal_acceptance_authority_pass2_attack.rs:379) panicked at line 396, "a goal reached its final review", after more than 60 s while the whole workspace's tests ran in parallel on a loaded host. It passed alone 2 of 2, with its own binary 3 of 3 (12-27 s), and 5 of 5 reruns afterwards. The test depends on the scripted fleet tick reaching final review within its run; under load that ordering did not happen once. An unattended run on a loaded host can take the same path, so the real run's report records host load (`/proc/loadavg`) beside its elapsed time. Recorded here, not as an issue.
