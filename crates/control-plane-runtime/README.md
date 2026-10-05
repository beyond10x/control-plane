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
review. Rejected acceptance stays durable and idle until inputs change. Real model,
tool, check and publication progress is retained in bounded activity history.
`max_minutes` bounds each execution or acceptance attempt, not cumulative lifetime
across restarts; the durable assignment attempt count bounds retries. A changed base
or unresolved publication is an explicit blocker, never an automatic force repair.

`CodexAgentModel` uses the foundation LLM crate's `codex_model` login resolver.
Authentication failures, refused Git operations, validator errors and limits are
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
