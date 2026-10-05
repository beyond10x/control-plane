---
format: aep.planning-md/3
id: story:contract-verification
kind: story
status: active
title: Real durable ESS conformance and generated drift gate
relations:
- decomposes: epic:bootstrap
- depends_on: story:workspace-host
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-xtask
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:47:07Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T20:47:07Z", actor: "human:timo", revision: 4}
---
## Outcome
Run the generated ESS suite against the real durable contract store and drift-check generated source before planner/fleet qualification. This extracts the independently executable contract gate from story:verification so feedback is available during the next implementation wave. The final two-repository qualification remains with story:verification.

## ESS first
Use ess/22 source and generated/conformance.json. The target drives production contract::ContractStore and observes actual generated outcomes, events and views backed by Eventlog SQLite. No interpreter, test-only state model, fabricated result or ignored scenario can count as conformance.

## Acceptance
All 149 current generated scenarios execute with zero failures and skips; a broken real adapter must make the suite fail. Regenerate Rust, OpenAPI and conformance into isolated output, compare emitted source bytes while excluding machine-local .ess-output ledgers, and fail on drift. Expose clap commands generated-check, generate and conformance for Taskfile.yml. Keep authored source in ordinary YAML. CI installs verified released ESS/AEP/Worktree tools and runs the repository checks.

## Scope
Inferred: crates/control-plane-xtask. Parent coordinator owns root manifests, Taskfile.yml, workflow and AEP. Story:verification may later extend this same gate after planner, fleet and console are integrated; it is dependent work, not concurrent ownership.

## Authorization
Covered by the operator's standing instruction to implement the existing standalone control-plane plan and use AEP waves. This moves contract feedback earlier without replacing the plan or reducing final qualification.
