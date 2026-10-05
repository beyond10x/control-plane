# Control plane

A local workspace for autonomous engineering: add a repository or a directory of repositories, set a goal, and follow its plan, implementation assignments and verified merge receipts.

The product is a standalone Rust service and CLI. Its domain is specified in [ESS](ess/system.yaml); the engineering plan lives in [AEP](.engineering/planning/epic/bootstrap.md). Browser and CLI commands use the same durable host.

## Run from source

Install Rust, ESS 0.53.0, AEP 0.68.0 and Worktree 0.8.2. `rust-toolchain.toml` selects the Rust version. The repository gate also needs Task 3.52.0.

```console
cargo run --locked -p control-plane-app -- serve
```

Open `http://127.0.0.1:8787/`. Use `--listen 127.0.0.1:8788` when that port is occupied. State defaults to `$XDG_STATE_HOME/control-plane/state.sqlite`, or `~/.local/state/control-plane/state.sqlite`; `serve --state <file>` chooses a different store.

In another terminal:

```console
cargo run --locked -p control-plane-app -- workspace add .
cargo run --locked -p control-plane-app -- workspace list
cargo run --locked -p control-plane-app -- status
```

Pass `--url http://127.0.0.1:8788` for a service using a different port. The CLI contacts the running service, so it shares the browser's state and command admission.

The browser exposes repository membership and settings; goal objective and acceptance; per-role models; worker, attempt and time limits; and merge authority. Goals start paused. Starting a goal and allowing merges are separate controls. Repository settings name the repository's own test and publication commands.

## Structure

| Surface | Responsibility |
| --- | --- |
| `ess/` | Workspace, repository, goal, assignment and publication contracts in readable ESS22 YAML |
| `generated/model/` | Generated domain types, behavior and command dispatch |
| `generated/api/` | Generated domain OpenAPI contract |
| `crates/control-plane-core/` | Operational admission, discovery, Eventlog storage and deterministic replay |
| `crates/control-plane-protocol/` | Canon planning and verified-merge protocols |
| `crates/control-plane-app/` | Local browser console and CLI |

Generated OpenAPI describes the domain contract. The local console intentionally exposes only Operator commands through `/api/commands/{command}`; it does not expose Supervisor commands.

The service binds to loopback and checks browser origin, host and request tokens. It holds one state-store lock. Command decisions become visible only after durable append, and restarting reconstructs the same state from the log.

## Development

```console
task check
task generate
```

`task check` validates ESS and AEP, checks Rust formatting and lints, runs tests, checks generated drift and exercises the real durable conformance target. `task generate` regenerates artifacts from the specification. Generated files are not edited directly. Repository changes use managed worktrees; see [AGENTS.md](AGENTS.md).

Bootstrap status: the host, protocols and operator surface are integrated. Autonomous planner/fleet integration and full repository qualification remain in the active AEP plan.
