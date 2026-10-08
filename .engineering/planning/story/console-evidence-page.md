---
format: aep.planning-md/3
id: story:console-evidence-page
kind: story
status: active
title: Evidence opens as a readable view with the raw JSON as a download
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-goal-cards
- supersedes: story:console-evidence-and-baseline
scope:
- confidence: inferred
  path: crates/control-plane-app/src/dashboard.rs
- confidence: inferred
  path: crates/control-plane-app/src/lib.rs
- confidence: inferred
  path: crates/control-plane-app/src/tests.rs
- confidence: inferred
  path: frontend/dist
- confidence: inferred
  path: frontend/src
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

"Inspect evidence" opens a Vue view instead of escaped JSON. Before: `GET /goals/{id}/evidence` returns the evidence JSON (crates/control-plane-app/src/dashboard.rs:12-35). After: `GET /goals/{id}/evidence` serves the console, which renders the view, and the unchanged JSON moves to `GET /api/goals/{id}/evidence`, which the view reads and offers as a download. The view shows, from what the JSON already holds: the goal's state and whether an acceptance is recorded, a timeline of recorded activity, the latest check command and the candidate it ran on, the reviewer run that approved each assignment, and the published commit from each merge receipt. No new route renders HTML on the server.

## Evidence

review-result:console-ux-review-2026-10-06 row 5: the evidence page serves 74,820 bytes of escaped JSON with no summary. The JSON holds the goal, its assignments and publications, and the planning receipt with its activity history (dashboard.rs:12-35).

## Acceptance

- `evidence_path_serves_the_console`: a server test requests `/goals/{id}/evidence` and receives the console HTML, not JSON.
- `evidence_json_moves_to_api_path`: a server test requests `/api/goals/{id}/evidence` and receives JSON equal to what `/goals/{id}/evidence` returned before the change, for the same fixture.
- `evidence_view_summarises_before_raw`: for a fixture of a merged assignment the view renders the goal state, the check command and candidate, the reviewer run and the published commit before the download link.
- `evidence_view_handles_a_goal_without_merges`: a cancelled goal with no assignments renders its timeline and says nothing was merged.

## Scope

Inferred: new frontend/src/Evidence.vue and its test, frontend/src/main.js (route), crates/control-plane-app/src/dashboard.rs and crates/control-plane-app/src/lib.rs (routes), crates/control-plane-app/src/tests.rs (server tests), frontend/dist. Ordered after story:console-goal-cards because every frontend story rebuilds frontend/dist.
