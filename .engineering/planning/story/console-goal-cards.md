---
format: aep.planning-md/3
id: story:console-goal-cards
kind: story
status: active
title: Goal cards carry state, step and controls
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- informed_by: review-result:console-ux-review-2026-10-06
- depends_on: story:console-status-and-attention
scope:
- confidence: inferred
  path: frontend/dist
- confidence: inferred
  path: frontend/src
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T00:11:59Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-08T00:12:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

Each goal is one card titled workspace · repository · short goal, showing the derived goal state built by story:console-status-and-attention (this story consumes it and defines none), the current step and its elapsed time, and its controls (start, pause, cancel, edit, merge authority) in the card header on every page. Every destructive action (cancel goal, delete goal, remove directory, disable repository) asks for confirmation naming its subject and the consequence. An edit that will cancel in-flight work says so before it is saved. Activity reads as sentences with relative times, refusals appear under the form that caused them, and ids, hashes and paths sit behind a details disclosure.

## Evidence

review-result:console-ux-review-2026-10-06 rows 3, 4 and 6: controls about 2,500 px down in a second goal list; cancel, delete goal, remove directory and disable act on one click; any goal save, including toggling merge authority, bumps the revision and the fleet then cancels in-flight assignments (frontend/src/GoalForm.vue, ess/domains/host.yaml UpdateGoal `revision: {increment: 1}`, fleet.rs `retire_stale`); activity titles are action ids such as `loom.event`; ids and hashes lead queue rows.

## Acceptance

- `goal_card_title_names_workspace_repository_and_goal`: a card renders "workspace · repository · short goal", the goal text cut to one line.
- `goal_card_shows_the_derived_state`: the card's badge equals the derived goal state for Running/Blocked, Paused and Cancelled goals.
- `goal_card_shows_step_and_elapsed_time`: a goal with an implementing assignment shows that step and the time since it started.
- `goal_controls_are_on_the_card`: the overview and the workspace page render start/pause/cancel/edit and the merge-authority control on each goal card.
- `destructive_actions_confirm_with_their_subject`: cancel goal, delete goal, remove directory and disable repository each render, before any request is sent, a confirmation naming the goal, directory or repository and stating the consequence (for cancel: no further work starts and in-flight assignments are cancelled; for remove directory: its repositories leave the workspace unless another directory covers them, and every in-flight assignment in the workspace is stopped and blocked, because the runtime compares the whole directory list (fleet.rs `workspace directory membership changed during execution`); for disable: no new work starts in that repository, and its in-flight assignments are stopped and blocked). Saving repository settings while that repository has in-flight assignments warns before the save that they will be stopped and blocked (fleet.rs `repository configuration changed or disabled`, `workspace directory membership changed during execution`).
- `merged_assignment_shows_merge_time`: a merged assignment on a card shows its merge time from `merged_at`.
- `revision_bump_is_announced`: editing a goal with in-flight assignments shows the warning that they will be cancelled before the save.
- `activity_reads_as_sentences`: each known action id renders a sentence and a relative time; an unknown id renders its id unchanged.
- `refusal_appears_under_its_form`: a refused goal save shows the server's error under the goal form, not at the top of the page.
- `raw_ids_are_behind_details`: story ids, candidate and run ids and worktree paths are absent from the visible card and queue text and present inside the details disclosure.

## Scope

Inferred: frontend/src/App.vue, frontend/src/GoalForm.vue, frontend/src/Activity.vue, new components and tests, frontend/dist.

## Open question

Whether granting merge authority should invalidate evidence at all is not settled (vision.md:30 says authority changes invalidate stale evidence and does not distinguish a grant from a revocation). This story only announces the effect; changing it needs a specification decision.

## Pre-existing findings

- review-result:adversary-console-goal-cards-pass-2 F9: the attention strip (`frontend/src/status.js:287`) shows assignment reasons without the identifier filter the cards and queue use, so candidate hashes stay visible there. Pre-existing and outside this story's card and queue; not fixed in wave 6.
