# Control-plane

Standalone Rust product: local autonomous planner, implementation fleet and web console. Read README.md for usage.

## Serves

- O1 — governed reach: check declared authority before effectful work.
- O5 — the generic agent platform: configure autonomous work and inspect its record.

## Working rules

- Backend and tooling are Rust; CLIs use clap derive. The frontend is Vue, bundled and embedded in the Rust binary. The Rust rule does not restrict frontend code (operator clarification, 2026-10-06). Live updates use SSE without page refresh.
- Use managed worktrees. Keep primary checkouts clean. All commits and GitHub writes use the bot and repository-required publishing tools; never bypass hooks.
- ESS under ess/ owns domain types and lifecycles. Specify first, record a failing test, generate Rust contracts, then implement adapters. Never hand-edit generated output.
- AEP under .engineering/ owns the engineering plan. Artifact writes use aep plan artifact. Stories name conformance scenarios, scope, dependencies and evidence.
- Runtime dependencies use foundation components. Repository-configured publishing tools are an operational exception.
- Loom owns model turns, compaction, structured proposals and sessions; Commission admits every planner, implementor and publication effect. Use CanonGovernor with admitted protocol inputs, never a private Governor or direct Canon admission loop. Host adapters authenticate evidence and bind provider, persistence, authority and repository operations.
- Model output cannot grant authority or supply test/merge evidence. Review must use a distinct execution context. Recheck authority and candidate before publication.
- One active change per Git common directory across workspaces. Effects use managed worktrees.
- Storage failures stop effects. Reconcile uncertain publication before retry. An empty queue does not prove goal completion.
- task check validates ESS, generated drift, AEP, formatting, clippy, tests and dependency boundaries. Tests use scripted models and disposable repositories.
- Build output is per worktree. Private paths, policy and credentials stay outside public source.
