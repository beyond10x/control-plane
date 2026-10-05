---
format: aep.planning-md/3
id: story:verification
kind: story
status: draft
title: Conformance, build gates and two-repository qualification
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:implementation-fleet
- depends_on: story:operator-console
scope:
- confidence: inferred
  path: crates/control-plane-xtask
revision: 2
---
## Outcome
Conformance, build gates and two-repository qualification, as part of the approved standalone control-plane plan.

## ESS first
Use ess/system.yaml and ess/domains/host.yaml (ess/22). Contracts are generated before implementation. Record a red acceptance test before filling adapters; add domain declarations before introducing any noun. Never edit generated output.

## Acceptance
Named conformance and adapter scenarios: generated_contracts_do_not_drift, real_adapter_conformance, two_repository_goal_delivery. Each must run against the real implementation with scripted model and integration ports, and be part of task check.

## Scope
Inferred: crates/control-plane-xtask. Shared root manifests, specifications, generated files and planning artifacts belong to the coordinator. Runtime planner and fleet share one implementation lane because their surfaces overlap.

## Authorization
Operator: Implement the plan. Product must live in the new public control-plane repository. No further wave confirmation is required within this approved scope.
