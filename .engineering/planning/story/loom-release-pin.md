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
revision: 5
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

## Breaks crossed

Move: `b10x-loom-sdk` from `rev = "1b25fe89…"` (after Loom 0.1.0, before 0.2.0) to `tag = "0.20.0"`
(`91a427dc`). With it, this repository's own pins move to the sources Loom 0.20.0 uses, so that
nothing resolves twice: `b10x-canon` rev `f6f843c` -> `tag = "0.4.0"` (control-plane-protocol) and
every `b10x-llm-*` rev `8618681` -> `tag = "0.4.0"` (control-plane-runtime, control-plane-app
dev-dependency). Both old revs are ancestors of those tags. The canon and llm moves needed no
source change.

`cargo check -p control-plane-runtime --all-targets` on the moved pins, before any adaptation
: 6 errors, exactly the rows below. After the adaptation the same command over
runtime, app and xtask: exit 0.

| Break | Adapted here | Loom version and CHANGELOG line |
|---|---|---|
| `EffectOutcomePerformed` gains `attempt: Option<ConnectorAttemptId>` (E0063) | crates/control-plane-runtime/src/engine.rs:860, src/fleet.rs:1727, src/fleet.rs:3030 (`attempt: None`: host operations, not Connector invocations) | 0.4.0: "`EffectOutcomePerformed` names the Connector attempt the invocation produced (`attempt`, an optional `ConnectorAttemptId`, present exactly when the effect was invoked through a Connector)" |
| `EffectOutcomePerformed` gains `audit: Option<ConnectorAuditRef>` (E0063) | same three sites (`audit: None`) | 0.9.0: "`EffectOutcomePerformed` names the read's execution audit record (`audit`, ...). Hosts that build `ActionBindingData` or `EffectOutcomePerformed` add the new field" |
| `CanonGovernor::open_case` takes `inputs: CaseInputs` (E0061) | crates/control-plane-runtime/src/governance.rs:38 (`CaseInputs::none()`: this repository's protocols declare no case inputs) | 0.20.0: "Breaking: `CanonGovernor::open` and `open_case` take the case inputs as a third argument" |
| `SessionData` gains `boundary_refusals: i64` (E0063) | crates/control-plane-runtime/src/loom_model.rs:88 (`boundary_refusals: 0`) | 0.15.0: "**Breaking:** the generated `SessionData` gains `boundary_refusals`. A caller that builds the session it hands `Loom::run_loop` or `LoopExecutor` sets it to `0`" |
| `LoopStop::is_completed` removed (E0599) | crates/control-plane-runtime/src/loom_model.rs:144 (`outcome.stop == LoopStop::Completed`) | 0.14.0: "`LoopStop::is_completed` is removed; compare with `LoopStop::Completed`." |
| Behaviour: a step performed under an authority grant no longer ends the Run `AwaitingApproval`; the executor runs again | none needed: the publication executor (crates/control-plane-runtime/src/fleet.rs, `impl AgentExecutor for Publication`) already answers a second call with `Suspended(Evidence["merge_observation"])`, the effect runs once (`invoked` flag), and the result is read from the effect, not from the Run's end. `native_loom_delivers_candidate_through_checks_review_and_observed_publication` and the publication attack suites pass | 0.16.0: "**Breaking:** a step performed under an authority grant passes Commission's approval gate." |

Foundation graph (crates/control-plane-xtask/src/foundation.rs, `fn admitted`, Loom names only):
Loom 0.20.0 brings `b10x-loom-connectors` (0.8.0), `b10x-loom-protocols` (0.4.0) and the generated
`intake` crate into the SDK's graph; all three are added. `foundation-check` then refuses
non-Loom crates the SDK now pulls in (`b10x-canon-expr`, `b10x-canon-engineering-assertions`,
`b10x-assertion-providers`, `connectors-client`, `connectors-core`, `b10x-substrate-host`,
`b10x-substrate-wire`); they are admitted inside the SDK only, in the unit commit after the repin
(patch `foundation-non-loom.patch`, measured green when applied).

## Loom defects

- CHANGELOG 0.20.0 says `open_case` takes "the case inputs as a third argument". The signature
  (`crates/loom-governor/src/lib.rs:643` at 0.20.0) is
  `open_case(&self, case: CaseId, protocol: &str, revisions: BTreeMap<String, String>, inputs: CaseInputs)`:
  the inputs are the fourth parameter after `self` (rustc: "argument #4 of type
  `loom_sdk::CaseInputs` is missing"). The entry names the break, but its position is wrong.
- No break was found that no CHANGELOG entry names.
