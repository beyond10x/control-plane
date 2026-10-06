---
format: aep.planning-md/3
id: story:console-projection
kind: story
status: draft
title: The SSE projection carries planner activity, waits, clock and recorded outcomes
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:bounded-progress-records
- depends_on: story:acceptance-traceability
scope:
- confidence: cited
  path: crates/control-plane-app/src/live.rs
revision: 6
---
## Outcome

The SSE projection the browser receives carries what the console needs to tell the truth, derived only from records that already exist: the planner's own latest activity separate from each worker's, a derived waiting state (role, model, since) when an agent waits for a model response, the server's clock for elapsed times, whether an acceptance is recorded, when an assignment merged, and a stable id per activity entry.

## Evidence

- review-result:console-ux-review-2026-10-06 rows 1, 3 and 5.
- Worker events overwrite the goal's `last_activity`, and the planner card renders it as planner activity (crates/control-plane-runtime/src/fleet.rs `Host::progress`; crates/control-plane-app/src/live.rs `compact`; frontend/src/App.vue). Every activity entry already carries its `role`, so the projection can select the newest planner entry without a runtime change.
- live.rs `compact` removes `server_observed_at` and every field containing `receipt`, so the browser cannot compute elapsed time or show a recorded acceptance or merge.

## Acceptance

- `planner_activity_is_the_planner_event`: after a planner event then a worker event on one goal, the projection's planner activity is the planner event.
- `worker_event_is_listed_under_its_assignment`: in the same sequence the worker event appears under its assignment and not as planner activity.
- `model_wait_is_projected`: between a model request and its completion the projection carries `waiting` with role, model and start time; after completion it carries none.
- `projection_carries_server_clock`: every SSE frame carries the server time it was built at.
- `recorded_acceptance_reaches_the_browser`: a satisfied goal projects `acceptance_recorded: true` without its receipt contents.
- `merge_time_reaches_the_browser`: a merged assignment projects `merged_at` from its receipt without the receipt contents.
- `activity_entries_have_distinct_ids`: two entries recorded in the same second have different ids.
- `activity_ids_are_stable_across_projections`: the same recorded entry carries the same id in two projections built from one store state and after a further entry is appended.

## Consumers

story:console-status-and-attention reads the planner activity, `waiting`, the server clock and `acceptance_recorded`; story:console-goal-cards reads the planner activity on the planner card and `merged_at` on assignments.

## Scope

Cited: crates/control-plane-app/src/live.rs (`compact` and its test module). No runtime file changes; ordered after story:bounded-progress-records, which also edits live.rs, and story:acceptance-traceability, which may rename tests in crates/control-plane-app.
