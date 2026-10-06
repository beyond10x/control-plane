---
format: aep.planning-md/3
id: story:candidate-process-environment
kind: story
status: active
title: Host-started processes get an allowlisted environment
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:control-plane-review-2026-10-06
scope:
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/control-plane-app/src/cli.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/src/lib.rs
- confidence: cited
  path: crates/control-plane-runtime/src/process.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T02:29:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-review-session"}
- {from: "proposed", to: "active", at: "2026-10-06T02:29:50Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-review-session"}
---
## Outcome

Every process the host starts gets an explicit environment: a fixed allowlist (PATH, HOME, LANG, TMPDIR and the tool caches the gates need) plus the configured entries. Only the publish command receives credentials, and only the variables the operator names with a serve flag.

## Narrowing of the epic's promise

The publish command receives the named credentials and runs in the candidate worktree. If the operator configures a publish command that executes files from the candidate tree (for example a Taskfile target or a script in the repository), those files are model-written and receive the named credentials. This story does not prevent that; the README states it, and the operator chooses a publish command that runs only host-installed programs (such as `b10x-gates bot … git push`).

## Evidence

- review-result:control-plane-review-2026-10-06 finding 1: `ProcessRunner::run` adds `environment` on top of the inherited environment (crates/control-plane-runtime/src/process.rs:61-70); nothing calls `env_clear`.
- Test command, publish command (fleet.rs:237-259) and the eval-only `go test ./...` (fleet.rs:1280-1301) run in the candidate worktree; the test command and `go test` execute model-written code.
- The running service carries 21 credential-named variables (names read, values not).
- Probe `probe_child_processes_do_not_inherit_service_credentials` (recorded in the review, not committed) failed on f510e7f and passed with `env_clear()` plus PATH.

## Acceptance

- `processes_start_from_allowlisted_environment`: a parent variable outside the allowlist is absent from `env` run through ProcessRunner.
- `publish_command_receives_only_named_credentials`: a variable named by the new serve flag reaches the publish command.
- `named_credentials_stay_out_of_other_commands`: the same variable does not reach the test command or an implementor-run command.
- `native_tools_run_with_cleared_environment`: the existing planner and fleet native tests (git, aep, ess, worktree, go) pass unchanged.
- README.md states the narrowing above.

## Scope

Cited: crates/control-plane-runtime/src/process.rs, crates/control-plane-runtime/src/lib.rs (RuntimeConfig), crates/control-plane-runtime/src/fleet.rs (`command`). Inferred: crates/control-plane-app/src/cli.rs (serve flag), crates/control-plane-runtime/tests/, README.md.

## Out of scope

Filesystem and network confinement, including the loopback API (review finding 8): same-user processes can still read files and reach 127.0.0.1. That belongs to Substrate.

## Authorization

Operator approval 2026-10-06 ("commit + fix", option A: this story, then story:model-input-refusals, before eval round 7).
