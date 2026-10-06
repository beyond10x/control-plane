---
format: aep.planning-md/3
id: story:bounded-progress-records
kind: story
status: draft
title: Progress records stay bounded and restart replay stays fast
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:control-plane-review-2026-10-06
- depends_on: story:model-input-refusals
scope:
- confidence: cited
  path: crates/control-plane-app/src/live.rs
- confidence: cited
  path: crates/control-plane-core/src/lib.rs
- confidence: inferred
  path: crates/control-plane-core/src/memory.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/src/supervisor.rs
revision: 6
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
