---
format: aep.planning-md/3
id: review-result:adversary-typed-satisfaction-receipt-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:typed-satisfaction-receipt on 5623c54 plus working tree'
relations:
- reviews: story:typed-satisfaction-receipt
revision: 1
---
Adversary pass 1 on story:typed-satisfaction-receipt, uncommitted working tree on base 5623c54 in tree cp-wave7-typed-satisfaction-receipt-2. The pass stopped before reporting (its last command, a cleanup, never returned) and was stopped by the coordinator; it returned no findings block. Its results are taken from the tree.

```
unit: story:typed-satisfaction-receipt, uncommitted working tree against 5623c54
verdict: green (no report; results read from the tree)
cases: 6 added, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: one mktemp TMPDIR under the cache directory (4 KiB, deleted by the coordinator)
needs-coordinator: no
```

Cases added: crates/control-plane-core/tests/typed_satisfaction_receipt_attack.rs (359 lines), all green: no_malformed_receipt_revision_satisfies_a_running_goal, recorded_stale_refusal_changes_no_view, receipt_and_receipt_revision_must_agree_on_the_current_revision, contract_admission_requires_and_compares_the_revision, every_other_integer_revision_is_stale, finished_or_paused_goals_answer_wrong_state_not_stale_revision (`test result: ok. 6 passed; 0 failed`). The console case live::tests::stale_satisfaction_is_refused_and_the_browser_sees_the_edit passed (1 passed). Areas 3, 4 and 6 of the brief were not reached; pass 2 took them.

```findings
[]
```
