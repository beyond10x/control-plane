---
format: aep.planning-md/3
id: story:admission-conformance-target
kind: story
status: draft
title: The full conformance suite runs through admission and repository-missing is declared
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: decision-blocker:admission-selectors-not-synthesized
- depends_on: story:spec-owned-admission
revision: 1
---
## Outcome

The full synthesized conformance suite runs through the same admission path as the console and runtime, and QueueAssignment's "repository missing" rule is declared in the specification, so the host fact for it in the specification README is removed.

## Evidence

- decision-blocker:admission-selectors-not-synthesized and the wave-7 page: through admission the suite gave 84 passed, 119 errored of 203; 109 were "repository not found", because QueueAssignment's repository guard cannot be declared beside its goal guards.
- ess 0.56.0 `generate synthesize --target rust` keeps such a command as an obligation (`136 capabilities: 135 generated, 1 obligation(s)`), construct "`when_related:` reading several related rows in one command"; synthesis accepts it (204 scenarios, 0 refusals).
- upstream-blocker:ess-generates-several-related-rows names the ESS story that lifts it.

## Acceptance

- `repository_missing_is_declared`: QueueAssignment declares `repository-not-found` beside its goal guards, and `xtask generate` leaves no unmet capability.
- `full_suite_passes_through_admission`: `cargo run --locked -p control-plane-xtask -- conformance` runs every synthesized scenario through the console/runtime admission path and reports 0 failed/error/unsupported/skipped.
- The "repository missing" row leaves the host-facts section of ess/README.md.

## Scope

Inferred: ess/domains/host.yaml, ess/README.md, ess/spec-acknowledgements.json, generated, crates/control-plane-xtask/src/target.rs, crates/control-plane-core/src/guards.rs.
