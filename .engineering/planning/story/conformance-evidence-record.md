---
format: aep.planning-md/3
id: story:conformance-evidence-record
kind: story
status: implemented
title: The gate's conformance report becomes AEP evidence with its real run instant
relations:
- serves: vision:autonomous-engineering
- informed_by: review-result:plan-audit-2026-10-06
- depends_on: story:spec-owned-admission
- decomposes: epic:bootstrap
scope:
- confidence: inferred
  path: .github/workflows/check.yml
- confidence: inferred
  path: README.md
- confidence: cited
  path: crates/control-plane-xtask/src/target.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-10-08T14:47:10Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome

The conformance report `task check` writes is recorded with `aep plan artifact evidence --from`, carrying the real run instant, and executable-system-specification:control-plane moves to conforming on that record. This completes epic:bootstrap's ESS commitment ("Generate contracts; named story scenarios extend generated lifecycle coverage").

## Evidence

- AEP 0.68.0 refuses the ESS 0.53 suite: `Error: UnknownField at $suite.provenance.scenario_initial_state: closed count-stage vocabulary` (re-run 2026-10-06 on a throwaway copy of the store). Held as dependency-blocker:aep-imports-ess-053-suites.
- crates/control-plane-xtask/src/target.rs:229 (at 89c62c8) runs `Runner::for_suite`, whose default clock starts at 2023-11-14T22:13:20Z; report.json `completed_at` is 1700000062500, and the import takes that as the evidence instant.
- aep 0.69.0 release notes: "`aep plan artifact evidence --suite` admits the `scenario_initial_state` and `synthesis_seeds` provenance members ... that ESS 0.55.0 writes". The repository pins ESS 0.56.0 and, in CI, aep 0.68.0 (.github/workflows/check.yml:62, :109); the newest aep is 0.69.1. Whether 0.69.1 admits the ESS 0.56.0 suite is not yet observed.

## Acceptance

- `conformance_report_carries_run_instant`: the report's `completed_at` falls between the wall-clock start and end of the `task check` conformance step.
- Checked by reading the files, not by a test: .github/workflows/check.yml (both jobs) and README.md name aep 0.69.1 with the release checksum, and the pull request CI passes with it.
- Checked by the store, not by a test: with aep 0.69.1, `aep plan artifact evidence executable-system-specification:control-plane --from .scratch/conformance/report.json --suite generated/conformance.json` records the evidence, and `aep plan artifact move executable-system-specification:control-plane --to conforming` succeeds (the coordinator does this on the integration branch; the record is on executable-system-specification:control-plane).

## Scope

Cited: crates/control-plane-xtask/src/target.rs. Inferred: .github/workflows/check.yml, README.md (the aep pin). If aep 0.69.1 still refuses the suite, the refusal text goes verbatim into dependency-blocker:aep-imports-ess-053-suites and the clock fix ships alone.
