---
format: aep.planning-md/3
id: review-result:adversary-console-status-and-attention-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:console-status-and-attention at 343561b'
relations:
- reviews: story:console-status-and-attention
revision: 1
---
unit: story:console-status-and-attention, worktree cp-wave4-console-status-and-attention at 343561b (base e88ee4f), plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 16→22, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths in the assigned scratch directory, plus state the worktree tool and npm manage themselves (part 6)
needs-coordinator: none

The "before" count (16) comes from the implementor's correction run, `Tests  16 passed (16)` in `/dev/shm/cp-wave4-console-status-and-attention-scratch/d2-test.log`. I did not run the suite before writing my cases.

## 1. Diff proof

```
$ git status --short --ignored
?? frontend/src/status.pass2.attack.test.js
$ git --no-pager diff --stat
(empty: no tracked file changed)
$ git --no-pager diff --no-index --stat /dev/null frontend/src/status.pass2.attack.test.js
 .../src/status.pass2.attack.test.js | 135 +++++++++++++++++++++
 1 file changed, 135 insertions(+)
```

I added one test file and changed nothing else. No implementation file, no existing test and nothing in `frontend/dist` was touched. I installed `frontend/node_modules`, used it and removed it. No `/home/` path appears in the file.

## 2. Cases added

All six are in `frontend/src/status.pass2.attack.test.js` in the unit's worktree, and all six are red now.

| case | what it asserts |
|---|---|
| `attack2_planner_model_stop_inside_an_outage_is_rendered_as_words` | The real planner outcome `Suspended(RunOutcomeSuspended { reason: ExternalAvailability(Object([("error", Text("Loom stopped: Deadline { limit_ms: 600000 }"))])) })` produces a row with no braces, no snake_case and no "object". |
| `attack2_implementor_model_stop_inside_an_outage_is_rendered_as_words` | The same outcome form with `MaxTurns { limit: 40 }` as an assignment reason reads "max turns", with no identifier and no braces. |
| `attack2_reviewer_prose_keeps_its_code_references` | A reviewer's rejection that quotes `Ok(())` and `Some(0)` keeps both quotes. |
| `attack2_goal_acceptance_checks_are_not_a_stall` | All current assignments are Merged and the fleet entry is `goal.checks`, running. The card does not say Stalled, there are no attention rows, and the header does not say Blocked. |
| `attack2_stalled_goal_after_rejected_acceptance_states_the_rejection` | All current assignments are Merged and the fleet entry is `blocked` from `goal_reviewer`. The stall row names the goal-review rejection. |
| `attack2_every_struct_in_a_list_keeps_its_name` | `Stops([MaxTurns {…}, Deadline {…}])` names both "max turns" and "deadline". |

First red run, the first five cases alone (`npx vitest run src/status.pass2.attack.test.js`). Output is verbatim except that code frames are left out; the full log is `adv2-case-alone.log`.
```
 ❯ src/status.pass2.attack.test.js (5 tests | 5 failed) 149ms
   × attack2_planner_model_stop_inside_an_outage_is_rendered_as_words 66ms
   × attack2_implementor_model_stop_inside_an_outage_is_rendered_as_words 19ms
   × attack2_reviewer_prose_keeps_its_code_references 23ms
   × attack2_goal_acceptance_checks_are_not_a_stall 23ms
   × attack2_stalled_goal_after_rejected_acceptance_states_the_rejection 16ms
AssertionError: expected 'Planner ended without a validated pla…' not to match /[{}]|\b[a-z]+_[a-z]+\b/
"Planner ended without a validated plan: suspended: external availability: object: error: Loom stopped: Deadline { limit_ms: 600000 }"
AssertionError: expected 'Planner ended without a validated pla…' not to match /\bobject\b/i
AssertionError: expected 'Implementation ended before a candida…' to match /max turns/i
"Implementation ended before a candidate proposal: suspended: external availability: object: error: Loom stopped: MaxTurns { limit: 40 }"
AssertionError: expected 'Implementation ended before a candida…' not to match /\b[A-Z][a-z]+[A-Z]\w*\b|\b[A-Z]\w*\(/
AssertionError: expected 'Implementation ended before a candida…' not to match /[{}]/
AssertionError: expected 'Independent review rejected candidate…' to contain 'returns Ok(()) even when the push fai…'
Received: "Independent review rejected candidate: {"approved":false,"reason":"publish() returns even when the push fails, and count() returns 0 for an empty queue"}"
AssertionError: expected 'Independent review rejected candidate…' to contain 'returns Some(0) for an empty queue'
AssertionError: expected 'Stalled' not to be 'Stalled' // Object.is equality
AssertionError: expected [ DOMWrapper{ …(3) } ] to have a length of +0 but got 1
AssertionError: expected 'Blocked' not to be 'Blocked' // Object.is equality
AssertionError: expected 'The plan is queued, but no assignment…' to match /goal review rejected/i
"The plan is queued, but no assignment of this goal is queued or running."
 Test Files  1 failed (1)
      Tests  5 failed (5)
EXIT=1
```

Second red run, the sixth case alone, which I added after the first five had run (full log `adv2-case6-alone.log`):
```
   × attack2_every_struct_in_a_list_keeps_its_name 10ms
AssertionError: expected 'Budgets exceeded: stops (max turns (l…' to match /deadline/i
"Budgets exceeded: stops (max turns (limit 1), limit 600 s)"
      Tests  1 failed | 5 skipped (6)
EXIT=1
```

## 3. Suite run (after the cases existed)

Command: `npm run test --prefix frontend`. The final run is in `adv2-suite2.log`; an earlier run with five cases, in `adv2-suite.log`, read `5 failed | 16 passed (21)`.
```
 ❯ src/status.pass2.attack.test.js (6 tests | 6 failed) 197ms
 Test Files  1 failed | 3 passed (4)
      Tests  6 failed | 16 passed (22)
EXIT=1
```
The suite is red, which is the outcome this pass aims for. All 16 existing tests still pass, including pass 1's 3 attack cases.

## 4. Findings (all cover 343561b)

All six are "introduced": `git show e88ee4f:frontend/src/status.js` and `git show e88ee4f:frontend/src/goalState.js` both exit with 128, because neither file exists at the base.

| # | file:line | verdict · severity | what was measured | what reaches it |
|---|---|---|---|---|
| 1 | goalState.js:40 | NEEDS-CHANGE · blocker | `goal_acceptance_checks_are_not_a_stall`: the card reads Stalled, there is 1 row, and the header reads Blocked | `run` calls `satisfy_goals` at fleet.rs:513 in the same tick as the last merge. It runs once every assignment is Merged (fleet.rs:2001-2006), records `goal.checks` (fleet.rs:2104) and runs the test command for each repository (fleet.rs:2105-2111) before any goal-review wait. Ticks run one after another (supervisor.rs:44-47), so no planner wait covers this window. **Every completing goal** shows "needs you" for the length of its test suite, and the row offers the controls that would interrupt acceptance. Traced through the code, not observed at runtime. |
| 2 | status.js:246 | NEEDS-CHANGE · warning | `stalled_goal_after_rejected_acceptance…`: the reason shown is the generic fallback | A latched rejection records `blocked` with the reason on the assignment's fleet entry (fleet.rs:2255-2262). After that, the supervisor (supervisor.rs:100-112) and the fleet (fleet.rs:2037-2039) skip the goal for good, and `planning_reason` stays `""` (supervisor.rs:176-178). The projection carries the reason as the fleet entry's `detail` (live.rs:388-401). |
| 3 | status.js:151 | NEEDS-CHANGE · warning | the two outage cases: `MaxTurns { limit: 40 }` and `Deadline { limit_ms: … }` stay raw | A planner model failure becomes `SelectorError::Unavailable(e.to_string())` (engine.rs:764). Loom wraps that as `ExternalAvailability(Object([("error", Text(..))]))` (loom-executor lib.rs:173-180, :221-222). engine.rs:192-196 records it, and supervisor.rs:160/186 store it. The implementor path is fleet.rs:1236 → :1153-1157 → :497-499. The budget behind these stops is set at loom_model.rs:113. Pass 1's F2 was fixed only for the unwrapped form, but this wrapped form is the one the planner and the implementor actually produce. |
| 4 | status.js:118 | CONFIRMED · note | the planner case: "object:" appears | The `render` doc (status.js:142-144) says a map is rendered without a name. Loom's map is `Object(Vec<(String, Value)>)` (Loom json.rs:23-42), a tuple, which `anonymous` does not cover. It reaches every outage reason and every step-budget reason (`Budget(Object([("max_steps", …)]))`, loom-commission runtime.rs:576). |
| 5 | status.js:184 | NEEDS-CHANGE · warning | `reviewer_prose_keeps_its_code_references`: `Ok(())` is deleted and `Some(0)` becomes `0` | A rejected review blocks the assignment with `independent review rejected candidate: {review}` (fleet.rs:955-961, model.rs:62-63, fleet.rs:497-499). Reviewer prose that quotes Rust is plausible but was built for this case, not observed. The same rewriting reaches the rejection JSON, which is otherwise shown raw. |
| 6 | status.js:157 | INFEASIBLE · note | `every_struct_in_a_list_keeps_its_name` | Nothing found: no runtime reason carries a list of structs. The cause is that `items.map(render)` passes each item's index as the `unnamed` flag. |

Fixes I would suggest, none applied:
- **#1:** treat Running/Queued whose current assignments are all Merged, with a current fleet entry `goal.checks` or `goal.review` (status `running`), as executing, and have the header say the checks are running. The projection already carries those entries, so no new field is needed.
- **#2:** for a stalled goal, use the reason from the newest `blocked` fleet entry with role `goal_reviewer` at the goal's revision.
- **#3:** pass a string node's text through `words()` again.
- **#4:** treat the `Object` and `Array` tuples as nameless.
- **#5:** only render a Debug value that runs to the end of a reason link, and never render a value as empty text.
- **#6:** change the call to `items.map(item => render(item))`.

## 5. Attacked and held

- **frontend/dist:** a fresh `vite build --outDir` into scratch is identical (`diff -r` is empty; app.js 96152 B, index.css 9389 B, index.html 318 B).
- **Pass 1 F1, F2 and F3:** all three of pass 1's attack cases pass in the suite.
- **Header with several open calls and mixed lanes:** calls are sorted oldest `since` first. The label names the oldest call's model, the detail lists lanes in the same order, and `also` counts the working lanes. A `since` that does not parse sorts last. I checked this by reading the code; no case was written.
- **Callers of deriveGoalState:** App.vue:78 (both calls) and status.js:242 and :297 all pass assignments, and none relies on the default.
- **Stalled versus an assignment the fleet is about to claim:**
  - A current Queued assignment derives `executing`.
  - `plan_repository` creates the assignments before the goal's phase is recorded as Queued, so there is no stalled window while planning.
  - A Blocked assignment with attempts left is claimed again on the next tick (fleet.rs:419-460, :808-816) but still gets a row. Acceptance `attention_lists_each_blocked_item` requires that row, so I did not raise it.
- **Strictness on prose:** `Name (`, `Template {name}` and `HTTP(S)` stay as written.
- **A bare unit variant such as `GovernorUnavailable`:** it stays raw. The only path I found needs a case-store failure, so I did not raise it.

## 6. Paths written outside the worktree

- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-case-alone.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-case6-alone.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-suite.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-suite2.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-build.log`
- `/dev/shm/cp-wave4-console-status-and-attention-scratch/adv2-dist/` (app.js, index.css, index.html)
- Tool-managed, not chosen by me:
  - `/dev/shm/cp-wave4-console-status-and-attention-scratch/tmp/` (Node's cache, refreshed)
  - npm's shared download cache from `npm ci`
  - the worktree lease record for session `adversary-cp-wave4-csa-pass2`, which I acquired and then released

No cargo build was run, and the build directory was not touched.

## 7. Findings block

```findings
- file: frontend/src/goalState.js
  line: 40
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A Running goal in phase Queued whose current assignments are all Merged derives stalled while the fleet runs its goal acceptance checks, so every completing goal shows a red Stalled row and a Blocked header for as long as its test command runs."
- file: frontend/src/status.js
  line: 246
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The stalled row takes its reason only from planning_reason, so a goal parked by a rejected final goal review reads 'The plan is queued, but no assignment of this goal is queued or running.' and never states the rejection recorded in its fleet entry."
- file: frontend/src/status.js
  line: 151
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Debug text carried inside a Debug string is kept verbatim, so planner and implementor model stops, which arrive wrapped in Loom's ExternalAvailability outage, reach the attention row as 'Loom stopped: MaxTurns { limit: 40 }'."
- file: frontend/src/status.js
  line: 118
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The render doc says a map is rendered without a name, but Loom's map is the tuple form Object([(key, value)]), which renders as 'object:' in every outage reason and every step-budget reason."
- file: frontend/src/status.js
  line: 184
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "words() rewrites any capitalised name followed by a bracket inside prose, so a reviewer reason that quotes Ok(()) loses it and Some(0) becomes 0, which changes what the rejection says."
- file: frontend/src/status.js
  line: 157
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "List items are rendered with items.map(render), which passes each index as the unnamed flag, so every struct after the first in a list loses its name; no runtime reason that carries a list of structs was found."
```
