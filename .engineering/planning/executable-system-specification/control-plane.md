---
format: aep.planning-md/3
id: executable-system-specification:control-plane
kind: executable-system-specification
status: validated
title: Control-plane ess/22 domain
model_digest: 3d7e5edad026a769d94fad7e6af37d426a599672c637ccd2389868c2ed11448b
revision: 6
transitions:
- {from: "draft", to: "validated", at: "2026-10-05T20:35:57Z", actor: "human:timo", revision: 4}
---
The authored specification is ess/system.yaml, ess/domains/host.yaml and ess/components.yaml, using ess/22 with ESS 0.53.0. It declares Workspace, RepositoryRegistration, Goal, Assignment and PublicationIntent, including mutable configuration, revision-bound evidence, durable planning progress and publication reconciliation. Validation passes. Synthesis reports 113 generated capabilities, zero obligations and zero refusals, and 152 generated scenarios. Generated contracts live in generated/model. The conformance target is the real durable contract::ContractStore; operational admission policies have additional real adapter tests. ESS interpreter runs are not evidence of product conformance. AEP remains the planning authority; the standalone product owns its service and runtime integration.
