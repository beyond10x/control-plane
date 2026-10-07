---
format: aep.planning-md/3
id: story:conformance-evidence-record
kind: story
status: draft
title: The gate's conformance report becomes AEP evidence with its real run instant
relations:
- serves: vision:autonomous-engineering
- informed_by: review-result:plan-audit-2026-10-06
- depends_on: story:spec-owned-admission
- decomposes: epic:bootstrap
scope:
- confidence: cited
  path: crates/control-plane-xtask/src/target.rs
revision: 3
---
## Outcome

The conformance report `task check` writes is recorded with `aep plan artifact evidence --from`, carrying the real run instant, and executable-system-specification:control-plane moves to conforming on that record. This completes epic:bootstrap's ESS commitment ("Generate contracts; named story scenarios extend generated lifecycle coverage").

## Evidence

- AEP 0.68.0 refuses the ESS 0.53 suite: `Error: UnknownField at $suite.provenance.scenario_initial_state: closed count-stage vocabulary` (re-run 2026-10-06 on a throwaway copy of the store). Held as dependency-blocker:aep-imports-ess-053-suites.
- crates/control-plane-xtask/src/target.rs:207 runs `Runner::for_suite`, whose default clock starts at 2023-11-14T22:13:20Z; report.json `completed_at` is 1700000062500, and the import takes that as the evidence instant.

## Acceptance

- `conformance_report_carries_run_instant`: the report's `completed_at` falls between the wall-clock start and end of the `task check` conformance step.
- `conformance_report_imports_as_evidence`: after the blocker clears, `aep plan artifact evidence executable-system-specification:control-plane --from .scratch/conformance/report.json --suite generated/conformance.json` records the evidence, and `aep plan artifact move executable-system-specification:control-plane --to conforming` succeeds.

## Scope

Cited: crates/control-plane-xtask/src/target.rs. Ordered after story:spec-owned-admission, which changes the same file.
