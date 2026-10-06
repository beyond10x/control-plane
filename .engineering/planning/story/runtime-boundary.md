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
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: .gitignore
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: README.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/control-plane-app/
- confidence: cited
  path: crates/control-plane-core/
- confidence: inferred
  path: crates/control-plane-protocol/
- confidence: inferred
  path: crates/control-plane-runtime/
- confidence: cited
  path: crates/control-plane-xtask/
- confidence: inferred
  path: docs/
- confidence: inferred
  path: ess/
- confidence: cited
  path: frontend/
revision: 17
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

## Operator-authorized improvement cycle

The operator supersedes the previous single-attempt allocation: run as many eval rounds as useful to improve the system. Each round remains bounded and retains its observed failure, usage and runtime evidence. Start with the existing isolated Go auth workspace, current implementation and unchanged black-box acceptance.

After each round, diagnose and present concrete suggested changes with their responsible foundation or host owner and verification criteria. Ask the operator before implementing those changes. Once approved, dispatch a sub-agent for the approved implementation, integrate its result, verify it, and run the next eval. This approval applies to code changes proposed from each round; running the evals and recording their results is already authorized. Do not silently patch between rounds or mistake passing regressions for real application delivery.

Round 4 begins on the already-published correction 82aef4175944168ca8cf419428055b934b910517, with one worker, one attempt per round, gpt-5.6-sol for all roles and a ten-minute total watchdog. The historical failures remain in the external eval results. Successful outcome requires real runtime planning, implementation, trusted checks, independent review, observed publication and the auth verifier passing against the published application.

## Round 4 outcome and proposed correction

Round 4 ran for 569 seconds on control-plane 82aef4175944168ca8cf419428055b934b910517. Real provider planning recovered from ESS refusals, created one scoped story, passed independent plan review, implemented the Go app, executed tests and the trusted external auth verifier, passed independent code review and published c11fa5d62e059a3f731dcd0eaeed1b91c06f3681 to the isolated local origin. Checked, reviewed, candidate and observed target revisions match. The coordinator reran the trusted verifier against the clean published candidate: PASS go-auth-web. No manual implementation of the example was substituted.

Goal acceptance was rejected: the coordinator incorrectly added eval-round evidence retention to the application acceptance. The final reviewer supported all application requirements but could not verify retained prior eval failures from the app diff. Those records correctly live outside the candidate. This is an eval setup error, not evidence that the auth implementation failed. The coordinator stopped this eval goal, retaining its merged assignment, app and all workspaces; it is not marked Satisfied.

Recorded provider totals: 51 turns, 2,216,026 input tokens, 24,034 output tokens, 772,608 cached input tokens; dollar cost unavailable. Initial planning used 28 turns and 11 specification writes. Implementation used 12 turns but accumulated 1,611,572 input tokens. A deterministic inspection of its filed session found 12 user messages totalling 823,423 bytes, including the identical 56,134-byte planning receipt in every message (673,608 receipt bytes). fleet.rs serializes the entire Goal on each proposal and Host.respond sets continuation to None. Loom preserves that supplied history correctly; the host must stop supplying redundant operational records.

After final rejection, Supervisor.tick scheduled another planner while fleet acceptance itself remained blocked. Eight further planner turns were filed before the round was stopped. The existing rejected_goal_acceptance_remains_durable_and_idle_until_inputs_change test exercises only fleet_tick after restart, omitting tick. This is a scheduler coverage gap, not grounds to replace Loom execution.

### Proposed changes — awaiting operator approval

1. Control-plane model-input adapter: use a compact projection of the objective, acceptance and necessary authority/context references. Supply the role brief once and only new observations/frontier information on continuation. Exclude UI/activity receipts from every role prompt while retaining them in storage and the UI. Regression: N actions do not duplicate the role brief or planner receipts in the native session; authority and current-goal checks still use authoritative host state.
2. Control-plane scheduler: preserve failed final acceptance as an explicit gate across both planner and fleet scheduling, including restart. Resume only after relevant operator/repository inputs change. Regression drives tick plus fleet_tick after a real merged assignment/rejected final review and proves zero additional provider calls on unchanged inputs; a relevant change permits progress.
3. Control-plane authoring-context binding: expose bounded read-only lookup of the existing ESS-owned authoring schema and version-matched syntax/examples. ESS already generates schemas/generated/ess.schema.json from ess_domain::spec::RawSpecFile via cargo xtask schema; do not hand-write a parallel schema or insert the entire schema into every prompt. Validate the reference pin and exercise lifecycle transition/outcome/payload lookup through the native runtime.
4. Eval configuration: keep orchestration/evidence-retention instructions outside the app acceptance, which must contain only the fixed TASK.md and trusted verifier obligations. Preserve all previous evidence. Compare the next round on an isolated seed while retaining the successfully published application.

No implementation of these suggestions is authorized yet. After operator approval, delegate the approved scope to a sub-agent, independently inspect and integrate its changes, run regressions and repository gates, then run the next bounded real eval. Further eval rounds are already authorized; the per-round change approval is the operator's explicit workflow requirement.

## Approved implementation batch

The operator explicitly approved all four round-4 proposals and additionally required SSE instead of the refreshing dashboard. Implement the approval with delegated workers, then coordinator integration and verification, followed by another isolated eval. Do not ask for the same approval again.

Runtime worker owns crates/control-plane-runtime/ and its tests: compact role inputs and delta continuation, scheduler hold after final rejection, bounded read-only lookup of the existing ESS-owned authoring schema. These are host bindings/scheduling, not new Loom algorithms. Preserve foundation ownership, evidence, authority and session guarantees.

Console worker owns crates/control-plane-app/ and its tests: replace the one-second HTML refresh/iframe presentation with SSE-driven in-place operations updates, preserving entered forms, focus and scroll. Push committed state/progress, support reconnect and accurate disconnected status, and retain per-workspace isolation. Existing ESS host Goal/Assignment/progress views own the data; SSE is the presentation transport over those views, not another source of state. Any new typed contract must be specified before implementation.

Coordinator owns AEP, integration, evaluation configuration and final verification. Correct eval acceptance by keeping round bookkeeping in external results, and use an isolated fresh seed while retaining the successful round-4 app. Runtime and console worker file scopes do not overlap. Checks must establish bounded native-session input growth, no model calls after rejected final acceptance on unchanged inputs, correct schema reference lookup, streamed updates without page reload and preserved UI editing state, followed by the real auth eval.

## Frontend technology clarification

The operator clarified that Rust is required for tooling/backend code, not frontends, and explicitly requested a Vue application bundled and embedded in the binary. The console worker will replace the refreshing iframe with a reactive Vue app consuming SSE; no WebAssembly or CDN is requested. Node is a frontend build dependency only. Committed deterministic assets are embedded for a single Rust runtime binary. The coordinator adds a Rust build/drift gate and CI setup; the worker owns frontend source, assets, SSE app routes and the post-commit Store notification seam. Existing ESS host data/lifecycles remain authoritative.

Round 5 uses a newly seeded independent auth repository and local origin outside this repository; round 4's published app is preserved. The goal is created paused until integrated gates and browser checks pass. Application acceptance contains only TASK.md/verifier obligations; round metadata and retained failure evidence live outside the app.

## Integrated Vue/SSE and runtime verification

The approved worker changes are integrated. Backend/tooling remain Rust; the frontend is Vue with deterministic local assets embedded into the binary. Runtime source policy also permits accepted frontend/ and web/ JS/TS/JSX/TSX/Vue assets while preserving scope, backend/tooling restrictions and the explicit external Go eval exception.

Full task check passed: strict ESS and AEP validation, frontend byte drift, formatting, clippy with denied warnings, 152 Rust tests, generated byte drift, 167 durable conformance scenarios with zero failed/error/unsupported/skipped and foundation dependency qualification. A deliberate embedded CSS mutation failed frontend-check and was restored. Native-session and scheduler regressions previously demonstrated red against duplicated briefs and the missing tick rejection guard; corrected tests now pass.

The actual console on port 8788 was restarted with the new binary; executable digest matches the build. All five workspaces and prior goal state survived. Chromium first reproduced the old iframe refresh (five frame navigations, one iframe, no SSE requests). With Vue, a real operator UpdateGoal committed through the API and appeared over SSE: one navigation, no iframe, no JavaScript error, unsaved draft retained, focus retained, selection 5..12 retained and identical scroll position. A real service shutdown produced Disconnected; reconnection after durable replay restored Live without navigation or losing the draft. This is actual running-browser evidence, not a mock stream. The external eval results retain sse-before.json, sse-after.json and sse-after.png.

Coordinator review: Store notifies only after Eventlog commits; the stream queries existing generated views, bounds operational details and filters foreign workspace fleet entries. Loom owns persisted sessions and compaction, Commission owns effect admission, ESS owns the exact referenced schema, and control-plane owns model-input projection, scheduling and UI transport. No new foundation execution replacement or unproven foundation gap was introduced.

Round 5 is now executing through the real runtime in a fresh external Go seed with one worker, one attempt, gpt-5.6-sol and a ten-minute total watchdog. Do not claim final outcome from the passing component/browser checks.
