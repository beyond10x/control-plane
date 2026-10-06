---
id: story:backticked-list
kind: story
status: active
title: Names listed in backticks beside commands, artifact ids and paths
---
## Acceptance

- `unacknowledged_spec_change_fails_gate`: on a branch that removes `Blocked` from `Assignment.repair.from`, `task check` fails and names `entity/controlplane.host.Assignment/transition-route-changed/repair`.
- `progress_decision_stays_under_16_kib`: 1,000 progress events, each with a 1 KiB detail; `restart_open_is_bounded` follows.
- `recorded_history_replays` (story:spec-history-gate) stays green; `story:spec-history-gate` is cited in backticks as well.
- `ess verify conform mutate --emit` is a command, and so is `cargo run --locked -p control-plane-xtask -- acceptance-check`.
- The probe runs `Store::execute` from crates/control-plane-xtask/src/acceptance.rs and `crates/control-plane-app/tests/operator-boundary-cases.rs`, configured in Taskfile.yml.
- `recorded_history_replays` is named twice and listed once.
