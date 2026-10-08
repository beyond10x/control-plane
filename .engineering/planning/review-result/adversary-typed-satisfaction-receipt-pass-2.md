---
format: aep.planning-md/3
id: review-result:adversary-typed-satisfaction-receipt-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:typed-satisfaction-receipt on 5623c54 plus working tree'
relations:
- reviews: story:typed-satisfaction-receipt
revision: 1
---
Adversary pass 2 on story:typed-satisfaction-receipt, uncommitted working tree on base 5623c54 in tree cp-wave7-typed-satisfaction-receipt-2, after the mutation exemption was added. Scope: replay of a recorded stale-revision refusal, the runtime path against the console path, the three changed attack test files, and the mutation exemption in crates/control-plane-xtask/src/mutation.rs.

```
unit: story:typed-satisfaction-receipt, pass 2, uncommitted working tree on 5623c54
verdict: INFEASIBLE (2 red cases; no reachable defect found)
cases: executed 208→220, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (its TMPDIR, deleted by the coordinator)
needs-coordinator: none
```

Cases added: crates/control-plane-core/tests/typed_satisfaction_receipt_pass2_attack.rs (211 lines; stale_refusal_over_host_refusals_is_recorded_once_and_replays, stale_refusals_around_edits_replay_in_order, both green) and crates/control-plane-xtask/tests/mutation_exemption_attack.rs (79 lines; exemption_attack_duplicate_exempted_mutant_is_refused red, exemption_attack_stillborn_or_equivalent_count_is_refused red, exemption_attack_control_report_is_admitted green).

Red output: `a report exempting the one mutant twice was admitted: Ok(Verdict { mutants: 3, killed: 1, exempt: 2, guard_mutants: 1 })`; `stillborn: 1 beside the exemption was admitted`.

Observation, not a finding: no production caller reaches the generated stale-revision refusal; fleet.rs re-checks the revision under the store lock before SatisfyGoal and fails without recording, so the generated rule is a backstop below it.

```findings
[
  {"file": "crates/control-plane-xtask/src/mutation.rs", "line": 89, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "verdict admits the exempted mutant listed twice and reports exempt: 2, against its documented 0 or 1"},
  {"file": "crates/control-plane-xtask/src/mutation.rs", "line": 111, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "verdict ignores stillborn and equivalent counts and never checks that the counts sum to mutants, yet with the exemption it is the sole judge run_verb consults"}
]
```
