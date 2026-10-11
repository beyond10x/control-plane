---
format: aep.planning-md/3
id: approval-record:unattended-run-spending-2026-10-11
kind: approval-record
status: draft
title: One real-model run on go-auth-web approved by the operator
relations:
- decides: story:unattended-goal-evidence
- decides: story:report-test-scratch-cleanup
revision: 1
---
The operator answered decision-blocker:unattended-run-spending on 2026-10-11 with option A, for goal G1 (a goal accepted without operator input).

## Decision

- Exactly one real-model run, on the go-auth-web evaluation case.
- One worker, one attempt, a 10-minute goal budget.
- At most about 2.2M input tokens, the largest earlier round (story:runtime-boundary, round 4).
- On the existing codex login on this machine: no new account, key or secret.
- No GitHub publication by the run: the case publishes only to its local origin under the eval root.

If the run fails, it is kept as evidence and a second run is a new spending question for the operator; none starts without one.

## Units

1. story:unattended-goal-evidence: the run and its report.
2. story:report-test-scratch-cleanup: the report test leaves no fixture repositories behind.

On `control-plane/wave-9`; the page is `.engineering/waves/2026-10-11-wave-9.md`.

## Commits this approval authorises

Store and evidence commits on `control-plane/wave-9`, one commit per unit plus correction commits on unit branches, the merges into `control-plane/wave-9`, its push, one pull request into `main` and its merge with a merge commit once its checks pass. No tag, no release.
