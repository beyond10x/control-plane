---
format: aep.planning-md/3
id: story:implementation-fleet
kind: story
status: draft
title: Isolated implementation, independent review and verified merges
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:autonomous-planner
scope:
- confidence: inferred
  path: crates/control-plane-runtime
revision: 2
---
## Outcome
Isolated implementation, independent review and verified merges, as part of the approved standalone control-plane plan.

## ESS first
Use ess/system.yaml and ess/domains/host.yaml (ess/22). Contracts are generated before implementation. Record a red acceptance test before filling adapters; add domain declarations before introducing any noun. Never edit generated output.

## Acceptance
Named conformance and adapter scenarios: repository_execution_is_exclusive, review_is_independent, changed_revision_invalidates_evidence, merge_requires_current_authority, restart_reconciles_effects, pause_and_limits_stop_dispatch. Each must run against the real implementation with scripted model and integration ports, and be part of task check.

## Scope
Inferred: crates/control-plane-runtime. Shared root manifests, specifications, generated files and planning artifacts belong to the coordinator. Runtime planner and fleet share one implementation lane because their surfaces overlap.

## Authorization
Operator: Implement the plan. Product must live in the new public control-plane repository. No further wave confirmation is required within this approved scope.
