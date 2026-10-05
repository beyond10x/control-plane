---
format: aep.planning-md/3
id: story:workspace-host
kind: story
status: implemented
title: Workspace host and durable generated behavior
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: cited
  path: crates/control-plane-core
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:20:54Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T20:20:54Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-05T20:46:12Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":4,"verification":2}}}
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

## Implementation contract

Canonicalize paths in the operational Store, discover either the selected Git repository or immediate Git child repositories, and preserve one registration per canonical workspace path. A dedicated contract::ContractStore executes the same generated behavior and Eventlog SQLite persistence without filesystem admission: this is the exact ESS conformance boundary, never a browser-selected mode. Stage deep-cloned memory, append with compare-and-swap, then publish state; failed append leaves visible state unchanged. Keep a database lifetime lock to exclude a second service. Operational admission enforces one running goal per workspace and one active assignment per canonical common Git directory across workspaces, current goal revision, independent review and recorded publication reconciliation. Validate these policies with real adapter tests beside generated conformance.
