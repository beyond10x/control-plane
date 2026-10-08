---
format: aep.planning-md/3
id: review-result:adversary-console-evidence-page-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:console-evidence-page at af990b2'
relations:
- reviews: story:console-evidence-page
revision: 1
---
unit: story:console-evidence-page
verdict: red
cases: executed 105→114, red 2
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: yes

The findings cover commit af990b2 plus my two untracked test files, in worktree `<tree>/cp-wave8-console-evidence-page`.

**1. Diff (`git diff --stat` is empty, `git status --short`)**
```
?? crates/control-plane-app/tests/console_evidence_page_attack.rs
?? frontend/src/evidence.attack.test.js
```
I added only these two test files and edited no implementation file.

**2. Cases added (each run alone first)**

| case | what it asserts | now |
|---|---|---|
| `adversary_latest_check_survives_a_minute_of_review_streaming` | runs `checks.run`, then `review.request`, then 64 `loom.event` through `Supervisor::record_progress`, then `ReadyAssignment`; after that, `history.activity` still holds a check that has a command | **red** |
| `adversary_merged_fixture_matches_the_server_shape` | every field the view reads exists in the server's real JSON for a merged assignment, with the same type as in `evidence-merged.json` | green |
| `adversary_api_path_keeps_the_guard_and_answers_a_missing_goal` | `/api/...` refuses cross-site and foreign-origin requests (403); an unknown id gets an error in JSON form; the console path returns 200 HTML | green |
| `adversary_console_is_served_for_a_bare_percent_id` | `/goals/50%/evidence` returns 200 HTML | green |
| `adversary_bare_percent_in_the_console_path_does_not_throw` (vitest) | `evidenceGoal('/goals/50%/evidence')` does not throw | **red** |
| `adversary_missing_goal_shows_the_server_error`, `adversary_goal_checks_after_checks_run_is_the_latest`, `adversary_same_second_checks_keep_recording_order`, `adversary_reviewer_without_review_revision_is_not_an_approval` (vitest) | the error alert, `goal.checks` supplying `observed_head`, tie order, approval needing `review_revision` | green |

Red output from the solo runs, verbatim:
```
panicked at crates/control-plane-app/tests/console_evidence_page_attack.rs:215:5:
the checks ran on cand1234 with `task check`, yet the evidence JSON the view reads holds no check activity, so the view says 'No check recorded'; JSON elsewhere mentions the command: false
test result: FAILED. 3 passed; 1 failed
```
```
FAIL src/evidence.attack.test.js > adversary_bare_percent_in_the_console_path_does_not_throw
AssertionError: expected [Function] to not throw an error but 'URIError: URI malformed' was thrown
```

**3. Suite runs (after the cases existed)**
- `cargo test --locked --no-fail-fast -p control-plane-app` gave EXIT=101. Results: lib 46 ok, `console_evidence_page_attack` 3 passed and 1 failed, `console_projection_attack` 2 ok, `console_projection_pass2_attack` 3 ok, `operator_boundary` 3 ok. Without my file that is 54 cases; with it, 58.
- `npx vitest run` gave EXIT=1: `Tests 1 failed | 55 passed (56)`. Without my file that is 51; with it, 56.
- The "before" counts are these same runs with my file subtracted. I did not run the suite before writing the cases.

**4. Findings**
- **`frontend/src/evidence.js:24` — the "latest check" is lost in an ordinary run.** In a normal run the view says "No check recorded" for an assignment whose checks ran.
  - Measured: the history keeps only 64 activities (`memory.rs:50`). After `checks.run` (`fleet.rs:1287`), the review model's stream logs one `loom.event` per second (`fleet.rs:78`). After about a minute of reviewing, no check, and not even the string `task check`, is left anywhere in the JSON.
  - What reaches it: every review that streams for more than about 64 seconds, because `review.request` follows the checks (`fleet.rs:1357`). On a satisfied goal the same happens after `goal.checks`, because `goal.review` and its stream follow it (`fleet.rs:2673` → `2699`).
  - Per the brief, the JSON lacks the field, so this is a decision about the spec or the acceptance, not something to fix in the view. That is why `needs-coordinator: yes`.
  - NEEDS-CHANGE, introduced. The JSON bounding already existed; this unit's acceptance is what relies on it.
- **`frontend/src/evidence.js:8` — a bare `%` in the URL throws.** `decodeURIComponent` throws `URIError` for an id containing a bare `%`. `main.js` calls it before mounting, so the page stays blank. The server does serve the console shell for that path (green case). Only a hand-typed URL reaches it, since goal ids are UUIDs. INFEASIBLE, introduced, note.
- **`crates/control-plane-app/src/tests.rs:425` — the JSON-equality test cannot fail on the filtering.** `evidence_json_moves_to_api_path` builds its expected value from the same `state.snapshot()` the handler reads. Its fixture has no assignments and no publications (`:450`), so dropping or inverting the handler's assignment or publication filter would leave it green. The old path was never compared. `dashboard.rs` is unchanged, which keeps the actual risk low. CONFIRMED, introduced, note.

**5. Attacked, could not break**
- The fixtures against the real server shape: every field the view reads is present with a matching type, including `merge_receipt.observed_head`, `target` and `candidate`.
- The guard on the new `/api` path: cross-site requests and foreign origins get 403.
- An unknown goal: 409 with a JSON `error`, and the view shows it.
- Old-path callers: `GoalCard.vue:44`, `status.js:297` and `dashboard.rs:287` link to the view, which is intended. The CLI has no caller.
- `index.html` uses absolute asset paths, so the deeper path still loads the stylesheet and script.
- The download link points at the encoded `/api` path.
- Approval needs `review_revision`; `goal.checks` takes precedence through `observed_head`; tie order on equal timestamps is correct.
- The real merge receipt is always JSON (`fleet.rs:2357`). The legacy `"observed-target c1"` string comes only from a hand-written fixture.

**6. Paths written outside the worktree:** none. Scratch and the npm cache are under `<tree>/.scratch`. I removed `frontend/node_modules`.

```findings
- file: frontend/src/evidence.js
  line: 24
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: after about a minute of reviewer streaming the 64-entry history no longer holds any checks.run or goal.checks, so the view shows "No check recorded" for checks that ran, and the evidence JSON carries the command nowhere else
- file: frontend/src/evidence.js
  line: 8
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: decodeURIComponent throws URIError on a bare percent in the console path, blanking the page, reachable only by a hand-typed non-UUID id
- file: crates/control-plane-app/src/tests.rs
  line: 425
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: evidence_json_moves_to_api_path derives its expected value from the handler's own snapshot with zero assignments and publications, so a broken assignment or publication filter stays green
```
