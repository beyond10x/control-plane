---
format: aep.planning-md/3
id: story:precise-outcome-acknowledgements
kind: story
status: active
title: Added-outcome acknowledgements bind the reviewed condition and answer
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-terminal-goal-edits-pass-1
scope:
- confidence: inferred
  path: crates/control-plane-xtask/src/spec_history.rs
- confidence: inferred
  path: crates/control-plane-xtask/tests
- confidence: inferred
  path: ess/spec-acknowledgements.json
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 5, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 6, executor: "agent:claude-wave-coordinator"}
---
## Outcome

A specification-history acknowledgement for an added outcome admits only the outcome that was reviewed: its condition and its answer, not just its name.

## Evidence

review-result:adversary-terminal-goal-edits-pass-1, finding on ess/spec-acknowledgements.json (pre-existing in crates/control-plane-xtask/src/spec_history.rs, note, INFEASIBLE today). `ess verify diff` reports an `outcome-added` change with the outcome name only, so the two outcome-added acknowledgements story:terminal-goal-edits wrote also admit a later `satisfied` or `cancelled` outcome with a different state or error. The adversary's case is kept at `$HOME/.cache/cp-wave2/terminal-goal-edits/terminal_goal_edits_ack_attack.rs`. `recorded_history_replays` still catches such a change today, because the recorded fixture holds both refusals.

## Acceptance

- `added_outcome_acknowledgement_binds_its_condition`: an acknowledgement reviewed for an added outcome refuses a later change that keeps the outcome name and changes its state condition or its error.

## Scope

Inferred: crates/control-plane-xtask/src/spec_history.rs, crates/control-plane-xtask/tests/, and ess/spec-acknowledgements.json (its three existing entries must carry the reviewed condition and answer in the new form). Every story that changes the specification also writes that file (story:publication-exit, story:spec-owned-admission), so none of them shares a wave with this one. Possibly an issue on beyond10x/ess if the diff should carry the outcome's condition and answer.
