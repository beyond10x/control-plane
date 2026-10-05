# Control plane

A local workspace for autonomous engineering: add a repository or a directory of repositories, set a goal, and follow its plan, implementation assignments and verified merge receipts.

The product is a standalone Rust service and CLI. Its domain is specified in [ESS](ess/system.yaml); the engineering plan lives in [AEP](.engineering/planning/epic/bootstrap.md). Browser and CLI commands use the same durable host.

The [product vision](docs/vision.md) defines the live operator experience and its delivery standard.

## Run from source

Install Rust, ESS 0.53.0, AEP 0.68.0 and Worktree 0.8.2. `rust-toolchain.toml` selects the Rust version. The repository gate also needs Task 3.52.0.

```console
cargo run --locked -p control-plane-app -- serve
```

Open `http://127.0.0.1:8787/`. Use `--listen 127.0.0.1:8788` when that port is occupied. State defaults to `$XDG_STATE_HOME/control-plane/state.sqlite`, or `~/.local/state/control-plane/state.sqlite`; `serve --state <file>` chooses a different store.

Startup adds the current directory as a workspace. Repeat `serve --workspace PATH` to select other startup roots. Existing workspaces and goals remain in the same store across restarts. A single service manages all of them.

In another terminal:

```console
cargo run --locked -p control-plane-app -- workspace add .
cargo run --locked -p control-plane-app -- workspace list
cargo run --locked -p control-plane-app -- workspace add-directory WORKSPACE_ID /path/to/context
cargo run --locked -p control-plane-app -- workspace directories WORKSPACE_ID
cargo run --locked -p control-plane-app -- status
```

Pass `--url http://127.0.0.1:8788` for a service using a different port. The CLI contacts the running service, so it shares the browser's state and command admission.

The browser exposes repository membership and settings; goal objective and acceptance; per-role models; worker, attempt and time limits; and merge authority. Goals start paused. Starting a goal and allowing merges are separate controls. Repository settings name the repository's own test and publication commands.

Each workspace can contain several directories, including context directories without Git. Adding a directory discovers its repository or immediate child repositories. Removing membership preserves repositories still covered by another directory and manually registered repositories. Active work must finish or be cancelled before its directory can be removed.

The service starts its planner alongside the console. The planner uses the existing Codex login through the foundation LLM library and creates isolated managed worktrees. The goal page shows planning progress and concrete blockers, including unavailable repositories, dirty source trees, rejected plans and tool failures. Configure the bot's private policy with `serve --gates-policy PATH` or `B10X_GATES_POLICY`; keep policy files and credentials outside the repository.

The operations dashboard refreshes live status every second while keeping goal forms and unsaved edits in place. Planner cards show the model, current stage, actual model-call waits, elapsed time and recorded blockers. Assignment state and durable activity history appear beneath them; full receipts are available through **Inspect evidence**. The server timestamp measures connection freshness, while work timestamps advance only when an operation is recorded.

Planner context uses bounded repository indexes and observations. Repeated reads refresh the same observation, and file pages allow later content to be inspected without accumulating entire files in every request.

The HTTP API shares the browser and CLI state:

| Request | Result |
| --- | --- |
| `GET /api/workspaces` | List workspaces |
| `POST /api/workspaces` | Register `{ "path": "...", "name": "..." }` |
| `GET /api/workspaces/{id}` | Workspace, directories, repositories, goals and assignments |
| `GET /api/workspaces/{id}/directories` | List active directory memberships |
| `POST /api/workspaces/{id}/directories` | Add `{ "path": "..." }` |
| `DELETE /api/workspaces/{id}/directories/{directory}` | Remove membership |
| `GET /api/state` | Inspect the complete local state and runtime errors |
| `GET /workspaces/{id}/live` | Automatically refreshed operations dashboard for one workspace |
| `GET /goals/{id}/evidence` | Inspect a goal's stored planning and execution evidence |

Mutation clients obtain `csrf_token` from `GET /api/session` and send it in `x-csrf-token`. The bundled CLI handles this automatically.

## Structure

| Surface | Responsibility |
| --- | --- |
| `ess/` | Workspace, directory, repository, goal, assignment and publication contracts in readable ESS22 YAML |
| `generated/model/` | Generated domain types, behavior and command dispatch |
| `generated/api/` | Generated domain OpenAPI contract |
| `crates/control-plane-core/` | Operational admission, discovery, Eventlog storage and deterministic replay |
| `crates/control-plane-protocol/` | Canon planning and verified-merge protocols |
| `crates/control-plane-runtime/` | Supervised planning and repository execution adapters |
| `crates/control-plane-app/` | Local browser console and CLI |
| `crates/control-plane-xtask/` | Generated drift, durable conformance and foundation dependency gates |

Generated OpenAPI describes the domain contract. The local console intentionally exposes only Operator commands through `/api/commands/{command}`; it does not expose Supervisor commands.

The service binds to loopback and checks browser origin, host and request tokens. It holds one state-store lock. Command decisions become visible only after durable append, and restarting reconstructs the same state from the log.

## Development

```console
task check
task generate
```

`task check` validates ESS and AEP, checks Rust formatting and lints, runs tests, checks generated drift and exercises the real durable conformance target. `task generate` regenerates artifacts from the specification. Generated files are not edited directly. Repository changes use managed worktrees; see [AGENTS.md](AGENTS.md).

Bootstrap status: the host, workspace directories, protocols, autonomous planner, reviewed implementation fleet and live operations dashboard are integrated. Full repository qualification and publication remain in the AEP plan.
