---
format: aep.planning-md/3
id: story:goal-acceptance-authority
kind: story
status: implemented
title: Goal acceptance satisfies only the revision it checked
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-terminal-goal-edits-pass-1
scope:
- confidence: cited
  path: crates/control-plane-core/src/guards.rs
- confidence: cited
  path: crates/control-plane-core/src/tests.rs
- confidence: cited
  path: crates/control-plane-core/tests/fixtures/recorded-history.db
- confidence: cited
  path: crates/control-plane-core/tests/fixtures/recorded-history.views.json
- confidence: cited
  path: crates/control-plane-core/tests/goal_acceptance_authority_attack.rs
- confidence: cited
  path: crates/control-plane-core/tests/terminal_goal_edits_attack.rs
- confidence: cited
  path: crates/control-plane-core/tests/terminal_goal_edits_pass2_attack.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/tests/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/tests/goal_acceptance_authority_attack.rs
- confidence: cited
  path: crates/control-plane-runtime/tests/goal_acceptance_authority_pass2_attack.rs
- confidence: cited
  path: crates/control-plane-xtask/src/history.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 6, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 7, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T20:24:22Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

The fleet satisfies a goal only at the revision it checked: an operator edit that lands while acceptance runs is not swallowed by a SatisfyGoal for the older revision.

## Evidence

review-result:adversary-terminal-goal-edits-pass-1, finding at crates/control-plane-runtime/src/fleet.rs:2079 (pre-existing, warning, from reading the code; no run observed). Goal acceptance checks state and revision, releases the store lock across `git ls-remote` per repository (fleet.rs:2098), then sends SatisfyGoal (fleet.rs:2117). An UpdateGoal applied in that window leaves a Satisfied goal whose receipt names the old revision, and since story:terminal-goal-edits a Satisfied goal refuses every edit.

## Acceptance

- `goal_edit_during_acceptance_is_not_satisfied`: an UpdateGoal applied between the acceptance check and SatisfyGoal leaves the goal Running at the new revision, with no receipt.
- `unchanged_goal_is_satisfied_after_acceptance`: with no edit in the window, the goal is Satisfied with a receipt naming the checked revision.

## Scope

Cited (wave 3, `git diff --name-only 9691f41 control-plane/impl/goal-acceptance-authority`): crates/control-plane-core/src/guards.rs (SatisfyGoal receipt must name the stored revision), crates/control-plane-core/src/tests.rs, crates/control-plane-runtime/src/fleet.rs (goal acceptance: locked final comparison and SatisfyGoal, `interrupted` versus latching `failed`, skip an edited goal), crates/control-plane-runtime/tests/fleet.rs, crates/control-plane-xtask/src/history.rs (JSON receipts), the recorded-history fixtures (re-recorded, 98 decisions), and the attack files crates/control-plane-core/tests/{goal_acceptance_authority_attack,terminal_goal_edits_attack,terminal_goal_edits_pass2_attack}.rs and crates/control-plane-runtime/tests/goal_acceptance_authority_{,pass2_}attack.rs.

Corrections to the earlier inferred scope: the fix needed no ESS, generated or acknowledgement change, as decided; it did need crates/control-plane-xtask/src/history.rs and the recorded-history fixtures, which the inferred scope did not list. Two adversary passes widened it from the UpdateGoal window to configuration changes, pauses, edits of another goal and transient observation failures, all in fleet.rs goal acceptance.
