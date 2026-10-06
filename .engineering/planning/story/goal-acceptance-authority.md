---
format: aep.planning-md/3
id: story:goal-acceptance-authority
kind: story
status: draft
title: Goal acceptance satisfies only the revision it checked
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-terminal-goal-edits-pass-1
scope:
- confidence: inferred
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
revision: 3
---
## Outcome

The fleet satisfies a goal only at the revision it checked: an operator edit that lands while acceptance runs is not swallowed by a SatisfyGoal for the older revision.

## Evidence

review-result:adversary-terminal-goal-edits-pass-1, finding at crates/control-plane-runtime/src/fleet.rs:2079 (pre-existing, warning, from reading the code; no run observed). Goal acceptance checks state and revision, releases the store lock across `git ls-remote` per repository (fleet.rs:2098), then sends SatisfyGoal (fleet.rs:2117). An UpdateGoal applied in that window leaves a Satisfied goal whose receipt names the old revision, and since story:terminal-goal-edits a Satisfied goal refuses every edit.

## Acceptance

- `goal_edit_during_acceptance_is_not_satisfied`: an UpdateGoal applied between the acceptance check and SatisfyGoal leaves the goal Running at the new revision, with no receipt.
- `unchanged_goal_is_satisfied_after_acceptance`: with no edit in the window, the goal is Satisfied with a receipt naming the checked revision.

## Scope

Inferred: crates/control-plane-runtime/src/fleet.rs (goal acceptance), crates/control-plane-runtime/tests/fleet.rs; if SatisfyGoal gains a revision input, ess/domains/host.yaml, generated/ and ess/spec-acknowledgements.json.
