---
format: aep.planning-md/3
id: review-result:control-plane-review-2026-10-06
kind: review-result
status: active
title: 'Source and runtime review at f510e7f: credentials, store growth, publication dead end'
relations:
- reviews: epic:bootstrap
- reviews: story:implementation-fleet
- reviews: story:workspace-host
- reviews: story:runtime-boundary
- reviews: story:operator-console
revision: 1
---
# Control-plane source and runtime review, 2026-10-06

Reviewed commit f510e7f (integration head of `control-plane/bootstrap`). Read: crates/control-plane-core, control-plane-app (Rust side), control-plane-runtime (process, fleet, engine confinement, supervisor scheduling), control-plane-xtask (conformance target), ess/. The frontend was reviewed separately.

Gate at f510e7f: `task check` exit 0 — strict ESS and AEP validation, frontend drift, fmt, clippy, 157 Rust tests, generated drift, 167/167 conformance scenarios, foundation check.

## How the findings were established

- Live store: read-only queries of the running service's state database on 2026-10-06 03:45 CEST: 3,602 host decisions, 812 MB; 3,524 `RecordPlanningProgress` decisions hold 809.8 MB of event data; the largest single decision is 2.44 MB.
- Replay: `Store::open` on a copy of that database took 78.8 s (debug build, the profile the live service runs).
- Live service environment: 21 variables whose names mark credentials (provider API keys, a Git hosting token, chat and issue-tracker tokens, the SSH agent socket). Names only were read.
- Three probe tests, run once and not committed; each failed on f510e7f:
  - `probe_child_processes_do_not_inherit_service_credentials`: a variable set in the parent appears in `env` run through `ProcessRunner`. Passes with `env_clear()` plus `PATH`.
  - `probe_satisfied_goal_keeps_its_acceptance_receipt`: UpdateGoal on a Satisfied goal is applied; state stays Satisfied, objective changes, `satisfaction_receipt` becomes empty. Passes with a one-line guard refusing UpdateGoal outside Paused/Running.
  - `probe_unpublished_intent_has_an_exit`: after MarkPublicationUncertain and BlockAssignment, RepairAssignment, CancelAssignment and ReconcileAssignment are all refused; after CancelGoal, a second workspace's ClaimAssignment on the same repository is refused with "repository already has an active change".

## Findings

1. Credentials reach candidate code. `ProcessRunner::run` adds configured variables on top of the inherited environment and never clears it. The repository test command, the publish command and the eval-only `go test ./...` all execute model-written code, so that code receives every credential of the service process.
2. Progress records grow without bound. `Host::progress` rewrites the whole `planning_receipt` (with a 64-entry activity history) as one `RecordPlanningProgress` decision on every Loom event. History is never compacted, and `Store::open` replays every decision through generated dispatch, deep-cloning memory per decision. Restart now blocks the console for about 79 s and the cost grows with each eval round.
3. A publication the remote never received has no exit. PublicationIntent can only reach Confirmed; Repair and Cancel refuse while any intent exists; Reconcile needs a confirmed receipt; `active()` counts Blocked, so the common-directory slot stays held in every workspace. Only a manual merge of the candidate frees it.
4. Model-input refusals suspend runs. Only `ReadPathSyntax` becomes `EffectOutcome::Refused`. Twelve other checks on model input stay `EffectError`, which Commission treats as external unavailability: too many reads, read over 256 KiB, non-UTF-8 file, unregistered context directory, write outside scope or language policy, write over 256 KiB, delete outside scope or of a missing file, Go argument policy, inspection command grammar, empty Finish summary, changed files outside scope at Finish; plus process timeout, output over 2 MiB and non-UTF-8 output. Eval rounds 3, 5 and 6 each stopped on one member of this class.
5. Terminal goals accept edits (probe above). UpdateGoal declares no wrong_state outcome and `guard` checks only field values.
6. Conformance runs below admission. The 167 scenarios drive `ContractStore`, which calls `Store::apply` and skips `prepare`/`guard`. About 25 admission rules in guards.rs and directories.rs (one running goal per workspace, one active change per common directory, worker and attempt limits, goal revision binding, independent review, merge authority, publication intent before merge, confirmed receipt before completion, satisfaction preconditions, delete preconditions, directory removal with active work) are tested only by hand-written tests and are invisible to ESS mutation.
7. Any spec tightening can make existing state unopenable. Replay refuses a recorded outcome that differs from the current generated behaviour ("migration required"), and nothing in `task check` classifies spec changes.
8. The local API hands its mutation token to any local client. `GET /api/session` returns the token to every request with a loopback Host header. Combined with finding 1 (candidate code runs as the same user with loopback network access), a candidate's tests could set `merge_authority` or rewrite `publish_command`. Inferred chain; each link observed separately. The architecture assigns process confinement to Substrate.
9. Workspace context reads admit secrets. A `workspace:<directory>/file` read admits any non-symlink file outside `.git` under a registered directory, including ignored files, and the content goes to the provider.
10. Stuck publications write every tick. `reconcile_publications` calls `block()` on every fleet tick for an unresolved intent, and `block()` always appends a progress decision.
11. Every committed decision re-snapshots six views per SSE connection; each view query deep-clones the full memory, including multi-megabyte receipts, while holding the store mutex.
12. One child repository whose detached HEAD has an ambiguous base branch fails discovery of its whole directory, and startup registration propagates the error.
13. The Supervisor actor is granted eight commands the runtime never sends as Supervisor, including StartGoal, which the README treats as an operator control.
14. Conformance reports carry the ESS runner's default clock: `completed_at` is 2023-11-14T22:13:20Z plus run steps, and `aep plan artifact evidence --from` takes that as the evidence instant.

## Not covered

Frontend source and visual UX (separate reviews). The planner engine's ESS/AEP effect paths beyond confinement and refusal classification. Loom, Commission and the Canon governor internals.

```findings
[
{"file":"crates/control-plane-runtime/src/process.rs","line":61,"category":"security","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"Child processes inherit the full service environment; test, publish and go test commands run model-written code with every service credential. Probe red, green with env_clear."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":79,"category":"efficiency","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"Every Loom event appends a full planning_receipt copy; live store 812 MB, 3,524 progress decisions hold 809.8 MB; replay of a copy took 78.8 s."},
{"file":"ess/domains/host.yaml","line":299,"category":"correctness","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"A publication the remote never received has no exit: Repair, Cancel and Reconcile refused, repository slot held across workspaces after goal cancel. Probe red."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":1154,"category":"correctness","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Only ReadPathSyntax is a Refused outcome; twelve other model-input checks become EffectError and suspend the run (eval rounds 3, 5, 6)."},
{"file":"crates/control-plane-core/src/guards.rs","line":104,"category":"correctness","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"UpdateGoal applies to Satisfied goals and erases satisfaction_receipt. Probe red, green with a one-line guard."},
{"file":"crates/control-plane-xtask/src/target.rs","line":106,"category":"test-coverage","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Conformance drives ContractStore below guard(); about 25 admission rules are absent from ESS and from the 167 scenarios."},
{"file":"crates/control-plane-core/src/lib.rs","line":214,"category":"compatibility","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Replay refuses any changed outcome and no gate classifies spec changes; spec tightening can make existing state unopenable."},
{"file":"crates/control-plane-app/src/lib.rs","line":306,"category":"security","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"GET /api/session hands the mutation token to any local client; with inherited environment and loopback access a candidate's tests could grant merge authority. Inferred chain."},
{"file":"crates/control-plane-runtime/src/engine.rs","line":1070,"category":"security","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Workspace context reads admit any non-symlink file outside .git under a registered directory, including ignored secret files, and send it to the provider."},
{"file":"crates/control-plane-runtime/src/fleet.rs","line":1726,"category":"efficiency","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"An unresolved publication appends a progress decision on every fleet tick."},
{"file":"crates/control-plane-core/src/lib.rs","line":320,"category":"efficiency","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Each view query deep-clones all memory including multi-megabyte receipts under the store mutex; SSE runs six per committed decision per connection."},
{"file":"crates/control-plane-core/src/discovery.rs","line":36,"category":"correctness","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"One child repository with an ambiguous detached HEAD fails discovery of the whole directory and startup registration."},
{"file":"ess/domains/host.yaml","line":334,"category":"security","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Supervisor holds eight grants the runtime never uses, including StartGoal."},
{"file":"crates/control-plane-xtask/src/target.rs","line":207,"category":"evidence","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Conformance report completed_at is the ESS runner default clock (2023-11-14), which AEP evidence import takes as the run instant."}
]
```
