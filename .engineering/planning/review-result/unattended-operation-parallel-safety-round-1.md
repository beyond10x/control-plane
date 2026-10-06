---
format: aep.planning-md/3
id: review-result:unattended-operation-parallel-safety-round-1
kind: review-result
status: active
title: 'Plan critic (parallel-safety), round 1: epic:unattended-operation decomposition'
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
story:bounded-progress-records — it shares `crates/control-plane-runtime/src/fleet.rs` (`reconcile_publications`, cited in both bodies) with story:publication-exit, `crates/control-plane-core/src/lib.rs` (cited in both) with story:spec-owned-admission, and `ess/domains/host.yaml` with story:terminal-goal-edits, publication-exit and spec-owned-admission. The `depends_on` chain bounded-progress-records → model-input-refusals → candidate-process-environment never meets the chain terminal-goal-edits → publication-exit → spec-owned-admission, and no body names the overlap. Fix it with an ordering edge that records the shared file as its reason, or by splitting the surface — .engineering/planning/story/bounded-progress-records.md:49
story:publication-exit — it edits `crates/control-plane-runtime/src/fleet.rs` (cited in all three bodies) and `crates/control-plane-runtime/tests/fleet.rs` (inferred on both sides). The first overlaps with story:model-input-refusals and story:candidate-process-environment. publication-exit's only `depends_on` is terminal-goal-edits, which has no path to either story, and its body does not mention them. Fix it with an ordering edge or by splitting the surface — .engineering/planning/story/publication-exit.md:47
story:conformance-evidence-record — it changes `crates/control-plane-xtask/src/target.rs` (cited in both bodies), which story:spec-owned-admission also changes. conformance-evidence-record has no `depends_on` edge at all (only the blocker's `blocks`), and neither body mentions the other. Fix it with an ordering edge or by splitting the surface — .engineering/planning/story/conformance-evidence-record.md:30

What I read: 10 store artifacts (the nine stories plus the blocker in the graph) via `aep plan artifact show` on each story, `aep plan artifact graph` and `aep plan artifact waves --kind story --status draft`. I also checked the xtask, core and runtime file layout and the function locations in `fleet.rs`. Surfaces: 9 stories cited, 0 inferred-only, 0 unplaced. `waves` reports 0 unassessed.

What I could not establish:
- **Overlap I am not reporting:** `waves` reports no collision for conformance-evidence-record against spec-history-gate or acceptance-traceability. `crates/control-plane-xtask/src/` (inferred in those two bodies) contains `target.rs`, and all three plausibly edit `main.rs`. I cannot tell from the bodies whether they do, so it is not a finding.
- **Overlaps already ordered by an edge:** the collisions that already have a direct or transitive `depends_on` path are not findings, for example `spec-history-gate` with `acceptance-traceability` on `Taskfile.yml`. The edges carry no recorded reason.
- **Out of my lane:** `crates/control-plane-core/tests` (in spec-history-gate's scope) does not exist yet, so it is a created file, not a collision. The wave ordering that `waves` computes is the operator's call, not mine.

```findings
- file: .engineering/planning/story/bounded-progress-records.md
  line: 49
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'shares fleet.rs (reconcile_publications) with story:publication-exit, core/src/lib.rs with story:spec-owned-admission and ess/domains/host.yaml (inferred on this side) with story:terminal-goal-edits, publication-exit and spec-owned-admission, with no depends_on path between the two chains and no body naming it; add an ordering edge recording the file or split the surface'
- file: .engineering/planning/story/publication-exit.md
  line: 47
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'edits fleet.rs (cited in all three bodies) and tests/fleet.rs (inferred) that story:model-input-refusals and story:candidate-process-environment also edit, with no depends_on path to either and no mention in the body; add an ordering edge recording the file or split the surface'
- file: .engineering/planning/story/conformance-evidence-record.md
  line: 30
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'changes crates/control-plane-xtask/src/target.rs (cited in both bodies) that story:spec-owned-admission also changes, and has no depends_on edge or body mention; add an ordering edge recording the file or split the surface'
```
