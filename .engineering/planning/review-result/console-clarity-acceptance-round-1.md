---
format: aep.planning-md/3
id: review-result:console-clarity-acceptance-round-1
kind: review-result
status: active
title: 'Plan critic (acceptance), round 1: epic:console-clarity decomposition'
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
story:console-projection — `recorded_acceptance_and_merge_reach_the_browser` joins two independent outcomes (`acceptance_recorded: true` and `merged_at`), so one can pass while the other fails; split it into one scenario each — .engineering/planning/story/console-projection.md:38
story:console-component-tests — the third bullet, "A planted failing assertion makes `frontend-check` exit non-zero.", is an unnamed scenario that restates the first one ("fails when a test fails"); it cannot be re-checked once the plant is removed, so name it as a scenario or fold it into `component_tests_run_in_the_gate` — .engineering/planning/story/console-component-tests.md:34
story:console-status-and-attention — the Outcome promises the header states paused, working and blocked and "the age of the last real activity", but the only header scenarios are disconnected, waiting and idle, so those states and the age can ship unobserved; add scenarios for them — .engineering/planning/story/console-status-and-attention.md:28
story:console-goal-cards — the Outcome promises the card title "workspace · repository · short goal", one derived state, and the current step with elapsed time, but no scenario observes any of them; add scenarios — .engineering/planning/story/console-goal-cards.md:27
story:console-goal-cards — `goal_controls_are_on_the_card` lists start/pause/cancel/edit, so merge authority in the card header, which the Outcome promises, is never observed; add it to the scenario — .engineering/planning/story/console-goal-cards.md:29
story:console-goal-cards — the Outcome says "Destructive actions ask for confirmation" and the Evidence names Remove directory and Disable as one-click, but `destructive_actions_confirm_with_the_goal_name` covers only cancel and delete; add the others or narrow the Outcome — .engineering/planning/story/console-goal-cards.md:30
story:console-evidence-and-baseline — the Outcome's "one colour per state" and "danger styling for destructive buttons" have no scenario (`text_meets_contrast_and_size` covers only size and contrast), so they cannot be told done; add observable scenarios — .engineering/planning/story/console-evidence-and-baseline.md:29
story:console-evidence-and-baseline — `one_renderer` bundles two outcomes (embedded snapshot with no server-rendered goal markup, and "the duplicate stylesheet is gone") in one sentence; split them — .engineering/planning/story/console-evidence-and-baseline.md:39
story:console-evidence-and-baseline — `narrow_queue_shows_reasons` ("at 390 px the queue renders cards") has no checking mechanism: the only test tooling in the set is the component-test story, which has no browser, and the review says browser checks are unsettled (`review-result:console-ux-review-2026-10-06` "Not settled"; vision.md:40); name the harness or state the check as a stylesheet or markup assertion — .engineering/planning/story/console-evidence-and-baseline.md:40

What I read: all 5 stories, in full, with `aep plan artifact show` on each id. I also read `aep plan artifact kinds`, `aep plan artifact lifecycle story`, `review-result:console-ux-review-2026-10-06`, `crates/control-plane-xtask/src/frontend.rs`, `frontend/package.json`, `docs/vision.md` and `Taskfile.yml`. The `frontend-check` command exists in `Taskfile.yml:10` and `crates/control-plane-xtask/src/main.rs:36`.

What I could not establish: whether the frontend test runner the component-tests story will pick can evaluate CSS media queries. Nothing is chosen yet, and `package.json` has only `vite build`. Out of my lane: `story:console-evidence-and-baseline` removes `dashboard.rs` and the server page while still serving the evidence page from `dashboard.rs`, which is scope or coupling for `plan-critic-scope` and `plan-critic-design`. It did not set my verdict.

```findings
- file: .engineering/planning/story/console-projection.md
  line: 38
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`recorded_acceptance_and_merge_reach_the_browser` joins two independent outcomes (`acceptance_recorded: true` and `merged_at`), so one can pass while the other fails; split it into one scenario each'
- file: .engineering/planning/story/console-component-tests.md
  line: 34
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the third bullet, "A planted failing assertion makes `frontend-check` exit non-zero.", is an unnamed scenario that restates the first one ("fails when a test fails"); it cannot be re-checked once the plant is removed, so name it as a scenario or fold it into `component_tests_run_in_the_gate`'
- file: .engineering/planning/story/console-status-and-attention.md
  line: 28
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the Outcome promises the header states paused, working and blocked and "the age of the last real activity", but the only header scenarios are disconnected, waiting and idle, so those states and the age can ship unobserved; add scenarios for them'
- file: .engineering/planning/story/console-goal-cards.md
  line: 27
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the Outcome promises the card title "workspace · repository · short goal", one derived state, and the current step with elapsed time, but no scenario observes any of them; add scenarios'
- file: .engineering/planning/story/console-goal-cards.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`goal_controls_are_on_the_card` lists start/pause/cancel/edit, so merge authority in the card header, which the Outcome promises, is never observed; add it to the scenario'
- file: .engineering/planning/story/console-goal-cards.md
  line: 30
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the Outcome says "Destructive actions ask for confirmation" and the Evidence names Remove directory and Disable as one-click, but `destructive_actions_confirm_with_the_goal_name` covers only cancel and delete; add the others or narrow the Outcome'
- file: .engineering/planning/story/console-evidence-and-baseline.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the Outcome''s "one colour per state" and "danger styling for destructive buttons" have no scenario (`text_meets_contrast_and_size` covers only size and contrast), so they cannot be told done; add observable scenarios'
- file: .engineering/planning/story/console-evidence-and-baseline.md
  line: 39
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`one_renderer` bundles two outcomes (embedded snapshot with no server-rendered goal markup, and "the duplicate stylesheet is gone") in one sentence; split them'
- file: .engineering/planning/story/console-evidence-and-baseline.md
  line: 40
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '`narrow_queue_shows_reasons` ("at 390 px the queue renders cards") has no checking mechanism, because the only test tooling in the set is the component-test story, which has no browser, and the review says browser checks are unsettled; name the harness or state the check as a stylesheet or markup assertion'
```
