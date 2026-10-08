---
format: aep.planning-md/3
id: decision-blocker:typed-receipt-replay
kind: decision-blocker
status: open
title: 'Typed SatisfyGoal receipt: migrate stored receipts in host code, keep the host guard, or wait for ESS?'
relations:
- blocks: story:typed-satisfaction-receipt
revision: 1
---
## Question

Typing SatisfyGoal's receipt makes the stored SatisfyGoal decisions unreadable, and ESS 0.56.0 has no declared way to read an older input. Migrate stored receipts in host code, keep the host guard, or wait for ESS?

## State

Implementor trial on 2026-10-08, ESS 0.56.0, unit tree cp-wave7-typed-satisfaction-receipt (uncommitted, 27 files):

- `Store::open` on the recorded history fixture under the typed input: `generated command refused (400): {"refused":"body.satisfaction_receipt: expected an object, found a string"}`. The fixture holds 4 SatisfyGoal decisions (event versions 12, 90, 91, 102), each receipt JSON text such as `{"goal_revision":2,"kind":"goal_acceptance"}`.
- The story's design (struct, `conversions:` to String, `stale-revision`) validates but does not generate: `Error: generation left unmet capabilities` (131 generated, 2 obligations); synthesis: `InvalidSuite: the payload holds {"evidence":…,"goal_revision":1.0} and the step's own shape declares a String`.
- A form that generates: no conversion; Goal and SatisfyGoalApplied store `input.satisfaction_receipt.evidence`; `stale-revision` guarded by `all: [state == Running, revision != input.satisfaction_receipt.goal_revision]`. 132 capabilities all generated; synthesis 181 scenarios, 0 refusals (base 180, 0). It still meets the replay refusal above.
- Senders of SatisfyGoal: `crates/control-plane-runtime/src/fleet.rs` (`satisfy_goals`), `crates/control-plane-app/src/live.rs`, `crates/control-plane-xtask/src/history.rs`.

## Options

| option | what happens | cost |
|---|---|---|
| A | host-side replay migration in `Store::open`: a stored text receipt `s` becomes `{goal_revision: <parsed from s>, evidence: s}`; re-record the fixture with a `stale-revision` refusal | host code that reinterprets stored inputs; every existing store is read through it |
| B | keep the host guard; the story leaves the wave unimplemented | the precondition stays outside the specification |
| C | B now, and ask ESS for a declared way to read older command inputs (an input version or upcast); the story waits on it | waits on an ESS release |

Recommendation: C. It keeps stored history read only through declared behaviour, and the guard already enforces the rule today.
