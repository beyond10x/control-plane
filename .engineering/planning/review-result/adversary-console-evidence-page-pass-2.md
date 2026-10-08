---
format: aep.planning-md/3
id: review-result:adversary-console-evidence-page-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:console-evidence-page at 1936742'
relations:
- reviews: story:console-evidence-page
revision: 1
---
unit: story:console-evidence-page
verdict: red
cases: executed 115→119, red 2
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: no

These findings cover commit 1936742 plus my two new test files, in worktree `$HOME/.local/state/worktree/trees/b10x/control-plane/cp-wave8-console-evidence-page`.

**1. Diff.** `git diff --stat` is empty. `git status --short` shows only my two new test files:
```
?? crates/control-plane-app/tests/console_evidence_page_pass2_attack.rs
?? frontend/src/evidence.pass2.attack.test.js
```
I changed no implementation file.

**2. Cases added (each run on its own first)**

| case | what it checks | now |
|---|---|---|
| `adversary_latest_check_of_a_second_story_is_not_hidden_by_an_earlier_tested_story` (vitest) | story A merged on a1, then story B's check ran on b1 and failed. The "Latest check" panel should show b1. | **red** |
| `adversary_goal_acceptance_over_two_repositories_shows_both_heads` (vitest) | a goal over two repositories records two `goal.checks`. The panel should show both heads. | **red** |
| `adversary_console_path_cannot_steer_the_fetch_outside_the_goal_segment` (vitest) | for bare `%`, `%2e%2e`, an encoded `../`, `?#` and a truncated UTF-8 escape, the fetch URL stays as one segment under `/api/goals/` | green |
| `adversary_test_revision_is_cleared_by_repair_and_never_set_without_review` (Rust, real store and router) | a repair clears `test_revision`. A block keeps it, and it was set only after passed checks. A check that failed leaves it `''`. A mismatched `test_revision` is refused. | green |

Red output from the solo run:
```
AssertionError: the latest recorded check ran on b1: expected 'Latest checkstory:a · tests passed on…' to contain 'b1'
Received: "Latest checkstory:a · tests passed on candidate a1 · task check · 2026-10-06T09:00:00Z"
AssertionError: first repository acceptance head: expected 'Latest checkstory:a · tests passed on…' to contain 'head-repo-one'
Received: "…Goal acceptance check cargo test on head-repo-two · 2026-10-06T10:00:30Z"
Tests  2 failed | 1 passed (3)
```
The Rust case needed two fixes to its own setup before it ran (it did not accept the `rebased` outcome, and the store allows only one active change per repository). After those fixes it was green.

**3. Suite runs (after my cases existed)**
- `cargo test --locked --no-fail-fast -p control-plane-app` gave EXIT=0: 46 + 0 + 4 + 1 + 2 + 3 + 3 = 59 cases. Without my file that is 58.
- `npx vitest run` gave EXIT=1: `Tests 2 failed | 58 passed (60)`. Without my file that is 57.
- The before count of 115 is these same runs minus my 4 cases. I did not run the suite before writing the cases. Your brief asked for a suite run first, but the adversary procedure forbids that (hard rule 3), so I followed the procedure.

**4. Findings**
- **`frontend/src/Evidence.vue:14` — the "Latest check" panel hides the newest check (NEEDS-CHANGE, introduced by 1936742).**
  - **Cause:** the panel suppresses `latestCheck` whenever any assignment has a `test_revision` or a goal check exists.
  - **Effect:** a later story whose check ran and failed (its `test_revision` stays `''`, `fleet.rs:1287`→`1353`) is missing from the panel. The panel shows only the older passed story. At af990b2 the panel showed b1.
  - **What reaches it:** any goal with more than one story, or a re-plan after a goal review is rejected. Case: `evidence.pass2.attack.test.js:41`.
  - **Fix:** always show the latest retained check alongside the tested candidates.
- **`frontend/src/evidence.js:56` — `goalCheck` keeps only the last `goal.checks` (CONFIRMED, introduced, warning).**
  - **Effect:** in a goal over several repositories, the first repository's acceptance head and command disappear from the panel.
  - **What reaches it:** `fleet.rs:2634`, which loops `for repo in &repos` and records one `goal.checks` per repository with merged work. Case: `:63`.
  - **Fix:** take every `goal.checks` from the latest acceptance pass.
- **Residue, no case: `crates/control-plane-app/tests/console_evidence_page_attack.rs:224` — the re-pinned assertion proves only half of pass 1's intent (CONFIRMED, introduced, note).**
  - The re-pinned assertion checks the candidate, which is in `assignments[].test_revision` in the JSON. The command is still unrecoverable once the history drops `checks.run`. The view says so in words rather than showing it.
  - The server was unchanged, so this assertion could not have failed before the correction either.
  - Treating the command as optional is now an acceptance decision. No new defect.

**5. Attacked, could not break**
- `test_revision` semantics: it is set only after both check runs pass. Every repair (`rebased` and `applied`) clears it. A block keeps it, which is true because those tests did pass. Claim happens only from Queued, where `test_revision` is `''`, so a stale candidate from an earlier attempt never shows as passed.
- Matching `checks.run` to its assignment and candidate: the first-commit `checks.run` never matches. `.at(-1)` takes the final one.
- The decode fallback: it cannot inject anything into the fetch URL. The raw segment is re-encoded, and the URL parser normalises dot segments before `main.js` reads the path.
- Large JSON: the view renders no raw JSON, and the timeline is bounded at 64 entries.

**6. Paths written outside the worktree:** none. Logs (`p2-*.log`) and the npm cache are under `<tree>/.scratch`. I removed `frontend/node_modules`. I took no worktree lease.

```findings
- file: frontend/src/Evidence.vue
  line: 14
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the Latest check panel suppresses latestCheck whenever any assignment has a test_revision or a goal check exists, so a later story's failed check on a new candidate is hidden behind an older passed one
- file: frontend/src/evidence.js
  line: 56
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: goalCheck keeps only the last goal.checks, so a goal over several repositories shows one repository's acceptance head and drops the others recorded in the same pass
- file: crates/control-plane-app/tests/console_evidence_page_attack.rs
  line: 224
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the re-pinned assertion checks only test_revision, which the unchanged server always carried, so the command half of the pass-1 intent is not asserted anywhere and the view only states that it was lost
```
