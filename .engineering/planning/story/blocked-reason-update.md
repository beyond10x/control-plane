---
format: aep.planning-md/3
id: story:blocked-reason-update
kind: story
status: draft
title: A Blocked assignment's reason is replaced when its cause changes
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-publication-exit-pass-2
- depends_on: story:publication-exit
scope:
- confidence: inferred
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
- confidence: inferred
  path: ess/domains/host.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 2
---
## Outcome

A Blocked assignment's reason can be replaced when the cause changes, so the console's "Current reason", the attention strip and the goal card show the cause that holds now, not the first one recorded.

## Evidence

- review-result:adversary-publication-exit-pass-2 finding 4: after a publication intent is closed as not published, frontend/src/App.vue:66 still shows "Publication outcome unresolved…".
- crates/control-plane-runtime/src/fleet.rs:254 (at 2c810ff) sends BlockAssignment only when the assignment is not already Blocked; ess/domains/host.yaml declares no transition or command that replaces a Blocked assignment's reason.
- decision-blocker:stale-blocked-reason chose to close wave 4 without this fix and to plan it for wave 5.

## Acceptance

- ess/domains/host.yaml declares how a Blocked assignment's reason is replaced (a Blocked to Blocked transition of BlockAssignment, or a separate command), with its actor and outcomes, and the specification validates (`ess specify validate --path ess --strict-requires`, a `task check` step).
- The scenarios `ess verify conform synthesize` emits for it pass against the durable conformance target (`cargo run --locked -p control-plane-xtask -- conformance`, a `task check` step).
- The change ids `ess verify diff` reports are acknowledged in ess/spec-acknowledgements.json (`spec-history-check`, a `task check` step).
- `closed_publication_replaces_the_blocked_reason`: after an intent closes as not published, the assignment's reason names the close, and ten further ticks over the unchanged state replace nothing.
- `unchanged_blocked_reason_is_not_replaced`: a Blocked assignment whose cause is unchanged keeps its reason and appends no record.

## Scope

Inferred: ess/domains/host.yaml, generated/, ess/spec-acknowledgements.json, crates/control-plane-core/src/guards.rs, crates/control-plane-runtime/src/fleet.rs (`block`), crates/control-plane-runtime/tests/fleet.rs.

## Planning

Planned for wave 5, beside story:spec-owned-admission and story:console-goal-cards. Both this story and story:spec-owned-admission write ess/domains/host.yaml, generated/ and ess/spec-acknowledgements.json, so the wave derivation will keep them apart unless one lands first.
