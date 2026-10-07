---
format: aep.planning-md/3
id: review-result:ess-design-review-2026-10-06
kind: review-result
status: active
title: 'ESS design review: 16 missing, 2 contradicting, 2 spec-only declarations'
relations:
- reviews: executable-system-specification:control-plane
revision: 1
---
# ESS design review (hardening technique 8), 2026-10-06

Specification: ess/ at f510e7f (`ess specify validate --path ess --strict-requires`: "controlplane v1 — 3 file(s), valid"). Design: docs/vision.md, README.md, epic:bootstrap, and the implementation contracts of story:workspace-host, story:workspace-directories, story:implementation-fleet and story:autonomous-planner. No mapping document exists.

The reviewer is the session that also read guards.rs, so this review is not independent of the code review. The planted-defect check of the brief was not run: a reviewer who planted the defect cannot test whether the review finds it.

| # | classification | design (quoted) | spec (file:line) | note |
|---|---|---|---|---|
| 1 | missing | epic:bootstrap Constraints "One active goal per workspace" | host.yaml:644 StartGoal declares no guard | enforced only in guards.rs:120 |
| 2 | missing | story:workspace-host "one active assignment per canonical common Git directory across workspaces" | host.yaml:812 ClaimAssignment, no guard | guards.rs:263; spans Assignment → RepositoryRegistration.common_dir |
| 3 | missing | story:implementation-fleet "Run at most the configured worker limit" | host.yaml:812, 874 | guards.rs:282 |
| 4 | missing | story:implementation-fleet "Enforce attempt/time budgets" | host.yaml:812, 874; Goal.max_attempts host.yaml:111 | guards.rs:255 |
| 5 | missing | vision.md:30 "Changes to a goal, repository configuration or authority invalidate stale evidence" | host.yaml:751, 812–953: goal_revision stored, never compared | guards.rs:152, 235, 240 |
| 6 | missing | AGENTS.md:18 "Review must use a distinct execution context" | host.yaml:907 ReadyAssignment, no guard | guards.rs:314 |
| 7 | missing | story:implementation-fleet "Bind test and review evidence to the exact candidate" | host.yaml:846, 907 | guards.rs:307, 320 |
| 8 | missing | vision.md:32 "merge authority are explicit operator controls"; story:implementation-fleet "Recheck standing merge authority" | host.yaml:935 MergeAssignment, 1131 PreparePublication, no guard | guards.rs:328 |
| 9 | missing | story:implementation-fleet "Persist a PublicationIntent before invocation" | host.yaml:935 | guards.rs:358 |
| 10 | missing | story:implementation-fleet "exit zero is not a merge receipt" | host.yaml:954 CompleteAssignment, 1107 ReconcileAssignment | guards.rs:371 |
| 11 | missing | vision.md:22 "An empty assignment queue cannot establish goal completion" | host.yaml:682 SatisfyGoal, no guard | guards.rs:180 checks only "no unfinished assignment" and a non-empty receipt; zero assignments passes |
| 12 | contradicts | vision.md:22 "Completion requires verified results and a recorded acceptance decision" | host.yaml:1021–1060 UpdateGoal has no wrong_state and sets `satisfaction_receipt: ''` | a Satisfied goal loses its receipt (probe) |
| 13 | contradicts | epic:bootstrap "ambiguous publication reconciles before retry" | host.yaml:299–316 PublicationIntent: Prepared → Uncertain → Confirmed only | no recorded "not published" outcome, so retry is impossible (probe) |
| 14 | missing | README.md:91 "removes a cancelled goal with no assignment history" | host.yaml:725 DeleteGoal: Cancelled only | guards.rs:86; Goal owns Assignment (host.yaml:172) and the delete leaves the relation's fate undeclared |
| 15 | missing | story:workspace-host "preserve one registration per canonical workspace path" | host.yaml:454 RegisterWorkspace creates unconditionally | guards.rs:47 returns the first receipt |
| 16 | missing | story:workspace-directories "do not remove directories with active assignments" | host.yaml:435 RemoveWorkspaceDirectory | directories.rs:117 |
| 17 | missing | epic:bootstrap limits are positive counts ("three workers … three repairs and sixty minutes") | host.yaml:573, 1021 inputs typed Integer, no `when:` | guards.rs:105 |
| 18 | spec-only | none: no design statement gives the Supervisor goal or workspace lifecycle control | host.yaml:334–345 Supervisor may RegisterWorkspace, ArchiveWorkspace, RegisterRepository, Disable/EnableRepositoryRegistration, CreateGoal, StartGoal, PauseGoal, CancelGoal | the runtime sends none of them as Supervisor |
| 19 | spec-only | none: no design statement mentions archiving a workspace | host.yaml:23 Workspace.archive, 475 ArchiveWorkspace | archived paths can never be re-registered (guards.rs:57) |
| 20 | unclear | vision.md:30 "Pausing or cancelling stops further effects" | host.yaml:1205 RecordPlanningProgress applies in every Goal state | whether a progress record is an "effect" is not stated |

Counts: missing 16, contradicts 2, spec-only 2, unclear 1. Stale mapping: none (no mapping exists).

Every `missing` row is enforced today by host code outside the specification. ESS 0.53 can express rows 1, 3–11, 14–17 with `when:`, `when_subject:` and `when_related:` (ess/20–22); row 2 needs a model decision because the common directory belongs to RepositoryRegistration, two relations away from the claimed Assignment.

```findings
[
{"file":"ess/domains/host.yaml","line":644,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"One running goal per workspace is enforced only in guards.rs:120."},
{"file":"ess/domains/host.yaml","line":812,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"One active change per common Git directory, worker limit and attempt limit are enforced only in guards.rs:255-295; the common-directory rule spans Assignment to RepositoryRegistration."},
{"file":"ess/domains/host.yaml","line":751,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Goal revision binding of assignment evidence is stored but never compared in the specification (guards.rs:152, 235, 240)."},
{"file":"ess/domains/host.yaml","line":907,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Independent review context and candidate-bound tests and review are enforced only in guards.rs:307-326."},
{"file":"ess/domains/host.yaml","line":935,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Merge authority, publication intent before merge and confirmed receipt before completion are enforced only in guards.rs:327-385."},
{"file":"ess/domains/host.yaml","line":682,"category":"missing","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"SatisfyGoal preconditions are host-only; with zero assignments the host guard admits satisfaction (vision.md:22)."},
{"file":"ess/domains/host.yaml","line":1059,"category":"contradicts","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"UpdateGoal has no wrong_state outcome and clears satisfaction_receipt on Satisfied goals, contradicting vision.md:22."},
{"file":"ess/domains/host.yaml","line":299,"category":"contradicts","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"PublicationIntent has no recorded not-published outcome, contradicting epic:bootstrap 'ambiguous publication reconciles before retry'."},
{"file":"ess/domains/host.yaml","line":725,"category":"missing","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"DeleteGoal admits deletion with assignment history; README.md:91 rule lives in guards.rs:86."},
{"file":"ess/domains/host.yaml","line":454,"category":"missing","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Registration idempotency per canonical path, directory removal with active work and positive limits are host-only (guards.rs:47, 105; directories.rs:117)."},
{"file":"ess/domains/host.yaml","line":334,"category":"spec-only","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Eight Supervisor grants have no design statement and no runtime caller."},
{"file":"ess/domains/host.yaml","line":23,"category":"spec-only","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Workspace archive has no design statement; archived paths can never be re-registered."},
{"file":"ess/domains/host.yaml","line":1205,"category":"unclear","severity":"note","verdict":"NEEDS-CHANGE","origin":"undecided","message":"RecordPlanningProgress applies in every Goal state; whether progress is an effect under vision.md:30 is not stated."}
]
```
