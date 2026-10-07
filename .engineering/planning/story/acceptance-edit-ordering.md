---
format: aep.planning-md/3
id: story:acceptance-edit-ordering
kind: story
status: active
title: A goal edit and its acceptance are ordered, and both orders are shown
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- depends_on: story:goal-acceptance-authority
scope:
- confidence: inferred
  path: crates/control-plane-core/src/guards.rs
- confidence: inferred
  path: crates/control-plane-core/src/tests.rs
- confidence: inferred
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
- confidence: inferred
  path: ess/domains/host.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T02:47:22Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-07T02:47:22Z", actor: "human:timo", revision: 5}
---
## Outcome

The order between an operator's goal edit and the goal's acceptance is stated as an invariant and shown in both directions: a goal is satisfied only at the revision its acceptance checked, and an edit that arrives after satisfaction is refused with a reason the operator sees. If the code allows any other outcome, the race is fixed in the specification first.

## Evidence

- CI run 37561352666 (pull request 2, commit 587a4f4) failed `goal_edit_during_acceptance_is_not_satisfied` at crates/control-plane-runtime/tests/fleet.rs:946 with `{"error":"controlplane.host.GoalStateConflict","outcome":"satisfied","payload":{"state":"Satisfied"}}`: the operator's UpdateGoal reached the store after SatisfyGoal and was refused. The same test passed in the push run on that commit, in 50 local runs (20 pinned to one CPU) and in both wave-3 runs.
- crates/control-plane-runtime/src/fleet.rs:2575-2600 (at 587a4f4): under one store lock the fleet re-reads the goal, requires state Running and the checked revision, then sends SatisfyGoal.
- crates/control-plane-core/src/guards.rs:235-265 (at 587a4f4): SatisfyGoal requires a receipt, every assignment of the goal Merged or Cancelled, and a receipt that names the goal's current revision. This is host code, not a declaration in ess/domains/host.yaml.
- Inferred, not verified: the test places its edit by handing the store lock to the next waiter, and on a loaded runner the editor thread can queue after SatisfyGoal. Whether any product path accepts a goal on content that is no longer current is the question this story answers.

## Acceptance

- `satisfy_after_edit_names_the_old_revision_and_is_refused`: after UpdateGoal moves a goal to revision 2, SatisfyGoal with a receipt for revision 1 is refused and the goal stays Running at revision 2.
- `edit_after_satisfy_is_refused_and_says_why`: after SatisfyGoal applies, UpdateGoal is refused with the satisfied outcome, the goal keeps its satisfaction receipt for the checked revision, and the refusal reaches the operator's command result.
- `goal_edit_during_acceptance_is_not_satisfied` reaches the window between the fleet's post-review check and SatisfyGoal by a seam that does not depend on thread scheduling, and asserts the outcome of each ordering instead of assuming one.
- The precondition "SatisfyGoal's receipt names the goal's current revision" is declared in ess/domains/host.yaml where the specification can express it, and `ess specify validate --path ess --strict-requires` passes; if it cannot be expressed, the story records that as a decision before any code change.

## Scope

Inferred: crates/control-plane-core/src/guards.rs, crates/control-plane-core/src/tests.rs, crates/control-plane-runtime/src/fleet.rs (`satisfy_goals`), crates/control-plane-runtime/tests/fleet.rs, ess/domains/host.yaml.

## Planning

Planned for wave 5. If the race is real, the fix goes through the specification first.
