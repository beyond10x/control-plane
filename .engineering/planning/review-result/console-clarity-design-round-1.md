---
format: aep.planning-md/3
id: review-result:console-clarity-design-round-1
kind: review-result
status: active
title: 'Plan critic (design), round 1: epic:console-clarity decomposition'
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
story:console-evidence-and-baseline — two things in one item: the readable evidence page and the visual baseline plus one-renderer cut. The body never says which renderer draws the evidence page, so it cannot be built without breaking `one_renderer` (delete the server page and its stylesheet) or deciding the renderer itself. Today `/goals/{id}/evidence` is a JSON endpoint, and the server-rendered pages draw all their CSS from `dashboard.css` — name the renderer in the Outcome (a Vue view over the existing JSON), or split the evidence page into its own story with no edge to goal-cards — .engineering/planning/story/console-evidence-and-baseline.md:29, .engineering/planning/story/console-evidence-and-baseline.md:39, crates/control-plane-app/src/dashboard.rs:12-34, crates/control-plane-app/src/web.rs:25-27
story:console-goal-cards — both this story and story:console-status-and-attention claim the goal's derived state (lifecycle plus planning phase). The set has no single owner for the derivation, and goal-cards has no acceptance on it. State that goal-cards consumes the derivation `console-status-and-attention` builds (`depends_on` already records the order), or move the claim wholly to one story — .engineering/planning/story/console-goal-cards.md:21, .engineering/planning/story/console-status-and-attention.md:22

What I read: 5 stories, `epic:console-clarity`, `story:bounded-progress-records`, `story:spec-history-gate`, `review-result:console-ux-review-2026-10-06`. I ran `aep plan artifact show` on each, then `relations`, `graph` and `validate`. I walked every edge in the graph, including those outside the set. There is no cycle. `validate` reports valid. I checked `web.rs` and `dashboard.rs` in the checkout.

What I could not establish:
- Whether the five-story order is a serialising chain: the roots are projection and component-tests, then three stories follow in sequence. Each edge has its shared-file reason in the epic's "Stories and order" table, so I did not report it. The cost is that evidence-and-baseline waits on goal-cards only for `frontend/src`. Splitting the evidence page out of that story would remove the wait for that half.
- Out of my lane, for the acceptance and parallel-safety critics: the "Cancelled card shows Stage Queued / Recorded running" evidence (review row 2) has no matching acceptance in `console-status-and-attention`. Review improvement 4 (translated blocker errors, ids behind a disclosure) is only partly covered, which is a scope question. The attention-row control link targets controls that `console-goal-cards` later relocates. `tests.rs` is used by the projection story while `dashboard::operations` tests there are touched by the one-renderer removal.

```findings
- file: .engineering/planning/story/console-evidence-and-baseline.md
  line: 29
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'two things in one item: the readable evidence page and the visual baseline plus one-renderer cut. The body never says which renderer draws the evidence page, so it cannot be built without breaking `one_renderer` (delete the server page and its stylesheet) or deciding the renderer itself. Today `/goals/{id}/evidence` is a JSON endpoint, and the server-rendered pages draw all their CSS from `dashboard.css`. Name the renderer in the Outcome (a Vue view over the existing JSON), or split the evidence page into its own story with no edge to goal-cards.'
- file: .engineering/planning/story/console-goal-cards.md
  line: 21
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'both this story and story:console-status-and-attention claim the goal''s derived state (lifecycle plus planning phase). The set has no single owner for the derivation, and goal-cards has no acceptance on it. State that goal-cards consumes the derivation `console-status-and-attention` builds (`depends_on` already records the order), or move the claim wholly to one story.'
```
