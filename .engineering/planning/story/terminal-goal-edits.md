---
format: aep.planning-md/3
id: story:terminal-goal-edits
kind: story
status: implemented
title: Satisfied and cancelled goals refuse edits
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:ess-design-review-2026-10-06
- depends_on: story:spec-history-gate
scope:
- confidence: inferred
  path: crates/control-plane-core/src/tests.rs
- confidence: cited
  path: ess/domains/host.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T05:17:10Z", actor: "human:timo", revision: 5, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T05:17:10Z", actor: "human:timo", revision: 6, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T06:33:37Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

UpdateGoal is refused for Satisfied and Cancelled goals, so a satisfied goal keeps its acceptance receipt.

## Evidence

- ess/domains/host.yaml:1021-1077: UpdateGoal declares no wrong_state outcome and sets `satisfaction_receipt: ''`.
- crates/control-plane-core/src/guards.rs:104-119 checks only field values.
- Probe `probe_satisfied_goal_keeps_its_acceptance_receipt` failed on f510e7f (state Satisfied, objective changed, receipt empty) and passed with a one-line host guard.
- review-result:ess-design-review-2026-10-06 row 12 (contradicts vision.md:22).

## Acceptance

- Generated conformance scenarios for UpdateGoal refusing in Satisfied and in Cancelled pass against the durable target.
- `satisfied_goal_refuses_edit`: the probe sequence through `Store::execute` leaves objective and receipt unchanged.
- `recorded_history_replays` (story:spec-history-gate) stays green, or the acknowledgement names the change.

## Scope

Cited: ess/domains/host.yaml. Inferred: generated/, crates/control-plane-core/src/tests.rs.
