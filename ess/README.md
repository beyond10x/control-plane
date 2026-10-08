# Control-plane specification

`domains/host.yaml` declares the host's entities, commands, refusals and views. The Rust model,
the OpenAPI contract and the conformance suite in `generated/` come from it
(`cargo run --locked -p control-plane-xtask -- generate`). Conformance runs the whole synthesized
suite against the durable generated store (`contract::ContractStore`), below the host rules. The
host's admission (`Store::admit` in `crates/control-plane-core/src/lib.rs`: the grant, the host
rules below, then the generated decision) is held to the declared QueueAssignment refusals by
`queue_guards_are_refused_in_conformance` (`crates/control-plane-xtask/src/target.rs`).

## Declared admission rules

The rows of the design review (review-result:ess-design-review-2026-10-06) that the
specification declares, each with synthesized scenarios:

| row | rule | declared as |
|---|---|---|
| 5 | a queued assignment names a running goal at its current revision | QueueAssignment `goal-not-found`, `goal-not-current` |
| 6 | review uses an execution context other than the implementor's | ReadyAssignment `reviewer-missing`, `review-not-independent` |
| 7 | tests and review cover the assignment's candidate | ReviewAssignment `tests-not-current`, ReadyAssignment `evidence-not-current` |
| 10 | completion and reconciliation carry a merge receipt | CompleteAssignment and ReconcileAssignment `receipt-missing` |
| 11 | goal satisfaction names the goal's current revision | SatisfyGoal `stale-revision`, when `receipt_revision` is given |
| 17 | worker, attempt and minute limits are positive | CreateGoal `workers-invalid`, `attempts-invalid`, `minutes-invalid` |

## Host facts

Rules the host enforces in `crates/control-plane-core/src/guards.rs` and `directories.rs`
because ESS 0.56.0 (`ess/22`) cannot declare them with synthesized scenarios, or because the
generated Rust cannot implement the declaration. Each quotation is what ESS printed in a trial
of this specification with the rule added; the trials and their full output are kept with the
unit's working records (`host-facts/<row>.txt`). A candidate count depends on the trial: it is
quoted with the trial that printed it.

| row | rule kept by the host | why the specification does not hold it |
|---|---|---|
| 1 | one running goal per workspace (StartGoal) | trial `1-startgoal-busy`: the selector `goal_id != subject.goal_id and state == Running and workspace_id == subject.workspace_id` validates, and synthesis answers "`(goal_id != subject.goal_id and state == Running and workspace_id == subject.workspace_id)` reads a subject field the steps leave undetermined` is `row set`, which has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch", refusing StartGoal/applied and every scenario after it |
| 2 | one active change per Git common directory, across workspaces (ClaimAssignment, RepairAssignment) | trials `2-common-dir-stamped` and `2-common-dir-input`: beside ClaimAssignment's declared evidence guard the selector is refused at validation, "`controlplane.host.ClaimAssignment` selects on a row set (`when_related: {entity, where, …}`) and on a `when_subject` guard; one command reads one kind of related row in this cut" (ESS-COMMAND-009). Without that guard: trial `2-common-dir-stamped-alone` (`common_dir` stamped on Assignment from the repository) answers "reads a subject field the steps leave undetermined" (152 scenarios, 45 refusals); trial `2-common-dir-input-alone` (`common_dir` as a QueueAssignment input) answers for the busy branch "its own row set is False` is `row set`, which has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch" (197 scenarios, 6 refusals) |
| 3 | at most the goal's worker limit occupied (ClaimAssignment, RepairAssignment from Blocked) | trial `3-worker-limit`: `count: {gte: subject.max_workers}` is refused at validation, "invalid type: string "subject.max_workers", expected i64"; a count bound is a literal |
| 4 | attempt budget; a queued assignment names a registered, enabled repository of the goal's workspace (repository missing, repository disabled) | trial `4-attempt-limit` (the goal's limit stamped on Assignment, `when_subject: attempt >= max_attempts`): "no candidate of the 8 tried satisfies ``controlplane.host.Assignment` stored attempt,max_attempts selecting this branch, over the rows 8 bounded arrangements left`" (count varies by trial). Trial `4-repository-disabled` (`when_related: {via: input.repository_id, predicate: state == Disabled}` beside the goal guards) is refused at validation: "`controlplane.host.QueueAssignment` reads the fields of the row `input.repository_id` names, and a missing row makes every predicate over it unknown: no branch answers when that row does not exist" (ESS-COMMAND-005). Trial `4-repository-missing` (`when_related: {via: input.repository_id, exists: false}` beside the goal guards) validates and synthesizes, but `ess generate synthesize --target rust` prints "136 capabilities: 135 generated, 1 obligation(s), 0 refused" and keeps QueueAssignment an obligation, class `undetermined`, for "`when_related:` reading several related rows in one command"; `xtask generate` then stops with "generation left unmet capabilities". Repository missing is to be declared by story:admission-conformance-target. The time budget is enforced by the runtime, not at admission |
| 5 | assignment commands act only while their goal is running at the revision they were queued under | the goal is selected by `subject.goal_id`, a selector comparing with a subject field (the form of rows 1 and 2), and on QueueAssignment trial `5-selector-beside-identity` shows a selector beside the declared goal guards refused at validation: "`controlplane.host.QueueAssignment` selects on a row set (`when_related: {entity, where, …}`) and on an identity-addressed `when_related`; one command reads one kind of related row in this cut" (ESS-COMMAND-009) |
| 8 | merging and preparing publication need the goal's merge authority | the goal is selected by `subject.goal_id` (row 5) |
| 9 | merging needs a prepared publication intent for the candidate, base and target | a selector over PublicationIntent comparing with subject fields (rows 1 and 2) |
| 10 | the merge receipt is the receipt of a confirmed publication of the candidate | a selector over PublicationIntent comparing with subject fields (rows 1 and 2); the empty receipt is declared |
| 11 | goal satisfaction: a receipt, no unfinished assignment; a new satisfaction of a Running goal names `receipt_revision`, and its receipt names the same `goal_revision` | `receipt_revision` is optional so that SatisfyGoal decisions recorded before it existed replay with their recorded answers, so the specification cannot require it of a new call; the receipt stays text (decision-blocker:typed-receipt-replay), so its `goal_revision` is not a field ESS can read; unfinished assignments are a selector `goal_id == subject.goal_id` over Assignment (rows 1 and 2) |
| 14 | a goal with assignment history is not deleted | a selector `goal_id == subject.goal_id` over Assignment (rows 1 and 2) |
| 15 | one registration per canonical workspace path, answered with its first receipt | canonical path discovery reads the filesystem, and a repeat answers the recorded receipt rather than a refusal |
| 16 | a workspace directory with active assignments or planning is not removed | the directory's repositories come from a filesystem scan (`repository_common_dirs`), and the rule spans Directory, RepositoryRegistration, Assignment and Goal |
| 17 | UpdateGoal limits and non-blank objective and acceptance | an input guard answers before the held state, so it would replace the declared `satisfied` and `cancelled` refusals of a finished goal; ESS compares text exactly, so a blank (whitespace-only) text is a host rule on CreateGoal too |

Rules 6 and 7 are also checked again by the host before merging and publication. ESS cannot
synthesize that repeat: trial `6-7-merge-evidence` (the evidence check declared on
MergeAssignment) answers "no candidate of the 41 tried satisfies ``controlplane.host.Assignment`
stored candidate,implementor_run,review_revision,reviewer_run,test_revision selecting this
branch, over the rows 41 bounded arrangements left`" (count varies by trial): once
ReadyAssignment refuses stale evidence, no ReadyToMerge assignment holds it.
