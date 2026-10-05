---
format: aep.planning-md/3
id: story:implementation-fleet
kind: story
status: implemented
title: Isolated implementation, independent review and verified merges
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:autonomous-planner
scope:
- confidence: inferred
  path: crates/control-plane-runtime
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T21:31:09Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-05T21:31:09Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-05T22:15:46Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Implementation contract

Run at most the configured worker limit, default three, while holding one active repository common-directory slot globally across workspaces. Use managed worktrees and fresh run identities for implementations and independent reviews. Bind test and review evidence to the exact candidate, base and goal revision; edits invalidate prior evidence. Recheck standing merge authority immediately before effect invocation. Publication adapters receive explicit candidate, expected base, target and operation id. Persist a PublicationIntent before invocation; exit zero is not a merge receipt. Observe Git publication and save the receipt; after a crash or ambiguous result reconcile that exact intent before another attempt. Pause/cancel/revocation stop new dispatch. Enforce attempt/time budgets and expose concrete blocked reasons. Never release or deploy.
