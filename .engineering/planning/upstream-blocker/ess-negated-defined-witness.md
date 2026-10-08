---
format: aep.planning-md/3
id: upstream-blocker:ess-negated-defined-witness
kind: upstream-blocker
status: open
title: ESS mutation witness for a negated defined() guard on an optional input
relations:
- blocks: story:remove-stale-revision-mutation-exemption
revision: 3
---
## What is blocked

story:remove-stale-revision-mutation-exemption: `ess mutate` on ess 0.56.0 synthesizes no witness for `guard-negate/controlplane.host.SatisfyGoal/stale-revision` (`when: defined(receipt_revision)` becomes `when: not (defined(receipt_revision))`, while `when_subject` reads `input.receipt_revision`) and adds refusal ESS-SYNTH-003 on `controlplane.host.SatisfyGoal/outcome/stale-revision`.

## Upstream

- ess `story:negated-defined-optional-input-witnessed-beside-when-subject` (draft), after ess #501, in the ESS minor after 0.57.0. Reproduction shape given to ESS; original: `ess/domains/host.yaml` SatisfyGoal on the wave-7 merge of story:typed-satisfaction-receipt, `task mutation`.

## Clears when

An ESS release reports that mutant killed or witnessed; the pin moves to it.
