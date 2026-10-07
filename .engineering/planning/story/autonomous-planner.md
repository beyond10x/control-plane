---
format: aep.planning-md/3
id: story:autonomous-planner
kind: story
status: implemented
title: Goal-driven ESS-first AEP planner
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:workspace-host
- depends_on: story:governed-protocols
scope:
- confidence: inferred
  path: crates/control-plane-runtime
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:46:56Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-05T20:46:56Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-05T21:31:09Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Implementation contract

The planner wakes on a changed goal, repository inventory, newly completed assignment or blocker; an unchanged idle workspace makes no model call. Inspect each selected repository and its existing ESS/AEP artifacts before mutation. Adopt missing stores using their CLIs, specify new nouns before writing stories, validate ESS and AEP, and request independent critique. Use ready existing stories rather than duplicate a backlog. Planner work runs in a managed checkout. Store assignment references to authoritative AEP story ids, not a second task backlog. Current goal/configuration revision binds planning and execution. Completion requires passing goal acceptance and observed merge receipts for every delivered assignment; an empty queue alone cannot satisfy the goal.
