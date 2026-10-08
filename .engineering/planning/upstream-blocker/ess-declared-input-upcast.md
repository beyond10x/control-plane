---
format: aep.planning-md/3
id: upstream-blocker:ess-declared-input-upcast
kind: upstream-blocker
status: open
title: ESS declared upcast of older command inputs on replay
relations:
- blocks: story:typed-satisfaction-receipt
revision: 1
---
## What is blocked

story:typed-satisfaction-receipt cannot type SatisfyGoal's receipt: stored SatisfyGoal decisions carry the receipt as text, and under a typed input `Store::open` refuses them (`body.satisfaction_receipt: expected an object, found a string`). ESS 0.56.0 has no declared way to read an older command input.

## Upstream

- ess `story:an-older-command-input-is-read-on-replay-through-a-declared-upcast` (draft), planned after ESS 0.57.0 and 0.58.0. ESS holds the reproduction from the archive of tree cp-wave7-typed-satisfaction-receipt.

## Clears when

An ESS release declares an upcast from the text receipt to the typed one, and the recorded history fixture opens under the typed input with every view equal to its recorded views.
