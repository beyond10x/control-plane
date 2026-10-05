---
format: aep.planning-md/3
id: story:planner-feedback-recovery
kind: story
status: active
title: Recover planner syntax errors and preserve action history
relations:
- decomposes: epic:bootstrap
- informed_by: story:autonomous-planner
- serves: vision:autonomous-engineering
scope:
- confidence: inferred
  path: crates/control-plane-runtime/
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T22:25:18Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T22:25:18Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Make the autonomous planner recover from ordinary command syntax mistakes and retain enough action history to avoid repeating unchanged observations. The live calculator goal must progress beyond planning; a restarted model request or a green scripted happy path is not evidence of recovery.

## Evidence

The live goal failed after requesting AEP args [artifact, --help], despite the earlier narrow help fix. Durable events also show repeated reads of the same unrelated bootstrap stories. In crates/control-plane-runtime/src/context.rs, deduplicating and re-appending the latest unchanged observation can yield an identical transcript. engine.rs supplies no action journal, and model.rs starts each request with one fresh user prompt. ProcessRunner previously erased the observed exit status into an untyped error.

## Existing specification

Use the existing ESS Goal revision, planning phase, planning reason, planning receipt, Assignment and RecordPlanningProgress contracts. This is execution-loop correction within those declared behaviors, not a new domain entity. Authority, storage, cancellation, forbidden effects and path confinement remain fail-closed.

## Acceptance

Named regression scenarios: aep_prefix_help_is_read_only; command_syntax_feedback_allows_correction; syntax_recovery_has_an_attempt_budget; unchanged_reads_preserve_action_history; alternating_reads_receive_stall_feedback; process_exit_retains_actual_status_for_syntax_recovery. Exercise the real Supervisor and installed AEP CLI with a model that reacts to observed feedback, retaining the existing forbidden-path and manufactured-evidence regressions. Confirm the live user goal produces a validated assignment or identify and repair the next reproducible product failure; do not claim successful delivery from a Planning label.

## Scope

Cited: crates/control-plane-runtime/src/engine.rs, context.rs, model.rs, process.rs and tests/planner.rs. The coordinator owns ProcessExit and live verification; the implementation lane owns planner feedback and its regressions; independent review checks recovery boundaries and repeated-read handling. AEP writes remain coordinator-owned.

## Authorization

The operator explicitly reports recurring failures and requests continuation of the previously authorized implementation goal. No pause or goal cancellation is authorized.
