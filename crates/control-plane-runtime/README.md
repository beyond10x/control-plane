# Autonomous planner and fleet runtime

`Supervisor` shares the durable core `Store` and an operator wake notification.
`tick()` inspects running goals and repository changes; `run(shutdown)` repeats
that reconciliation on notifications and bounded polling. Persisted Goal progress
precedes effects and makes unchanged completed or blocked planning idle across
restarts. An interrupted planning checkout is reused by its managed-worktree id.
Queued work from a superseded goal revision is cancelled with a retained reason
before replanning. Unavailable repositories become durable blockers; unrelated
workspaces keep running and do not wake each other's idle planners.

A bounded Loom commission selects narrow read, ESS-source, AEP CLI, and validation
actions. Canon evaluates the actual planning contract. A separate model context
critiques observed specification validation and selected authoritative AEP story
bodies. Actual ESS/AEP processes must succeed before the host accepts stories.
Assignments retain repository-qualified story ids, goal revision, managed tree id,
and immutable plan commit, retained on a named branch in the managed checkout.
Empty selections leave the goal running with a reason. `fleet_tick()` implements
queued stories with a configurable worker limit and exclusive Git common-directory
ownership across workspace aliases; `run()` drives both planner and fleet.

Workers use managed checkouts and accepted AEP scope, execute configured checks,
and obtain independent review in a fresh model context. Canon admits publication
only for the exact checked and reviewed candidate under current standing authority.
A durable publication intent precedes the repository-configured command. Its argv
supports `{candidate}`, `{expected_base}`, `{target}` and `{operation_id}`; the same
values are supplied as `CONTROL_PLANE_*` environment variables. Commands are parsed
as argv without a shell. The runtime observes `origin` itself: a successful command
exit alone is not merge evidence. Ambiguous effects reconcile before further writes.

Goal satisfaction requires every relevant assignment to have an observed merge,
real checks against every current repository target, and a separate final acceptance
review. Both scheduling paths share the durable acceptance-rejection guard, including
after database reopen. A goal revision, repository configuration, directory membership
or observed published target change permits reconsideration; activity timestamps and
unavailable target observations do not. Real model,
tool, check and publication progress is retained in bounded activity history.
`max_minutes` bounds each execution or acceptance attempt, not cumulative lifetime
across restarts; the durable assignment attempt count bounds retries. A changed base
or unresolved publication is an explicit blocker, never an automatic force repair.

`CodexAgentModel` uses the foundation LLM crate's `codex_model` login resolver.
Its single-turn `ModelPort` projection preserves LLM continuation provenance and streams
real activity to the host. Loom's `AgentLoop` validates structured proposals, compacts
context and files durable `SessionFile` records beside the host database. Planner,
implementor and all independent reviewers use this same path, with separate sessions.
Reopening a session deducts its recorded turns from the host's remaining turn limit.
Each native role receives its compact task brief once. Planning receipts and UI activity
are excluded; subsequent planner/implementor turns append new host observations and the
implementation frontier. The durable host store remains authoritative for effect checks.
Missing files, ESS validation diagnostics and unsuccessful inspection/build exits return observations; authority,
confinement, persistence and cancellation failures stop effects.

Read paths use the managed worktree namespace (`AGENTS.md`, `TASK.md`, `go.mod`), or
`workspace:<directory_id>/relative-file` for registered read-only context. Absolute
paths and parent traversal are refused without reading or remapping them. A typed
read-syntax refusal uses Commission's `EffectOutcome::Refused` and reaches the next
native Loom turn as bounded corrective feedback. An implementation attempt stops
after three malformed read requests; planners retain their existing unchanged-read
limit. These are adapter input limits, not a replacement agent loop. Unknown context
registrations, root/symlink escapes, permissions and other operational failures stay
fatal. Every path in a read batch is syntax-checked before any file in that batch is read.

The planner's `ess_schema` action reads bounded JSON-pointer fragments from the exact
ESS-owned authoring schema recorded in `resources/ess-0.53.0/README.md`. An empty pointer
returns the definition/property index. The resource must match `ess specify toolchain
which` in the specification directory; mismatches are explicit feedback. This adapter
neither invents syntax nor replaces ESS semantic validation.

Commission drives planner and implementation proposals and admits publication. Its
`CanonGovernor` evaluates host-admitted planning/source-delivery profiles; the host
authenticates verifier/reviewer receipts before submitting them as evidence. Case state
belongs to a bounded attempt and is journalled before updates become visible. Recovery
revalidates a fresh case against durable product state; a trace never authorizes replay
of an uncertain publication. The SDK pin includes the tested fallible store and supplied
protocol APIs on Loom's `feat/hosted-governor-contract` branch; it is not a new release.
Authentication failures, refused Git operations, unavailable validators and limits are
reported as blockers. `RuntimeConfig.commit_command` is trusted service/operator
configuration; its default invokes `b10x-gates bot`. Supply normal Gates policy
configuration or `B10X_GATES_POLICY`; model output cannot change commit authority.
Plain Git commits in tests are restricted to local disposable fixture repositories.

The model cannot invoke a shell, select an AEP store override, create evidence or
approval artifacts, write arbitrary repository YAML, or submit check/merge receipts.
Processes have output and time limits and terminate their process groups on
cancellation. Paused goals, changed revisions, and changed repository configuration
are checked before effects and queueing. Worktree leases cover planning and commits.

Run `cargo test -p control-plane-runtime`, `cargo fmt -p control-plane-runtime --check`,
and `cargo clippy -p control-plane-runtime --all-targets -- -D warnings`.
Integration fixtures use real installed AEP, ESS, Git and worktree CLIs. Their
isolated profiles and linked checkouts remain under `.scratch/planner` and
`.scratch/fleet` for review;
cleanup must use the worktree archive/finish/gc protocol with each fixture's XDG
configuration, never recursive removal of an active managed checkout.
