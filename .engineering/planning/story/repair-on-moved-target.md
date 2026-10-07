---
format: aep.planning-md/3
id: story:repair-on-moved-target
kind: story
status: draft
title: A repair after a moved target takes the target's current head
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

An assignment whose publication was closed as not published after its target moved can be attempted again on the target's current head, instead of staying Blocked until an operator cancels it.

## Evidence

- crates/control-plane-runtime/src/fleet.rs:938-946 (at 2c810ff): a repair keeps the claimed `base_revision`, and deliver refuses "target base changed; queued plan must be reconciled before another attempt" once the target has moved. No command refreshes `base_revision`.
- review-result:adversary-publication-exit-pass-2 finding 1: the case `publication_closed_after_its_target_moved_is_retried_with_a_new_intent` measured the assignment Blocked at attempt 2 with no second intent.
- story:publication-exit settled this for wave 4 by blocking such an assignment once with a plain reason, without spending an attempt; this story replaces that block with a real retry.

## Acceptance

- The specification declares how a repair takes a fresh base (an input of RepairAssignment or a separate command) in ess/domains/host.yaml, and it validates (`ess specify validate --path ess --strict-requires`, a `task check` step).
- `repair_after_moved_target_takes_the_current_head`: after a close because the target moved, a repair records the target's current head as the assignment's base, and the next attempt creates a second publication intent.
- `repair_on_unchanged_target_keeps_its_base`: a repair after a grace-period close on an unchanged target keeps the claimed base.
- The change ids `ess verify diff` reports are acknowledged in ess/spec-acknowledgements.json (`spec-history-check`, a `task check` step).

## Scope

Inferred: ess/domains/host.yaml, generated/, ess/spec-acknowledgements.json, crates/control-plane-core/src/guards.rs, crates/control-plane-runtime/src/fleet.rs (`deliver`), crates/control-plane-runtime/tests/fleet.rs.
