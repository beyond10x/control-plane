---
format: aep.planning-md/3
id: epic:console-clarity
kind: epic
status: active
title: The console says what is happening, what needs the operator and what to do
summary: Turn the live console from committed rows into an operating view, from the 2026-10-06 UI/UX reviews.
relations:
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T13:13:07Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-08T13:13:07Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Opening the console answers the questions in docs/vision.md:5-12 within seconds: whether the system is connected and working, what the planner and each worker are doing now and since when, what is blocked and what resolves it, and where to pause, cancel, edit a goal or change merge authority. Raw identifiers, hashes, paths, receipts and JSON stay one click away (behind a details disclosure or a download) instead of in the main view. One renderer draws the console: the Vue application, started from a snapshot embedded in the page, so the server-rendered copy and its duplicated stylesheet are removed.

## Why now

review-result:console-ux-review-2026-10-06 (two independent reviews, code and visual): the console fails the vision's 5-second test. A disconnected stream looks like a healthy idle system; a blocked goal keeps a green Running badge; the planner card shows worker events; activity titles are internal action ids with raw UTC timestamps; goal controls sit about 2,500 px down; destructive actions (cancel, delete goal, remove directory, disable repository) act on one click; evidence is 74,820 bytes of raw JSON; secondary text is 9-11 px at contrast 3.0-3.95; the server-rendered page and the Vue app have drifted.

## Stories and order

| story | size | depends on (shared file) |
|---|---|---|
| story:console-projection | M | story:bounded-progress-records (live.rs `compact`), story:acceptance-traceability (test renames in crates/control-plane-app) |
| story:console-component-tests | S | story:spec-history-gate (crates/control-plane-xtask/src) |
| story:console-status-and-attention | M | console-projection, console-component-tests (frontend/src) |
| story:console-goal-cards | L | console-status-and-attention (frontend/src/App.vue; consumes its derived goal state) |
| story:console-evidence-page | M | console-goal-cards (frontend/dist, rebuilt by every frontend story) |
| story:console-visual-baseline | M | console-evidence-page (frontend/src, frontend/dist, crates/control-plane-app/src) |

## Constraints

Vue frontend bundled into the Rust binary, live updates over SSE, no CDN and no Node at runtime (AGENTS.md). Frontend code and its component tests may be JavaScript/Vue; backend and tooling stay Rust. ESS views stay the source of the data; the projection only selects and derives from records that already exist. The live operator experience is part of acceptance (vision.md:40): each story names the behaviour a component test or a server test observes.

## Out of scope

New domain nouns, changes to what the runtime records, authentication of the local API (story:candidate-process-environment's out-of-scope note).
