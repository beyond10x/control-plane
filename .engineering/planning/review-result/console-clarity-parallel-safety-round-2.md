---
format: aep.planning-md/3
id: review-result:console-clarity-parallel-safety-round-2
kind: review-result
status: active
title: 'Plan critic (parallel-safety), round 2: epic:console-clarity decomposition'
relations:
- reviews: epic:console-clarity
- reviews: story:console-projection
- reviews: story:console-component-tests
- reviews: story:console-status-and-attention
- reviews: story:console-goal-cards
- reviews: story:console-evidence-page
- reviews: story:console-visual-baseline
revision: 1
---
approve

What I read: 6 stories, plus the 7 other draft stories and epic:unattended-operation for collisions. I ran `aep plan artifact waves --kind story --status draft`, `show` on each of the 13 stories, `graph`, and `show review-result:console-clarity-parallel-safety-round-1`. I also ran `git grep` and `cat` on `crates/control-plane-xtask/src/frontend.rs`, `main.rs`, `Taskfile.yml`, `crates/control-plane-app/src/live.rs` and `frontend/`.

Round-1 findings:
- **Fleet collision, resolved.** story:console-projection now claims only `crates/control-plane-app/src/live.rs`. Its Scope section says "No runtime file changes". The `fleet.rs` overlap with story:publication-exit is gone, and `waves` no longer prints it.
- **App `tests.rs` collision, resolved.** story:console-projection now has `depends_on story:acceptance-traceability`, and its Scope section names the reason (test renames in `crates/control-plane-app`).
- **Edge to bounded-progress-records, in place.** The `live.rs` collision `waves` prints is ordered by `depends_on story:bounded-progress-records`, and the body says so.

The other collisions `waves` prints inside the set are all ordered by one chain: component-tests, then status-and-attention, goal-cards, evidence-page, visual-baseline. Projection feeds status-and-attention in the same chain. `dashboard.rs` (evidence-page and visual-baseline) and `frontend/src` and `frontend/dist` (every frontend story) are therefore not concurrent. Visual-baseline's `crates/control-plane-app/src/tests.rs` also sits downstream of acceptance-traceability through that chain. None of the other draft stories claims a `frontend/`, `dashboard.rs`, `web.rs` or app `tests.rs` surface.

What I could not establish:
- Surfaces placed: all 6, 0 unplaceable. The store tags 5 as inferred, but each body names its files, so I count them as cited. Projection, evidence-page, visual-baseline and component-tests (for `frontend.rs`) are fully cited. Goal-cards and status-and-attention cite App.vue and the other frontend files.
- story:acceptance-traceability and story:console-component-tests are both in wave 2 with no edge. Acceptance-traceability claims the `crates/control-plane-xtask/src` directory (inferred). Component-tests changes only `frontend.rs`, which has no tests to rename, and `main.rs` already wires `FrontendCheck` (`main.rs:36,51`). I found no shared file, so I am not reporting it. It would collide if acceptance-traceability edits `frontend.rs`.
- Out of my lane (design): component-tests still `depends_on story:spec-history-gate` although it needs only `frontend.rs`. The round-1 note stands.

```findings
[]
```
