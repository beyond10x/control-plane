---
format: aep.planning-md/3
id: review-result:plan-audit-2026-10-08
kind: review-result
status: active
title: 'Planning store audit: stale blockers, drafted epics, prose-only upstream edges'
relations:
- reviews: epic:unattended-operation
- reviews: epic:console-clarity
- reviews: story:conformance-evidence-record
- reviews: story:runtime-boundary
- reviews: story:external-evaluations
- reviews: story:planner-feedback-recovery
- reviews: dependency-blocker:aep-imports-ess-053-suites
- reviews: executable-system-specification:control-plane
- reviews: story:spec-owned-busy-rules
- reviews: story:remove-stale-revision-mutation-exemption
- reviews: story:unattended-goal-evidence
- reviews: upstream-blocker:ess-generates-several-related-rows
- reviews: upstream-blocker:ess-negated-defined-witness
- reviews: upstream-blocker:ess-subject-field-selectors
revision: 1
---
needs-revision

dependency-blocker:aep-imports-ess-053-suites — The blocker's cause is gone because AEP 0.69.0 now admits the `scenario_initial_state` provenance member, so it can be cleared once CI's aep pin moves from 0.68.0 to 0.69.x or later and the import is re-run. — .engineering/planning/dependency-blocker/aep-imports-ess-053-suites.md:27-29; `gh release view 0.69.0 -R beyond10x/aep`; .github/workflows/check.yml:62,109
executable-system-specification:control-plane — The body describes ESS 0.53.0 with 125 capabilities, zero obligations and 159 scenarios, but main runs ESS 0.56.0 with 136 capabilities, 1 obligation and 204 synthesized scenarios, so the body and `model_digest` need refreshing. — .engineering/planning/executable-system-specification/control-plane.md:12-14; .engineering/planning/story/admission-conformance-target.md:21; .github/workflows/check.yml:61
story:conformance-evidence-record — The evidence still cites the AEP 0.68.0 refusal, which 0.69.0 lifted, and cites `target.rs:207`, which is line 229 on 89c62c8. — .engineering/planning/story/conformance-evidence-record.md:23-24; `git show 89c62c8:crates/control-plane-xtask/src/target.rs` line 229
story:runtime-boundary — The story has been active since 2026-10-05 and the body was last edited 2026-10-06. Its final round-6 proposal still says "awaiting operator approval", but story:model-input-refusals delivered that proposal (implemented 2026-10-06T02:50Z), and the body never records this. — .engineering/planning/story/runtime-boundary.md:223-239; .engineering/planning/story/model-input-refusals.md frontmatter transitions
story:runtime-boundary — Two acceptance lines read "Not yet a named test", so they name no checkable scenario. — .engineering/planning/story/runtime-boundary.md:65,71
story:external-evaluations — Everything this story delivers is on main (DeleteGoal, the three eval cases, the trusted verifier, and the regression `cancelled_goal_can_be_deleted_without_deleting_its_workspace`). The real-run evidence now belongs to story:unattended-goal-evidence, yet this story is still active. — .engineering/planning/story/external-evaluations.md:30-41; crates/control-plane-xtask/src/eval.rs:46-52; crates/control-plane-core/src/tests.rs:91
story:planner-feedback-recovery — All six named regressions exist on main, and later eval rounds progressed past planning (round 4 delivered an application), yet the story is still active and still blocked by story:runtime-boundary. — .engineering/planning/story/planner-feedback-recovery.md:33; crates/control-plane-runtime/tests/planner.rs; crates/control-plane-runtime/src/process.rs
epic:unattended-operation — The epic is still draft although 13 of its 17 stories are implemented, so it should be active. — `aep plan artifact list`
epic:unattended-operation — The "Stories and order" table and the first-wave sentence list only the first 7 stories. Ten later stories that are part of this epic are missing from the body and from the Outcome list. — .engineering/planning/epic/unattended-operation.md:16-24,36-48
epic:console-clarity — The epic is still draft although 4 of its 6 stories are implemented, so it should be active. — `aep plan artifact list`
story:spec-owned-busy-rules — This story and story:admission-conformance-target both change ess/domains/host.yaml, ess/README.md, ess/spec-acknowledgements.json, generated and crates/control-plane-core/src/guards.rs, and no `depends_on` edge orders them. — .engineering/planning/story/spec-owned-busy-rules.md:34; .engineering/planning/story/admission-conformance-target.md:32
story:remove-stale-revision-mutation-exemption — It is the only story with no `serves: vision:autonomous-engineering` edge. — .engineering/planning/story/remove-stale-revision-mutation-exemption.md:7-9
story:unattended-goal-evidence — The body says it is "Planned for the wave after wave 6", but wave 7 left it out and moved it to wave 8 behind decision-blocker:unattended-run-spending. — .engineering/planning/story/unattended-goal-evidence.md:50; .engineering/waves/2026-10-08-wave-7.md:46
upstream-blocker:ess-generates-several-related-rows — The ESS story it waits on is named only in prose. It could be a cross-member edge, but the store has no `.engineering/workspace.yaml` declaring an ess member, and the local aep is 0.69.0 (cross-member `relate` needs 0.69.1). — .engineering/planning/upstream-blocker/ess-generates-several-related-rows.md:17; `ls .engineering/workspace.yaml` (absent); `aep --version`
upstream-blocker:ess-negated-defined-witness — The ESS story it waits on is named only in prose, so it is not an edge and cannot be one until a workspace file exists and aep is 0.69.1 or later. — .engineering/planning/upstream-blocker/ess-negated-defined-witness.md:17
upstream-blocker:ess-subject-field-selectors — The two ESS stories it waits on are named only in prose, so they are not edges and cannot be until a workspace file exists and aep is 0.69.1 or later. — .engineering/planning/upstream-blocker/ess-subject-field-selectors.md:17-18

**Proposed commands** (written out, none run):
- `aep plan artifact move epic:unattended-operation --to proposed`, then `--to active`. Do the same for epic:console-clarity.
- `aep plan artifact move story:external-evaluations --to implemented` and `aep plan artifact move story:planner-feedback-recovery --to implemented`, once you confirm their acceptance.
- `aep plan artifact relate story:spec-owned-busy-rules depends_on story:admission-conformance-target`
- `aep plan artifact relate story:remove-stale-revision-mutation-exemption serves vision:autonomous-engineering`
- First move the CI aep pin to 0.69.x or later, then `aep plan artifact move dependency-blocker:aep-imports-ess-053-suites --to cleared`.
- For the three upstream blockers: add `.engineering/workspace.yaml` declaring the ess member, pin aep 0.69.1 or later, then `aep plan artifact relate <story> depends_on ess/story:<name>`.

**What I read:** 128 artifacts. I ran `aep plan artifact validate`, `list`, `graph` and `lifecycle epic|story|review-result`, read every in-flight and draft story, epic and open blocker, and checked them with `git grep`/`git show` against 89c62c8 and with `gh release view` for aep and ess.

**What I could not establish:**
- Whether the ESS story ids named in the upstream blockers exist in the ESS store. A guard refused reads of ~/beyond10x/ess.
- Whether AEP 0.69.x admits the current ESS 0.56.0 suite. Proving it needs `aep plan artifact evidence`, which is a write verb.
- Whether aep 0.68.0's `validate` accepts cross-member edges.
- Whether `model_digest` matches the current spec.
- I left the 61 active review-result artifacts alone. The store's conventions say nothing about when to archive them.

`aep plan artifact validate`, verbatim:
```
128 file(s) in <checkout>/.engineering/planning: 128 artifact(s)
valid
```

```findings
- file: .engineering/planning/dependency-blocker/aep-imports-ess-053-suites.md
  line: 27
  category: drift
  severity: blocker
  verdict: needs-revision
  origin: undecided
  message: "The blocker's cause is gone because AEP 0.69.0 now admits the scenario_initial_state provenance member, so it can be cleared once CI's aep pin moves from 0.68.0 to 0.69.x or later and the import is re-run."
- file: .engineering/planning/executable-system-specification/control-plane.md
  line: 12
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The body describes ESS 0.53.0 with 125 capabilities, zero obligations and 159 scenarios, but main runs ESS 0.56.0 with 136 capabilities, 1 obligation and 204 synthesized scenarios, so the body and model_digest need refreshing."
- file: .engineering/planning/story/conformance-evidence-record.md
  line: 23
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The evidence still cites the AEP 0.68.0 refusal, which 0.69.0 lifted, and cites target.rs:207, which is line 229 on 89c62c8."
- file: .engineering/planning/story/runtime-boundary.md
  line: 223
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The story has been active since 2026-10-05 and the body was last edited 2026-10-06. Its final round-6 proposal still says awaiting operator approval, but story:model-input-refusals delivered that proposal (implemented 2026-10-06T02:50Z), and the body never records this."
- file: .engineering/planning/story/runtime-boundary.md
  line: 65
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "Two acceptance lines read 'Not yet a named test', so they name no checkable scenario."
- file: .engineering/planning/story/external-evaluations.md
  line: 30
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "Everything this story delivers is on main (DeleteGoal, the three eval cases, the trusted verifier, and the regression cancelled_goal_can_be_deleted_without_deleting_its_workspace). The real-run evidence now belongs to story:unattended-goal-evidence, yet this story is still active."
- file: .engineering/planning/story/planner-feedback-recovery.md
  line: 33
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "All six named regressions exist on main, and later eval rounds progressed past planning (round 4 delivered an application), yet the story is still active and still blocked by story:runtime-boundary."
- file: aep plan artifact list
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "epic:unattended-operation is still draft although 13 of its 17 stories are implemented, so it should be active."
- file: .engineering/planning/epic/unattended-operation.md
  line: 36
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The Stories and order table and the first-wave sentence list only the first 7 stories. Ten later stories that are part of this epic are missing from the body and from the Outcome list."
- file: aep plan artifact list
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "epic:console-clarity is still draft although 4 of its 6 stories are implemented, so it should be active."
- file: .engineering/planning/story/spec-owned-busy-rules.md
  line: 34
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "This story and story:admission-conformance-target both change ess/domains/host.yaml, ess/README.md, ess/spec-acknowledgements.json, generated and crates/control-plane-core/src/guards.rs, and no depends_on edge orders them."
- file: .engineering/planning/story/remove-stale-revision-mutation-exemption.md
  line: 7
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "It is the only story with no serves: vision:autonomous-engineering edge."
- file: .engineering/planning/story/unattended-goal-evidence.md
  line: 50
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The body says it is 'Planned for the wave after wave 6', but wave 7 left it out and moved it to wave 8 behind decision-blocker:unattended-run-spending."
- file: .engineering/planning/upstream-blocker/ess-generates-several-related-rows.md
  line: 17
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The ESS story it waits on is named only in prose. It could be a cross-member edge, but the store has no .engineering/workspace.yaml declaring an ess member, and the local aep is 0.69.0 (cross-member relate needs 0.69.1)."
- file: .engineering/planning/upstream-blocker/ess-negated-defined-witness.md
  line: 17
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The ESS story it waits on is named only in prose, so it is not an edge and cannot be one until a workspace file exists and aep is 0.69.1 or later."
- file: .engineering/planning/upstream-blocker/ess-subject-field-selectors.md
  line: 17
  category: drift
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "The two ESS stories it waits on are named only in prose, so they are not edges and cannot be until a workspace file exists and aep is 0.69.1 or later."
```
