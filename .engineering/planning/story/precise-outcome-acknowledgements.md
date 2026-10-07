---
format: aep.planning-md/3
id: story:precise-outcome-acknowledgements
kind: story
status: implemented
title: Added-outcome acknowledgements bind the reviewed condition and answer
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-terminal-goal-edits-pass-1
scope:
- confidence: cited
  path: crates/control-plane-xtask/README.md
- confidence: cited
  path: crates/control-plane-xtask/src/spec_history.rs
- confidence: cited
  path: crates/control-plane-xtask/tests/precise_outcome_acknowledgements_attack.rs
- confidence: cited
  path: crates/control-plane-xtask/tests/precise_outcome_acknowledgements_pass2_attack.rs
- confidence: cited
  path: crates/control-plane-xtask/tests/spec_history_attack.rs
- confidence: cited
  path: crates/control-plane-xtask/tests/spec_history_pass2_attack.rs
- confidence: cited
  path: ess/spec-acknowledgements.json
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 5, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 6, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T20:24:22Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

A specification-history acknowledgement for an added outcome admits only the outcome that was reviewed: its condition and its answer, not just its name.

## Evidence

review-result:adversary-terminal-goal-edits-pass-1, finding on ess/spec-acknowledgements.json (pre-existing in crates/control-plane-xtask/src/spec_history.rs, note, INFEASIBLE today). `ess verify diff` reports an `outcome-added` change with the outcome name only, so the two outcome-added acknowledgements story:terminal-goal-edits wrote also admit a later `satisfied` or `cancelled` outcome with a different state or error. The adversary's case is kept at `$HOME/.cache/cp-wave2/terminal-goal-edits/terminal_goal_edits_ack_attack.rs`. `recorded_history_replays` still catches such a change today, because the recorded fixture holds both refusals.

## Acceptance

- `added_outcome_acknowledgement_binds_its_condition`: an acknowledgement reviewed for an added outcome refuses a later change that keeps the outcome name and changes its state condition or its error.

## Scope

Cited (wave 3, `git diff --name-only 9691f41 control-plane/impl/precise-outcome-acknowledgements`): crates/control-plane-xtask/src/spec_history.rs (format control-plane-spec-acknowledgements/2: an outcome-added entry records the compiled `outcome` and the command's ordered `command_outcomes`), ess/spec-acknowledgements.json (the two outcome-added entries migrated), crates/control-plane-xtask/README.md, and the test files crates/control-plane-xtask/tests/{spec_history_attack,spec_history_pass2_attack,precise_outcome_acknowledgements_attack,precise_outcome_acknowledgements_pass2_attack}.rs.

Corrections to the earlier inferred scope: the two existing spec-history attack files had to change (format string and fixtures); the README paragraph changed. No ESS issue was filed: `ess verify diff` reports an added outcome by name only (ess-diff outcome_changes), and the gate now binds the compiled outcome itself.
