---
format: aep.planning-md/3
id: story:typed-satisfaction-receipt
kind: story
status: implemented
title: SatisfyGoal's revision precondition is declared through a typed receipt
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: decision-blocker:satisfy-goal-precondition-in-ess
- depends_on: story:acceptance-edit-ordering
- depends_on: story:ess-055-upgrade
scope:
- confidence: inferred
  path: crates/control-plane-core/src/guards.rs
- confidence: inferred
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: ess/domains/host.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T03:13:39Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-08T03:13:39Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-08T12:52:33Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

SatisfyGoal's precondition "the receipt names the goal's current revision" is declared in the specification and enforced by the generated behaviour, because the receipt is a declared structure instead of free text. The host guard that enforces it today (crates/control-plane-core/src/guards.rs:248-277) is then replaced by the generated rule.

## Evidence

- decision-blocker:satisfy-goal-precondition-in-ess: keep the host guard for wave 5; type the receipt in the specification next, without adding an input to SatisfyGoal.
- With the receipt typed as String, ESS 0.53.0 and 0.55.0 both refuse the predicate `revision != input.satisfaction_receipt.goal_revision`: `[unobservable_fact] … cannot select goal_revision from String` (domains/host.yaml:700, coordinator trial 2026-10-07).
- A trial that types only SatisfyGoal's input validates under ESS 0.55.0 and 0.53.0 (`controlplane v1 — 3 file(s), valid`): the diff against ess/domains/host.yaml at 05d0f45 is

```text
697c697
<     type: String
---
>     type: controlplane.host.SatisfactionReceipt
698a699,702
>   - name: stale-revision
>     when_subject:
>       predicate: revision != input.satisfaction_receipt.goal_revision
>     error: controlplane.host.GoalStateConflict
1736a1741,1744
> conversions:
> - from: controlplane.host.SatisfactionReceipt
>   to: String
>   because: the goal keeps the receipt it was satisfied with as text, and an edit clears it
1737a1746,1752
> - name: controlplane.host.SatisfactionReceipt
>   kind: struct
>   fields:
>   - name: goal_revision
>     type: Integer
>   - name: evidence
>     type: String
```

- Typing the Goal's and the event's receipt fields as well is refused: CreateGoal and UpdateGoal set the receipt to the literal `''`, and a `sets:` literal cannot be a structure. The trial therefore keeps those fields as text through the declared conversion.
- Not yet checked: the receipt the fleet sends today (crates/control-plane-runtime/src/fleet.rs:2596) carries more than a revision, and the stored SatisfyGoal decisions hold it as text; whether they still replay against a typed input is this story's first question.

## Acceptance

Revised on 2026-10-08 (decision-blocker:typed-receipt-replay, revised decision): the receipt stays text and SatisfyGoal gains an optional revision input.

- ess/domains/host.yaml keeps `satisfaction_receipt: String` and adds `receipt_revision: Optional<Integer>` to SatisfyGoal; the `stale-revision` outcome is guarded by `when: defined(receipt_revision)` together with `revision != input.receipt_revision` on a Running goal; `ess specify validate --path ess --strict-requires` passes and `xtask generate` leaves no unmet capability.
- `satisfy_with_a_receipt_for_an_old_revision_is_a_declared_refusal`: SatisfyGoal with `receipt_revision` naming an earlier revision gets the declared `stale-revision` outcome from the generated behaviour, and the goal stays Running.
- `recorded_satisfactions_still_replay`: the recorded history fixture opens with no host migration and every view equals its recorded views after the change; its four stored SatisfyGoal decisions carry no `receipt_revision`.
- The change ids `ess verify diff` reports are acknowledged in ess/spec-acknowledgements.json (`cargo run --locked -p control-plane-xtask -- spec-history-check`), and the synthesized conformance scenarios for the new outcome pass (`cargo run --locked -p control-plane-xtask -- conformance`).
- Every SatisfyGoal sender (crates/control-plane-runtime/src/fleet.rs `satisfy_goals`, crates/control-plane-app/src/live.rs, crates/control-plane-xtask/src/history.rs where it records new decisions) sends `receipt_revision`; the revision comparison in guards.rs for SatisfyGoal is removed once the generated rule covers it, and story:acceptance-edit-ordering's cases still pass.

## Scope

Inferred: ess/domains/host.yaml, generated/, ess/spec-acknowledgements.json, crates/control-plane-core/src/guards.rs, crates/control-plane-core/src/tests.rs, crates/control-plane-runtime/src/fleet.rs (`satisfy_goals`), crates/control-plane-core/tests/fixtures (recorded history).

## Planning

Wave 7 (2026-10-08): unparked with the optional-input form (decision-blocker:typed-receipt-replay, revised). Runs after story:spec-owned-admission merges into control-plane/wave-7, because both write ess/domains/host.yaml and generated/. The earlier typed-receipt trial is in the archive of tree cp-wave7-typed-satisfaction-receipt.
