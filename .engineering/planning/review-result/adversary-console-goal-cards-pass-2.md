---
format: aep.planning-md/3
id: review-result:adversary-console-goal-cards-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:console-goal-cards correction 1 on ae6053e plus working tree'
relations:
- reviews: story:console-goal-cards
revision: 1
---
Adversary pass 2 on story:console-goal-cards, correction 1, uncommitted working tree on base ae6053e in tree cp-wave6-console-goal-cards. Report as returned.

```
unit: story:console-goal-cards, correction 1
verdict: NEEDS-CHANGE
cases: executed 43→49, red 6
origin: introduced 7 / pre-existing 1 / undecided 0
wrote-outside-worktree: none
needs-coordinator: F7: whether the repository-settings warning must say that affected assignments can only be cancelled or re-planned, which is more than the story's "stopped and blocked"
```

## Cases added

`frontend/src/goalCards.pass2.attack.test.js`, run alone first: 6 tests, 6 failed.

| case | asserts | red output |
|---|---|---|
| runtime story id | with the runtime's `"<repository_id>::story:<slug>"` (supervisor.rs:448) neither card nor queue shows UUID fragments or `::` | received `Bdceb31a 273b 4248 8e69 f88446681e4e::story:status endpoint` |
| waiting detail | with a runtime story id the `waiting` detail (fleet.rs:813) leaves no `::` in the open | received `assignment (::“status docs”) holds it…` |
| planner.intent | "Reading workspace:<uuid>/README.md" shows no UUID segments | received `Reading “ 273b 4248 8e69 ”/README.md` |
| line number / port | `src/lib.rs:12:5` and `localhost:11434` pass unchanged | received `src/lib.“12”:5` and `(http://“11434”)`, `/api/chat` gone |
| " ./" spacing | `withoutIds('run ./scripts/check failed in .git/worktrees')` unchanged | received `run./scripts/check failed in.git/worktrees` |
| ReadyToMerge warning | the repository-settings warning appears for a ReadyToMerge assignment | `expected false to be true` |

Suite: `Test Files 1 failed | 6 passed (7)`, `Tests 6 failed | 43 passed (49)`, exit 1. Without the attack file: `Tests 43 passed (43)`.

## Findings

| id | file:line | verdict | origin | measured / what reaches it |
|---|---|---|---|---|
| F1 | frontend/src/identifiers.js:7 | NEEDS-CHANGE | introduced | `storyName` strips only a leading `word:`; the runtime id `<uuid>::story:x` shows the UUID and `::story:` on the card heading and queue. Every queued assignment (supervisor.rs:448; tests/planner.rs:336). The fixture's bare `story:status-endpoint` hides it |
| F2 | frontend/src/identifiers.js:19 | NEEDS-CHANGE | introduced | `waiting` detail renders `(::“status docs”)` in the open (fleet.rs:813) |
| F3 | frontend/src/identifiers.js:19 | NEEDS-CHANGE | introduced | artifact pattern runs before UUID removal: `workspace:<uuid>` keeps UUID segments (planner.intent, engine.rs:285) |
| F4 | frontend/src/identifiers.js:19 | NEEDS-CHANGE | introduced | artifact pattern rewrites `rs:12` and `localhost:11434`, drops the URL path; compiler output in Blocked reasons (process.rs:32); provider host:port inferred only |
| F5 | frontend/src/identifiers.js:33 | CONFIRMED | introduced | the space before `.` is deleted (`run./x`, `in.git`); cosmetic |
| F6 | frontend/src/goalCard.js:47 | NEEDS-CHANGE | introduced | `executingIn` omits ReadyToMerge, which the fleet still delivers (fleet.rs:742) and whose merge and publication the store refuses after a settings change (guards.rs:325-334) |
| F7 | frontend/src/RepoForm.vue | CONFIRMED | introduced | from code: after a settings change every later step is refused (guards.rs:63-78, fleet.rs:1145-1149), so affected assignments, including already-Blocked claimed ones, can only be cancelled or re-planned |
| F8 | frontend/src/App.vue:87 | CONFIRMED | introduced | from code: removing a directory or changing repository settings also stops a running planner (supervisor.rs:393-402) |
| F9 | frontend/src/status.js:287 | CONFIRMED | pre-existing | the attention strip shows reasons without `withoutIds`; status.js unchanged from the base, outside the card and queue the acceptance names |

Attacked without a break: full text in every details disclosure; `12:30` survives; activity.js covers every action id in engine.rs, supervisor.rs, fleet.rs and governance.rs; Paused goals' in-flight counts; disable wording.

```findings
[
  {"file": "frontend/src/identifiers.js", "line": 7, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F1: storyName on the runtime story id shape <repository_uuid>::story:<slug> (supervisor.rs:448) shows the repository UUID and ::story: on the card heading and in the queue"},
  {"file": "frontend/src/identifiers.js", "line": 19, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F2: the waiting detail with a runtime story id renders as (::“status docs”) in the open"},
  {"file": "frontend/src/identifiers.js", "line": 19, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F3: workspace:<uuid>/file in planner.intent becomes a quoted name made of UUID segments because the artifact pattern runs before UUID removal"},
  {"file": "frontend/src/identifiers.js", "line": 19, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F4: the artifact pattern rewrites line numbers (src/lib.rs:12:5) and ports (localhost:11434) into quoted names and drops the URL path"},
  {"file": "frontend/src/identifiers.js", "line": 33, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F5: the space before a dot is deleted, so 'run ./x' reads 'run./x' and 'in .git' reads 'in.git'"},
  {"file": "frontend/src/goalCard.js", "line": 47, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F6: executingIn omits ReadyToMerge, whose merge and publication the store refuses after a settings change, so the repository-settings save warns nothing for it"},
  {"file": "frontend/src/RepoForm.vue", "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "F7: inferred from code: after a settings change the store refuses every later step (guards.rs:63-78), so affected assignments, including already-Blocked claimed ones, can only be cancelled or re-planned, which the warning does not say"},
  {"file": "frontend/src/App.vue", "line": 87, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F8: inferred from code: removing a directory or changing repository settings also stops a running planner (supervisor.rs:393-402), which no confirmation states"},
  {"file": "frontend/src/status.js", "line": 287, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "F9: the attention strip shows assignment reasons without withoutIds, so hashes stay visible there"}
]
```
