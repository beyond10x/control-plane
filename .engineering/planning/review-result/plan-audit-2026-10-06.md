---
format: aep.planning-md/3
id: review-result:plan-audit-2026-10-06
kind: review-result
status: active
title: 'Planning store audit: acceptance traceability, attribution, journals, stale constraints'
relations:
- reviews: epic:bootstrap
- reviews: story:planner-feedback-recovery
- reviews: story:verification
- reviews: story:runtime-boundary
- reviews: story:external-evaluations
- reviews: executable-system-specification:control-plane
revision: 1
---
# Planning store audit, 2026-10-06

Store at f510e7f: 31 artifacts, `aep plan artifact validate` valid. This audit covers what `validate` cannot see.

1. **Acceptance names that no test carries.** The Acceptance sections of the stories name 52 scenarios. 21 of those names occur nowhere in crates/ (exact search, hyphens also tried as underscores). Several have renamed equivalents, for example `planner_activity_survives_restart` inside `model_wait_is_visible_before_response_and_activity_survives_restart`, `two_repository_goal_delivery` inside `review_is_independent_and_two_repositories_reach_observed_goal_completion`, `alternating_reads_receive_stall_feedback` as `alternating_reads_retain_unchanged_feedback_across_actions`. Three implemented stories carry unmatched names: story:operator-observability (5 of 6), story:verification (3 of 3) and story:workspace-directories (1 of 6). The other unmatched names belong to active stories: story:planner-feedback-recovery (5 of 6) and story:runtime-boundary (7 of 8). Nothing checks the mapping, so an acceptance list can drift from the suite silently.
2. **Every move is attributed to the operator.** All 36 recorded transitions say `actor: "human:timo"`; two carry `executor: "agent:control-plane-coordinator"`. Moves the coordinator made at one second (story:verification draft → proposed → active → implemented, all at 2026-10-05T22:17:16Z) read as the operator's decisions. Future agent moves should pass `--executor`.
3. **Two story bodies are run journals.** story:runtime-boundary (239 lines) and story:external-evaluations (149 lines) carry eval-round narratives; six round sections appear verbatim in both ("Corrected runtime and third auth eval", "Operator-authorized improvement cycle", "Round 4…", "Round 5…", "Approved round 5 recovery", "Round 6…"). Acceptance is hard to find, and the duplication doubles every future edit. One verification-report per round, related to the story it verifies, would hold the same facts once.
4. **story:planner-feedback-recovery cannot close as written.** It is active and blocked by story:runtime-boundary. Its acceptance requires "the live calculator goal" to progress; story:external-evaluations records that goal as cancelled. Five of its six scenario names have no exact test; renamed equivalents exist. Closing it, or rewriting its acceptance against the auth eval, is the coordinator's decision.
5. **epic:bootstrap Constraints are stale.** "server-rendered local UI" was replaced by the operator's Vue/SSE clarification recorded in story:runtime-boundary ("Frontend technology clarification") and AGENTS.md.
6. **The ESS artifact cannot reach conforming.** executable-system-specification:control-plane stays validated. Re-run on a throwaway copy of the store on 2026-10-06 with AEP 0.68.0, `aep plan artifact evidence … --from report.json --suite generated/conformance.json` still answers `Error: UnknownField at $suite.provenance.scenario_initial_state: closed count-stage vocabulary`. No beyond10x/aep issue matches "scenario_initial_state" or "count-stage" (searched 2026-10-06). Once it is admitted, the report's `completed_at` (2023-11-14, the ESS runner's default clock) would date the evidence wrongly.
7. **The eval loop fixes one member of a class per round.** Rounds 3, 5 and 6 each stopped because a refused model input became `EffectError`: ESS validation, read-path syntax, command grammar. Each correction admitted one member as a refusal. The round-6 proposal again covers one member; the code review lists twelve more.

```findings
[
{"file":".engineering/planning/story/verification.md","line":30,"category":"traceability","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"21 of 52 named acceptance scenarios occur nowhere in crates/; implemented stories operator-observability (5/6), verification (3/3), workspace-directories (1/6)."},
{"file":".engineering/planning/story/verification.md","line":20,"category":"provenance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"All 36 transitions record actor human:timo, including agent moves made within one second; 2 carry an executor."},
{"file":".engineering/planning/story/runtime-boundary.md","line":1,"category":"maintainability","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Run journals in story bodies; six round sections duplicated verbatim in runtime-boundary and external-evaluations."},
{"file":".engineering/planning/story/planner-feedback-recovery.md","line":31,"category":"stale","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Acceptance depends on the cancelled live calculator goal; story cannot close as written."},
{"file":".engineering/planning/epic/bootstrap.md","line":18,"category":"stale","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Constraints still say server-rendered local UI; the operator replaced it with an embedded Vue/SSE console."},
{"file":".engineering/planning/executable-system-specification/control-plane.md","line":14,"category":"dependency","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"AEP 0.68.0 still refuses the ESS 0.53 report: UnknownField at $suite.provenance.scenario_initial_state; no AEP issue found."},
{"file":".engineering/planning/story/runtime-boundary.md","line":223,"category":"scope","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Eval rounds 3, 5 and 6 each fixed one member of one refusal class; the round-6 proposal covers one more member, twelve remain."}
]
```
