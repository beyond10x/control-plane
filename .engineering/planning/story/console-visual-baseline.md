---
format: aep.planning-md/3
id: story:console-visual-baseline
kind: story
status: implemented
title: Visual baseline and one renderer
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-evidence-page
- supersedes: story:console-evidence-and-baseline
scope:
- confidence: cited
  path: crates/control-plane-app/src/dashboard.css
- confidence: cited
  path: crates/control-plane-app/src/dashboard.rs
- confidence: inferred
  path: crates/control-plane-app/src/lib.rs
- confidence: inferred
  path: crates/control-plane-app/src/live.rs
- confidence: cited
  path: crates/control-plane-app/src/tests.rs
- confidence: cited
  path: crates/control-plane-app/src/web.rs
- confidence: cited
  path: frontend/dist
- confidence: inferred
  path: frontend/index.html
- confidence: inferred
  path: frontend/src/Activity.vue
- confidence: cited
  path: frontend/src/App.vue
- confidence: inferred
  path: frontend/src/AttentionStrip.vue
- confidence: cited
  path: frontend/src/ConfirmButton.vue
- confidence: inferred
  path: frontend/src/Evidence.vue
- confidence: cited
  path: frontend/src/GoalCard.vue
- confidence: cited
  path: frontend/src/goalState.js
- confidence: inferred
  path: frontend/src/main.js
- confidence: inferred
  path: frontend/src/status.js
- confidence: cited
  path: frontend/src/style.css
revision: 29
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T06:29:21Z", actor: "human:timo", revision: 26, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-13T06:29:21Z", actor: "human:timo", revision: 27, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-13T07:44:56Z", actor: "human:timo", revision: 29, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Outcome

The console meets a visual baseline: text at least 12 px, contrast at least 4.5:1, one colour per derived state (idle, paused, planning, waiting, working, blocked, cancelled, merged, disconnected), danger styling for destructive buttons, and queue items as cards that keep the blocker reason visible at any width. One renderer remains: the Vue application starts from a snapshot embedded in the page, and the server-rendered console markup and its duplicated stylesheet are removed.

## Evidence

review-result:console-ux-review-2026-10-06 rows 7 and 8: secondary text 9-11 px at contrast 3.0-3.95 (frontend/src/style.css); every badge grey except Merged; at 390 px the queue table hides the reason column; the server page and Vue drifted (default limits 3/3/60 against 1/1/10) and the stylesheet is duplicated byte for byte (crates/control-plane-app/src/web.rs, crates/control-plane-app/src/dashboard.css).

## Acceptance

- `text_meets_contrast`: a test over the compiled stylesheet (`frontend/dist/index.css`, media queries included) finds no rule whose text colour is under 4.5:1 against its own background, or, where the rule sets no background, against both the card background `#fff` and the page background; a rule that sets only a background is measured against the text colour it inherits; rules inside every at-rule that holds style rules (`@media`, `@supports`, `@container`, `@layer`) count.
- `text_meets_minimum_size`: the same test finds no `font-size` and no `font:` shorthand under 12 px, media queries included.
- `each_derived_state_has_its_own_colour`: the goal badge for each derived goal state in `frontend/src/goalState.js` (starting, planning, executing, stalled, blocked, paused, cancelled, satisfied) resolves to its own background colour, merged assignments have their own badge colour, and the status header's states (idle, waiting, working, paused, blocked, disconnected) resolve to six distinct colours. One badge mapper serves every component.
- `destructive_buttons_are_styled_as_danger`: the trigger and the confirm button of cancel goal, delete goal and remove directory carry the danger class; the reversible toggles (disable/enable, grant/withhold merge authority) and every other button do not.
- `queue_items_keep_their_reason`: the queue renders each item as a card element containing its blocker reason as text; the Vue application renders no table, and no stylesheet rule hides the reason at any width.
- `page_embeds_a_snapshot_for_vue`: the HTML served for the overview and for a workspace page holds the console payload in `<script type="application/json" id="console-snapshot">` with `<` escaped, Vue mounts from it before its first SSE frame, and the page holds no server-rendered goal markup (no `class="operation"`, no objective text outside the JSON).
- `duplicate_stylesheet_is_removed`: crates/control-plane-app/src/dashboard.css no longer exists; the refused-form error page links `/index.css`.
- `served_page_links_only_the_bundled_stylesheet`: the HTML served for the overview, a workspace page and the error page links exactly one stylesheet, the bundled one. This already holds for the overview on main, so it is a guard, not a red-first test.
- `concurrent_workers_and_blocked_stopped_states_are_distinct` (named by story:operator-observability) keeps its name and its meaning, rewritten against the projection Vue renders.

This changes no ESS entity: the derived states are projections of the existing Goal and Assignment lifecycles in controlplane.host.

## Scope

Inferred: frontend/src/style.css, frontend/src/**, frontend/index.html, frontend/dist, crates/control-plane-app/src/web.rs, crates/control-plane-app/src/dashboard.rs, crates/control-plane-app/src/dashboard.css, crates/control-plane-app/src/tests.rs.
