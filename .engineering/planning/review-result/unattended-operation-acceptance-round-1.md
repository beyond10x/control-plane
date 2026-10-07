---
format: aep.planning-md/3
id: review-result:unattended-operation-acceptance-round-1
kind: review-result
status: active
title: 'Plan critic (acceptance), round 1: epic:unattended-operation decomposition'
relations:
- reviews: epic:unattended-operation
- reviews: story:candidate-process-environment
- reviews: story:model-input-refusals
- reviews: story:bounded-progress-records
- reviews: story:spec-history-gate
- reviews: story:terminal-goal-edits
- reviews: story:publication-exit
- reviews: story:spec-owned-admission
- reviews: story:acceptance-traceability
- reviews: story:conformance-evidence-record
revision: 1
---
needs-revision
story:model-input-refusals — the acceptance says each refusal "starts no process and changes no file", but the listed refusals include a command that timed out and one that printed over 2 MiB, and both have already run a process, so the scenario cannot pass as written — .engineering/planning/story/model-input-refusals.md:39
story:model-input-refusals — `input_refusals_share_one_budget` says "a stated number of consecutive refusals", but no number is stated anywhere, so a budget of 10,000 passes it — .engineering/planning/story/model-input-refusals.md:40
story:bounded-progress-records — `progress_record_size_is_bounded` compares growth to "a stated per-event cap" that is never stated, so any cap passes and the bound is not checkable — .engineering/planning/story/bounded-progress-records.md:39
story:bounded-progress-records — `restart_open_is_bounded` does not say how the 10,000 decisions are built or how large each is, so it reads the same before and after the work (the 78.8 s open came from 812 MB of large receipts, not from the decision count) — .engineering/planning/story/bounded-progress-records.md:40
story:publication-exit — the last bullet joins three outcomes (new state and command declared "first", generated conformance passes, spec-history-gate acknowledges), and "first" is an ordering no one can observe afterwards — .engineering/planning/story/publication-exit.md:45
story:spec-history-gate — `spec_diff_gate_refuses_unacknowledged_change` packs three independent outcomes into one scenario (unacknowledged change fails naming the id, listed id passes, stale id fails), so one can pass while another fails — .engineering/planning/story/spec-history-gate.md:35
story:spec-owned-admission — "the conformance target executes commands through the admitted Store path" describes the implementation and names no output or refusal that tells the current ContractStore path apart from the admitted one — .engineering/planning/story/spec-owned-admission.md:44
story:conformance-evidence-record — "byte-reproducible apart from its timestamps, or the deviation is stated" passes whatever the report does, and it is a second outcome the story's Outcome never names — .engineering/planning/story/conformance-evidence-record.md:28
story:acceptance-traceability — `acceptance_names_resolve_to_tests` joins building the gate with clearing the 21-name backlog "by renaming tests or rewriting the story text", but the Scope says story text changes belong to the coordinator, and it gives no rule for what counts as a scenario name (Acceptance sections also hold backticked commands such as `ess verify conform mutate --emit`) — .engineering/planning/story/acceptance-traceability.md:29

What I read: all 10 ids (9 stories and dependency-blocker:aep-imports-ess-053-suites) with `aep plan artifact show`. I also ran `aep plan artifact kinds` and `aep plan artifact lifecycle story`, read both cited review-results, and checked `process.rs:100-190` and `git grep` for the probe and scenario names.

What I could not establish:
- The three `probe_*` tests named in Evidence are not in the tree (`git grep probe_` at f510e7f finds nothing), so "the probe sequence" in terminal-goal-edits and publication-exit rests on the prose at `.engineering/planning/review-result/control-plane-review-2026-10-06.md:27-29`. I did not count that as a finding.
- I did not re-verify the "21 of 52" count or the `fleet.rs` line references.
- Out of my lane, not counted in the verdict: scope for the drafts that overlap the round-6 proposal in story:runtime-boundary, and the draft copies under `.engineering/drafts/` that duplicate these stories (scope or parallel-safety).

```findings
- file: .engineering/planning/story/model-input-refusals.md
  line: 39
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the acceptance says each refusal "starts no process and changes no file", but the listed refusals include a command that timed out and one that printed over 2 MiB, and both have already run a process, so the scenario cannot pass as written'
- file: .engineering/planning/story/model-input-refusals.md
  line: 40
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`input_refusals_share_one_budget` says "a stated number of consecutive refusals", but no number is stated anywhere, so a budget of 10,000 passes it'
- file: .engineering/planning/story/bounded-progress-records.md
  line: 39
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`progress_record_size_is_bounded` compares growth to "a stated per-event cap" that is never stated, so any cap passes and the bound is not checkable'
- file: .engineering/planning/story/bounded-progress-records.md
  line: 40
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`restart_open_is_bounded` does not say how the 10,000 decisions are built or how large each is, so it reads the same before and after the work (the 78.8 s open came from 812 MB of large receipts, not from the decision count)'
- file: .engineering/planning/story/publication-exit.md
  line: 45
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the last bullet joins three outcomes (new state and command declared "first", generated conformance passes, spec-history-gate acknowledges), and "first" is an ordering no one can observe afterwards'
- file: .engineering/planning/story/spec-history-gate.md
  line: 35
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`spec_diff_gate_refuses_unacknowledged_change` packs three independent outcomes into one scenario (unacknowledged change fails naming the id, listed id passes, stale id fails), so one can pass while another fails'
- file: .engineering/planning/story/spec-owned-admission.md
  line: 44
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"the conformance target executes commands through the admitted Store path" describes the implementation and names no output or refusal that tells the current ContractStore path apart from the admitted one'
- file: .engineering/planning/story/conformance-evidence-record.md
  line: 28
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"byte-reproducible apart from its timestamps, or the deviation is stated" passes whatever the report does, and it is a second outcome the story''s Outcome never names'
- file: .engineering/planning/story/acceptance-traceability.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`acceptance_names_resolve_to_tests` joins building the gate with clearing the 21-name backlog "by renaming tests or rewriting the story text", but the Scope says story text changes belong to the coordinator, and it gives no rule for what counts as a scenario name (Acceptance sections also hold backticked commands such as `ess verify conform mutate --emit`)'
```
