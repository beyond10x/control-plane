---
format: aep.planning-md/3
id: review-result:adversary-repair-on-moved-target-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:repair-on-moved-target on 16bcc9d plus working tree'
relations:
- reviews: story:repair-on-moved-target
revision: 1
---
unit: story:repair-on-moved-target, worktree cp-wave5-repair-on-moved-target, uncommitted on base 16bcc9d
verdict: NEEDS-CHANGE
cases: 6 total, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: none (a disposable copy of 16bcc9d under the unit tree's `.scratch/base-16bcc9d`, with its own 709M `target/`)
needs-coordinator: no

The pass was interrupted once after five cases had each run red, and resumed from the untracked attack file; the resumed pass reran the five, added the sixth and classified all six.

## Cases (crates/control-plane-runtime/tests/repair_on_moved_target_attack.rs, untracked in the unit tree, all red)

Classification: the same file run in a copy of 16bcc9d stops cases 1 to 5 at attempt 1 with the wave-4 block reason "The target main moved from … since this attempt started", so the new merge path is never reached there; case 6 passes there.

1. `conflicting_moved_target_is_not_attempted_on_every_tick`: a target change on the candidate's own line. The assignment is Blocked at attempt 4 of 4 and the implementor was asked 0 more times; each tick spent an attempt on a merge that conflicts.
2. `failed_attempt_leftovers_do_not_wedge_the_repair_on_a_moved_target`: a failed attempt leaves uncommitted writes and the target changes the same file. Blocked at attempt 3 of 3: "Your local changes … would be overwritten by merge", then "There is no merge to abort (MERGE_HEAD missing)".
3. `integration_merge_carries_no_change_of_its_own`: a failed attempt leaves uncommitted writes and the target changes another file. The merge commit "Integrate the target's current head" carries the leftover implementation change (`git add --all` before the commit).
4. `rewound_target_is_not_republished_with_the_commit_it_dropped`: the target is rewound while a publication is unresolved. The retry takes the rewound head as its base, skips the merge because the head is an ancestor, and publishes a candidate that still carries the commit the target dropped.
5. `failed_integration_commit_leaves_no_merge_in_progress`: the commit command fails once during the integration. The merge stays in progress, and the next attempt fails on "You have not concluded your merge (MERGE_HEAD exists)" until attempts run out.
6. `squashed_candidate_is_not_published_again_as_an_empty_change`: the publisher squashed the candidate onto the target. The retry merges the squash and publishes an integration merge that changes nothing against its expected base; a merging publisher would record the story merged by a commit carrying none of its work.

## Gate after the cases

validate 0, generated-check 0, spec-history-check 0 (4 changes since 6341398, 3 could break replay, 2 acknowledged), fmt 0, clippy 0, test 101 (the attack binary: 0 passed, 6 failed; three frontend_check binaries failed 18 cases on "frontend/node_modules is missing", the unit tree's setup, not this change), conformance 0 (180 scenarios passed).

## Findings

1. crates/control-plane-runtime/src/fleet.rs:1131, NEEDS-CHANGE, blocker, introduced (case 6). After a squashed publication closes, the retry publishes a candidate with no change against its base, and its review reads an empty diff. Fix named: block when the candidate changes nothing on the new base.
2. crates/control-plane-runtime/src/fleet.rs:1088, CONFIRMED, warning, introduced (case 1). A conflicting moved target spends an attempt per tick until max_attempts, where wave 4 blocked once without spending one. Fix named: check mergeability before the repair.
3. crates/control-plane-runtime/src/fleet.rs:577, CONFIRMED, warning, introduced (case 2). A failed attempt's uncommitted writes make the merge fail on every attempt, and the abort of a merge that never started fails too.
4. crates/control-plane-runtime/src/fleet.rs:586, CONFIRMED, note, introduced (case 3). `integrate` stages everything, so the integration merge carries a failed attempt's leftover change.
5. crates/control-plane-runtime/src/fleet.rs:1088, CONFIRMED, warning, introduced (case 4). A rewound target is taken as the new base and the retried candidate still carries the commit the target dropped. Fix named: take a new base only when it descends from the old one.
6. crates/control-plane-runtime/src/fleet.rs:587, CONFIRMED, warning, introduced (case 5). A failed commit during `integrate` leaves the merge in progress, contrary to its doc comment.

## Held

The planner fingerprint: every repair also increments `attempt`, so a new base adds no planner run that `applied` did not already cause, and re-planning skips stories that already have an assignment (supervisor.rs:446-458). Replay of a stored `rebased` repair: `repair_takes_a_new_base_only_when_it_names_one` checks the row after a restart, and the reopened store in publication_exit_pass2_attack.rs keeps the new base. Another story landing on the target merges cleanly.

```findings
[
{"file":"crates/control-plane-runtime/src/fleet.rs","line":1131,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"after a squashed publication closes, the retry merges the squash and publishes an integration merge with no change against its expected base, reviewed on an empty diff"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":1088,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"a conflicting moved target spends an attempt on every tick without asking the implementor until max_attempts is exhausted, where wave 4 blocked once without spending one"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":577,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"uncommitted writes left by a failed attempt make the merge onto a moved target that touches the same file fail on every attempt, and the abort of a merge that never started fails too"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":586,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"integrate stages everything, so the merge commit named Integrate the target's current head carries a failed attempt's leftover implementation change"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":1088,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"a rewound target is taken as the new base, and the retried candidate still carries the commit the target dropped"},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":587,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"a commit command that fails during integrate leaves the merge in progress, contrary to integrate's doc comment, so the next attempt fails on MERGE_HEAD as well"}
]
```
