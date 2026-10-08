---
format: aep.planning-md/3
id: story:spec-owned-admission
kind: story
status: active
title: Admission rules are declared in ESS and exercised by conformance
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:ess-design-review-2026-10-06
- informed_by: verification-report:ess-hardening-2026-10-06
- depends_on: story:publication-exit
scope:
- confidence: cited
  path: crates/control-plane-core/src/directories.rs
- confidence: cited
  path: crates/control-plane-core/src/guards.rs
- confidence: cited
  path: crates/control-plane-core/src/lib.rs
- confidence: cited
  path: crates/control-plane-xtask/src/target.rs
- confidence: inferred
  path: ess/components.yaml
- confidence: cited
  path: ess/domains/host.yaml
- confidence: inferred
  path: ess/spec-acknowledgements.json
- confidence: inferred
  path: generated
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T03:13:39Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-08T03:13:39Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The admission rules the product relies on are declared in the specification where ESS 0.53 can express them, conformance runs through the same admission path as the console and runtime, the rules that stay host-only are listed with their reason, and the Supervisor holds only the grants the runtime uses.

## Evidence

- review-result:ess-design-review-2026-10-06: 16 missing rows, each enforced today in crates/control-plane-core/src/guards.rs or directories.rs; row 18: eight Supervisor grants with no design statement and no runtime caller.
- crates/control-plane-xtask/src/target.rs:106 drives `ContractStore`, which calls `Store::apply` below `prepare`/`guard` (crates/control-plane-core/src/lib.rs:112-135).
- verification-report:ess-hardening-2026-10-06, technique 1: 213 mutants, none in the guard classes, because the specification declares no input or stored-field guard.
- ESS 0.53 offers `when:`, `when_subject:` and `when_related:` (`exists`, `count`, `forall`, related fields and held state).

## Acceptance

- `admission_rules_are_declared`: each missing row of the design review is either a declared guard with synthesized scenarios, or listed in a host-facts section of the specification's README with the reason (for example canonical path discovery).
- `second_running_goal_is_refused_in_conformance`: the synthesized scenario that sends StartGoal for a workspace that already has a Running goal passes against the conformance target and observes the declared refusal; run against today's `ContractStore` target the same scenario fails.
- `guard_mutants_are_killed`: `ess verify conform mutate --emit` / `--collect`, run through the xtask runner, reports guard-class mutants and no survivor.
- `supervisor_grants_are_least_privilege`: the Supervisor's `may` list equals the set of commands the runtime executes as Supervisor at the time of the change.
- `recorded_history_replays` stays green, or each change id is acknowledged.

## Open question

Resolved by a coordinator trial on 2026-10-08 against ESS 0.56.0 (`ess/22`), in a scratch copy of `ess/`:

- A selector over Assignment rows is accepted: `when_related: {entity: controlplane.host.Assignment, where: {all: [assignment_id != subject.assignment_id, {state: {in: [Implementing, Reviewing, ReadyToMerge, Merging, Blocked]}}, common_dir == subject.common_dir]}, exists: true}` on ClaimAssignment.
- It does not validate while `common_dir` lives only on RepositoryRegistration: `[unobservable_fact] ... common_dir reads common_dir, which is not a declared observable root` and `subject.common_dir: the subject has no field common_dir`.
- It validates (`controlplane v1 — 3 file(s), valid`, `--strict-requires`) once Assignment declares `common_dir: String` and QueueAssignment's `created` outcome sets it with `common_dir: {related: {via: input.repository_id, field: common_dir}}`.

Decision: Assignment carries the common directory, stamped when it is queued; the selector above is the declared refusal. Not yet checked: recorded Assignment rows that predate the field (`recorded_history_replays`) and whether conformance synthesis covers the selector branch.

## Scope

Cited: ess/domains/host.yaml, crates/control-plane-core/src/guards.rs, crates/control-plane-core/src/directories.rs, crates/control-plane-core/src/lib.rs, crates/control-plane-xtask/src/target.rs. Inferred: ess/components.yaml, generated/.
