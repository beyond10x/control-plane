---
format: aep.planning-md/3
id: story:remove-stale-revision-mutation-exemption
kind: story
status: draft
title: The mutation audit admits no exemption once ESS witnesses the negated stale-revision guard
relations:
- depends_on: story:typed-satisfaction-receipt
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
revision: 2
---
## Outcome

The mutation audit admits no exemption: `guard-negate/controlplane.host.SatisfyGoal/stale-revision` is killed or witnessed by ESS-synthesized scenarios, and the one-id exemption in `crates/control-plane-xtask/src/mutation.rs` is gone.

## Evidence

- story:typed-satisfaction-receipt added `stale-revision` guarded by `when: defined(receipt_revision)`; on ess 0.56.0 `task mutation` reports 13 mutants, 12 killed, 1 unwitnessed (ESS-SYNTH-003).
- decision-blocker:stale-revision-guard-unwitnessed (cleared, B) admitted that one mutant id until ESS witnesses it.

## Acceptance

- The ESS pin moves to the release that clears upstream-blocker:ess-negated-defined-witness.
- `guard_mutants_are_killed` passes with the refreshed report and with the exemption constant deleted; the report shows 0 unwitnessed.
- `mutation_audit_admits_no_exemption`: a report with any unwitnessed mutant fails the audit.

## Out of scope

Any other mutation class or guard; changes to SatisfyGoal's behaviour.

## Scope

Inferred: crates/control-plane-xtask/src/mutation.rs, crates/control-plane-xtask/tests/fixtures/guard-mutation-report.json, crates/control-plane-xtask/tests/, the ESS pin (Taskfile.yml / CI workflow).
