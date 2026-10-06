---
format: aep.planning-md/3
id: story:ess-054-upgrade
kind: story
status: draft
title: Move control-plane to ESS 0.54.0
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: .github/workflows/check.yml
- confidence: inferred
  path: crates/control-plane-runtime/resources
- confidence: inferred
  path: crates/control-plane-runtime/src/ess_reference.rs
- confidence: inferred
  path: ess/ess-inputs.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 2
---
## Outcome

control-plane runs on the newest ESS release, 0.54.0: the specification manifest, the CI install, the authoring reference the runtime hands to the planner, and the generated contracts all name 0.54.0, and `task check` passes with it.

## Evidence

- ESS 0.54.0 is the installed `ess` on the operator's machine since 2026-10-06 19:54 (`ess --version`). The workspace rule is to always move to the newest ESS release.
- ess/ess-inputs.yaml declares `requires: ess 0.53.0`; .github/workflows/check.yml:61 installs `ess 0.53.0` with its checksum.
- crates/control-plane-runtime/src/ess_reference.rs pins `VERSION = "0.53.0"` and embeds `resources/ess-0.53.0/ess.schema.json`; it refuses any other toolchain. With 0.54.0 first on PATH, `native_loom_recovers_missing_and_invalid_specification_and_queues_validated_plan` (crates/control-plane-runtime/tests/planner.rs) fails (wave-3 implementor run, 2026-10-06 ~21:50); with 0.53.0 it passes (36 planner tests, coordinator run 22:00).
- Waves 3 to 5 gate with ESS 0.53.0 first on PATH (`ess specify toolchain install 0.53.0`, binary in the ESS toolchain cache), matching CI.

## Acceptance

- `ess specify validate --path ess --strict-requires` passes with `requires: ess 0.54.0`, and CI installs 0.54.0 with its published checksum.
- The runtime's authoring reference names 0.54.0 and embeds that release's schema; the planner test above passes with 0.54.0 first on PATH.
- `generated-check`, `conformance` and `spec-history-check` pass with 0.54.0; every change `ess verify diff` reports between the two compiled models is acknowledged or shown to be none.

## Scope

Inferred: ess/ess-inputs.yaml, .github/workflows/check.yml, crates/control-plane-runtime/src/ess_reference.rs, crates/control-plane-runtime/resources/, generated/, ess/spec-acknowledgements.json.
