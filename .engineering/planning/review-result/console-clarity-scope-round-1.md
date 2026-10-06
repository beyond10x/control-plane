---
format: aep.planning-md/3
id: review-result:console-clarity-scope-round-1
kind: review-result
status: active
title: 'Plan critic (scope), round 1: epic:console-clarity decomposition'
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
epic:console-clarity — the promise "Raw identifiers, receipts and JSON stay one click away instead of in the main view" is only partly claimed: JSON (evidence download) and receipts (projection) are claimed, but no item moves raw ids and hashes out of the main view, such as the story id, candidate and run ids, and worktree path on queue rows and goal cards. Most natural owner is story:console-evidence-and-baseline, or story:console-goal-cards. — .engineering/planning/epic/console-clarity.md:15
story:console-goal-cards — the epic's "destructive actions act on one click" is narrowed to cancel and delete: the evidence lists "Remove directory and Disable" as one-click actions, but outcome and `destructive_actions_confirm_with_the_goal_name` cover only cancel and delete, and nothing records dropping the other two — .engineering/planning/story/console-goal-cards.md:30 (epic line 19)
story:console-goal-cards — "one derived state" in the outcome claims the same promise as story:console-status-and-attention's "A goal's badge is derived from its lifecycle state and planning phase together", so both will be marked done; the card should consume the status story's derivation rather than claim it — .engineering/planning/story/console-goal-cards.md:21 (status story line 22)
story:console-projection — scope cites `fleet.rs` (`Host::progress`) and `supervisor.rs`, the runtime's record-writing path, but the epic excludes "any change to what the runtime does" and says "the projection only selects and derives"; the body should derive planner activity and entry ids in `compact` from the existing `activity` history, or name the recorded-shape change as a deliberate exception — .engineering/planning/story/console-projection.md:18 (epic lines 33 and 37)
story:console-evidence-and-baseline — "one renderer remains: Vue starts from a snapshot embedded in the page, and the duplicated server-rendered page and stylesheet are removed" traces to no sentence in the epic's outcome, why-now or constraints; either the epic gains the promise or the story drops it. — .engineering/planning/story/console-evidence-and-baseline.md:29

What I read: 7 artifacts (the epic, the 5 stories and the review result), plus `docs/vision.md`, `frontend/src/App.vue`, `live.rs` and `fleet.rs`. Commands: `aep plan artifact show` on each, and `aep plan artifact graph`. I extracted 14 promises from the epic (5 from the outcome, 8 from why-now, 1 from constraints). I traced 12 fully to an item and 2 partly: raw identifiers, and the destructive actions.

What I could not establish:
- Whether the planner-only activity needs a write-path change in `fleet.rs`. The story does not say, and I only inferred from `engine.rs:223` and the `activity` history that it can be derived.
- Out of my lane: `goal_controls_are_on_the_card` does not list merge authority, although the outcome does. That belongs to the acceptance critic.
- Out of my lane: both `story:console-projection` and `story:bounded-progress-records` edit `Host::progress`. That belongs to the parallel-safety critic.

```findings
- file: .engineering/planning/epic/console-clarity.md
  line: 15
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the promise "Raw identifiers, receipts and JSON stay one click away instead of in the main view" is only partly claimed; JSON (evidence download) and receipts (projection) are claimed, but no item moves raw ids and hashes out of the main view (most natural owner story:console-evidence-and-baseline or story:console-goal-cards)'
- file: .engineering/planning/story/console-goal-cards.md
  line: 30
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the epic''s "destructive actions act on one click" is narrowed to cancel and delete; the evidence names Remove directory and Disable too, and nothing records dropping them'
- file: .engineering/planning/story/console-goal-cards.md
  line: 21
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"one derived state" in the outcome claims the same promise as story:console-status-and-attention''s goal badge derived from lifecycle state and planning phase together, so both will be marked done'
- file: .engineering/planning/story/console-projection.md
  line: 18
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'scope cites fleet.rs Host::progress and supervisor.rs, the runtime''s record-writing path, while the epic excludes any change to what the runtime does and says the projection only selects and derives; the body should derive from existing records or name the exception'
- file: .engineering/planning/story/console-evidence-and-baseline.md
  line: 29
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"one renderer" (Vue from an embedded snapshot, duplicated server-rendered page and stylesheet removed) traces to no sentence in the epic''s outcome, why-now or constraints'
```
