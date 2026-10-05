---
format: aep.planning-md/3
id: epic:bootstrap
kind: epic
status: draft
title: Standalone autonomous control-plane with local web console
relations:
- serves: vision:autonomous-engineering
revision: 1
---
## Outcome
Public beyond10x/control-plane hosts workspace registration, goal-driven planning, implementation across repositories, independent review and autonomous verified merge. The UI, service, specification, planning store and implementation are all in this repository. The operator explicitly approved the plan and clarified a new repository, not an implementation inside Loom. Use ess/22.

## Constraints
Rust only, clap CLI, server-rendered local UI. Foundation runtime dependencies only; repository publishing tools are an explicit exception. One active goal per workspace, three workers, one active repository change, three repairs and sixty minutes per attempt by default. Existing Codex login via LLM. No release/deployment or hosted tenancy. Store failures stop effects; ambiguous publication reconciles before retry.

## ESS
Typed model: ess/system.yaml, ess/domains/host.yaml. Generate contracts; named story scenarios extend generated lifecycle coverage with real adapter tests.
