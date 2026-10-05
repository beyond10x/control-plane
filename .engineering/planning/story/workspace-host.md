---
format: aep.planning-md/3
id: story:workspace-host
kind: story
status: draft
title: Workspace host and durable generated behavior
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-core
revision: 2
---
## Outcome
Workspace host and durable generated behavior, as part of the approved standalone control-plane plan.

## ESS first
Use ess/system.yaml and ess/domains/host.yaml (ess/22). Contracts are generated before implementation. Record a red acceptance test before filling adapters; add domain declarations before introducing any noun. Never edit generated output.

## Acceptance
Named conformance and adapter scenarios: workspace_registration_is_idempotent, durable_state_survives_restart, one_active_goal_per_workspace, storage_failure_stops_effects. Each must run against the real implementation with scripted model and integration ports, and be part of task check.

## Scope
Inferred: crates/control-plane-core. Shared root manifests, specifications, generated files and planning artifacts belong to the coordinator. Runtime planner and fleet share one implementation lane because their surfaces overlap.

## Authorization
Operator: Implement the plan. Product must live in the new public control-plane repository. No further wave confirmation is required within this approved scope.
