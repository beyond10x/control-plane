---
format: aep.planning-md/3
id: story:autonomous-planner
kind: story
status: draft
title: Goal-driven ESS-first AEP planner
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:workspace-host
scope:
- confidence: inferred
  path: crates/control-plane-runtime
revision: 2
---
## Outcome
Goal-driven ESS-first AEP planner, as part of the approved standalone control-plane plan.

## ESS first
Use ess/system.yaml and ess/domains/host.yaml (ess/22). Contracts are generated before implementation. Record a red acceptance test before filling adapters; add domain declarations before introducing any noun. Never edit generated output.

## Acceptance
Named conformance and adapter scenarios: goal_drives_plan, existing_backlog_is_not_duplicated, goal_completion_requires_evidence. Each must run against the real implementation with scripted model and integration ports, and be part of task check.

## Scope
Inferred: crates/control-plane-runtime. Shared root manifests, specifications, generated files and planning artifacts belong to the coordinator. Runtime planner and fleet share one implementation lane because their surfaces overlap.

## Authorization
Operator: Implement the plan. Product must live in the new public control-plane repository. No further wave confirmation is required within this approved scope.
