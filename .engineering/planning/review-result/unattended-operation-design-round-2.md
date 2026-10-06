---
format: aep.planning-md/3
id: review-result:unattended-operation-design-round-2
kind: review-result
status: active
title: 'Plan critic (design), round 2: epic:unattended-operation decomposition'
relations:
- reviews: epic:unattended-operation
- reviews: story:candidate-process-environment
- reviews: story:model-input-refusals
- reviews: story:bounded-progress-records
- reviews: story:spec-history-gate
- reviews: story:terminal-goal-edits
- reviews: story:publication-exit
- reviews: story:spec-owned-admission
revision: 1
---
approve

What I read: the epic and the seven stories with `aep plan artifact show`, plus `review-result:unattended-operation-design-round-1`. I also ran `relations`, `graph` (the whole store, including edges outside the set), `validate` ("50 artifact(s) … valid"; nothing for me to relay) and `waves --kind story --status draft`.

Round-1 findings, each checked:
- **Edge for `reconcile_publications`:** fixed. `story:publication-exit` now has `depends_on story:bounded-progress-records`. `bounded-progress-records.md` Evidence says the per-tick append belongs to `publication-exit`, and that story's acceptance `unchanged_unresolved_publication_appends_once` owns it. Each half of the abstraction now sits in one item.
- **Hidden dependency on the gate:** fixed. The bounded-progress body now says any new stored record type becomes a separate ESS story ordered after `story:spec-history-gate`, so it no longer depends on that gate.
- **"Disjoint chains" sentence:** fixed. The epic table now gives a shared-file reason per edge, and its first wave matches the derived waves.

Shape of the set:
- **No cycle.** The `depends_on` edges are cpe → (none), mir → cpe, bpr → mir, tge → shg, pe → {tge, bpr}, soa → pe. Nothing returns to an item already passed.
- **No chain without a reason.** The longest path is five of seven items (cpe → mir → bpr → pe → soa). Every edge has its shared file written beside it in the epic table. The files are `process.rs`, `fleet.rs`, `host.yaml` and `guards.rs`. The `waves` collision list shows no pair left unordered inside the set.
- **No split abstraction.** The slices are by outcome, not layer. Each can be shown working on its own: credential isolation, refusals, bounded records, publication exit, terminal edits, the gate and admission.

What I could not establish:
- **Unease, not a finding:** `story:spec-owned-admission` is L and still bundles declaring the guards, re-pointing conformance at the admission path, the mutation gate and the grants trim. I cannot name a seam that has to be cut.
- **Out of my lane (parallel-safety):** `story:spec-history-gate` lists the whole `ess` directory as scope, which overlaps `ess/domains/host.yaml` in three later stories. Those stories depend on it directly or transitively, so the order is already declared.
- **Out of my lane (scope):** the bounded-progress "separate ESS story if a new record type is needed" is conditional and not yet drafted. I treat that as outside this set.

```findings
[]
```
