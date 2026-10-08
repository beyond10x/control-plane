---
format: aep.planning-md/3
id: review-result:adversary-spec-owned-admission-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:spec-owned-admission on 61eed7f plus working tree'
relations:
- reviews: story:spec-owned-admission
revision: 1
---
Adversary pass 1 on story:spec-owned-admission, uncommitted working tree on base 61eed7f in tree cp-wave7-spec-owned-admission (implementor correction 1). Report header and findings as returned.

```
unit: story:spec-owned-admission, uncommitted working tree against 61eed7f in cp-wave7-spec-owned-admission
verdict: red (strongest: NEEDS-CHANGE)
cases: executed 184→188, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: yes. The acceptance names a "repository missing" QueueAssignment refusal, but the README records why it cannot be declared. Either the acceptance text is re-scoped or the refusal is declared.
```

Cases added (untracked): crates/control-plane-core/tests/spec_owned_admission_attack.rs (260 lines), crates/control-plane-xtask/tests/spec_owned_admission_attack.rs (27 lines).

| case | asserts | now |
|---|---|---|
| core `non_integer_goal_limits_are_still_refused` | CreateGoal is refused with limits of 1.5, 2.0, u64::MAX, "3" or -0.5, and creates no goal | green |
| core `repair_with_null_base_keeps_its_base` | a repair with `base_revision: null` answers `applied` and keeps the base | green |
| core `history_recorded_before_the_added_input_guards_still_opens` | a history the base commit could record still opens, with a control step | red: `a history the base commit recorded no longer opens: Some(generated behavior disagrees with durable history; migration required)` |
| xtask `queue_assignment_declares_a_missing_repository_refusal` | QueueAssignment has a synthesized refusal for a missing repository | red: `QueueAssignment declares no repository-missing refusal; its outcome scenarios: ["…/created", "…/goal-not-current", "…/goal-not-found"]` |

Suites: `cargo test --locked --no-fail-fast -p control-plane-core` exit 101, passed 69, failed 1; `cargo test --locked --no-fail-fast -p control-plane-xtask -- --skip guard_mutants_are_killed` exit 101, passed 117, failed 1.

Attacked without a break: admission order (no host-refused command applied; subject not-found stays an error); console and runtime both go through `Store::admit`; no runtime or app path sends any of the nine removed Supervisor grants; the 14 `outcome-added` acknowledgements equal the set the new replay test exercises; guard boundaries; negative and non-integer limits refused on `Store::execute`; all 15 missing review rows declared or listed.

What reaches each finding: F1 the story's own acceptance text; F2 no caller found (the runtime sends only existing assignment ids with non-empty receipts; `OPERATOR_COMMANDS` has no Supervisor command), acknowledged as `outcome-added`; F3 the acceptance requires the README to quote the refusal each rule met.

The findings block below is the adversary's corrected response (the first one used field names the store does not read).

```findings
[
{"file":"ess/domains/host.yaml","line":802,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Acceptance queue_guards_are_refused_in_conformance names a declared repository-missing QueueAssignment refusal; only goal-not-found and goal-not-current exist, so either the refusal is declared or the acceptance is re-scoped to match README row 4."},
{"file":"ess/domains/host.yaml","line":1063,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"The added input guards answer before not-found, so a stored decision on an unknown assignment with empty evidence (recorded as not-found at the base) makes Store::open fail with migration required; no caller sending such a command was found."},
{"file":"ess/README.md","line":48,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The quoted ESS refusals 'no candidate of the 41 tried' and 'no candidate of the 4 tried' (line 35) appear in no captured output; a scratch trial of the MergeAssignment evidence guard printed 'no candidate of the 19 tried'."}
]
```
