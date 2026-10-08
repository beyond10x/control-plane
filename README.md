# Control plane

A local workspace for autonomous engineering: add a repository or a directory of repositories, set a goal, and follow its plan, implementation assignments and verified merge receipts.

The product is a standalone Rust service and CLI with a Vue console bundled into the binary. Its domain is specified in [ESS](ess/system.yaml); the engineering plan lives in [AEP](.engineering/planning/epic/bootstrap.md). Browser and CLI commands use the same durable host.

The [product vision](docs/vision.md) defines the live operator experience and its delivery standard.

## Run from source

Install Rust, ESS 0.56.0, AEP 0.68.0 and Worktree 0.8.2. `rust-toolchain.toml` selects the Rust version. The repository gate also needs Task 3.52.0.

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

The browser exposes repository membership and settings; goal objective and acceptance; per-role models; worker, attempt and time limits; and merge authority. Goals start paused. Starting a goal and allowing merges are separate controls. Repository settings name the repository's own test and publication commands. The publication command must exit only after its merge is visible on the repository's fetch URL (`origin`), where the service looks for the candidate. The publication counts only when the candidate commit itself reaches the target; a publisher that squashes or rebases it is recorded as not published. While the candidate is missing there, the assignment waits and the command is not run again. Once the target moves past the head seen when the command exited, still without the candidate, or 10 minutes have passed, the publication is recorded as not published; a merge that becomes visible after that is not recorded. The assignment is then attempted again on the target's current head: if the target moved since its attempt started, that head becomes the assignment's base and the earlier work is merged onto it before the next attempt, unless the target was rewound and no longer contains the base the work was built on, the earlier work conflicts with the new head, or its changes are already on the target, in which case the assignment is blocked instead, without spending an attempt.

Each workspace can contain several directories, including context directories without Git. Adding a directory discovers its repository or immediate child repositories. Removing membership preserves repositories still covered by another directory and manually registered repositories. Active work must finish or be cancelled before its directory can be removed.

The service starts its planner alongside the console. The planner uses the existing Codex login through the foundation LLM library and creates isolated managed worktrees. The goal page shows planning progress and concrete blockers, including unavailable repositories, dirty source trees, rejected plans and tool failures. Configure the bot's private policy with `serve --gates-policy PATH` or `B10X_GATES_POLICY`; keep policy files and credentials outside the repository.

Processes the service starts get a cleared environment: only `PATH`, `HOME`, locale, XDG directories and toolchain or build-cache variables are inherited. Test commands and model tool commands run model-written code and never receive credentials. The Gates settings (`B10X_GATES_POLICY`, `B10X_GATES_KEY`, `B10X_GATES_GITLEAKS`) and each variable named with `serve --publish-env NAME` reach only the commit command and the repository's publish command. The publish command runs in the candidate worktree, so configure one that runs host-installed programs only: a publish command that executes a script or task from the repository hands those credentials to model-written code. This is not a sandbox; candidate processes still run as your user.

The Vue console receives committed state changes through SSE and updates cards in place, preserving form edits, focus and scroll. It shows connection and reconnect status separately from recorded work activity. Planner cards show the model, current stage and actual runtime observations. Assignment state and durable activity history appear beneath them; full receipts remain available through **Inspect evidence**. Stream heartbeats do not claim work progress. The embedded frontend needs no CDN, Node process or separate frontend server at runtime.

Loom owns model turns, compaction and durable sessions; Commission admits planner, implementation and publication effects. Control-plane owns workspace scheduling, persistence, authority inputs and UI projections. Planner context uses bounded repository indexes and observations. Repeated reads refresh the same observation, and file pages allow later content to be inspected without accumulating entire files in every request.

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
| `GET /events` | SSE stream of compact committed operations across workspaces |
| `GET /workspaces/{id}/events` | SSE stream scoped to one workspace |
| `GET /workspaces/{id}/live` | Vue operations console for one workspace |
| `GET /goals/{id}/evidence` | Vue evidence view for one goal: state, acceptance, checks, reviews, merges and activity |
| `GET /api/goals/{id}/evidence` | A goal's stored planning and execution evidence as JSON |

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

Isolated model evaluations use external local repositories:

```console
cargo run -p control-plane-xtask -- eval init --root "$HOME/control-plane-evals"
control-plane serve --local-eval-root "$HOME/control-plane-evals"
control-plane workspace add "$HOME/control-plane-evals/repos/go-auth-web"
cargo run -p control-plane-xtask -- eval verify --case go-auth-web --repo /path/to/candidate
```

The cases are `go-cli`, `go-json-http` and `go-auth-web`; each seed contains its fixed `TASK.md`. Configure the repository's test command to run the built verifier outside the candidate tree. Use one worker, one attempt and a bounded time budget, and retain failures before retrying. The verifier requires Go tests and checks real CLI or HTTP behavior, including login, session rejection, logout revocation, frontend and README requirements. Blank seeds fail. Explicit `--local-eval-root` permits Go and local commits only when both the repository's Git common directory and its absolute local origin are beneath that root. Other repositories retain their normal policy.

`control-plane goal delete <id>` removes a cancelled goal with no assignment history while retaining its workspace. Cancel it first with `control-plane goal cancel <id>`.

```console
task check
task generate
task frontend
```

`task check` validates ESS and AEP, checks Rust formatting and lints, runs tests, checks generated drift and exercises the real durable conformance target. It also rebuilds Vue, verifies that the embedded assets match their source, then runs the Vue component tests (Vitest with happy-dom, no browser). Both steps run in `cargo run -p control-plane-xtask -- frontend-check`, which fails if any test fails, if any test or suite is skipped or marked todo, if a test is marked to fail as expected (`test.fails`), if Vitest reports an unhandled error, or if no test passed. `npm run test --prefix frontend` alone checks less: it exits 0 when a test is skipped, marked todo or marked `test.fails`. Frontend development and this gate need Node 22.23.2 and npm; ordinary Cargo builds use the committed bundle. `cargo test -p control-plane-xtask` and `cargo test --workspace` run the frontend check on temporary copies of `frontend/`, so they also need Node and npm, and `npm ci --prefix frontend` first. `task frontend` installs the locked frontend dependencies and rebuilds `frontend/dist`, which must be committed with frontend changes. `task generate` regenerates artifacts from the specification. Generated files are not edited directly. Repository changes use managed worktrees; see [AGENTS.md](AGENTS.md).

Bootstrap status: the real auth eval recovered through Loom, implemented and published a Go application, and passed the trusted external verifier. Final goal acceptance rejected an eval-bookkeeping requirement mistakenly included in application acceptance. The [boundary correction](.engineering/planning/story/runtime-boundary.md) records the result, token costs and the approved next round of runtime and console improvements. Passing scripted tests alone is not evidence of a delivered autonomous application.
