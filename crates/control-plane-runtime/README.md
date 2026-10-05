# Autonomous planner runtime

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
Empty selections leave the goal running with a reason;
implementation, publication, and final satisfaction belong to the fleet lane.

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
isolated profiles and linked checkouts remain under `.scratch/planner` for review;
cleanup must use the worktree archive/finish/gc protocol with each fixture's XDG
configuration, never recursive removal of an active managed checkout.
