---
format: aep.planning-md/3
id: story:governed-protocols
kind: story
status: implemented
title: Planning and verified-merge protocols over Canon
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-protocol
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:30:36Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T20:30:36Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-05T20:40:47Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Outcome
Compile control-plane's planning and verified-merge protocols with the existing Canon foundation. This is a dependency of the existing autonomous-planner and implementation-fleet stories, extracted after inspection showed stock engineering protocols have broader release/deployment outcomes.

## ESS first
The host entities and publication lifecycle are declared in ess/domains/host.yaml. Protocol cases, artifact revisions and evidence records use Canon's existing typed model; this unit introduces no second persistent domain.

## Acceptance
Named tests planning_requires_current_validation, merge_requires_current_test_and_independent_review, stale_evidence_cannot_complete, merge_observation_required, authority_is_checked_at_effect. Compile ordinary protocol YAML through Canon and exercise its real evaluator; no fabricated pass flags or alternate protocol interpreter.

## Scope
Inferred: crates/control-plane-protocol. Own product-local protocol YAML and a small Rust adapter for Canon compilation/evaluation. No changes to Loom, Canon or engineering-protocols. Runtime adapters remain in crates/control-plane-runtime. Root manifests and AEP belong to the coordinator.

## Authorization
Implementation is part of the operator-approved standalone control-plane plan. Defaults follow epic:bootstrap. This extraction preserves that scope.
