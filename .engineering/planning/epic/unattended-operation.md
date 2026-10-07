---
format: aep.planning-md/3
id: epic:unattended-operation
kind: epic
status: draft
title: 'Unattended operation: credential isolation, recoverable refusals, bounded state, spec-owned admission'
summary: Make goals safe to run without an operator watching, from the 2026-10-06 review and hardening findings.
relations:
- serves: vision:autonomous-engineering
- informed_by: review-result:control-plane-review-2026-10-06
- informed_by: review-result:ess-design-review-2026-10-06
- informed_by: review-result:plan-audit-2026-10-06
- informed_by: verification-report:ess-hardening-2026-10-06
revision: 3
---
## Outcome

The control plane can run goals unattended:

- model-written code never receives the service's credentials, except the variables the operator explicitly names for the publish command;
- a model action the host refuses for its input returns to the model instead of ending the run (five consecutive refusals in one attempt block the assignment);
- each progress record has a fixed size limit, and restart replay stays fast;
- a publication the remote never received never locks a repository;
- the admission rules the product depends on are declared in ESS and exercised by conformance, behind a gate that keeps the existing state store openable.

## Why now

review-result:control-plane-review-2026-10-06, review-result:ess-design-review-2026-10-06 and verification-report:ess-hardening-2026-10-06 record the evidence. In short:

- Processes that run candidate code inherit every credential of the service process (probe red).
- The live state store reached 812 MB in one night, 99.99 % of it progress records; restart replay takes 78.8 s.
- A publication the remote never received leaves its repository locked in every workspace (probe red).
- Eval rounds 3, 5 and 6 each stopped on one member of one class: a refused model input classified as external unavailability. Twelve members remain.
- About 25 admission rules live only in host code; conformance runs below them and mutation finds no guard to test.

## Stories and order

| story | size | depends on (shared file) |
|---|---|---|
| story:candidate-process-environment | S | — |
| story:spec-history-gate | S | — |
| story:model-input-refusals | M | candidate-process-environment (process.rs, fleet.rs) |
| story:terminal-goal-edits | S | spec-history-gate (gate must exist before a spec change) |
| story:bounded-progress-records | M | model-input-refusals (fleet.rs) |
| story:publication-exit | M | bounded-progress-records (fleet.rs `reconcile_publications`), terminal-goal-edits (host.yaml) |
| story:spec-owned-admission | L | publication-exit (host.yaml, guards.rs; core lib.rs via bounded-progress-records) |

Every pair of stories that touch the same file is ordered by a `depends_on` path; `aep plan artifact waves --kind story --status draft` derives the waves. The first wave is candidate-process-environment and spec-history-gate.

## Constraints

Those of epic:bootstrap. Process and filesystem confinement stays with Substrate (architecture-design:runtime-ownership); these stories narrow exposure, they do not claim a sandbox. Every specification change runs behind story:spec-history-gate so the existing store stays openable. Each story gets its own operator approval before implementation, as the eval-round workflow in story:runtime-boundary requires.

## Out of scope

Deleting or rewriting the existing live state store; release or deployment; process confinement; UI layout work; acceptance-name traceability and conformance evidence import (story:acceptance-traceability and story:conformance-evidence-record, under epic:bootstrap).
