---
format: aep.planning-md/3
id: review-result:adversary-acceptance-traceability-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:acceptance-traceability (dea51c3): red, 6 introduced'
relations:
- reviews: story:acceptance-traceability
revision: 1
---
unit: story:acceptance-traceability
verdict: red
cases: executed 173→180, red 5
origin: introduced 6, pre-existing 0, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-acceptance-traceability-scratch/{attack-alone.log, live.out, live.err, fns.txt, suite.log, mutant-status/, tmp/}; /dev/shm/cp-wave2-acceptance-traceability-scratch/mutant-target/ (created, then deleted)
needs-coordinator: no

Findings cover head dea51c3 plus one new, uncommitted test file.

**1. What I touched**

`git --no-pager diff --stat` is empty, so no tracked file changed. `git status --short`:
```
?? crates/control-plane-xtask/tests/acceptance_traceability_attack.rs   (166 lines, test only)
```
Every path I added is a test file.

**2. Cases added**

All are in `crates/control-plane-xtask/tests/acceptance_traceability_attack.rs`. I wrote them first and ran them alone (`cargo test --locked -p control-plane-xtask --test acceptance_traceability_attack`) before running anything else:

| case | asserts | now |
|---|---|---|
| `fenced_shell_comment_does_not_end_acceptance` | a `# ` line inside a fenced block does not end the Acceptance section | red |
| `single_quoted_status_is_still_checked` | a story with `status: 'active'` is checked | red |
| `tests_the_compiler_never_builds_do_not_resolve` | a `#[test]` under `#[cfg(any())]` or inside a `macro_rules!` body that is never expanded does not resolve | red |
| `backticked_name_with_call_parentheses_is_a_name` | `` `goal_drives_plan()` `` gives the same name as the plain `goal_drives_plan()` | red |
| `quote_in_a_regex_literal_neither_hides_nor_invents_titles` | `/'/` in a JS test neither hides the next title nor invents one from a string | red |
| `proposed_and_archived_stories_are_not_checked` | only `active` and `implemented` stories are checked | green; red against the mutant below |
| `names_in_markdown_decorations_are_extracted` | names in nested bullets, links, bold and wrapped items, with Acceptance as the last section | green |

Red output from that first run, verbatim:
```
---- backticked_name_with_call_parentheses_is_a_name stdout ----
  left: []
 right: ["goal_drives_plan"]
---- fenced_shell_comment_does_not_end_acceptance stdout ----
assertion `left == right` failed: section seen by the extractor: "\n```console"
  left: []
 right: ["name_listed_after_the_fence"]
---- quote_in_a_regex_literal_neither_hides_nor_invents_titles stdout ----
  left: ["apostrophe_regex_scenario", "phantom_title_scenario"]
 right: ["apostrophe_regex_scenario", "title_after_the_regex"]
---- tests_the_compiler_never_builds_do_not_resolve stdout ----
  left: ["disabled_module_scenario", "disabled_function_scenario", "unexpanded_macro_scenario", "compiled_scenario_runs"]
 right: ["compiled_scenario_runs"]
---- single_quoted_status_is_still_checked stdout ----
  left: []
 right: ["story:quoted"]
test result: FAILED. 2 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Mutant probe.** I copied `acceptance.rs` into `scratch/mutant-status/` and changed line 86 to `if status != "draft" {`. The worktree was not touched. Against that copy, the unit's `acceptance_traceability.rs` passed 3 of 3. My status case failed:
```
  left: ["story:active", "story:archived", "story:dquoted", "story:implemented", "story:proposed"]
 right: ["story:active", "story:dquoted", "story:implemented"]
```

**3. Suite run (after the cases existed)**

Command: `cargo test --locked -p control-plane-xtask -p control-plane-app -p control-plane-runtime --no-fail-fast`, exit 101.
- Every binary passed except mine: app 30, app operator_boundary 3, runtime 38, fleet 23, planner 36, xtask 14, acceptance_traceability 3, generation_ownership 4, record_history_attack 2, spec_history_attack 12, spec_history_pass2_attack 8.
- `acceptance_traceability_attack`: `test result: FAILED. 2 passed; 5 failed`.
- The 173 "before" is the sum of the other binaries in this run. It equals the implementor's `green-summary.txt`.
- I deleted the worktree's `.scratch/` (375M) afterwards, as the brief says.

**4. Findings**

| file:line | finding | verdict / origin | what reaches it |
|---|---|---|---|
| src/acceptance.rs:121 | The section ends at any line starting `# ` or `## `, even inside a fence. Names after the fence go unchecked with no error. | INFEASIBLE / introduced | Nothing found: none of the 28 stories has a fence. |
| src/acceptance.rs:107 | Only `"` is stripped from the status. A story with `'active'` is skipped silently. | INFEASIBLE / introduced | Nothing found: all 28 status lines are unquoted. |
| src/acceptance.rs:250 | The resolver reads source text, so tests that never compile still resolve. The fix is to resolve against `cargo test -- --list`. | INFEASIBLE / introduced | Nothing found: crates/ has no `cfg(any())`, `cfg(feature` or `cfg_attr`. The only `macro_rules!` (core/memory.rs) has no `#[test]`. |
| src/acceptance.rs:141 | A backticked span is judged whole, so `` `name()` `` is dropped while plain `name()` is kept. | INFEASIBLE / introduced | Nothing found in the live Acceptance sections. |
| src/acceptance.rs:429 | The JS scanner has no regex-literal state. A quote in a regex hides a real title and can invent one, which means a silent pass. | INFEASIBLE / introduced | No `*.test.js` exists at dea51c3. story:console-component-tests adds the first ones. |
| src/acceptance.rs:86 | Status filtering is only tested for `draft`. The unit's suite passes with `status != "draft"`. | CONFIRMED / introduced | Any later edit to this filter. |

**5. Attacked and could not break**
- **Renames:** all 12 diffs change only the `fn` line, so the test bodies are unchanged. No old name is referenced anywhere except the historical review-result. All 12 new names appear in active or implemented stories.
- **Live store:** the check extracts 78 names. My hand recount of every Acceptance section also gives 78, so no names are invented or missed. Of the 22 unresolved, 18 need story text and 4 are expected after merge, matching the commit message.
- **Several unresolved names map to one test that covers two of them**, so no single rename can resolve them: `review_is_independent` and `two_repository_goal_delivery`; `model_wait…` and `planner_activity…`; `non_git…` and `duplicate…`. These are the coordinator's story rewrites, not a defect.
- **Rust test detection:** `#[test]` separated by other attributes or doc comments, `#[tokio::test(flavor=…)]`, nested modules, comments, and raw or plain strings are all handled. `#[ignore]` tests resolve; that fits the rule's wording ("a Rust test function") and the unit asserts it.
- **Frontend titles:** `describe` and `test.skip` are not counted, which is correct. `it.only` is not counted either, but that can only cause a loud false refusal. A template literal without `${` is counted.
- **CRLF or a BOM:** by reading line 100 (not run), the check fails closed with "has no story header".
- **Exit status:** the live run exits 1 and prints `<story id>: <name>` lines to stderr. The success path exits 0 (the unit's own test).

**6. Paths written outside the worktree**
- `/dev/shm/cp-wave2-acceptance-traceability-scratch/`: `attack-alone.log`, `live.out`, `live.err`, `fns.txt`, `suite.log`, `mutant-status/` (the mutant crate), `tmp/`.
- `/dev/shm/cp-wave2-acceptance-traceability-scratch/mutant-target/`: created for the mutant build, already deleted.

```findings
[
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 121, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A line starting with '# ' inside a fenced block ends the Acceptance section, so names listed after the fence are silently unchecked."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 107, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A single-quoted YAML status such as 'active' is not unquoted, so that story is skipped without error."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 250, "category": "acceptance", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A test under #[cfg(any())] or inside an unexpanded macro_rules body resolves although it never compiles; resolving against cargo test --list would not."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 141, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A backticked name with call parentheses is dropped while the same text in plain prose yields the name."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 429, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A quote inside a JS regex literal is read as a string opener, hiding the next real test title and inventing one from a later string literal."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 86, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The unit's suite stays green when the status filter is loosened to status != draft; only draft is covered, and proposed and archived are not."}
]
```
