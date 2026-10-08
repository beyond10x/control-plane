---
format: aep.planning-md/3
id: decision-blocker:admission-selectors-not-synthesized
kind: decision-blocker
status: cleared
title: Re-scope story:spec-owned-admission to the guards ESS 0.56.0 synthesizes, or park it?
relations:
- blocks: story:spec-owned-admission
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T03:28:43Z", actor: "human:timo", revision: 3}
---
## Question

ESS 0.56.0 validates the admission rules story:spec-owned-admission needs, but conformance synthesis cannot produce a scenario for a selector that compares a row with a subject field holding an owner identity or a related value. Re-scope the story to what synthesizes, or park it until ESS synthesizes those selectors?

## State

Implementor trials on 2026-10-08 against copies of `ess/` at d363a2e, ESS 0.56.0 (outputs kept under the unit tree's `.scratch/trial/`):

| trial | result |
|---|---|
| Assignment carries `common_dir` (stamped at QueueAssignment) and ClaimAssignment has the `repository-busy` selector | validates; synthesis 141 scenarios, 39 refusals (base 180, 0): "reads a subject field the steps leave undetermined"; ClaimAssignment/applied and every later assignment scenario lost |
| `common_dir` as a required QueueAssignment input | `applied` synthesizes; the busy branch: "its own row set is False … has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch" |
| StartGoal selector `state == Running and workspace_id == subject.workspace_id` (row 1) | 159 scenarios, 20 refusals; StartGoal/applied and workspace-busy lost; same under `ess/23` |
| two selectors on one command | synthesis: "reads more than one row set: a scenario arranges the rows of one selector per command in this cut" |
| selector beside `when_subject` or an identity-addressed guard | `ESS-COMMAND-009 … one command reads one kind of related row in this cut` |
| worker limit as `count: {gte: subject.max_workers}` | `invalid type: string "subject.max_workers", expected i64` |
| QueueAssignment guards addressed by input id (goal missing, goal not Running or stale revision, repository missing) | 184 scenarios, 0 refusals |
| the same plus "repository disabled" | `ESS-SYNTH-003 no candidate of the 2 tried satisfies … RepositoryRegistration stored related row` |

Routing the conformance target through the console's admission path on today's specification: 68 passed, 112 errored (101 "repository not found", 11 the SatisfyGoal receipt guard).

The design review's table marks 15 missing rows (1–11, 14–17), not 16.

## Options

| option | what happens | cost |
|---|---|---|
| A | Re-scope: declare the guards that synthesize (input-addressed QueueAssignment guards and the like); list rows 1–4 (second running goal, one change per common directory, worker limit, repository disabled) in the specification README's host facts with the refusals above; replace `second_running_goal_is_refused_in_conformance`; ask ESS for synthesis of subject-field selectors | the two central rules stay host code until ESS ships it |
| B | Park the story until an ESS release synthesizes those selectors | no admission work in this wave; the rules stay host code anyway |

Recommendation: A, with the ESS request either way.

## Decided

Option A, decided 2026-10-08 under the operator's delegated decision authority: story:spec-owned-admission is re-scoped to the guards ESS 0.56.0 synthesizes. The input-addressed QueueAssignment guards are declared (184 scenarios, 0 refusals in the trial). Rows 1-4 of the design review (second running goal per workspace, one active change per Git common directory, worker limit, repository disabled) are listed as host facts in the specification README, quoting the ESS 0.56.0 refusals. The acceptance `second_running_goal_is_refused_in_conformance` is replaced by one that checks what synthesizes. Synthesis of selectors that compare a row with a subject field has been requested from ESS; the trial outputs stay in the unit tree's `.scratch/trial/` until ESS has copied them.
