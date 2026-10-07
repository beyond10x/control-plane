---
format: aep.planning-md/3
id: review-result:adversary-console-status-and-attention-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:console-status-and-attention at 40b04a2'
relations:
- reviews: story:console-status-and-attention
revision: 1
---
unit: story:console-status-and-attention, control-plane worktree cp-wave4-console-status-and-attention at 40b04a2 (base e88ee4f), plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 12→15, red 3
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: the projection has no field for when an assignment became Blocked or a publication became Uncertain, so finding 4 cannot be fixed inside frontend/

The "before" count (12) comes from the implementor's own Vitest summary, `Tests  12 passed (12)`, in `/dev/shm/cp-wave4-console-status-and-attention-scratch/f2-test.log`. I did not run the suite before adding my cases.

## 1. Diff proof

```
$ git status --short
?? frontend/src/status.attack.test.js
$ git --no-pager diff --stat
(empty: no tracked file changed)
$ git --no-pager diff --no-index --stat /dev/null frontend/src/status.attack.test.js
 /dev/null => frontend/src/status.attack.test.js | 81 +++++++++++++++++++++++++
 1 file changed, 81 insertions(+)
```

Only one test file was added; no implementation file was touched. `frontend/node_modules` was installed and then removed.

## 2. Cases added

All three are in `frontend/src/status.attack.test.js` in the unit's worktree. All three are red now.

| case | what it asserts |
|---|---|
| `attack_blocked_goal_reason_keeps_the_words_of_the_outcome` | A Blocked goal whose reason is `planner ended without a validated plan: NoAdmissibleAction(Unit(true))` gets an attention row that still says "admissible" and does not show `true` or an identifier. |
| `attack_blocked_assignment_reason_is_not_a_debug_struct` | Blocked assignments with reasons `Loom stopped: MaxTurns { limit: 1 }` and `Loom stopped: Deadline { limit_ms: 600000 }` must not match the unit's own identifier regex, and must not contain `{}` or snake_case field names. |
| `attack_open_model_call_is_named_while_another_worker_runs` | `alpha` has `waiting` set and `beta` is Implementing between calls. The header must contain "waiting for fixture-implementor". |

Red run of the file alone (`npx vitest run src/status.attack.test.js`), verbatim except that code frames are omitted. The full log is `adv1-case-alone.log`.
```
 ❯ src/status.attack.test.js (3 tests | 3 failed) 78ms
   × attack_blocked_goal_reason_keeps_the_words_of_the_outcome 49ms
   × attack_blocked_assignment_reason_is_not_a_debug_struct 17ms
   × attack_open_model_call_is_named_while_another_worker_runs 10ms
AssertionError: expected 'Planner ended without a validated pla…' to match /admissible/i
+ Received:
"Planner ended without a validated plan: true"
AssertionError: expected 'Loom stopped: MaxTurns { limit: 1 }' not to match /\b[A-Z][a-z]+[A-Z]\w*\b|\b[A-Z]\w*\(/
AssertionError: expected 'Loom stopped: MaxTurns { limit: 1 }' not to match /[{}]|\b[a-z]+_[a-z]+\b/
AssertionError: expected 'Loom stopped: Deadline { limit_ms: 60…' not to match /[{}]|\b[a-z]+_[a-z]+\b/
AssertionError: expected 'Workingimplementor in betaLast activi…' to match /waiting for fixture-implementor/i
+ Received:
"Workingimplementor in betaLast activity 5 min agoLive · committed observationsPlanner Add a status endpoint · Planning phase: Queued · 10 min ago"
 Test Files  1 failed (1)
      Tests  3 failed (3)
EXIT=1
```

## 3. Suite run (after the cases existed)

Command: `npm run test --prefix frontend`. Full log: `adv1-suite.log`.
```
 ❯ src/status.attack.test.js (3 tests | 3 failed) 81ms
 Test Files  1 failed | 2 passed (3)
      Tests  3 failed | 12 passed (15)
EXIT=1
```
The suite is red, which is the intended outcome of this pass. All 12 existing tests still pass.

## 4. Findings

All five cover 40b04a2. All five are "introduced": `status.js` and `goalState.js` do not exist at e88ee4f, and the base `App.vue` has no header and no attention strip.

**F1 · NEEDS-CHANGE · blocker** (`frontend/src/status.js:65`)
- **Defect:** the rule for wrappers like `Variant(…)` keeps only what is inside them. `NoAdmissibleAction(Unit(true))` therefore becomes `true`, so the operator reads "Planner ended without a validated plan: true".
- **Measured:** `status.attack.test.js:50`.
- **What reaches it:** `engine.rs:192-195` formats the Loom run outcome into the error. Loom ends any run in which the model proposes nothing admissible on two passes in a row with `NoAdmissibleAction(Unit(true))` (`loom-commission/src/runtime.rs:555`; `NoPerformableAction(Unit(true))` at :444 and :526). `supervisor.rs:160` stores that error as the planning reason with phase Blocked. The same message shape reaches assignment reasons through `fleet.rs:1155` and `:2354`.
- **Fix:** render a variant whose contents are not words by its name split into words ("no admissible action").
- This path is traced through the code; I did not observe it at runtime.

**F2 · NEEDS-CHANGE · warning** (`frontend/src/status.js:54`)
- **Defect:** the documented rule keeps a debug struct with no string fields unchanged. `MaxTurns { limit: 1 }` and `Deadline { limit_ms: 600000 }` reach the attention row as raw identifiers.
- **Measured:** `status.attack.test.js:66-67`.
- **What reaches it:** `loom_model.rs:144-148` fails a model call with `Loom stopped:` followed by Loom's stop reason. The turn budget and call timeout are set at `loom_model.rs:113`. The reviewer call at `fleet.rs:954` passes the error up, and `fleet.rs:498` blocks the assignment with it as the reason. Traced through the code only.

**F3 · NEEDS-CHANGE · warning** (`frontend/src/status.js:178`)
- **Defect:** "working" is checked before "waiting". When one worker runs tests and another has an open model call, the header says "Working · implementor in beta" and the open call disappears: no "waiting for <model>" and no "asked … ago".
- **Measured:** `status.attack.test.js:80`.
- **What reaches it:** two concurrent workers. The fixture's `max_workers` is 2, and the projection sets `waiting` per assignment. This contradicts acceptance `idle_and_waiting_are_distinct` and brief line 38.

**F4 · INFEASIBLE (cannot be fixed here) · warning** (`frontend/src/status.js:133`; also `:124` and `:191`)
- **Defect:** ages for an Uncertain publication, its Blocked assignment, and the header's "Last activity" come from the assignment's newest fleet entry. That entry is re-written on every fleet tick, so these ages stay at or below roughly 15 s even when the item has been stuck for hours.
- **What reaches it:**
  - `fleet.rs:390` runs `reconcile_publications` at the start of every tick (default poll interval 15 s, `control-plane-runtime/src/lib.rs:82`).
  - For a publication still not observed on its target, it calls `host.block` (`fleet.rs:1973-1976`).
  - `block` records a new `blocked` entry stamped with the current time before it checks the state (`fleet.rs:219-221` and `:300`).
- **Why not here:** the projection carries no blocked-since or uncertain-since time, and it keeps only the newest 24 activity entries. The brief says to stop and name the missing field in this situation; the unit used the newest entry instead.
- Traced through the code only. A Rust probe was out of scope for this pass.

**F5 · CONFIRMED · note** (`frontend/src/goalState.js:13`)
- **Defect:** Running plus Queued always derives "executing", whether or not any work exists. `supervisor.rs:178-186` records phase Queued with the reason "No ready story selected; …" when nothing is queued, and `:115-118` does not plan again while nothing changes. That goal stalls, yet it shows a green "Executing" badge, the header says "Idle", and there is no attention row.
- story:console-goal-cards will inherit this mapping.

## 5. Attacked and held

- **frontend/dist:** a fresh `vite build` into scratch matches it exactly (`diff -r` empty: app.js 91873 B, index.css 9389 B, index.html 318 B).
- **Never 0 while disconnected:** every path through error, unavailable, parse failure, connecting and navigation leaves the stream not live, and `App.vue:75` then shows "—". The strip is hidden when it has no items.
- **Derived-state table:** goal states (4) and planning phases (6) are closed sets (`ess/domains/host.yaml:145-149` and `:1696-1704`). All 24 combinations are in the table, so a Running goal with an unknown phase cannot occur.
- **Ages:**
  - A missing `server_time` hides "Last activity" and shows "Age unknown".
  - An entry newer than `server_time` shows "0 s ago".
  - The runtime writes RFC 3339 times, which `parseTime` handles.
- **Waits:** the projection sets `waiting` only on Implementing and Reviewing assignments. The goal-review wait arrives on the goal and is shown.
- **Attention filter:** the only Blocked assignments it leaves out for good have a publication (`retire_stale` skips those). Their Uncertain publication row still appears.
- **Workspace page:** publications are limited to that page's assignments (`control-plane-app/src/lib.rs:396-401`), so "Unknown goal" rows cannot appear there.

## 6. Paths written outside the worktree

- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv1-case-alone.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv1-suite.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv1-build.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv1-dist/` (app.js, index.css, index.html)
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/tmp/node-compile-cache/` (Node's cache, written or refreshed by these runs)

No cargo build was run and the build directory was not touched. My worktree session lease was acquired and then released. A `.scratch/` directory at the worktree root comes from the implementor's `frontend-check` run; I did not create it and left it in place.

## 7. Findings block

```findings
- file: frontend/src/status.js
  line: 65
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "plainReason replaces a tuple variant by its contents, so Loom's NoAdmissibleAction(Unit(true)) planner outcome reaches the attention row as \"Planner ended without a validated plan: true\"."
- file: frontend/src/status.js
  line: 54
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "plainReason keeps debug structs without string fields verbatim, so Loom stop reasons such as \"MaxTurns { limit: 1 }\" and \"Deadline { limit_ms: 600000 }\" are shown as error identifiers."
- file: frontend/src/status.js
  line: 178
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: working outranks waiting, so a projection with an open assignment model call renders "Working" and never "waiting for <model>" while another worker runs between calls.
- file: frontend/src/status.js
  line: 133
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: ages for uncertain publications, their blocked assignments and the header's last activity come from the newest fleet entry, which the fleet re-records on every tick, and the projection carries no blocked-since field to fix it from the frontend.
- file: frontend/src/goalState.js
  line: 13
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: Running with phase Queued always derives "executing", so a goal the supervisor left queued with nothing to run shows a green Executing badge, an Idle header and no attention row.
```
