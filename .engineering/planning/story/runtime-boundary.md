---
format: aep.planning-md/3
id: story:runtime-boundary
kind: story
status: draft
title: Run every agent through the governed Loom boundary
summary: Remove private turn loops and duplicated governor behavior while preserving workspace/UI orchestration; fix foundation composition gaps at their owners.
relations:
- decomposes: epic:bootstrap
- informed_by: architecture-design:runtime-ownership
- serves: vision:autonomous-engineering
- blocks: story:planner-feedback-recovery
scope:
- confidence: inferred
  path: crates/control-plane-protocol/
- confidence: inferred
  path: crates/control-plane-runtime/
- confidence: inferred
  path: docs/
- confidence: inferred
  path: ess/
revision: 2
---
# Restore the runtime ownership boundary

## Outcome

The operator gets planner, implementor and reviewer runs through the existing governed Loom foundation, with usable tool feedback, durable budgets and visible runtime events, while retaining all registered workspaces and product data.

## Basis

Follow architecture-design:runtime-ownership. The two external auth-eval failures and the direct fleet model loop demonstrate the current mismatch. Local recovery patches are provisional regression evidence; moving more generic execution behavior into this product is not completion.

## Scope and owners

Control-plane owns workspace/goal APIs, run scheduling, contention, operator authority inputs, provider/store/effect bindings and UI projections. Loom owns turn/context/session/budget machinery. Its Commission runtime owns invocation/revalidation. Its governor owns frontier/completion with Canon. Engineering protocols own reusable planning/source-delivery semantics. ESS and AEP own specification and planning behavior. Worktree, Eventlog, LLM and execution substrate retain their existing responsibilities.

Resolve foundation gaps through their repositories first: a demonstrated governed native-session bridge, a fallible durable CaseStore, and admitted planning/source-delivery profiles. Do not weaken authority, independent review, source revision binding or publication reconciliation to fit an existing example. Do not start a new repository or discard the current host.

## Acceptance

- shared-governed-executor
- missing-file-refusal-roundtrip
- no-change-is-not-progress
- stale-authority-before-effect
- session-budget-restart
- storage-failure-stops-effects
- visible-runtime-events
- current-review-and-publication

The architecture artifact defines each scenario's observable result. First specify any changed public entities/contracts in their owning ESS domain, then implement the named regressions using the actual SDK and scripted provider responses. Production dependency/architecture checks reject direct model-loop implementation in control-plane after migration. A single bounded real external eval follows those passing checks; failed runs remain failed.

## Sequence

1. Pin the ownership design and reproduce the observed failures at the SDK seams.
2. Deliver any required foundation contract/composition changes in their owner, with their own ESS and gates.
3. Bind both roles through the shared runtime; preserve the existing UI, API, data, worktree records and publication reconciliation.
4. Replace handcrafted activity/session data with actual runtime events and run the integration scenarios.
5. Run one bounded auth example through its existing trusted verifier. Report exact result and spend evidence, including unknowns.

## Out of scope

A second control-plane, replacement agent framework, vendor-harness orchestration, release/deployment, and repeated paid retries while deterministic seam checks fail.
