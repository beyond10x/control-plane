---
format: aep.planning-md/3
id: review-result:unattended-operation-design-round-1
kind: review-result
status: active
title: 'Plan critic (design), round 1: epic:unattended-operation decomposition'
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
story:bounded-progress-records — the body edits `reconcile_publications` (fleet.rs:1684-1735, the unresolved-publication tick) and so does story:publication-exit, which changes that same tick to close the intent, and no edge orders them; add `depends_on story:publication-exit`, or move the unresolved-publication append change into story:publication-exit so each half of that abstraction lives in one item — .engineering/planning/story/bounded-progress-records.md:42 and :51, .engineering/planning/story/publication-exit.md:36
story:bounded-progress-records — its "ESS first" section requires a possible new host.yaml record noun to pass story:spec-history-gate, but only `depends_on story:model-input-refusals` is declared, so the gate it names is a hidden dependency; add `depends_on story:spec-history-gate` — .engineering/planning/story/bounded-progress-records.md:47 and `aep plan artifact graph`
epic:unattended-operation — the body says the two chains "touch disjoint files and can run side by side", but story:bounded-progress-records (runtime chain) lists fleet.rs `reconcile_publications`, ess/domains/host.yaml and crates/control-plane-core/src/lib.rs, which story:publication-exit and story:spec-owned-admission (specification chain) also list, so the sentence has to be corrected to match whichever edges the revision adds — .engineering/planning/epic/unattended-operation.md:44

What I read: 11 artifacts (the epic, the 9 stories and the blocker), plus review-result:ess-design-review-2026-10-06 in full. Commands run: `aep plan artifact show` on each, `relations`, `graph` and `validate`. `validate` reported "46 artifact(s) … valid", so nothing it reports counts as a finding. I walked all 130 or so edges in the whole store graph, including edges to artifacts outside the set. There is no cycle. The `depends_on` edges among the nine form two chains (3 deep and 4 deep) plus two leaves, so the set is not a queue. Each chain edge carries a shared-file reason in the epic table (.engineering/planning/epic/unattended-operation.md, Stories table). I confirmed against the code that `reconcile_publications` is at fleet.rs:1684 and that the Ok(None) branch is where both stories act.

What I could not establish:
- Parallel-safety, not mine and not setting the verdict: story:spec-owned-admission (target.rs:106) and story:conformance-evidence-record (target.rs:207) both edit crates/control-plane-xtask/src/target.rs with no edge. story:bounded-progress-records and story:spec-owned-admission both edit crates/control-plane-core/src/lib.rs.
- Acceptance, not mine and not setting the verdict: story:spec-owned-admission's `supervisor_grants_are_least_privilege` (design review row 18, a spec-only row) is absent from its Outcome, and it depends on the set of commands the runtime sends. That set can change under story:bounded-progress-records and story:publication-exit.
- Unease, not a finding: story:spec-owned-admission is L and combines declaring 16 guards, re-pointing conformance at the admission path, a mutation gate and a grants trim. I could not name a seam that has to be cut, so I leave it.

```findings
- file: .engineering/planning/story/bounded-progress-records.md
  line: 42
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the body edits `reconcile_publications` (fleet.rs:1684-1735, the unresolved-publication tick) and so does story:publication-exit, which changes that same tick to close the intent, and no edge orders them; add `depends_on story:publication-exit`, or move the unresolved-publication append change into story:publication-exit so each half of that abstraction lives in one item'
- file: .engineering/planning/story/bounded-progress-records.md
  line: 47
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'its "ESS first" section requires a possible new host.yaml record noun to pass story:spec-history-gate, but only `depends_on story:model-input-refusals` is declared, so the gate it names is a hidden dependency; add `depends_on story:spec-history-gate`'
- file: .engineering/planning/epic/unattended-operation.md
  line: 44
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the body says the two chains "touch disjoint files and can run side by side", but story:bounded-progress-records (runtime chain) lists fleet.rs `reconcile_publications`, ess/domains/host.yaml and crates/control-plane-core/src/lib.rs, which story:publication-exit and story:spec-owned-admission (specification chain) also list, so the sentence has to be corrected to match whichever edges the revision adds'
```
