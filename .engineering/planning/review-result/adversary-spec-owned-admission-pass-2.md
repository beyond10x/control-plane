---
format: aep.planning-md/3
id: review-result:adversary-spec-owned-admission-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: story:spec-owned-admission on 61eed7f plus working tree'
relations:
- reviews: story:spec-owned-admission
revision: 1
---
Adversary pass 2 on story:spec-owned-admission, uncommitted working tree on base 61eed7f in tree cp-wave7-spec-owned-admission (implementor correction 2). Report header and findings as returned.

```
unit: story:spec-owned-admission (uncommitted working tree on 61eed7f, cp-wave7-spec-owned-admission)
verdict: red
cases: executed 191→198, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: no
```

Cases added (untracked): crates/control-plane-core/tests/spec_owned_admission_pass2_attack.rs (1), crates/control-plane-xtask/tests/spec_owned_admission_pass2_attack.rs (3; it compiles src/mutation.rs through #[path] with the runner stubbed).

Red, each alone (exit 101):
- core `acknowledged_reasons_that_cite_the_fixture_are_in_the_fixture` :87: `7 of 7 acknowledgements say "the re-recorded fixture records this refusal", and the committed fixture records none of these: [CreateGoal attempts-invalid, CreateGoal minutes-invalid, CreateGoal workers-invalid, QueueAssignment goal-not-current, QueueAssignment goal-not-found, ReadyAssignment evidence-not-current, ReadyAssignment review-not-independent]`
- xtask `verdict_refuses_a_report_whose_mutant_survived` :51: `a report listing mutant "guard-boundary/…/attempts-invalid/0" as survived was accepted: Ok(Verdict { mutants: 12, killed: 12, guard_mutants: 12 })`
- xtask `verdict_refuses_counts_that_disagree_with_the_mutants_listed` :66: `a report listing 1 mutant while counting 12 was accepted`
- xtask `verdict_refuses_an_audit_that_killed_no_guard_mutant` :87: `an audit with 12 stillborn and 0 killed mutants was accepted`

Suites: core exit 101, 71 run, 1 failed; xtask exit 101, 127 run, 3 failed; `task mutation` exit 0, `12 mutant(s), 12 killed, 12 of the guard classes; no survivor`, 7m43s.

Guard coverage: uncovered parts are an empty ClaimAssignment `worktree_id` and an empty ReviewAssignment `candidate`; Ready `evidence-not-current`'s candidate parts are unreachable (only `review` enters Reviewing and it requires an equal non-empty pair). Every other guard part is covered.

Attacked without a break: stored histories from the base open; new guards answer after not-found and before wrong-state; the CI mutation job copies the existing setup and fails on a survivor; mutate cleanup on `?` and panic, `--keep` keeps; all 13 README quotations match `.scratch/host-facts/`; the 23 acknowledged ids equal the non-compatible ids of `ess verify diff --compatibility`.

```findings
[
{"file":"ess/spec-acknowledgements.json","line":96,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Seven outcome-added reasons claim the re-recorded fixture records the refusal, but recorded-history.db is unchanged since 17cce0b and records none of the seven outcomes."},
{"file":"crates/control-plane-xtask/src/mutation.rs","line":49,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"verdict judges only the counts object, so guard_mutants_are_killed accepts a report that lists a survived mutant or lists fewer mutants than it counts."},
{"file":"crates/control-plane-xtask/src/mutation.rs","line":61,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"verdict accepts an audit with every guard mutant stillborn and none killed, which task mutation relies on alone unless ess --collect exits non-zero."},
{"file":"ess/domains/host.yaml","line":867,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"No test or scenario sets an empty ClaimAssignment worktree_id or an empty ReviewAssignment candidate (line 906) alone, so dropping either guard part survives."}
]
```
