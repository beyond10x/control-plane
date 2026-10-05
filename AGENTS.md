# Control-plane

Standalone Rust product: local autonomous planner, implementation fleet and web console. Read README.md for usage.

- Everything that runs is Rust; CLIs use clap derive. The UI is server-rendered HTML/CSS.
- Use managed worktrees. Keep primary checkouts clean. All commits and GitHub writes use the bot and repository-required publishing tools; never bypass hooks.
- ESS under ess/ owns domain types and lifecycles. Specify first, record a failing test, generate Rust contracts, then implement adapters. Never hand-edit generated output.
- AEP under .engineering/ owns the engineering plan. Artifact writes use aep plan artifact. Stories name conformance scenarios, scope, dependencies and evidence.
- Runtime dependencies use foundation components. Repository-configured publishing tools are an operational exception.
- Model output cannot grant authority or supply test/merge evidence. Review must use a distinct execution context. Recheck authority and candidate before publication.
- One active change per Git common directory across workspaces. Effects use managed worktrees.
- Storage failures stop effects. Reconcile uncertain publication before retry. An empty queue does not prove goal completion.
- task check validates ESS, generated drift, AEP, formatting, clippy, tests and dependency boundaries. Tests use scripted models and disposable repositories.
- Build output is per worktree. Private paths, policy and credentials stay outside public source.
