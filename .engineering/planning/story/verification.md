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
- depends_on: story:contract-verification
scope:
- confidence: inferred
  path: crates/control-plane-xtask
revision: 3
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

## Implementation contract

task check validates ESS and the AEP store, formats/lints/tests Rust, checks regenerated model/OpenAPI/conformance bytes for drift, and executes all generated scenarios against the real durable contract::ContractStore. Generated state-machine conformance is reported separately from operational host admission tests. Include real AEP, ESS, managed-worktree, checks and Git processes with scripted model decisions against two disposable repositories. Exercise changed candidates, independent-review rejection, authority revocation, restart after effect before receipt, and nonempty goal acceptance. Never call an interpreter run product conformance and never treat skipped scenarios as passing.
