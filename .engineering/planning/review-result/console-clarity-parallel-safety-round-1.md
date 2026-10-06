---
format: aep.planning-md/3
id: review-result:console-clarity-parallel-safety-round-1
kind: review-result
status: active
title: 'Plan critic (parallel-safety), round 1: epic:console-clarity decomposition'
relations:
- reviews: epic:console-clarity
- reviews: story:console-projection
- reviews: story:console-component-tests
- reviews: story:console-status-and-attention
- reviews: story:console-goal-cards
- reviews: story:console-evidence-and-baseline
revision: 1
---
needs-revision
story:console-projection — collides with story:publication-exit on `crates/control-plane-runtime/src/fleet.rs` (cited, both bodies). This story rewrites `Host::progress` (fleet.rs:79) and publication-exit changes `block()` and `reconcile_publications` (fleet.rs:205, 1835) so an unchanged intent stops appending a progress record every tick. No `depends_on` path joins them, neither body mentions the other, and `waves` prints the collision. Add an ordering edge that records fleet.rs as its reason, or split the surface — .engineering/planning/story/console-projection.md:43
story:console-projection — also lands on `crates/control-plane-app/src/tests.rs` (inferred). story:acceptance-traceability renames existing tests under `crates/` to match scenario names such as `planner_activity_survives_restart` and `workspace_activity_is_scoped`. The only candidates I found are `live_activity_is_durable_scoped_and_does_not_inline_model_receipts` and similar in that file. There is no edge between the two and neither body names the other. Record the shared file as an ordering reason, or have acceptance-traceability name the files it renames — .engineering/planning/story/console-projection.md:43

What I read: 5 console stories, epic:console-clarity, 7 other draft stories and epic:unattended-operation, via `aep plan artifact waves --kind story --status draft`, `show`, and `graph`. I also ran `git grep` on `fleet.rs`, `tests.rs` and the xtask sources.

Surfaces: 5 of 5 placed. 5 are cited by body text, though the store tags four of them inferred. 0 are unplaceable.

What I could not establish:
- Whether acceptance-traceability's renames reach `crates/control-plane-app/src/tests.rs`. Its scope says only "test files under crates/ whose names change", so the second finding rests on name matching.
- Everything else in the set is ordered by `depends_on`: console-component-tests, console-status-and-attention, console-goal-cards and console-evidence-and-baseline share `frontend/src` and `frontend/dist` along one chain. The projection story shares `crates/control-plane-app/src` with the evidence story through that same chain.
- Out of my lane: console-component-tests depends on spec-history-gate for `xtask/src`, but only `frontend.rs` is needed (`main.rs` already wires `FrontendCheck`). The design critic can judge whether that edge is necessary.

```findings
- file: .engineering/planning/story/console-projection.md
  line: 43
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'this story rewrites Host::progress in crates/control-plane-runtime/src/fleet.rs (cited) and story:publication-exit changes block() and reconcile_publications in the same file, with no depends_on path between them and neither body naming the other; add an ordering edge recording fleet.rs as its reason, or split the surface'
- file: .engineering/planning/story/console-projection.md
  line: 43
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'surface is inferred - this story adds tests to crates/control-plane-app/src/tests.rs while story:acceptance-traceability renames existing tests under crates/ (likely including that file) with no edge and no mention in either body; record the shared file as an ordering reason or have acceptance-traceability name the files it renames'
```
