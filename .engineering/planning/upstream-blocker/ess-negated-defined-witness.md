---
format: aep.planning-md/3
id: upstream-blocker:ess-negated-defined-witness
kind: upstream-blocker
status: cleared
title: ESS mutation witness for a negated defined() guard on an optional input
relations:
- blocks: story:remove-stale-revision-mutation-exemption
revision: 5
transitions:
- {from: "open", to: "cleared", at: "2026-10-12T00:08:19Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## What is blocked

story:remove-stale-revision-mutation-exemption: `ess mutate` on ess 0.56.0 synthesizes no witness for `guard-negate/controlplane.host.SatisfyGoal/stale-revision` (`when: defined(receipt_revision)` becomes `when: not (defined(receipt_revision))`, while `when_subject` reads `input.receipt_revision`) and adds refusal ESS-SYNTH-003 on `controlplane.host.SatisfyGoal/outcome/stale-revision`.

## Upstream

- ess `story:negated-defined-optional-input-witnessed-beside-when-subject` (draft), after ess #501, in the ESS minor after 0.57.0. Reproduction shape given to ESS; original: `ess/domains/host.yaml` SatisfyGoal on the wave-7 merge of story:typed-satisfaction-receipt, `task mutation`.

## Clears when

An ESS release reports that mutant killed or witnessed; the pin moves to it.

## Checked against ESS 0.59.0 (2026-10-11)

ESS 0.57.0 and later score this mutant `equivalent`, not unwitnessed: the ess-mutation-report/5 record carries `unsatisfiable_guard: (not (defined(receipt_revision)) and (state == Running and revision != input.receipt_revision))`, and the summary cites ESS-MUTATE-005 (no input satisfies the mutated guard). A mutant no input can reach needs no witness, so the gap this blocker stood for is closed: cleared on that evidence, beside "killed or witnessed". story:remove-stale-revision-mutation-exemption deletes the exemption and admits an equivalent mutant only with that evidence.
