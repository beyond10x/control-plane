---
format: aep.planning-md/3
id: review-result:unattended-operation-acceptance-round-2
kind: review-result
status: active
title: 'Plan critic (acceptance), round 2: epic:unattended-operation decomposition'
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
story:publication-exit — `close_command_is_declared` still joins two independent outcomes in one bullet (`ess specify validate` passes with the new state and command, and `ess verify conform synthesize` emits scenarios for the close command that pass against the durable target), so one can pass while the other fails. Round 1 flagged this bullet for three outcomes. The "first" ordering and the acknowledgement clause are gone, but this "and" remains — .engineering/planning/story/publication-exit.md:47
story:bounded-progress-records — the Outcome promises "the console still shows the same activity", but the only check is the unnamed bullet "The existing observability and SSE tests stay green". It names no test and no fixture, and the existing tests in `crates/control-plane-app/src/live.rs` do not cover the history-bearing receipt that this story changes. A bounded record that dropped history entries would still pass — .engineering/planning/story/bounded-progress-records.md:41

What I read: all 7 ids, with `aep plan artifact show` on each. I also read the round-1 acceptance review-result, ran `aep plan artifact kinds` and `aep plan artifact lifecycle story`, and used `git grep` on the tree for `gofmt`, `transition-route-changed`, the existing test names, and `ContractStore` in `target.rs:95-125`.

Round-1 findings fixed (not repeated):
- model-input-refusals: the admission and execution cases are split, and the budget is now a stated number (fifth refusal).
- bounded-progress-records: the cap is stated (16 KiB) and the fixture is specified.
- spec-history-gate: the gate scenario is split into three.
- spec-owned-admission: `second_running_goal_is_refused_in_conformance` now names an observable refusal.
- publication-exit: the "first" ordering and the three-way join are gone, except for the remainder in the finding above.

What I could not establish:
- I did not run `ess verify diff`, so I did not check that it reports the exact id `entity/controlplane.host.Assignment/transition-route-changed/repair`. The verification-report names only the class, `transition-route-changed`, at `.engineering/planning/verification-report/ess-hardening-2026-10-06.md:23`.
- `host_failures_stay_fatal` rests on "existing" regressions that I only partly found by name. I found symlink, cancellation and storage tests, but nothing for `.git` confinement or stale authority. I did not count this as a finding.
- The `probe_*` tests are still not in the tree, so "the probe sequence" in terminal-goal-edits and publication-exit relies on the Evidence prose. This is unchanged from round 1.
- `supervisor_grants_are_least_privilege` names no mechanism for computing "the set of commands the runtime executes as Supervisor". I judged it checkable by grep and did not count it.
- Out of my lane: a possible conflict between terminal-goal-edits, which asks for conformance scenarios to pass against ContractStore, and spec-owned-admission, which says that target bypasses guards. terminal-goal-edits does not depend on spec-owned-admission. That is design or parallel-safety territory, and it did not set my verdict.

```findings
- file: .engineering/planning/story/publication-exit.md
  line: 47
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`close_command_is_declared` still joins two independent outcomes in one bullet (`ess specify validate` passes with the new state and command, and `ess verify conform synthesize` emits scenarios for the close command that pass against the durable target), so one can pass while the other fails'
- file: .engineering/planning/story/bounded-progress-records.md
  line: 41
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the Outcome promises "the console still shows the same activity", but the only check is "The existing observability and SSE tests stay green", which names no test and no fixture, so a bounded record that dropped history entries would still pass'
```
