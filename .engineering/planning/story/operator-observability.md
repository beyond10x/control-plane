---
format: aep.planning-md/3
id: story:operator-observability
kind: story
status: implemented
title: Live operational dashboard and visible autonomous activity
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:operator-console
scope:
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/control-plane-app
- confidence: cited
  path: crates/control-plane-runtime
- confidence: cited
  path: docs/vision.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T21:48:32Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T21:48:32Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-05T22:15:46Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":5}}}
---
## Outcome

Deliver the operational dashboard and automatically updating activity described in docs/vision.md. The operator can tell what the system is doing without inspecting raw JSON or manually refreshing a form page.

## Existing ESS contract

Use the already declared Workspace, Goal and Assignment views, Goal.planning_phase, planning_reason and planning_receipt, Assignment lifecycle and evidence fields, and RecordPlanningProgress. This work projects existing runtime facts and records observations through the existing durable receipt contract; it introduces no new domain entity. Keep receipt provenance and real observations separate from server-response freshness. Any additional domain noun must be specified before use.

## Acceptance

Named executable scenarios: model_wait_is_visible_before_response_and_activity_survives_restart; concurrent_workers_and_blocked_stopped_states_are_distinct (concurrent worker activity, blocked and stopped states); live_activity_is_durable_scoped_and_does_not_inline_model_receipts and workspace_stream_filters_foreign_changes_and_recovers_durable_progress (workspace scoping). Not yet a named test: a live update keeps a form's entered value, and the console tells idle from disconnected (story:console-status-and-attention derives both). Observe real commands, model-call entry/exit, checks and publication results; no synthetic work events or invented percentages. A browser run must see automatic updates while a form's entered value remains intact. Show actionable blockers, elapsed time, role, repository and worktree context. Raw evidence belongs behind drill-down controls.

## Scope

Cited: crates/control-plane-app (dashboard, navigation, live fragment/API routes, CSS and browser integration tests); crates/control-plane-runtime (durable model/tool activity); docs/vision.md and README.md. Coordinator owns AEP and runtime progress integration. UI implementor owns the app surface. Fleet implementor coordinates its progress calls with the common receipt shape.

## Authorization

Operator explicitly requested live ticks/status/progress, a control-plane application look and feel, and an updated goal/agenda and docs/vision.md. This is required in the existing autonomous implementation goal, not optional follow-up work.
