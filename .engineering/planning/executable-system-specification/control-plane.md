---
format: aep.planning-md/3
id: executable-system-specification:control-plane
kind: executable-system-specification
status: validated
title: Control-plane ess/22 domain
model_digest: 528a7c48088b8ebb67277ee677106218efacfce1938bb9582406ba6a479b6902
revision: 9
transitions:
- {from: "draft", to: "validated", at: "2026-10-05T20:35:57Z", actor: "human:timo", revision: 4}
---
The authored specification is ess/system.yaml, ess/domains/host.yaml and ess/components.yaml, using ess/22 with ESS 0.53.0. It declares Workspace, WorkspaceDirectory, RepositoryRegistration, Goal, Assignment and PublicationIntent, including directory membership, mutable configuration, revision-bound evidence, durable planning progress and publication reconciliation. Validation passes. Synthesis reports 125 generated capabilities, zero obligations and zero refusals, and 159 generated scenarios with a complete compiler coverage inventory. Generated contracts live in generated/model. The conformance target is the real durable contract::ContractStore; operational admission policies have additional real adapter tests. ESS interpreter runs are not evidence of product conformance. AEP remains the planning authority; the standalone product owns its service and runtime integration.

The integration run passed all 159 scenarios with zero failed/error/unsupported/skipped and coverage qualification Passed. Its original report is retained at .scratch/conformance/report.json beside generated/conformance.json. Importing that ESS 0.53 suite/35 and report/2 through AEP 0.68.0 is currently refused: `error: UnknownField at $suite.provenance.scenario_initial_state: closed count-stage vocabulary`. The installed AEP CLI and plugin are the latest releases (0.68.0 and 0.19.2, verified by b10x upgrade). Keep this artifact validated until AEP admits the original report; do not strip fields, fabricate older evidence, or assert a conforming transition manually.
