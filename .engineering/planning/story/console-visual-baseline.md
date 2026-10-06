---
format: aep.planning-md/3
id: story:console-visual-baseline
kind: story
status: draft
title: Visual baseline and one renderer
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-evidence-page
- supersedes: story:console-evidence-and-baseline
scope:
- confidence: inferred
  path: crates/control-plane-app/src/dashboard.css
- confidence: inferred
  path: crates/control-plane-app/src/dashboard.rs
- confidence: inferred
  path: crates/control-plane-app/src/tests.rs
- confidence: inferred
  path: crates/control-plane-app/src/web.rs
- confidence: inferred
  path: frontend/dist
- confidence: inferred
  path: frontend/index.html
- confidence: inferred
  path: frontend/src
revision: 3
---
## Outcome

The console meets a visual baseline: text at least 12 px, contrast at least 4.5:1, one colour per derived state (idle, paused, planning, waiting, working, blocked, cancelled, merged, disconnected), danger styling for destructive buttons, and queue items as cards that keep the blocker reason visible at any width. One renderer remains: the Vue application starts from a snapshot embedded in the page, and the server-rendered console markup and its duplicated stylesheet are removed.

## Evidence

review-result:console-ux-review-2026-10-06 rows 7 and 8: secondary text 9-11 px at contrast 3.0-3.95 (frontend/src/style.css); every badge grey except Merged; at 390 px the queue table hides the reason column; the server page and Vue drifted (default limits 3/3/60 against 1/1/10) and the stylesheet is duplicated byte for byte (crates/control-plane-app/src/web.rs, crates/control-plane-app/src/dashboard.css).

## Acceptance

- `text_meets_contrast`: a test over the compiled stylesheet finds no text and background colour pair under 4.5:1.
- `text_meets_minimum_size`: the same test finds no font size under 12 px.
- `each_derived_state_has_its_own_colour`: the badge classes for idle, paused, planning, waiting, working, blocked, cancelled, merged and disconnected resolve to nine distinct colours.
- `destructive_buttons_are_styled_as_danger`: cancel, delete, remove and disable buttons carry the danger class and no other button does.
- `queue_items_keep_their_reason`: the queue renders each item as a card element containing its blocker reason; the markup contains no table.
- `page_embeds_a_snapshot_for_vue`: the HTML served for a workspace page contains the embedded snapshot and no server-rendered goal markup.
- `duplicate_stylesheet_is_removed`: crates/control-plane-app/src/dashboard.css no longer exists in the repository.
- `served_page_links_only_the_bundled_stylesheet`: the HTML served for the overview links exactly one stylesheet, the bundled one.

## Scope

Inferred: frontend/src/style.css, frontend/src/**, frontend/index.html, frontend/dist, crates/control-plane-app/src/web.rs, crates/control-plane-app/src/dashboard.rs, crates/control-plane-app/src/dashboard.css, crates/control-plane-app/src/tests.rs.
