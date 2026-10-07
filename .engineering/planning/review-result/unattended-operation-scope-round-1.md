---
format: aep.planning-md/3
id: review-result:unattended-operation-scope-round-1
kind: review-result
status: active
title: 'Plan critic (scope), round 1: epic:unattended-operation decomposition'
relations:
- reviews: epic:unattended-operation
- reviews: story:candidate-process-environment
- reviews: story:model-input-refusals
- reviews: story:bounded-progress-records
- reviews: story:spec-history-gate
- reviews: story:terminal-goal-edits
- reviews: story:publication-exit
- reviews: story:spec-owned-admission
- reviews: story:acceptance-traceability
- reviews: story:conformance-evidence-record
revision: 1
---
needs-revision
story:candidate-process-environment — "model-written code never receives the service's credentials" is promised without exception, but the story hands credentials to the publish command and its own evidence says the publish command executes model-written code, so the body must state this as a named narrowing of the parent's promise — .engineering/planning/story/candidate-process-environment.md:28 (and :33); promise at .engineering/planning/epic/unattended-operation.md:18
story:acceptance-traceability — no sentence of the epic's Outcome or Why now claims acceptance-name-to-test traceability; only the Stories table row at .engineering/planning/epic/unattended-operation.md:41 names it, so the epic Outcome must state it or the story must leave the epic — .engineering/planning/story/acceptance-traceability.md:21
story:conformance-evidence-record — recording the conformance report as AEP evidence and moving the specification to conforming is not among the Outcome's five promises, and the story is blocked by an external dependency the epic does not mention in its outcome, so the epic must state it or the story must leave — .engineering/planning/story/conformance-evidence-record.md:18

What I read: 15 artifacts (epic, nine stories, the blocker, four evidence records). Commands: `aep plan artifact show` on each, plus `aep plan artifact graph`. I also read `crates/control-plane-runtime/src/fleet.rs:225-265` and the store files for line numbers.

Promises extracted from the parent: 6. I traced 6 to an item.

| Promise | Claimed by |
|---|---|
| credentials (Outcome) | story:candidate-process-environment |
| refusals (Outcome) | story:model-input-refusals |
| bounded store and restart (Outcome) | story:bounded-progress-records |
| failed publication (Outcome) | story:publication-exit |
| admission in ESS and conformance (Outcome) | story:spec-owned-admission |
| history gate (Constraints) | story:spec-history-gate |

The twelve-member refusal class and the 16 missing design-review rows are fully claimed. I found no gap and no two items claiming one outcome.

What I could not establish:
- Whether the publish command in fact runs model-written code. The evidence says so (`control-plane-review-2026-10-06` finding 1). `fleet.rs:237-259` shows only that it is a configured command run in the candidate path.
- Out of lane:
  - `story:bounded-progress-records` bounds bytes per event but not total store size. Whether that meets "state store ... stay bounded" is for the acceptance critic.
  - `story:publication-exit` and `story:bounded-progress-records` both touch `reconcile_publications` in `fleet.rs`. That is for the parallel-safety critic.
  - `story:spec-owned-admission` acceptance `supervisor_grants_are_least_privilege` is a marginal fit with "admission rules". I left it out of the findings.

```findings
- file: .engineering/planning/story/candidate-process-environment.md
  line: 28
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"model-written code never receives the service''s credentials" is promised without exception, but the story hands credentials to the publish command and its own evidence says the publish command executes model-written code, so the body must state this as a named narrowing of the parent''s promise'
- file: .engineering/planning/story/acceptance-traceability.md
  line: 21
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'no sentence of the epic''s Outcome or Why now claims acceptance-name-to-test traceability; only the Stories table row at .engineering/planning/epic/unattended-operation.md:41 names it, so the epic Outcome must state it or the story must leave the epic'
- file: .engineering/planning/story/conformance-evidence-record.md
  line: 18
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'recording the conformance report as AEP evidence and moving the specification to conforming is not among the Outcome''s five promises, and the story is blocked by an external dependency the epic does not mention in its outcome, so the epic must state it or the story must leave'
```
