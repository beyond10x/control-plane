---
format: aep.planning-md/3
id: verification-report:ess-hardening-2026-10-06
kind: verification-report
status: draft
title: 'ESS hardening: techniques 8, 7, 1 and 6 with planted defects'
relations:
- verifies: executable-system-specification:control-plane
- informed_by: review-result:ess-design-review-2026-10-06
revision: 1
---
# ESS hardening run, 2026-10-06

Specification ess/ at f510e7f, ESS 0.53.0 (pinned by `requires: ess 0.53.0`). Baseline: `task check` exit 0 with 167/167 conformance scenarios. `ess verify conform synthesize`: 167 scenarios, 0 refusals.

## Technique 8: design review

Recorded in review-result:ess-design-review-2026-10-06: 16 missing, 2 contradicts, 2 spec-only, 1 unclear. Planted-defect check not run (the reviewer was not independent), so this result is unverified by the rule "a check nobody has seen fail is not evidence".

## Technique 7: spec diff in the gate

- History of ess/ on the branch, 17d4171 → f510e7f: `ess verify diff --compatibility` lists 146 changes, 41 compatible, 105 unknown, 0 breaking.
- Planted defect: `Blocked` removed from `Assignment.repair.from`. It is classified `transition-route-changed`, relation `changed`, verdict `unknown` (history `unknown`).
  - `--fail-on breaking`: exit 0. The narrowing passes.
  - `--fail-on breaking-or-unknown`: exit 4. Caught.
- Finding: the gate this product needs is `breaking-or-unknown` with an acknowledgement file, because replay refuses changed history (crates/control-plane-core/src/lib.rs:214). A from-state removal reported as `unknown` rather than `narrowed`/breaking is a candidate ESS classification issue; not filed.

## Technique 1: mutation audit

`ess verify conform mutate --path ess --target interpreted`: 213 mutants, 170 killed, 0 survived, 43 stillborn, 0 inconclusive, 0 unwitnessed, 0 equivalent; 19 emit-swap sites unavailable (`no_compatible_event_alternative`); exit 3 because of the unavailable sites; 59.9 s.
- Every declared lifecycle and `sets:` rule is pinned by some scenario.
- The guard classes (guard-boundary, guard-negate, guard-connective, precedence-swap) found no site: the specification declares no `when:` guard except DeleteGoal's `when_subject_state` branches. The admission rules live in guards.rs, so this audit cannot see them.
- Run against the interpreted reference target, not the durable ContractStore. Both execute the same generated behaviour; the durable target was not used because the xtask runner has no suite argument.

## Technique 6: guard analysis

Nothing to analyse beyond DeleteGoal: its four `when_subject_state` branches cover all four Goal states. No dead, overlapping or gap-leaving guard exists because no input or stored-field guard exists.

## Implementation probes (beside the catalogue)

Three failing tests, each green after a minimal fix and then reverted: credentials inherited by child processes; UpdateGoal on a Satisfied goal; no exit for an unpublished intent. Details in review-result:control-plane-review-2026-10-06.

## Not run

- Technique 2 (random sequences) and 4 (determinism): the explorer ships in the Go and TypeScript suite packages; this implementation is Rust, so a Target adapter over the HTTP or Store API would be needed first.
- Technique 3 (caller replay) and 5 (metamorphic relations): need a reference model; deferred until the admission rules are in the specification, since today the model would omit them.

## What changes the result

Moving the admission rules into the specification (story:spec-owned-admission) gives techniques 1 and 6 something to measure. The spec-history gate (story:spec-history-gate) has to land first, because replay of existing state refuses changed outcomes.
