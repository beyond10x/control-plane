---
format: aep.planning-md/3
id: decision-blocker:satisfy-goal-precondition-in-ess
kind: decision-blocker
status: open
title: Declare SatisfyGoal's receipt-revision precondition in ESS, or keep it as host code?
relations:
- blocks: story:acceptance-edit-ordering
revision: 1
---
## Question

ESS 0.53.0 cannot declare SatisfyGoal's precondition "the satisfaction receipt names the goal's current revision". Keep the rule as host code, or change the command so the specification can declare it?

## State

- The rule is enforced today in crates/control-plane-core/src/guards.rs:248-277 and re-checked by the fleet under the same store lock (crates/control-plane-runtime/src/fleet.rs:2573-2605); story:acceptance-edit-ordering (unit commit 3bc53cd) shows both orders and three planted defects that the cases catch.
- The implementor's trials against ESS 0.53.0 (report of 2026-10-07):
  - reading `goal_revision` inside the receipt: refused, `[unobservable_fact] … cannot select goal_revision from String`;
  - a required `goal_revision: Integer` input: valid, but the 4 recorded SatisfyGoal decisions lack it, so stored history would stop replaying (inferred from the generated type);
  - an optional `goal_revision` input: valid, but the generated comparison is undecided when absent, which fails replay;
  - the optional input checked only when present and Running: valid, 180 scenarios, 0 refusals, `ess verify diff` 2 changes, history compatible. It adds an input to the command and turns a stale call from an error that records nothing into a recorded refusal.
- What the language lacks: a predicate over the content of a String field.

## Options

| option | what it does | cost |
|---|---|---|
| A | keep the host guard; record that ESS 0.53.0 cannot express it; revisit when the repository moves to a newer ESS (story:ess-054-upgrade) or a typed receipt | the rule stays outside the specification |
| B | add the optional `goal_revision` input to SatisfyGoal, declared and checked when present | a command change with two acknowledged changes; stale calls become recorded refusals; the receipt link stays host code |
| C | ask the ESS repository for a predicate over a structured String field, then declare it | depends on another repository's release |

Recommendation: A, with C filed as a need. The rule is enforced and tested in both orders; B changes the command to fit the language rather than declaring the existing rule.
