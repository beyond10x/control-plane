---
format: aep.planning-md/3
id: executable-system-specification:control-plane
kind: executable-system-specification
status: conforming
title: Control-plane ess/22 domain
model_digest: 067305d07e71dad22be3826b880d520f1f1c41ed0bdd99b54d385d0d95f58dad
revision: 12
transitions:
- {from: "draft", to: "validated", at: "2026-10-05T20:35:57Z", actor: "human:timo", revision: 4}
- {from: "validated", to: "conforming", at: "2026-10-08T14:17:15Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1,"ess_conformance_coverage_v1":1}}}
---
The authored specification is ess/system.yaml, ess/domains/host.yaml and ess/components.yaml, in ess/22, validated with ESS 0.56.0. It declares Workspace, WorkspaceDirectory, RepositoryRegistration, Goal, Assignment and PublicationIntent, including directory membership, mutable configuration, revision-bound evidence, durable planning progress, publication reconciliation and the admission guards ESS can synthesize. Generated contracts live in generated/model; the model digest is 067305d07e71dad22be3826b880d520f1f1c41ed0bdd99b54d385d0d95f58dad (generated/model/PLAN.md:3). The conformance target is the real durable contract::ContractStore below host admission; admission rules ESS cannot yet express are host facts listed in ess/README.md and checked by queue_guards_are_refused_in_conformance. AEP remains the planning authority.

## Conformance

The ess-conformance-report/2 written by `cargo run --locked -p control-plane-xtask -- conformance` on the wave-8 conformance unit (suite ess-conformance/35, digest sha256:f9133c70cf5fcc258f4b2aaf28823a0c01287ad2a0495f44ad01b5940436992b) passed 204 of 204 generated scenarios, 0 failed, error, unsupported or skipped, coverage knowledge complete_inventory, spec digest equal to the model digest. Its completed_at (1791468732720, 2026-10-08T14:12:12Z) is the run's real instant: the runner reads the wall clock since story:conformance-evidence-record. It was recorded with aep 0.69.1 (`aep plan artifact evidence --from report.json --suite generated/conformance.json`, kind ess_conformance_coverage_v1), and the artifact moved validated -> conforming on that record.

aep 0.68.0 refused the ESS 0.53 suite provenance (scenario_initial_state); aep 0.69.0 admits it, and CI pins aep 0.69.1.
