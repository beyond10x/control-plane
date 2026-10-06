---
format: aep.planning-md/3
id: review-result:adversary-goal-acceptance-authority-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:goal-acceptance-authority at 65cba8c'
relations:
- reviews: story:goal-acceptance-authority
revision: 1
---
unit: story:goal-acceptance-authority; findings cover 65cba8c plus one untracked test file I added
verdict: CONFIRMED
cases: executed 154→157, red 3
origin: introduced 0 / pre-existing 0 / undecided 2
wrote-outside-worktree: 2 paths (my log dir under the assigned scratch, and the assigned build dir); scratch `tmp/` is empty
needs-coordinator: both findings fall outside the story's acceptance statement. The code behind each one is unchanged since 9691f41 (I read it with `git show`; I did not run it there). Decide whether they hold this unit or become their own story. My three red cases keep `cargo test -p control-plane-runtime` red until you decide.

**1. Diff stat** (worktree `$HOME/.local/state/worktree/trees/b10x/control-plane/cp-wave3-goal-acceptance-authority`)
```
$ git --no-pager diff --stat
(empty: no tracked file changed)
$ git status --short
?? crates/control-plane-runtime/tests/goal_acceptance_authority_pass2_attack.rs
```
The only path is a new test file (583 lines). No implementation file and no existing test was touched.

**2. Cases added** (each was written before anything ran, and each was run alone)

The fixture has two workspaces. Each has one repository and one Running goal whose single assignment merges in the first fleet tick. Case 3 uses one workspace.

| Case | Asserts | Now |
|---|---|---|
| `editing_another_goal_during_acceptance_does_not_fail_the_fleet_tick` | During goal A's review, an operator UpdateGoal moves goal B to revision 2. Preconditions (all passed): the edit was applied, B was Running at revision 1 with its assignment Merged at revision 1, A comes first in GoalList, and A ends Satisfied. The case then asserts `fleet_tick` returns Ok and B is re-planned at revision 2. | red |
| `editing_another_goal_during_acceptance_keeps_the_service_running` | The same edit under `Supervisor::run`. Asserts the service keeps running until B is planned at revision 2. | red |
| `transient_origin_failure_after_goal_review_does_not_latch_acceptance` | `origin` is unreachable only during the goal review, so the post-review `ls-remote` fails. After `origin` is restored, two ticks plus fleet ticks must satisfy the unchanged goal. | red |

Red output, each case alone (EXIT=101 for each). Line numbers moved by 5 after I added `fixture_with`; the assertions did not change.
```
panicked at …/goal_acceptance_authority_pass2_attack.rs:394:9:
an operator edit of one goal during another goal's acceptance failed the whole fleet tick: goal changed during acceptance
test result: FAILED. 0 passed; 1 failed; … finished in 4.21s

panicked at …/goal_acceptance_authority_pass2_attack.rs:438:17:
the service stopped before shutdown, after an operator edit of one goal during another goal's acceptance (edit {"outcome":"applied",…}): Err(goal changed during acceptance)
test result: FAILED. 0 passed; 1 failed; … finished in 4.28s

panicked at …/goal_acceptance_authority_pass2_attack.rs:575:5:
assertion `left == right` failed: origin answers again and nothing changed, yet acceptance never ran again (1 goal reviews); acceptance {"goal_revision":1,"status":"failed"}
  left: String("Running")
 right: "Satisfied"
```
For case 3, the recorded reason in the fixture store is `Goal acceptance blocked: git ["ls-remote", "--refs", "origin", "refs/heads/main"] exited Some(128) … does not appear to be a git repository`. So it failed for the reason claimed.

**3. Suite** (run after the cases existed)

`cargo test --locked --no-fail-fast -p control-plane-core -p control-plane-runtime` → EXIT=101
```
core: lib 44 | goal_acceptance_authority_attack 1 | recorded_history 2 | recorded_history_pass2_attack 1 | terminal_goal_edits_attack 1 | terminal_goal_edits_pass2_attack 4   (all ok)
runtime: lib 38 ok | fleet 25 ok | goal_acceptance_authority_attack 2 ok | goal_acceptance_authority_pass2_attack 0 passed; 3 failed (:575, :399, :443, same messages) | planner 36 ok | doc-tests 0, 0
```
The 154 "before" is this same run with my binary's 3 cases left out. I did not make a separate deselected run. rustfmt `--check` and clippy `-D warnings` on my file both exit 0.

**4. Findings** (these cover 65cba8c)

| file:line | Finding | Verdict | Origin | Severity |
|---|---|---|---|---|
| `crates/control-plane-runtime/src/fleet.rs:2013` | `satisfy_goals` reads GoalList once (:1961) and then runs each goal's acceptance in turn; one acceptance can take minutes. If an operator edits goal B during goal A's acceptance, B's first write, `acceptance_progress(…"running")?`, is refused ("goal changed during acceptance", :118). The `?` fails `satisfy_goals` and the fleet tick, and `Supervisor::run` returns Err (supervisor.rs:51). The app then reports "Autonomous processing stopped … Restart the service" (app lib.rs:234-240). This contradicts the correction's claim that an acceptance ended by a revision change is recorded as `interrupted`. | CONFIRMED | undecided | warning |
| `crates/control-plane-runtime/src/fleet.rs:2128` | A transient failure of the post-review `git ls-remote` is classified `failed`, with nothing about the goal, repository, directories or target changed. The latch (`acceptance_is_unchanged`, :728-741) then holds the goal Running for good: there is 1 review, and no edit or input change ever comes. This is a fourth latching cause beyond the three the correction lists and the model/provider limit it names. | CONFIRMED | undecided | warning |

- **Finding 1, measured:** both two-goal cases are red, :399 and :443.
- **Finding 1, what reaches it:** two workspaces, each with a Running goal whose work is all merged in the same tick, and an operator UpdateGoal on the second goal through the console's "Edit goal" or the CLI during the first goal's acceptance.
  - With a single goal, the same `?` fires if the edit lands during `acceptance_fingerprint`'s `ls-remote` (:2008). I did not test this.
  - The Err arm has the same mechanism in a smaller window: `now` is read at :2197, then `acceptance_progress` re-locks and checks the revision at :2213/:2215. I did not test this either.
  - Fix: treat `acceptance_progress`'s revision refusal as "goal changed" and skip the goal, at both sites, instead of `?`.
- **Finding 2, measured:** case 3 is red at :575.
- **Finding 2, what reaches it:** any network blip to `origin` after the review. The same applies by reading to fetch (:2051), worktree create and lease heartbeat (:2064-2066, :566), and a check that hits the process timeout.
  - Fix: latch only on a review verdict. Record host-observation failures as `interrupted`.

**5. Attacked and could not break**
- **Locked re-check:** goal state, revision, members and SatisfyGoal all run under one `&mut Store` guard (:2145-2177). The unit's edit test fails if the in-lock goal check is removed.
- **False refusal from the members comparison:** none found. No fleet write changes repository or directory rows (`managed_common_dirs` is set only in directories.rs:74), and view order is keyed by BTreeMap (memory.rs:13-18).
- **Shutdown versus deadline:** the outer token is a child of the supervisor's cancel token (supervisor.rs:61) and is cancelled only on shutdown (:52). The deadline and the lease heartbeat cancel only the worker's child token, so they record `failed` as claimed. The shutdown test fails if the cancel term is removed.
- **Consumers of `interrupted`:** both latches (:735, :2010) react only to `failed`. The dashboard badge shows it as neutral, and the frontend never reads the acceptance status.
- **Re-review on every tick:** none found. Non-Running goals are skipped (:1963), an edited goal has no merged work at its new revision (:1978), and the members snapshot is re-read every tick.
- **Cancel or delete during acceptance:** CancelGoal records `interrupted` without crashing, because RecordPlanningProgress has no state guard. DeleteGoal is refused when assignment history exists (guards.rs:86-94).
- **Inert terms (label only, not findings):** the members and revision terms in `interrupted` change only the label. A membership change already changes the fingerprint, and a changed revision already skips the record (:2211).

**6. Paths written outside the worktree**
- `/dev/shm/cp-wave3-goal-acceptance-authority-scratch/adversary-pass2/logs/`: case1-alone, case2-alone, case3-alone, suite, fmt and clippy logs.
- `/dev/shm/cp-wave3-goal-acceptance-authority-target`: the assigned build dir (my test binary and clippy metadata).
- `/dev/shm/cp-wave3-goal-acceptance-authority-scratch/tmp`: used as TMPDIR, now empty.
- `<worktree>/.scratch`: I removed the run directories (`fleet`, `planner`, `goal-acceptance-attack`, `goal-acceptance-pass2`); it is empty.
- Worktree lease `adversary-goal-acceptance-authority-w3p2`: acquired and released.

**7. Findings block**
```findings
- file: crates/control-plane-runtime/src/fleet.rs
  line: 2013
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: An operator edit of one goal during another goal's acceptance makes the edited goal's acceptance_progress refuse on the stale GoalList snapshot, and the `?` fails the fleet tick and ends Supervisor::run, so autonomous processing stops instead of recording an interruption.
- file: crates/control-plane-runtime/src/fleet.rs
  line: 2128
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: A transient failure of the post-review ls-remote is recorded as a failed acceptance for unchanged inputs, so the durable latch keeps the goal Running for good after origin recovers.
```
