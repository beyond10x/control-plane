---
format: aep.planning-md/3
id: story:goal-acceptance-authority
kind: story
status: active
title: Goal acceptance satisfies only the revision it checked
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:adversary-terminal-goal-edits-pass-1
scope:
- confidence: inferred
  path: crates/control-plane-core/src/guards.rs
- confidence: inferred
  path: crates/control-plane-core/src/tests.rs
- confidence: inferred
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 6, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T09:13:30Z", actor: "human:timo", revision: 7, executor: "agent:claude-wave-coordinator"}
---
## Outcome

The fleet satisfies a goal only at the revision it checked: an operator edit that lands while acceptance runs is not swallowed by a SatisfyGoal for the older revision.

## Evidence

review-result:adversary-terminal-goal-edits-pass-1, finding at crates/control-plane-runtime/src/fleet.rs:2079 (pre-existing, warning, from reading the code; no run observed). Goal acceptance checks state and revision, releases the store lock across `git ls-remote` per repository (fleet.rs:2098), then sends SatisfyGoal (fleet.rs:2117). An UpdateGoal applied in that window leaves a Satisfied goal whose receipt names the old revision, and since story:terminal-goal-edits a Satisfied goal refuses every edit.

## Acceptance

- `goal_edit_during_acceptance_is_not_satisfied`: an UpdateGoal applied between the acceptance check and SatisfyGoal leaves the goal Running at the new revision, with no receipt.
- `unchanged_goal_is_satisfied_after_acceptance`: with no edit in the window, the goal is Satisfied with a receipt naming the checked revision.

## Scope

Inferred: the fix is a host admission check, not a specification change. The SatisfyGoal guard in crates/control-plane-core/src/guards.rs (lines 187-199 at e238d48) runs inside `Store::execute`, so it compares the receipt's `goal_revision` with the goal's stored revision in the same step that applies the command; a mismatch is refused and the goal stays Running. crates/control-plane-runtime/src/fleet.rs (goal acceptance, 2060-2130) keeps its pre-check and handles the refusal; crates/control-plane-core/src/tests.rs and crates/control-plane-runtime/tests/fleet.rs carry the tests. No ess/, generated/ or ess/spec-acknowledgements.json change, so this story stays disjoint from story:precise-outcome-acknowledgements, which rewrites the acknowledgement file. Declaring the rule in ESS belongs to story:spec-owned-admission, which lists every host-only rule with its reason.
