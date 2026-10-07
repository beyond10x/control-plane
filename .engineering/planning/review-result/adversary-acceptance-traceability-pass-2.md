---
format: aep.planning-md/3
id: review-result:adversary-acceptance-traceability-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:acceptance-traceability (180d232): red, 5 introduced'
relations:
- reviews: story:acceptance-traceability
revision: 1
---
unit: story:acceptance-traceability
verdict: red
cases: executed 180→184, red 3
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: /dev/shm/cp-wave2-acceptance-traceability-scratch/p2/ (24 files); probe test binaries in the assigned build dir /dev/shm/cp-wave2-acceptance-traceability-target
needs-coordinator: no

The findings cover head 180d232 plus one new, uncommitted test file. All three red cases pass against dea51c3, so the 180d232 fixes introduced them. Nothing in today's store or crates reaches any of them.

**1. Diff**

`git --no-pager diff --stat` is empty. `git status --short`:
```
?? crates/control-plane-xtask/tests/acceptance_traceability_pass2_attack.rs
```
It is the only file left, and it is a test file. `cargo fmt --check` and `clippy -D warnings` both pass on it.

**2. Cases** (`crates/control-plane-xtask/tests/acceptance_traceability_pass2_attack.rs`)

| case | asserts | now | at dea51c3 |
|---|---|---|---|
| `unbalanced_fence_does_not_exempt_a_story` | `check` refuses two stories: one with an unclosed fence, one with a four-backtick fence closed by three. Each has an Acceptance section naming a missing test. | red | green |
| `fence_in_a_list_item_ends_with_the_item` | a fence inside a list item ends when the item ends, as CommonMark (and markdown-it) has it, so the next bullet's name is extracted | red | green |
| `test_in_an_invoked_macro_still_resolves` | a `#[test]` inside a `macro_rules!` body that is invoked still resolves | red | green |
| `fence_edges_follow_commonmark` | covers a decoy `## Acceptance` in an earlier fence, four backticks holding ``` and `# `, a tilde fence holding ```, an info-string line, and indented ``` and ~~~ fences in a list item. markdown-it renders it the same way. | green | red |

Red output from the first run, which executed only this file:
```
---- fence_in_a_list_item_ends_with_the_item stdout ----
  left: ["first_listed_scenario"]
 right: ["first_listed_scenario", "second_listed_scenario"]
---- test_in_an_invoked_macro_still_resolves stdout ----
  left: {"ordinary_scenario_runs"}
 right: {"invoked_macro_scenario", "ordinary_scenario_runs"}
---- unbalanced_fence_does_not_exempt_a_story stdout ----
the gate passed two checked stories whose Acceptance names no test: Ok("0 scenario names in 2 active and implemented stories resolve to tests")
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Mutant probe.** I made five one-line mutants as scratch copies, so the worktree was not edited. Each mutant was compiled in through temporary `#[path]` test files, which I deleted afterwards.
- m1 drops the `length >= opened` check (line 157).
- m2 lets any marker close a fence (line 157).
- m3 drops the info-string check (line 157).
- m4 drops `trim_start` (line 140).
- m5 drops `!code` (line 124).

The unit's suite stays at 3 of 3 and the pass-1 attack file at 7 of 7 under every mutant. `fence_edges_follow_commonmark` fails under all five. I widened that case with an indented `~~~` block after a first run showed m4 survived it, because backtick-span skipping masked an indented ``` fence.

**3. Suite** (run after the cases existed)

`cargo test --locked -p control-plane-xtask -p control-plane-app -p control-plane-runtime --no-fail-fast` exited 101.
- Every other binary is green: 30 + 0 + 3 + 38 + 23 + 36 + 14 + 3 + 7 + 4 + 2 + 12 + 8 = 180. That matches the implementor's r1 gate.
- `acceptance_traceability_pass2_attack`: `test result: FAILED. 1 passed; 3 failed`.
- I deleted the worktree's `.scratch/` afterwards.

**4. Findings** (the unit's base, bc1efdf, has no `acceptance.rs`, so everything below is `introduced`)

| file:line | finding | verdict | what reaches it |
|---|---|---|---|
| src/acceptance.rs:124 | A fence that is never closed, or four backticks closed by three, hides `## Acceptance`. `unwrap_or_default` (line 97) then exempts the story, and the gate reports success. dea51c3 refused the same story. Fix: refuse a checked story whose body ends inside a fence, or that has no Acceptance section. | INFEASIBLE, warning | Nothing: none of the 28 stories has a fence. |
| src/acceptance.rs:156 | A fence opened inside a list item never ends when the item ends, so every later name in the section is silently dropped. | INFEASIBLE, note | Nothing found. |
| src/acceptance.rs:302 | Every `macro_rules!` body is skipped, invoked or not, so a test that `cargo test` runs gets a false refusal. Fix: skip only macros the file never invokes, or resolve against `cargo test -- --list`. | INFEASIBLE, note | The only macro, `storage!` (core/memory.rs:359), contains no `#[test]`. |
| src/acceptance.rs:157 | The suite survives all five fence mutants above. | CONFIRMED, note | Any later edit to `Fence`. |
| src/generation.rs:237, src/target.rs:297, README.md:161 | The brief lists these as "not yours" and asks for a patch instead. The renames were committed, and the patch also sits in scratch. There is no merge conflict: terminal-goal-edits changes README lines 133–139 only. | CONFIRMED, note | The wave-2 merge. |

**5. Attacked and could not break**
- **Fences:** four-backtick fences, tilde versus backtick markers, info-string lines, indented fences in list items and a decoy heading inside a fence are all handled correctly.
- **cfg:** none of these hide a real test: `#[cfg(test)]` modules, `#[cfg(not(any()))]`, `#[cfg(all())]`, `#[cfg(any(unix, windows))]`, `#![cfg(test)]`, a test after a switched-off `use`, `const`, `impl` or `mod x;`, or a test after a module switched off with `#![cfg(any())]`.
- **JS division versus regex:** division after `)`, `]`, a name or a number is not read as a regex. Regex literals after `(`, `?`, `:`, `!` and `return`, and ones holding `/` and quotes inside a class or escaped, are read correctly. Comments containing quotes are skipped, and a title on its own line is found. The sibling unit's real `frontend/src/App.test.js` yields exactly `app_renders_a_snapshot_payload`.
- **Backticked spans:** 14 spans without spaces give no names: paths, artifact ids, flags, `KEY=value`, turbofish, attributes and `path:line`.
- **Live store:** 19 checked stories give 78 names, identical to dea51c3.
- **Real crates:** the resolver finds 230 names, identical to dea51c3. For xtask, app and runtime this equals `cargo test -- --list` exactly: 175 names, with no difference in either direction. For core and protocol it equals a grep: 55 names.

**6. Paths written outside the worktree**
- Scratch: `/dev/shm/cp-wave2-acceptance-traceability-scratch/p2/` holds logs (`alone.log`, `probe.log`, `probe2.log`, `probe3.log`, `probe-m4.log`, `suite.log`, `fmt.log`, `clippy.log`), code copies (`old.rs`, `m1.rs` to `m5.rs`, `probe.sh`), markdown-it inputs (`c1.md`, `c2.md`, `c2b.md`, `edges.md`) and name lists (`listed.txt`, `resolver.txt`, `only-resolver.txt`, `grep-core.txt`).
- Build dir: `/dev/shm/cp-wave2-acceptance-traceability-target/debug/deps/` gained 40 entries for the `zz_probe_*` and `zz_pass2_probe` binaries, about 2 MB each.
- Lease: my session lease `adversary-acceptance-traceability-pass-2` was taken and released through `worktree hook`.
- Inside the worktree: 35 temporary `tests/zz_*.rs` files were created and deleted.

```findings
[
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 124, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "An unclosed fence, or a four-backtick fence closed by three, hides the Acceptance heading and the gate passes a checked story naming absent tests, which dea51c3 refused."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 156, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A fence opened inside a list item stays open past the end of the item, so names in later bullets are silently dropped where CommonMark lists them."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 302, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "Every macro_rules body is skipped whether invoked or not, so a test an invoked macro expands to no longer resolves and is falsely refused."},
  {"file": "crates/control-plane-xtask/src/acceptance.rs", "line": 157, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Five one-line mutants of the fence code (lines 124, 140 and 157) leave the unit suite and the pass-1 attack suite green; fence_edges_follow_commonmark fails under each."},
  {"file": "crates/control-plane-xtask/src/generation.rs", "line": 237, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Test renames were committed in generation.rs, target.rs and the xtask README, which the brief lists as not the unit's files and asks for as a scratch patch."}
]
```
