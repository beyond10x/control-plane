---
format: aep.planning-md/3
id: story:bounded-progress-records
kind: story
status: implemented
title: Progress records stay bounded and restart replay stays fast
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:control-plane-review-2026-10-06
- depends_on: story:model-input-refusals
scope:
- confidence: cited
  path: crates/control-plane-app/src/dashboard.rs
- confidence: cited
  path: crates/control-plane-app/src/lib.rs
- confidence: cited
  path: crates/control-plane-app/src/live.rs
- confidence: cited
  path: crates/control-plane-app/src/tests.rs
- confidence: cited
  path: crates/control-plane-core/fixtures
- confidence: cited
  path: crates/control-plane-core/src/lib.rs
- confidence: cited
  path: crates/control-plane-core/src/memory.rs
- confidence: cited
  path: crates/control-plane-core/src/tests.rs
- confidence: cited
  path: crates/control-plane-runtime/src/context.rs
- confidence: cited
  path: crates/control-plane-runtime/src/engine.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/src/supervisor.rs
- confidence: cited
  path: crates/control-plane-runtime/tests
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T03:05:53Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":5}}, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T03:05:53Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":5}}, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T05:08:35Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":9,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

Recording runtime progress costs at most 16 KiB of event data per progress decision, restart does not replay an unbounded history, and the console still shows the same activity.

## Evidence

- Live store on 2026-10-06 03:45 CEST: 3,602 decisions, 812 MB; 3,524 `RecordPlanningProgress` decisions hold 809.8 MB; the largest is 2.44 MB. Current receipts of the three goals total 151 KB, so the rest is repeated copies.
- `Store::open` on a copy took 78.8 s in the debug profile the service runs.
- fleet.rs:79-97 rewrites the whole `planning_receipt` with a 64-entry history on every Loom event; crates/control-plane-core/src/lib.rs:183-222 replays every decision; lib.rs:320 deep-clones memory per view query; crates/control-plane-app/src/live.rs:52-81 parses every receipt per SSE update.
- The per-tick append for an unresolved publication (fleet.rs:1726) belongs to story:publication-exit, which owns `reconcile_publications`.

## Acceptance

- `progress_decision_stays_under_16_kib`: 1,000 progress events on one goal, each with a 1 KiB detail, each add no more than 16 KiB of event data.
- `restart_open_is_bounded`: a fixture built by sending 10,000 progress events, each with a 1 KiB detail, through the runtime's progress path for one goal opens with `Store::open` in under 5 s in the debug profile. On f510e7f the same fixture writes the full 64-entry history on every decision.
- `existing_receipts_still_open`: a committed fixture written with today's receipt shape opens after the change with identical WorkspaceList, GoalList, AssignmentList and PublicationIntentList rows.
- `console_activity_survives_bounding`: after 100 progress events sent through the progress path, the SSE projection for the goal lists the newest 24 activities in order, each with the same action, role, status and detail as the events sent, and `last_activity` equals the last event.
- The existing observability and SSE tests stay green.

## Specification

This story makes no specification change: `planning_receipt` stays a String field and the limit is a host rule. If the design turns out to need a new stored record type, that becomes a separate ESS story ordered after story:spec-history-gate, and this story waits for it.

## Scope

Cited: crates/control-plane-runtime/src/fleet.rs (`progress`), crates/control-plane-runtime/src/supervisor.rs (planner progress), crates/control-plane-core/src/lib.rs, crates/control-plane-app/src/live.rs. Inferred: crates/control-plane-core/src/memory.rs.

## Out of scope

Deleting or compacting the existing live store by hand: the migration must keep it openable, and removing history is the operator's decision.


## Narrowing decided during wave 1 (2026-10-06)

The 16 KiB bound applies to per-event progress decisions (Loom and fleet activity). Adversary pass 2 (review-result:adversary-bounded-progress-records-pass-2, P1) measured the validated-plan decision at 108,904 bytes: it is planning evidence recorded once per planning attempt, read by the evidence page and planner tests, and no later decision repeats it. The coordinator kept it, bounded it at 128 KiB, and required every later progress decision in the run to stay at or under 16 KiB. The acceptance record is capped at 5 KiB so its decision stays under 16 KiB (P2).
