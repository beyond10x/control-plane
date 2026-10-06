---
format: aep.planning-md/3
id: story:spec-history-gate
kind: story
status: active
title: Specification changes that could break stored history fail the gate
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: verification-report:ess-hardening-2026-10-06
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/control-plane-core/tests
- confidence: inferred
  path: crates/control-plane-xtask/src
- confidence: inferred
  path: ess
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T03:05:53Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T03:05:53Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

`task check` refuses a specification change that could break replay of stored decisions unless an acknowledgement names it, and proves that a recorded history still opens.

## Evidence

- crates/control-plane-core/src/lib.rs:212-215: replay refuses any recorded outcome that differs from current generated behaviour ("migration required"), so the service cannot start on its existing store after such a change.
- Taskfile.yml runs no specification diff.
- verification-report:ess-hardening-2026-10-06, technique 7: removing `Blocked` from `Assignment.repair.from` is classified `unknown`; `ess verify diff --fail-on breaking` exits 0, `--fail-on breaking-or-unknown` exits 4.
- The repository has no release tag, so the baseline is the specification at the merge base with origin/main.

## Acceptance

- `unacknowledged_spec_change_fails_gate`: on a branch that removes `Blocked` from `Assignment.repair.from`, `task check` fails and names `entity/controlplane.host.Assignment/transition-route-changed/repair`.
- `acknowledged_spec_change_passes_gate`: with that id in the acknowledgement file, the same branch passes.
- `stale_acknowledgement_fails_gate`: an acknowledgement whose id no longer appears in the diff fails the gate.
- `recorded_history_replays`: a committed fixture event log that applies every command at least once opens with `Store::open`.
- `changed_recorded_outcome_fails_replay`: the same fixture with one recorded outcome altered fails to open.

## Scope

Cited: Taskfile.yml. Inferred: crates/control-plane-xtask/src/ (a diff subcommand), an acknowledgement file beside ess/, crates/control-plane-core/tests (new fixture and test).
