---
format: aep.planning-md/3
id: review-result:adversary-console-goal-cards-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:console-goal-cards on ae6053e plus working tree'
relations:
- reviews: story:console-goal-cards
revision: 1
---
Adversary pass 1 on story:console-goal-cards, uncommitted working tree on base ae6053e in tree cp-wave6-console-goal-cards. Report as returned.

```
unit: story:console-goal-cards
verdict: NEEDS-CHANGE
cases: executed 37→41, red 4
origin: introduced 4 / pre-existing 0 / undecided 1
wrote-outside-worktree: none
needs-coordinator: yes. The story's wording for the "disable repository" and "remove directory" consequences looks understated (F5), and only the story can change that.
```

## Cases added

`frontend/src/goalCards.pass1.attack.test.js`, run alone first: 4 tests, 4 failed.

| line | asserts | red output |
|---|---|---|
| 48 | the runtime action `planner.prepare-branch` gets a sentence (engine.rs:87; supervisor.rs:637 records it as `planner.{kind}`) | `expected 'planner.prepare-branch' not to be 'planner.prepare-branch'` |
| 56 | the runtime action `waiting` gets a sentence (fleet.rs:819 `note`), and its detail does not show the holder's assignment id or story id | `expected 'waiting' not to be 'waiting'` |
| 68 | the `planner.accept-story` detail "Accepting story:status-endpoint" (engine.rs:641) is not visible on the card | `expected '…Accepting story:status-endpoint…' not to contain 'story:status-endpoint'` |
| 76 | a Blocked assignment whose reason names its candidate hash (fleet.rs:2434) does not show that hash on the card or in the queue | `card: expected '…does not contain candidate 9f86d081…' not to contain '9f86d081884c7d659a2feaa0c55ad015a3bf4…'` and the same for the queue |

Suite: `TMPDIR=<tree>/.scratch/tmp npx vitest run` in `frontend`: `Test Files  1 failed | 5 passed (6)`, `Tests  4 failed | 37 passed (41)`, exit 1. Without the attack file: `Tests  37 passed (37)`.

## Findings

| id | file:line | verdict | origin | measured / what reaches it |
|---|---|---|---|---|
| F1 | frontend/src/activity.js:6 | NEEDS-CHANGE | introduced | `planner.prepare-branch` and `waiting` have no sentence and render as raw ids; every planning run starts with prepare-branch, every assignment waiting behind a held repository records `waiting` |
| F2 | frontend/src/activity.js:47 | NEEDS-CHANGE | introduced | `detailIsProse` treats id-carrying details as prose: "Accepting story:<id>" and the `waiting` detail's assignment UUID and story id are visible; every approved plan records accept-story per story |
| F3 | frontend/src/GoalCard.vue:39 | NEEDS-CHANGE | introduced | the Blocked row shows `plainReason(assignment.reason)` in the open, so a candidate hash, target head hash or `{revision} does not merge` text (fleet.rs:600) is visible |
| F4 | frontend/src/App.vue:84 | NEEDS-CHANGE | undecided | the queue's "Current reason" column shows `a.reason` unfiltered; the same line exists at the base, not run there |
| F5 | frontend/src/App.vue:88 | CONFIRMED | introduced | inferred from code: the fleet guard (fleet.rs:196-208) fails in-flight work when a repository is disabled or reconfigured or the workspace's directories change (blocks it, fleet.rs:837-849); the confirmations say only "no new work starts" / repositories leave, and a repository settings save warns nothing. The wording follows the story |

Attacked without a break: derived state on the card for 5 lifecycle states × 7 phases × 5 assignment mixes; no request before confirmation (0 requests on "Keep"); refusal placement for Pause, Remove directory, Disable; elapsed time with no start; merge-authority toggle on a Paused goal with a held publication.

```findings
[
  {"file": "frontend/src/activity.js", "line": 6, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F1: runtime actions planner.prepare-branch (engine.rs:87) and waiting (fleet.rs:819) have no sentence and render as raw ids"},
  {"file": "frontend/src/activity.js", "line": 47, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F2: detailIsProse treats planner.accept-story and waiting details as prose, so story ids and assignment UUIDs show in the open on the card"},
  {"file": "frontend/src/GoalCard.vue", "line": 39, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F3: a Blocked assignment's reason is shown in the open on the card, including candidate and head hashes from fleet.rs:2434"},
  {"file": "frontend/src/App.vue", "line": 84, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "F4: the queue's Current reason column shows a.reason unfiltered, so candidate hashes are visible queue text"},
  {"file": "frontend/src/App.vue", "line": 88, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "F5: inferred from code: disabling or reconfiguring a repository, or removing a directory, fails in-flight work through the fleet guard (fleet.rs:196-208), and the confirmations do not say so"}
]
```
