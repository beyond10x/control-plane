---
format: aep.planning-md/3
id: review-result:console-clarity-design-round-2
kind: review-result
status: active
title: 'Plan critic (design), round 2: epic:console-clarity decomposition'
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
needs-revision
story:console-projection — three of its outcomes (`acceptance_recorded`, `merged_at`, and the planner activity kept separate from worker activity) are read by no story in the set; the body should name the consumer, or the other story's body should claim reading them, or the scenarios should be dropped; status-and-attention cites the "Recorded running" defect in its Evidence but has no scenario on it, and neither it nor goal-cards mentions the planner card at all — `grep -n 'acceptance_recorded|merged_at|planner' .engineering/planning/story/console-{status-and-attention,goal-cards}.md` returns nothing for the fields; .engineering/planning/story/console-projection.md:30-34

**Round-1 design findings**
- Evidence/baseline seam: fixed. `story:console-evidence-page` now states the renderer in its Outcome ("a Vue view over the existing evidence JSON … no new route renders HTML on the server"). `story:console-visual-baseline` owns removing the server-rendered markup and `dashboard.css`. The old story is archived, and both successors carry `supersedes` edges.
- Derived-state ownership: fixed. `story:console-status-and-attention` states it "owns the derived goal state". `story:console-goal-cards` says "this story consumes it and defines none". `depends_on` records the order.

**What I checked and did not flag**
- No cycle. I walked all the declared edges, including to artifacts outside the set (`bounded-progress-records`, `acceptance-traceability`, `spec-history-gate`, and the archived story). The set is a four-deep chain behind two roots. The epic's "Stories and order" table gives a shared-file reason for each edge.
- `console-evidence-page` serving the console at the evidence path does not collide with `console-visual-baseline` removing the server-rendered markup. Their ordering edge is recorded.

**What I read**
- 7 artifacts: the epic, the six stories, and the three round-1 reviews that touch design and scope.
- Commands: `aep plan artifact show` on each, then `relations`, `graph` and `validate`. `validate` reports valid.
- Code: `web.rs`, `dashboard.rs`, `lib.rs` routes, `frontend/src/main.js` and `App.vue`.

**What I could not establish**
- None for my lane.
- Unease, not a finding: `console-visual-baseline` holds the stylesheet and contrast work together with the one-renderer removal. They look separable, but the shared stylesheet dedupe ties them.
- Out of my lane (acceptance or scope): `console-evidence-page` says "keep the JSON at a stable URL", but the route `/goals/{id}/evidence` is the page after the change. The story does not name the new JSON URL.
- Out of my lane (acceptance or scope): the attention-row control links in `console-status-and-attention` target controls that `console-goal-cards` later moves. No scenario in goal-cards keeps those links working.

```findings
- file: .engineering/planning/story/console-projection.md
  line: 30
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'three of its outcomes (`acceptance_recorded`, `merged_at`, and the planner activity kept separate from worker activity) are read by no story in the set; the body should name the consumer, or the other story''s body should claim reading them, or the scenarios should be dropped; status-and-attention cites the "Recorded running" defect in its Evidence but has no scenario on it, and neither it nor goal-cards mentions the planner card at all'
```
