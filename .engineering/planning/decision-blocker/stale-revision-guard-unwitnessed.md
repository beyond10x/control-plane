---
format: aep.planning-md/3
id: decision-blocker:stale-revision-guard-unwitnessed
kind: decision-blocker
status: cleared
title: 'SatisfyGoal stale-revision: its negated guard has no ESS witness; wait, exempt, or move the guard?'
relations:
- blocks: story:typed-satisfaction-receipt
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T08:16:50Z", actor: "human:timo", revision: 4}
---
## Question

The `stale-revision` refusal on SatisfyGoal is guarded by `when: defined(receipt_revision)`. ESS 0.56.0 cannot synthesize a witness for its negated guard, so the repository's mutation audit (`task mutation`) refuses the unit. Wait for ESS, ship the unit with a named exemption, or move the guard into the predicate?

## State

Unit tree cp-wave7-typed-satisfaction-receipt-2 (uncommitted, 36 files), ESS 0.56.0, 2026-10-08:

- `task mutation -- --keep` exit 201: 13 mutants, 12 killed, 0 survived, 1 unwitnessed.
- Unwitnessed: `guard-negate/controlplane.host.SatisfyGoal/stale-revision`, change "`when: defined(receipt_revision)` becomes `when: not (defined(receipt_revision))`", added refusal ESS-SYNTH-003 on `controlplane.host.SatisfyGoal/outcome/stale-revision`.
- `verdict()` in `crates/control-plane-xtask/src/mutation.rs:61` refuses any survived, inconclusive or unwitnessed mutant; `guard_mutants_are_killed` stays red.
- Every other gate step exits 0: validate, generate (136 capabilities, 0 obligations), generated-check, spec-history-check (24 acknowledged), conformance (204 passed), acceptance-check (122 names), clippy and tests of core, runtime, app, xtask.
- Trial: `defined(input.receipt_revision)` inside `when_subject` validates and gives 12 mutants with none on SatisfyGoal: the guard is then not audited.

## Options

| option | what happens | cost |
|---|---|---|
| A | unit 2 leaves wave 7; wave 7 ships unit 1 alone; ask ESS to synthesize a witness for a negated `defined(...)` guard; unit 2 waits on that release | the revision precondition stays in the host guard; wave 8 waits too (depends_on) |
| B | ship unit 2 now; the mutation audit admits exactly this mutant id, citing the ESS-SYNTH-003 refusal text, and the exemption is removed when the ESS release carries the witness | one named exemption in the audit until ESS ships |
| C | move `defined(...)` into `when_subject` | the guard leaves the mutation audit silently |

Recommendation: B. The refusal is declared and tested by conformance and `satisfy_with_a_receipt_for_an_old_revision_is_a_declared_refusal`; only the witness for the negated guard is missing, and the exemption is one id that expires with the ESS release.

## Decided

Option B, decided 2026-10-08 under the operator's delegated decision authority: story:typed-satisfaction-receipt ships in wave 7. The mutation audit in `crates/control-plane-xtask/src/mutation.rs` admits exactly one unwitnessed mutant, `guard-negate/controlplane.host.SatisfyGoal/stale-revision`, only while its added refusal is ESS-SYNTH-003 on `controlplane.host.SatisfyGoal/outcome/stale-revision`, and cites that refusal beside it. story:remove-stale-revision-mutation-exemption removes it once an ESS release witnesses a negated `defined(...)` guard (upstream-blocker:ess-negated-defined-witness).
