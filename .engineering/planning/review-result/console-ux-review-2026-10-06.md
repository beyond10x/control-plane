---
format: aep.planning-md/3
id: review-result:console-ux-review-2026-10-06
kind: review-result
status: active
title: 'Operator console UI/UX: two independent reviews (code and visual), combined'
relations:
- reviews: story:operator-observability
- reviews: story:operator-console
- reviews: epic:bootstrap
- reviews: vision:autonomous-engineering
revision: 1
---
# Operator console UI/UX review, 2026-10-06

Two independent reviewers: one read the frontend and projection code at f510e7f (frontend/src, crates/control-plane-app/src/{web,dashboard,live}.rs) without running anything; one captured 21 screenshots of the running console on 127.0.0.1:8788 with GET requests only (overview, three workspaces with goals, the empty Go CLI workspace, /live, an evidence page, 1440 and 390 px widths, a simulated disconnect). All three goals were Cancelled at capture time, so a running goal's live view, model waits and its controls were not seen. Screenshots stay outside the repository.

Verdict of both: the console fails the vision's 5-second test (docs/vision.md:5-12). It shows committed rows; it does not tell the operator what is happening, what needs them or what to do.

## Where the two reviews agree

1. **No honest system state.** With the live stream down, tiles still read "00" and lists say "No assignments recorded"; the only signal is a 10 px line (visual: overview-1440-sse-blocked). No sentence says idle, paused, waiting, working, blocked or disconnected; "Need attention" reads 00 beside two failed assignments (visual: overview-1440). Code: App.vue:15,17,58,62-66 count only committed rows, and live.rs:39 strips the server clock.
2. **Contradictory goal state.** A Running goal whose planning phase is Blocked gets the green badge (App.vue:65-66, dashboard.rs:225); one card shows Cancelled, "Stage Queued", "Merge authority Granted" and "Recorded running" together (visual: overview-1440-full).
3. **Wrong subject on cards.** Planner cards never name workspace or repository; the headline is the full ~900-character goal text (visual). Worker events overwrite the goal's `last_activity`, and the planner card renders it as planner activity (fleet.rs:87-88, live.rs:55, App.vue:65).
4. **Internal vocabulary shown to the operator.** Activity titles are action ids (`loom.event`, `model.requested`); times are raw UTC strings, some with nanoseconds, with no elapsed time (Activity.vue:4); blockers show `ExternalAvailability(Text("effect failed: model command is not admitted"))` with no next step; ids and hashes lead queue rows (visual: ws-round06-1440-full). Refusals render as "command did not apply: {raw JSON}" at the top of the page, away from the form (lib.rs:108-111, App.vue:41-59).
5. **Evidence is a JSON dump.** "Inspect evidence" serves 74,820 bytes of escaped JSON with no summary (visual: evidence-round06-1440; dashboard.rs:12-35); `compact()` drops receipt fields, so the main view cannot say "acceptance recorded" or "merged at" (live.rs:82-88).
6. **Controls are far away and unsafe.** Goal controls sit about 2,500 px down, in a second goal list after the activity history, on workspace pages only (App.vue:57-69, visual). Merge authority is 11 px grey text. Cancel (permanent), Delete goal, Remove directory and Disable act on one click, look like harmless buttons and do not name their goal (App.vue:68-71, visual). Any goal save, including toggling merge authority, bumps the goal revision, and the fleet then cancels in-flight assignments without warning (GoalForm.vue:17, host.yaml:1057, fleet.rs:427-436).
7. **Legibility.** Secondary text is 9-11 px at contrast 3.0-3.95 (needs 4.5); input borders 1.49:1; status badges share one grey except Merged (style.css:1, visual). On 390 px the queue table hides the "Current reason" column, which is where blockers are (visual: overview-390-full).
8. **Two renderers.** The server-rendered page and the Vue app render the same screen and have drifted (default limits 3/3/60 vs 1/1/10, labels); Vue discards the server page and shows "00" until data arrives; the stylesheet is duplicated byte for byte (web.rs:29-38, GoalForm.vue:5,16, style.css = dashboard.css).

## Proposed improvements, by impact

| # | change | size | needs first |
|---|---|---|---|
| 1 | Fix the SSE projection (`compact()` in live.rs): planner-only latest activity, a derived `waiting {role, model, since}`, server clock, recorded-acceptance and merged flags, event ids | M | — |
| 2 | Status header and attention strip: coloured connection state, one sentence of system state with last-activity age, "—" for unknown counts, one row per blocked item with plain reason, age and the button that resolves it | M | 1 |
| 3 | Goal card as the unit: title "workspace · repository · short goal", one state derived from lifecycle plus phase, current step with elapsed time, controls (start, pause, cancel, edit, merge authority) in the card header, confirmations naming the goal, a warning before an edit that cancels in-flight work | L | 1 |
| 4 | Plain language: action ids mapped to sentences, relative times on a 1 s tick, streaming events grouped, internal errors translated to "what stopped / what to do", ids and hashes behind a details disclosure, errors shown under the form that caused them | M | 1 |
| 5 | Readable evidence page: verdict, timeline, checks, review, commit; raw JSON as a download | M | — |
| 6 | Visual baseline: at least 12 px, contrast at least 4.5:1, state colours, danger styling and spacing for destructive buttons, cards instead of the table on narrow screens; drop the duplicated server-rendered page and start Vue from an embedded snapshot | M | — |

Not settled by either review: whether granting merge authority should invalidate evidence at all (vision.md:30 says changes to authority invalidate stale evidence; it does not distinguish a grant from a revocation), and how browser checks required by vision.md:40 are run, since the repository carries no browser test tooling.

```findings
[
{"file":"frontend/src/App.vue","line":15,"category":"ux-state","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"No system state: a disconnected stream looks like a healthy idle system (00 tiles); no idle/waiting/working/blocked sentence; Need attention reads 00 beside failed assignments."},
{"file":"frontend/src/App.vue","line":65,"category":"ux-state","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"Goal badge ignores planning phase: Running+Blocked shows green; a Cancelled card also shows Stage Queued and Recorded running."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":87,"category":"ux-data","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Worker events overwrite the goal last_activity that the planner card renders as planner activity."},
{"file":"frontend/src/Activity.vue","line":4,"category":"ux-language","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Activity shows internal action ids and raw UTC timestamps; blockers show internal error debug strings with no next step."},
{"file":"crates/control-plane-app/src/dashboard.rs","line":12,"category":"ux-evidence","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Inspect evidence serves 74,820 bytes of escaped JSON with no summary; compact() drops receipt flags the main view would need."},
{"file":"frontend/src/App.vue","line":68,"category":"ux-controls","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"Goal controls sit about 2,500 px down; destructive actions act on one click without confirmation or goal name; a goal save that toggles merge authority bumps the revision and cancels in-flight assignments without warning."},
{"file":"frontend/src/style.css","line":1,"category":"accessibility","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Secondary text 9-11 px at contrast 3.0-3.95; badges share one grey; at 390 px the queue hides the reason column."},
{"file":"crates/control-plane-app/src/web.rs","line":29,"category":"maintainability","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Server-rendered page and Vue app render the same screen and have drifted (defaults, labels, duplicated stylesheet)."}
]
```
