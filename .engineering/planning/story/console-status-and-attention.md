---
format: aep.planning-md/3
id: story:console-status-and-attention
kind: story
status: implemented
title: Status header and attention strip
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-projection
- depends_on: story:console-component-tests
scope:
- confidence: inferred
  path: frontend/dist
- confidence: inferred
  path: frontend/src
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T20:26:27Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T20:26:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-07T02:17:28Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome

The top of every page says, in one line, whether the console is connected and what the system is doing (idle, paused, waiting for a model, working, blocked, disconnected) with the age of the last real activity; below it, one row per item that needs the operator (a blocked goal or assignment, an uncertain publication) with its reason in plain words, its age and the control that resolves it. This story owns the derived goal state: one value from the goal's lifecycle state and planning phase together, used by the header, the attention strip and later by the goal cards.

## Evidence

review-result:console-ux-review-2026-10-06 rows 1, 2 and 7: tiles read "00" while disconnected; "Need attention" reads 00 beside failed assignments; a Running goal in phase Blocked shows green; a Cancelled card also shows "Stage Queued" and "Recorded running".

## Acceptance

- `disconnected_is_not_idle`: with the stream closed the header shows Disconnected and every count shows "—", never 0.
- `paused_is_shown_as_paused`: with every goal Paused the header says paused.
- `working_is_shown_with_its_activity`: with an assignment Implementing and no wait the header says working and names the role and repository.
- `blocked_is_shown_as_blocked`: with a blocked assignment and nothing working the header says blocked.
- `idle_and_waiting_are_distinct`: a projection with `waiting` set renders "waiting for <model>"; with nothing running and nothing blocked it renders idle.
- `last_activity_age_is_shown`: the header shows the age of the newest activity computed from the projection's server clock.
- `derived_goal_state_combines_lifecycle_and_phase`: Running with phase Blocked derives blocked; Cancelled derives cancelled whatever its phase; Running with phase Planning derives planning.
- `blocked_goal_is_not_green`: a goal Running with planning phase Blocked renders the blocked badge and an attention row with its reason.
- `attention_lists_each_blocked_item`: two blocked assignments and one uncertain publication produce three rows, each naming its goal and repository, stating its reason in words (not an error identifier), showing its age from the server clock and linking the resolving control.
- `planner_status_uses_planner_activity`: with a worker event newer than the planner event, the planner line in the header shows the planner event.
- `satisfied_goal_shows_acceptance_recorded`: a satisfied goal shows "acceptance recorded" from `acceptance_recorded`, and a running goal does not.

## Scope

Inferred: frontend/src/App.vue, frontend/src/style.css, new frontend/src/*.vue components and their tests, frontend/dist (rebuilt).
