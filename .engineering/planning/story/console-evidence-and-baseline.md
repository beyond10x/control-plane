---
format: aep.planning-md/3
id: story:console-evidence-and-baseline
kind: story
status: archived
title: Readable evidence, visual baseline and one renderer
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-goal-cards
scope:
- confidence: inferred
  path: crates/control-plane-app/src/dashboard.css
- confidence: inferred
  path: crates/control-plane-app/src/dashboard.rs
- confidence: inferred
  path: crates/control-plane-app/src/web.rs
- confidence: inferred
  path: frontend/dist
- confidence: inferred
  path: frontend/index.html
- confidence: inferred
  path: frontend/src
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-06T03:12:10Z", actor: "human:timo", revision: 3, executor: "agent:claude-wave-coordinator"}
---
## Outcome

"Inspect evidence" shows a readable page (verdict, timeline, checks, review, published commit) with the raw JSON as a download; the console meets a visual baseline (text at least 12 px, contrast at least 4.5:1, one colour per state, danger styling for destructive buttons, cards instead of the queue table on narrow screens); and one renderer remains: Vue starts from a snapshot embedded in the page, and the duplicated server-rendered page and stylesheet are removed.

## Evidence

review-result:console-ux-review-2026-10-06 rows 5, 7 and 8: 74,820 bytes of escaped JSON on the evidence page (crates/control-plane-app/src/dashboard.rs); secondary text 9-11 px at contrast 3.0-3.95 (frontend/src/style.css); the server page and Vue drifted (default limits 3/3/60 against 1/1/10) and the stylesheet is duplicated byte for byte (crates/control-plane-app/src/web.rs, dashboard.css).

## Acceptance

- `evidence_page_summarises_before_raw`: the evidence page for a merged assignment renders its verdict, check result, reviewer verdict and published commit before a download link to the raw JSON.
- `text_meets_contrast_and_size`: a test over the compiled stylesheet finds no text colour pair under 4.5:1 and no font size under 12 px.
- `one_renderer`: the HTML served for a workspace page contains the embedded snapshot and no server-rendered goal markup; the duplicate stylesheet is gone.
- `narrow_queue_shows_reasons`: at 390 px the queue renders cards that include the blocker reason.

## Scope

Inferred: crates/control-plane-app/src/dashboard.rs, crates/control-plane-app/src/web.rs, crates/control-plane-app/src/dashboard.css, frontend/src/**, frontend/index.html, frontend/dist.
