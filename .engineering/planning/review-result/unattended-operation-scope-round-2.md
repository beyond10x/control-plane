---
format: aep.planning-md/3
id: review-result:unattended-operation-scope-round-2
kind: review-result
status: active
title: 'Plan critic (scope), round 2: epic:unattended-operation decomposition'
relations:
- reviews: epic:unattended-operation
- reviews: story:candidate-process-environment
- reviews: story:model-input-refusals
- reviews: story:bounded-progress-records
- reviews: story:spec-history-gate
- reviews: story:terminal-goal-edits
- reviews: story:publication-exit
- reviews: story:spec-owned-admission
revision: 1
---
needs-revision
story:model-input-refusals — the epic promises "a model action the host refuses for its input never ends a run", but the acceptance ends the attempt as Blocked on the fifth consecutive refusal, and the body names no narrowing of the promise (the Outcome says only "the run continues"). `BlockAssignment` is the state that needs an operator repair, so the body must name the cap as a narrowing, as story:candidate-process-environment does for its credential exception — .engineering/planning/story/model-input-refusals.md:41 (promise at .engineering/planning/epic/unattended-operation.md:21; Blocked path at crates/control-plane-runtime/src/fleet.rs:205-217)

What I read: 10 artifacts. These were the epic, the seven stories, review-result:unattended-operation-scope-round-1 and review-result:ess-design-review-2026-10-06. I ran `aep plan artifact show` on each and `aep plan artifact graph` once. I also read fleet.rs `block` and grepped host.yaml for Blocked. Promises extracted from the parent: 6 (five Outcome bullets plus the Constraint that every spec change runs behind the history gate). I traced 6 to an item. The round-1 findings have landed. The credential narrowing is now stated at story:candidate-process-environment, and the two stories that were claimed only by the epic's table row left the set. I found no gap and no double claim.

What I could not establish:
- Whether story:terminal-goal-edits (design-review row 12) and the Supervisor-grants outcome in story:spec-owned-admission (row 18) trace to a sentence of the epic. Both fit only loosely under "the admission rules the product depends on are declared in ESS". I left both out of the findings.
- Out of lane: story:bounded-progress-records bounds bytes per decision, but the epic title says "bounded state". Whether that bound is enough is for the acceptance critic.

```findings
- file: .engineering/planning/story/model-input-refusals.md
  line: 41
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the epic promises "a model action the host refuses for its input never ends a run" (.engineering/planning/epic/unattended-operation.md:21), but the acceptance ends the attempt as Blocked on the fifth consecutive refusal, and the body names no narrowing of the promise, so the cap must be stated as a named narrowing of the epic promise'
```
