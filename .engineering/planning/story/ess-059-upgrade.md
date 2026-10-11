---
format: aep.planning-md/3
id: story:ess-059-upgrade
kind: story
status: active
title: Move control-plane to the newest ESS release
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: .github/workflows/check.yml
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/control-plane-runtime/resources
- confidence: inferred
  path: crates/control-plane-runtime/src/ess_reference.rs
- confidence: inferred
  path: crates/control-plane-xtask/Cargo.toml
- confidence: inferred
  path: ess/ess-inputs.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-11T23:03:13Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-10-11T23:03:13Z", actor: "human:timo", revision: 12}
---
## Outcome

control-plane runs on the newest ESS release at the time this story starts (0.59.0, published 2026-10-11T10:58:22Z, `gh release list -R beyond10x/ess`, unless a newer one is listed when the unit starts): the specification manifest, the ESS crates the xtask links, the CI install, the authoring reference the runtime hands to the planner and the generated contracts all name it, and `task check` passes with it.

## Evidence

- ess/ess-inputs.yaml declares `requires: ess 0.56.0`; .github/workflows/check.yml installs `ess 0.56.0` in both jobs; crates/control-plane-xtask/Cargo.toml pins `ess-conformance` and `ess-primitives` at rev 84ee8d38 (0.56.0).
- The ess on this machine is 0.59.0. With it first on PATH, `cargo test -p control-plane-xtask` fails 5 cases (precise_outcome_acknowledgements_attack 3 of 14, precise_outcome_acknowledgements_pass2_attack 2 of 12), each with "ess/ess-inputs.yaml requires ess 0.56.0 and this is ess 0.59.0, which is newer, and --strict-requires refuses a newer release" (wave 9 implementor run, 2026-10-11). Wave 9 gated with 0.56.0 first on PATH.
- story:ess-055-upgrade moved the same surfaces from 0.53.0 to 0.56.0, including crates/control-plane-runtime/src/ess_reference.rs and its embedded schema under crates/control-plane-runtime/resources/.
- Three upstream-blockers wait on ESS releases after 0.56.0: upstream-blocker:ess-generates-several-related-rows, upstream-blocker:ess-negated-defined-witness and upstream-blocker:ess-subject-field-selectors.

## Acceptance

- `ess specify validate --path ess --strict-requires` passes with the new `requires:`; CI installs that release with the checksum from its `SHA256SUMS`.
- `cargo test -p control-plane-xtask` passes with the new release first on PATH, including the five cases above.
- The runtime's authoring reference names the new release and embeds its schema; `native_loom_recovers_missing_and_invalid_specification_and_queues_validated_plan` (crates/control-plane-runtime/tests/planner.rs) passes with it.
- `cargo run --locked -p control-plane-xtask -- generated-check`, `-- conformance` (at least the 204 scenarios recorded on executable-system-specification:control-plane) and `-- spec-history-check` pass; every change `ess verify diff` reports between the two compiled models is acknowledged or shown to be none.
- Each of the three upstream-blockers above is re-run against the new release with its recorded reproduction; one whose "Clears when" holds is cleared with that output, the others record the release they were checked against.

## Out of scope

Implementing the stories those blockers hold; changing domain behaviour.

## Scope

Inferred: ess/ess-inputs.yaml, .github/workflows/check.yml, crates/control-plane-xtask/Cargo.toml, Cargo.lock, crates/control-plane-runtime/src/ess_reference.rs, crates/control-plane-runtime/resources/, generated/, ess/spec-acknowledgements.json, README.md.
