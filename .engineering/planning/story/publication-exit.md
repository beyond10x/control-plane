---
format: aep.planning-md/3
id: story:publication-exit
kind: story
status: draft
title: A publication the remote never received can be closed and retried
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:ess-design-review-2026-10-06
- informed_by: review-result:control-plane-review-2026-10-06
- depends_on: story:terminal-goal-edits
- depends_on: story:bounded-progress-records
scope:
- confidence: cited
  path: crates/control-plane-core/src/guards.rs
- confidence: inferred
  path: crates/control-plane-core/src/tests.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
- confidence: cited
  path: ess/domains/host.yaml
- confidence: inferred
  path: generated
revision: 5
---
## Outcome

When the publisher has exited and the observed target does not contain the candidate, the intent is closed as not published. The assignment can then be repaired or cancelled, and its repository slot is released. An observation that fails (remote unreachable) never closes an intent, and an unchanged unresolved intent stops writing a progress record on every tick.

## Evidence

- ess/domains/host.yaml:284-316: PublicationIntent moves Prepared → Uncertain → Confirmed and has no other terminal state.
- crates/control-plane-core/src/guards.rs:297-306 refuses Repair and Cancel while any intent exists; guards.rs:371-385 requires a confirmed receipt for Reconcile; guards.rs:30-35 counts Blocked as active, so the common-directory slot stays held.
- crates/control-plane-runtime/src/fleet.rs:1684-1735 (`reconcile_publications`) marks Uncertain and calls `block()` on every tick; `block()` always appends a progress decision (fleet.rs:1726).
- Probe `probe_unpublished_intent_has_an_exit` (recorded in review-result:control-plane-review-2026-10-06, not committed) failed on f510e7f: Repair, Cancel and Reconcile refused; after CancelGoal a second workspace's claim on the same repository was refused.
- review-result:ess-design-review-2026-10-06 row 13 (contradicts epic:bootstrap "ambiguous publication reconciles before retry").

## Acceptance

- `unpublished_intent_closes_and_frees_repository`: after the probe sequence, the new close command applies; then CancelAssignment applies and a second workspace's ClaimAssignment on the same repository applies.
- `failed_observation_keeps_intent_open`: an observation error leaves the intent Uncertain and the assignment Blocked.
- `published_candidate_still_reconciles`: the existing uncertain-then-observed path still reaches Merged.
- `unchanged_unresolved_publication_appends_once`: ten fleet ticks over the same unresolved intent append at most one progress decision.
- `close_command_validates`: `ess specify validate --path ess --strict-requires` passes with the not-published state and the close command declared in ess/domains/host.yaml.
- `close_command_scenarios_pass`: the scenarios `ess verify conform synthesize` emits for the close command pass against the durable conformance target.
- The change ids `ess verify diff` reports for this change are listed in the acknowledgement file of story:spec-history-gate.

## Scope

Cited: ess/domains/host.yaml, crates/control-plane-core/src/guards.rs, crates/control-plane-runtime/src/fleet.rs (`reconcile_publications`, `block`, `publish`). Inferred: generated/, crates/control-plane-core/src/tests.rs, crates/control-plane-runtime/tests/fleet.rs.
