---
format: aep.planning-md/3
id: story:loom-release-pin
kind: story
status: active
title: Build on a released Loom
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/control-plane-runtime/Cargo.toml
- confidence: cited
  path: crates/control-plane-runtime/src/engine.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/src/governance.rs
- confidence: cited
  path: crates/control-plane-runtime/src/loom_model.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T02:58:09Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-14T02:58:09Z", actor: "human:timo", revision: 4}
---
# Build on a released Loom

## Outcome

Control-plane builds against a Loom release, not an unreleased commit, so the integration repository proves the same Loom crates and `b10x-loom` contract its consumers read. The `b10x-loom-sdk` pin in `crates/control-plane-runtime/Cargo.toml` names the tag of the newest Loom release (0.20.0 on 2026-10-14) instead of commit `1b25fe89` (after 0.1.0, before 0.2.0), and every Loom crate resolves once.

## Basis

The pin predates every Loom release from 0.2.0 to 0.20.0. Consumers read Loom as its released crates and as the `b10x-loom` command, whose `run --output jsonl` events are a stable contract; an integration repository on an unreleased commit proves neither.

## Acceptance

- The workspace pins `b10x-loom-sdk` with `tag = "0.20.0"` (or the newest Loom release when the unit starts), and `Cargo.lock` holds each `b10x-loom-*` package once, from that tag. `cargo tree --duplicates` lists no Loom crate.
- Every Loom break crossed between the old pin and the tag is adapted at its call site in `crates/control-plane-runtime/src/` and recorded on this story under "Breaks crossed", each with the CHANGELOG entry that names it. A break no CHANGELOG entry names is recorded as a foundation defect in Loom, not worked around silently.
- The existing runtime suite stays green on the new pin, with these named tests among it: every_role_runs_through_one_native_loom_session_path, native_runtime_events_reach_the_workspace_stream, native_loom_delivers_candidate_through_checks_review_and_observed_publication, review_and_publication_require_current_independent_evidence.
- Spec first: the move changes no ESS entity of this repository unless a crossed break changes a projected contract; such a change is modelled in `ess/` before code.

## Out of scope

The foundation checks that keep the pin a tag and the Loom crates single (story:loom-pin-foundation-checks). New Loom features beyond adapting to breaks. Any real-model run.
