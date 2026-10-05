---
format: aep.planning-md/3
id: story:operator-console
kind: story
status: draft
title: Local browser console and CLI over shared application handlers
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:workspace-host
scope:
- confidence: inferred
  path: crates/control-plane-app
revision: 2
---
## Outcome
Local browser console and CLI over shared application handlers, as part of the approved standalone control-plane plan.

## ESS first
Use ess/system.yaml and ess/domains/host.yaml (ess/22). Contracts are generated before implementation. Record a red acceptance test before filling adapters; add domain declarations before introducing any noun. Never edit generated output.

## Acceptance
Named conformance and adapter scenarios: local_operator_controls, cross_origin_mutation_refused, console_and_api_share_state. Each must run against the real implementation with scripted model and integration ports, and be part of task check.

## Scope
Inferred: crates/control-plane-app. Shared root manifests, specifications, generated files and planning artifacts belong to the coordinator. Runtime planner and fleet share one implementation lane because their surfaces overlap.

## Authorization
Operator: Implement the plan. Product must live in the new public control-plane repository. No further wave confirmation is required within this approved scope.
