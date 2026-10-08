---
format: aep.planning-md/3
id: story:ess-055-upgrade
kind: story
status: active
title: Move control-plane to ESS 0.56.0
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
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T00:11:58Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-10-08T00:11:58Z", actor: "human:timo", revision: 10}
---
## Outcome

control-plane runs on the newest ESS release, 0.56.0 (published 2026-10-07T23:16:28Z, `gh release list -R beyond10x/ess`): the specification manifest, the CI install, the authoring reference the runtime hands to the planner, and the generated contracts all name 0.56.0, and `task check` passes with it. The story id keeps its 0.55.0 name; 0.56.0 superseded 0.55.0 before the story ran.

## Evidence

- ESS 0.55.0 is the installed `ess` on the operator's machine on 2026-10-07 (`ess --version` printed `ess 0.55.0`) and in the toolchain cache (`$HOME/.cache/ess/toolchains/0.55.0`). The workspace rule is to always move to the newest ESS release.
- ess/ess-inputs.yaml declares `requires: ess 0.53.0`; .github/workflows/check.yml:61 installs `ess 0.53.0` with its checksum.
- crates/control-plane-runtime/src/ess_reference.rs pins `VERSION = "0.53.0"` and embeds `resources/ess-0.53.0/ess.schema.json`; it refuses any other toolchain. With 0.54.0 first on PATH, `native_loom_recovers_missing_and_invalid_specification_and_queues_validated_plan` (crates/control-plane-runtime/tests/planner.rs) fails (wave-3 implementor run, 2026-10-06 ~21:50); with 0.53.0 it passes (36 planner tests, coordinator run 22:00).
- Waves 3 to 5 gate with ESS 0.53.0 first on PATH (`ess specify toolchain install 0.53.0`, binary in the ESS toolchain cache), matching CI.

- story:typed-satisfaction-receipt's specification trial (a struct receipt with a declared conversion) validates under ESS 0.55.0 (`controlplane v1 — 3 file(s), valid`, 2026-10-07); this story lands first so that story runs on the new pin.

- ESS 0.55.0 takes an exclusive lock on the nearest existing ancestor of an `ess generate` output root that does not exist yet; with the shared temporary directory as that ancestor, concurrent runs from other sessions fail with "output ownership busy … (os error 11)" (seen on 2026-10-07 in another repository's gate). Until ESS releases its fix, this repository's gates run `ess generate` with `TMPDIR=<tree>/.scratch/tmp` or with the output's parent created first; a step that fails only on that error is rerun with the private TMPDIR, and the rerun is named in the pull request.

## Acceptance

- `ess specify validate --path ess --strict-requires` passes with `requires: ess 0.56.0`, and CI installs 0.56.0 with the checksum from the release's `SHA256SUMS`.
- The runtime's authoring reference names 0.56.0 and embeds that release's schema; the planner test above passes with 0.56.0 first on PATH.
- `cargo run --locked -p control-plane-xtask -- generated-check`, `cargo run --locked -p control-plane-xtask -- conformance` and `cargo run --locked -p control-plane-xtask -- spec-history-check` pass with 0.56.0; every change `ess verify diff` reports between the two compiled models is acknowledged or shown to be none.

## Scope

Inferred: ess/ess-inputs.yaml, .github/workflows/check.yml, crates/control-plane-runtime/src/ess_reference.rs, crates/control-plane-runtime/resources/, generated/, ess/spec-acknowledgements.json.

## Release changes to expect

Changes between 0.53.0 and 0.56.0 that can reach this repository, from the release notes of 0.54.0, 0.55.0 and 0.56.0:

- 0.55.0: a `{generated: true}` payload value of type `Optional<T>` is read from a new context port method `generate_optional_<t>`; a context that implements the port must add it.
- 0.56.0: `ESS-COMMAND-004` refuses an accepting or `external:` branch declared before a held-state branch when one request can satisfy both guards; the fix is to reorder.
- 0.56.0: generated Rust type libraries declare a default-on `exact-numbers` feature instead of forcing `serde_json/arbitrary_precision`; generated `Cargo.toml` changes.
- 0.56.0: `.ess-output/state.json` becomes `ess-output-state/3`; a CI pin older than 0.56.0 refuses it, so the CI pin moves in the same commit as the regeneration.
- 0.56.0: concurrent `ess generate` runs under one shared `$TMPDIR` no longer refuse each other as `output ownership busy`.
