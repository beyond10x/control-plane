---
format: aep.planning-md/3
id: review-result:console-clarity-acceptance-round-2
kind: review-result
status: active
title: 'Plan critic (acceptance), round 2: epic:console-clarity decomposition'
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
story:console-projection — `planner_card_receives_planner_activity_only` joins two independent outcomes (the planner activity is the planner event, and the worker event appears under that assignment), so one can pass while the other fails; split it into one scenario each — .engineering/planning/story/console-projection.md:30
story:console-projection — the Outcome promises "a stable id per activity entry", but `activity_entries_have_distinct_ids` only checks that two same-second entries differ, so an id that changes on every SSE frame passes; add a scenario that the same entry keeps its id across two projections — .engineering/planning/story/console-projection.md:35
story:console-status-and-attention — the Outcome promises each attention row carries "its reason in plain words, its age", but `attention_lists_each_blocked_item` observes only the goal, repository and resolving control, so reason text and age on assignment and publication rows can ship unobserved; add them to the scenario — .engineering/planning/story/console-status-and-attention.md:38
story:console-goal-cards — the Outcome says each confirmation names "its subject and the consequence", but `destructive_actions_confirm_with_their_subject` observes only the named goal, directory or repository, so a confirmation with no consequence text passes; add the consequence to the scenario — .engineering/planning/story/console-goal-cards.md:33
story:console-evidence-page — the scope changes `dashboard.rs` so the evidence path serves the console and the JSON moves to "a stable URL", yet no scenario observes it: `evidence_download_is_the_raw_json` ("the unchanged JSON the endpoint serves today") names no URL, so it reads the same whether or not "Inspect evidence" now opens the view instead of escaped JSON; name the before and after for the evidence path and the JSON URL, and add a server test for them — .engineering/planning/story/console-evidence-page.md:33
story:console-visual-baseline — `text_meets_contrast_and_size` joins two independent outcomes (no colour pair under 4.5:1, and no font size under 12 px), so one can pass while the other fails; split it into one scenario each — .engineering/planning/story/console-visual-baseline.md:40
story:console-visual-baseline — the Outcome says "one colour per derived state", but `each_derived_state_has_its_own_colour` lists idle, paused, waiting, working, blocked, cancelled and merged, which omits the `planning` and `disconnected` states that story:console-status-and-attention derives, so those can share a colour unobserved; add them or narrow the Outcome — .engineering/planning/story/console-visual-baseline.md:41
story:console-visual-baseline — `duplicate_stylesheet_is_removed` joins two independent outcomes (`dashboard.css` no longer exists, and the served page links only the bundled stylesheet), so one can pass while the other fails; split it into one scenario each — .engineering/planning/story/console-visual-baseline.md:45

What I read: all 6 ids (story:console-projection, story:console-component-tests, story:console-status-and-attention, story:console-goal-cards, story:console-evidence-page, story:console-visual-baseline), in full with `aep plan artifact show`. I also read `aep plan artifact kinds`, `aep plan artifact lifecycle story`, `review-result:console-clarity-acceptance-round-1`, `crates/control-plane-xtask/src/frontend.rs`, `crates/control-plane-app/src/dashboard.rs` and `frontend/package.json`.

Round-1 fixes that landed:
- Projection: acceptance and merge are split into two scenarios.
- Component tests: the plant is now the named `failing_component_test_fails_the_gate`.
- Status: header scenarios now cover paused, working, blocked and last-activity age.
- Cards: scenarios now cover title, derived state, step and elapsed time, merge authority, and all four destructive actions.
- Baseline: colour and danger scenarios were added, and the narrow-width check is now a markup assertion.

What I could not establish:
- Whether the chosen frontend test runner can read the compiled stylesheet well enough to compute contrast pairs. No runner is chosen yet and `package.json` has only `vite build`.
- Whether the evidence JSON carries a "check result" or "reviewer verdict" for the evidence view to render. `dashboard.rs:12-35` returns only goal, assignments and publications. That is feasibility, outside my lane, and it did not set my verdict.
- Out of my lane (scope and coupling): story:console-evidence-page and story:console-visual-baseline both edit `dashboard.rs`, and the baseline story removes the server page it depends on. This belongs to the scope, design and parallel-safety critics.

```findings
[
  {"file": ".engineering/planning/story/console-projection.md", "line": 30, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "`planner_card_receives_planner_activity_only` joins two independent outcomes (the planner activity is the planner event, and the worker event appears under that assignment), so one can pass while the other fails; split it into one scenario each"},
  {"file": ".engineering/planning/story/console-projection.md", "line": 35, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the Outcome promises \"a stable id per activity entry\", but `activity_entries_have_distinct_ids` only checks that two same-second entries differ, so an id that changes on every SSE frame passes; add a scenario that the same entry keeps its id across two projections"},
  {"file": ".engineering/planning/story/console-status-and-attention.md", "line": 38, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the Outcome promises each attention row carries \"its reason in plain words, its age\", but `attention_lists_each_blocked_item` observes only the goal, repository and resolving control, so reason text and age on assignment and publication rows can ship unobserved; add them to the scenario"},
  {"file": ".engineering/planning/story/console-goal-cards.md", "line": 33, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the Outcome says each confirmation names \"its subject and the consequence\", but `destructive_actions_confirm_with_their_subject` observes only the named goal, directory or repository, so a confirmation with no consequence text passes; add the consequence to the scenario"},
  {"file": ".engineering/planning/story/console-evidence-page.md", "line": 33, "category": "acceptance", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "the scope changes `dashboard.rs` so the evidence path serves the console and the JSON moves to \"a stable URL\", yet no scenario observes it: `evidence_download_is_the_raw_json` (\"the unchanged JSON the endpoint serves today\") names no URL, so it reads the same whether or not \"Inspect evidence\" now opens the view instead of escaped JSON; name the before and after for the evidence path and the JSON URL, and add a server test for them"},
  {"file": ".engineering/planning/story/console-visual-baseline.md", "line": 40, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "`text_meets_contrast_and_size` joins two independent outcomes (no colour pair under 4.5:1, and no font size under 12 px), so one can pass while the other fails; split it into one scenario each"},
  {"file": ".engineering/planning/story/console-visual-baseline.md", "line": 41, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the Outcome says \"one colour per derived state\", but `each_derived_state_has_its_own_colour` lists idle, paused, waiting, working, blocked, cancelled and merged, which omits the `planning` and `disconnected` states that story:console-status-and-attention derives, so those can share a colour unobserved; add them or narrow the Outcome"},
  {"file": ".engineering/planning/story/console-visual-baseline.md", "line": 45, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "`duplicate_stylesheet_is_removed` joins two independent outcomes (`dashboard.css` no longer exists, and the served page links only the bundled stylesheet), so one can pass while the other fails; split it into one scenario each"}
]
```
