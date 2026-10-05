---
format: aep.planning-md/3
id: story:runtime-boundary
kind: story
status: active
title: Run every agent through the governed Loom boundary
summary: Remove private turn loops and duplicated governor behavior while preserving workspace/UI orchestration; fix foundation composition gaps at their owners.
relations:
- decomposes: epic:bootstrap
- informed_by: architecture-design:runtime-ownership
- serves: vision:autonomous-engineering
- blocks: story:planner-feedback-recovery
scope:
- confidence: inferred
  path: crates/control-plane-protocol/
- confidence: inferred
  path: crates/control-plane-runtime/
- confidence: inferred
  path: docs/
- confidence: inferred
  path: ess/
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T23:11:49Z", actor: "human:timo", revision: 3, executor: "agent:control-plane-coordinator", correlation: "runtime-ownership-correction"}
- {from: "proposed", to: "active", at: "2026-10-05T23:11:49Z", actor: "human:timo", revision: 4, executor: "agent:control-plane-coordinator", correlation: "runtime-ownership-correction"}
---
# Restore the runtime ownership boundary

## Outcome

The operator gets planner, implementor and reviewer runs through the existing governed Loom foundation, with usable tool feedback, durable budgets and visible runtime events, while retaining all registered workspaces and product data.

## Basis

Follow architecture-design:runtime-ownership. The two external auth-eval failures and the direct fleet model loop demonstrate the current mismatch. Local recovery patches are provisional regression evidence; moving more generic execution behavior into this product is not completion.

## Scope and owners

Control-plane owns workspace/goal APIs, run scheduling, contention, operator authority inputs, provider/store/effect bindings and UI projections. Loom owns turn/context/session/budget machinery. Its Commission runtime owns invocation/revalidation. Its governor owns frontier/completion with Canon. Engineering protocols own reusable planning/source-delivery semantics. ESS and AEP own specification and planning behavior. Worktree, Eventlog, LLM and execution substrate retain their existing responsibilities.

Resolve foundation gaps through their repositories first: a demonstrated governed native-session bridge, a fallible durable CaseStore, and admitted planning/source-delivery profiles. Do not weaken authority, independent review, source revision binding or publication reconciliation to fit an existing example. Do not start a new repository or discard the current host.

## Acceptance

- shared-governed-executor
- missing-file-refusal-roundtrip
- no-change-is-not-progress
- stale-authority-before-effect
- session-budget-restart
- storage-failure-stops-effects
- visible-runtime-events
- current-review-and-publication

The architecture artifact defines each scenario's observable result. First specify any changed public entities/contracts in their owning ESS domain, then implement the named regressions using the actual SDK and scripted provider responses. Production dependency/architecture checks reject direct model-loop implementation in control-plane after migration. A single bounded real external eval follows those passing checks; failed runs remain failed.

## Sequence

1. Pin the ownership design and reproduce the observed failures at the SDK seams.
2. Deliver any required foundation contract/composition changes in their owner, with their own ESS and gates.
3. Bind both roles through the shared runtime; preserve the existing UI, API, data, worktree records and publication reconciliation.
4. Replace handcrafted activity/session data with actual runtime events and run the integration scenarios.
5. Run one bounded auth example through its existing trusted verifier. Report exact result and spend evidence, including unknowns.

## Out of scope

A second control-plane, replacement agent framework, vendor-harness orchestration, release/deployment, and repeated paid retries while deterministic seam checks fail.

## Authorized correction and review criteria (2026-10-06)

The operator explicitly authorized implementation and verification of this correction. The active unit is runtime-boundary; dependent external-evaluations remains active but cannot spend another model attempt until deterministic runtime checks pass.

The concrete integration uses Loom's existing native AgentLoop, OutputSchema, SessionFile and Commission runtime. Control-plane binds the LLM single-turn port and trusted effects; it must not implement model retry, compaction or tool-roundtrip algorithms. Opaque provider items retain their complete LLM binding provenance. All roles share this path, with separate reviewer sessions. Session filing errors stop effects; session spend is retained and deducted when binding remaining product limits.

One foundation worker owns the bounded Loom governor contract change: supplied validated protocols and explicit storage failure. The coordinator owns control-plane adapters and integration. Review must check the exact pinned API, lossless provider continuation, no model-controlled authority, no direct fleet execution loop, refusal recovery, durable session spend, and unchanged publication reconciliation. A framework written privately in the host is a rejection.

Current live outcome is still blocked: auth goal revision 2 failed on an admitted read of a file not yet created. No new paid run has started. Completion requires a real goal-to-published-Go-application result, trusted authentication verifier success and observable UI activity; unit tests alone do not close this story.

## Implemented ownership binding

The original generic-session-bridge gap is narrowed by implementation evidence: the existing public AgentLoop + OutputSchema + SessionFile API composes with Commission without a new framework. A thin LLM ModelPort projection preserves each opaque item's complete serving provenance, forwards streamed events, and supplies cancellation/deadline bindings. Actual retry/compaction/structured-output/session machinery stays in Loom. Cases are bounded-attempt governor state; restart retains the product record and Loom sessions, and opens fresh cases for trusted revalidation rather than replaying an uncertain effect.

The remaining demonstrated governor gaps are corrected in published Loom commit 1b25fe8939e1883f3e47c1292ac90cf75fc3f3a5: with_protocol_yaml, trusted evaluation time, and FallibleCaseStore. Control-plane pins that exact commit. Product-specific admitted profiles stay host configuration; this change does not claim a newly released engineering-protocols profile or a sandbox from the local adapter.

## ESS first

This adapter correction introduces no new product entity or lifecycle. Existing controlplane.host workspace, Goal, Assignment and PublicationIntent contracts remain unchanged. Loom's existing loom.run Session/Turn and Commission responsibility contracts govern the runtime integration. Foundation behavior/specification changes and ESS0.53 upgrade are recorded in Loom story:hosted-governor. Control-plane conformance must remain fully green without changing expected lifecycle outcomes.

## Verification in progress

Native runtime regression reproduces loss of continuation by deliberately discarding session items: the retention assertion fails. Restoring the integration passes. Native missing-file recovery drives LLM Model -> Loom AgentLoop/SessionFile -> Commission -> CanonGovernor -> trusted ESS/AEP effects and queues one validated story, with separate critic session and durable usage observation. Existing 32 planner and 10 fleet scenarios pass after owner migration; final integrated provider/fleet test, independent review and live external auth eval remain required.

## Independent review and cancellation counterexample

Independent source review found no admission/provenance/publication bypass, but correctly distinguished per-phase limits from the total eval ceiling. The real eval therefore gets a separate 600-second deadline: cancel its exact goal and signal the service to cancel in-flight provider work, including a silent stream.

The follow-up streaming test found a real adapter liveness defect: a midstream progress persistence refusal initially abandoned the receiver without cancelling the provider (red at its 2-second deadline). Explicit cancellation now reaches the provider; both progress refusal and external cancellation pass in 0.01 seconds. The reviewer confirmed the narrow fix. Planning session identity now derives from its retained worktree, so restart cannot silently discard a filed session's spent turns. Loom deliberately refuses a crash-left Active session; automatic recovery of unrecorded in-flight spend is not claimed.

## Corrected runtime and third auth eval

The ownership correction is implemented on published Loom 1b25fe8939e1883f3e47c1292ac90cf75fc3f3a5. Native model sessions, streamed observations, Commission effect admission and the supplied Canon governor replace the duplicated paths. All four registered workspaces and the existing blocked goal survived a real service restart. A headless browser capture showed the auth workspace, its goal revision and actual Loom turn/stream activity.

One authorized real auth eval ran on goal revision 3 with one worker, one attempt and a ten-minute total watchdog. It stopped after approximately 48 seconds and six planner turns: invalid ESS emitted dead_end_state, unreachable_state and conflicting_declaration diagnostics. The adapter incorrectly treated that observed validation refusal as external unavailability. No assignment, candidate or publication was produced. Observed usage: 37,523 input tokens, 1,852 output tokens, of which 1,536 input tokens were cached; dollar cost was not reported. Results and session evidence remain outside the source repository under the configured eval root. No extra paid retry was started.

The exact invalid domain now participates in native_loom_recovers_missing_and_invalid_specification_and_queues_validated_plan. It failed against the deployed adapter and passes after typed ESS-validation refusal handling. The test uses the real LLM port, Loom sessions, Commission, Canon governor, ESS and AEP; it proves diagnostics reach the next turn, invalid ESS does not mutate AEP, correction queues a validated story and independent review remains separate. Invalid Finish requests do not spend a critic review. Four unchanged validation refusals still stop; cancellation, transport/launch failures and non-validation process errors remain fatal. All 33 planner tests pass. This deterministic recovery is not a real-provider auth application success.

The end-to-end acceptance remains unmet. Another paid eval requires an explicit additional attempt allocation; the agreed one-attempt limit has been consumed. Preserve the failed goal and all workspaces; do not erase the failure or mark this story implemented from component tests.

## Installed verification candidate

The final feedback correction was built and installed in the local console. The running executable digest matches the built binary. A second real restart preserved all four workspaces, goal revision 3, its blocked failure receipt and zero assignments. Restart did not launch another model attempt. The failed goal remains visible; changing the installed code does not rewrite its historical outcome.
