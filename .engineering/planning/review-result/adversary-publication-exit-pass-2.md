---
format: aep.planning-md/3
id: review-result:adversary-publication-exit-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:publication-exit at 2c810ff'
relations:
- reviews: story:publication-exit
revision: 1
---
unit: story:publication-exit, control-plane worktree cp-wave4-publication-exit-p2 at 2c810ff plus 3 test-file changes
verdict: NEEDS-CHANGE
cases: executed 175→180, red 3
origin: introduced 2 / pre-existing 0 / undecided 4
wrote-outside-worktree: 3 harness background-output files under `$HOME/.cache/claude-tmp/`; the worktree session lease (acquired, renewed, released)
needs-coordinator: should a publication closed because its target moved be retried at all? If yes, finding 1 is a behaviour change. If no, it is a README sentence plus cancelling or blocking the assignment with a reason.

**1. Diff proof**

```
 M crates/control-plane-runtime/tests/publication_exit_attack.rs
?? crates/control-plane-core/tests/publication_exit_pass2_attack.rs
?? crates/control-plane-runtime/tests/publication_exit_pass2_attack.rs
```

All three are test files. The rename asked for: `candidate_landing_after_its_close_still_reconciles` is now `candidate_landing_within_the_grace_window_still_reconciles`, with a rewritten doc comment; nothing else in that file changed. Logs are in `.scratch/p2-*.log`; every fixture directory was deleted.

**2. Cases (each run alone before the suite)**

| Case | Asserts | Now |
|---|---|---|
| core `history_recorded_before_publication_exit_still_replays` | the e88ee4f `recorded-history.db` (read with `git show`) opens on 2c810ff and every view equals the e88ee4f views JSON | green |
| runtime `publication_closed_after_its_target_moved_is_retried_with_a_new_intent` | target moves, intent closes with the goal running, the retry creates a second intent | red |
| runtime `grace_period_counts_from_before_a_restart` | grace 4 s, store reopened mid-window, intent closes at the original deadline | green |
| runtime `unreachable_remote_with_changing_diagnostics_appends_once` | 10 ticks over an unreachable remote whose error text changes append at most 1 decision | red |
| runtime `reconfigured_repository_does_not_append_on_every_tick` | after a close, a ConfigureRepository and a resumed goal, 10 ticks append at most 2 decisions | red |

The core case needs object e88ee4f in the clone, so it fails on a shallow clone.

Red output, verbatim:

```
panicked at crates/control-plane-runtime/tests/publication_exit_pass2_attack.rs:339:5:
assertion `left == right` failed: the intent closed because main moved to a7ee3643f1195360407038565c36b334ea7f6dac, and the retry published nothing: the assignment is "Blocked" at attempt 2 ("target base changed; queued plan must be reconciled before another attempt")
  left: 1
 right: 2
```

```
panicked at crates/control-plane-runtime/tests/publication_exit_pass2_attack.rs:473:5:
ten fleet ticks over one unresolved publication whose remote cannot be reached appended 10 progress decisions; newest: "Publication observation unavailable: git [\"ls-remote\", \"--refs\", \"origin\", \"refs/heads/main\"] exited Some(128): ... Failed to connect to git.example.invalid port 443 after 448331455 ms: Could not connect to server ..."
```

```
panicked at crates/control-plane-runtime/tests/publication_exit_pass2_attack.rs:529:5:
ten fleet ticks over an unchanged "Blocked" assignment (attempt 1) appended 20 progress decisions; its reason: "Publication outcome unresolved: publisher returned without an observed merge: candidate not on target"
```

Coordinator recheck: the runtime attack binary rerun as built, with `GIT_CEILING_DIRECTORIES=<tree>/.scratch` and `TMPDIR=<tree>/.scratch/tmp`: `test result: FAILED. 1 passed; 3 failed`, the same three cases at lines 339, 529 and 473.

**3. Gate** (after the cases existed; log `.scratch/p2-gate.log`)

| Step | Exit | Summary (verbatim) |
|---|---|---|
| ess validate | 0 | `controlplane v1 — 3 file(s), valid` |
| generated-check | 0 | `generated contracts match emitter bytes` |
| spec-history-check | 0 | `specification history gate passed: 13 change(s) since baseline eaa37d4057b4c6b90c6cebccefa28dca4abd411e (recorded in ess/spec-acknowledgements.json, on origin/control-plane/bootstrap), 7 could break replay, 7 acknowledged` |
| fmt --check | 0 | (no output) |
| clippy -D warnings | 0 | (no output) |
| cargo test core+runtime | 101 | `error: 1 target failed:` `-p control-plane-runtime --test publication_exit_pass2_attack`, `test result: FAILED. 1 passed; 3 failed` |
| conformance | 0 | `179 scenarios passed; 0 failed/error/unsupported/skipped; coverage qualification: Passed` |

Every other test binary passed. Core: 50+1+2+1+2+1+1+4 = 62. Runtime: 39+31+2+3+36+3+4 = 118. 180 is the total; 175 is the same run without the two new test targets. `restart_open_is_bounded` ran once and passed. `--list` confirms the case names exist in this tree.

**4. Findings**

1. A publication closed because the target moved can never be retried (`fleet.rs:945`). The retry's `RepairAssignment` applies and uses up an attempt; deliver then refuses "target base changed", because no command refreshes `base_revision`. With attempts used up the assignment stays Blocked and holds the repository until an operator cancels it. This contradicts the story title ("closed and retried") and README:33 ("may be attempted again"). Reached by every close under the "moved" rule while the goal runs, and by any grace-period close after the target moved since the claim. `target_moved_past_uncertain_head_closes_intent` pauses the goal before the close, so it never shows the retry. Fix options: (a) README states this, and the close cancels the assignment or blocks it once with a reason instead of retrying; (b) let a repair take a fresh base, a specification change and its own story. The check is unchanged since e88ee4f (read, not run); at base no intent ever closed. NEEDS-CHANGE, undecided.
2. Observation errors whose text changes defeat the deduplication (`fleet.rs:2194`). The blocker carries the raw git stderr and `same_blocker` compares the whole text: 10 decisions in 10 ticks. curl's connect error includes the elapsed time; against a remote host that refuses, that number varies with round-trip time (inferred, not measured without external network; the fixture fakes it with `remote.origin.uploadpack`). Fix: record a stable error class (program and exit code) and keep the raw text out of the comparison. e88ee4f's `block()` appended on every tick (read, not run). INFEASIBLE, undecided.
3. A closed assignment whose repository was reconfigured appends 2 decisions per tick, forever (`fleet.rs:601`, with `guards.rs:306`). Each tick records "Recover interrupted…", then the store refuses with "repository configuration changed; assignment evidence is stale"; the two alternate, so deduplication never matches. Reached by an operator correcting the publish or test command after a not-published close, then resuming the goal. Fix: the fleet skips, or blocks once, an assignment whose admitted configuration differs from the repository's (the check at guards.rs:301-307), and records "Recover…" only after Repair applies. The same loop existed for any Blocked assignment at base (read, not run). CONFIRMED, undecided.
4. The console's "Current reason" stays stale after a close (`frontend/src/App.vue:66`, via `fleet.rs:254`): it still shows "Publication outcome unresolved…" after the intent became NotPublished, because `BlockAssignment` is sent only when the assignment is not already Blocked. CONFIRMED, undecided, note.
5. A queued assignment waiting for the repository records nothing (`fleet.rs:601`). When the holding assignment is used up or its goal is paused, the waiting assignment sits Queued with an empty reason indefinitely. CONFIRMED (read), introduced, note.
6. README:33 does not say the candidate commit itself must reach the target; a squash or rebase publisher's work is recorded NotPublished, where at base it stayed Uncertain. CONFIRMED (read), introduced, note.

**5. Attacked and not broken**

- Replay: history recorded at e88ee4f replays on 2c810ff and all views match.
- Grace across restart: the grace start survives a restart.
- Declared refusals: `not-found` and `wrong-state` come before the reason and receipt checks; the pass-1 core cases are green.
- Slot rule: the fleet's state list equals guards.rs `active()`; no fresh path produces two Blocked assignments on one common directory (read).
- Failed observations: no Err path closes an intent; a rewound target waits for the grace period; "moved" requires `merge-base before head == before`.
- Bounded records: a bounded blocker keeps `uncertain_since` and `uncertain_head` (detail under 1 KiB).
- Activity writers: only the fleet writes assignment activity in production, so the grace start read from the newest blocker is the intent's own.

```findings
[
{"file":"crates/control-plane-runtime/src/fleet.rs","line":945,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"A publication closed because its target moved is never retried; the retry's repair applies, deliver refuses \"target base changed\", attempts are used up and the assignment stays Blocked holding the repository, contrary to the story title and README \"may be attempted again\"."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":2194,"category":"acceptance","severity":"warning","verdict":"INFEASIBLE","origin":"undecided","message":"The observation-error blocker embeds raw git stderr, so a remote whose diagnostics vary between ticks (curl's elapsed ms) appends one progress decision per tick despite an unchanged intent."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":601,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"After a close and a ConfigureRepository, the fleet attempts the stale-evidence assignment every tick and records two alternating blockers per tick forever while the store refuses its repair."},
{"file":"frontend/src/App.vue","line":66,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"undecided","message":"The assignment's Current reason keeps \"Publication outcome unresolved\" after the intent is closed, because BlockAssignment is sent only when the assignment is not already Blocked."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":601,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A queued assignment waiting for a repository held by a used-up or paused Blocked assignment records nothing, so it shows Queued with no reason indefinitely."},
{"file":"README.md","line":33,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The README does not say the candidate commit itself must reach the target, so a squash or rebase publisher's merged work is recorded as not published."}
]
```
