---
format: aep.planning-md/3
id: story:model-input-refusals
kind: story
status: implemented
title: Every model-input refusal returns to the model instead of suspending the run
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: review-result:control-plane-review-2026-10-06
- informed_by: story:runtime-boundary
- depends_on: story:candidate-process-environment
scope:
- confidence: cited
  path: crates/control-plane-runtime/src/engine.rs
- confidence: cited
  path: crates/control-plane-runtime/src/fleet.rs
- confidence: cited
  path: crates/control-plane-runtime/src/process.rs
- confidence: cited
  path: crates/control-plane-runtime/src/read_request.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/fleet.rs
- confidence: inferred
  path: crates/control-plane-runtime/tests/planner.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T02:29:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}, executor: "agent:claude-review-session"}
- {from: "proposed", to: "active", at: "2026-10-06T02:29:50Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":3}}, executor: "agent:claude-review-session"}
- {from: "active", to: "implemented", at: "2026-10-06T02:50:05Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}, executor: "agent:claude-review-session"}
---
## Outcome

When the host refuses a model action because of what the model asked for (path, size, scope, language policy, command grammar, Finish preconditions, a command that timed out or printed too much), the planner and the implementor both receive a bounded refusal through `EffectOutcome::Refused` and the run continues. Only host failures stay fatal: cancellation, stale goal or authority, symlink or `.git` confinement, storage, provider and process-launch failures.

## Evidence

- review-result:control-plane-review-2026-10-06 finding 4: fleet.rs:1154 turns only `ReadPathSyntax` into a refusal. Fatal today on model input: fleet.rs:1235 (more than 32 reads), 1245 (read over 256 KiB), 1246 (non-UTF-8 file), engine.rs:1083 (unregistered context directory), fleet.rs:1255 and 885 (write outside scope or language policy), 1256 (write over 256 KiB), 1269-1270 (delete outside scope or of a missing file), 1286 (Go arguments), 1290 and 1393 (inspection grammar), 1321 (empty Finish summary), 1328 (changed files outside scope at Finish), process.rs:108 (timeout), 186 (output over 2 MiB), 188 (non-UTF-8 output).
- review-result:plan-audit-2026-10-06 item 7: rounds 3, 5 and 6 each fixed one member of this class. The round-6 proposal in story:runtime-boundary covers the command-grammar member only.

## Acceptance

- `admission_refusals_reach_the_model`: one native-runtime case per refusal decided before execution (every entry above except timeout, output size and output encoding); each yields a Refused observation in the next model turn, starts no process and changes no file.
- `execution_refusals_reach_the_model`: one case each for a command that exceeds its timeout, one that prints over 2 MiB and one that prints non-UTF-8; each yields a Refused observation naming the limit, the command's process group is gone, and the run continues.
- `fifth_consecutive_refusal_blocks_the_attempt`: four consecutive refusals in one attempt continue; the fifth ends the attempt as Blocked with that refusal as the reason; an admitted action in between resets the count.
- `host_failures_stay_fatal`: the existing cancellation, stale-authority, symlink, `.git`, storage and provider regressions still suspend.
- `round_six_gofmt_proposal_recovers`: the captured `gofmt -w` proposal is refused, the scripted next turn proposes an admitted command, and the run reaches independent review.

## Narrowing of the epic's promise

The epic says a refused model action never ends a run. This story keeps one exception: the fifth consecutive refusal in one attempt blocks the assignment (`BlockAssignment`), which then waits for the existing repair path. A model that keeps proposing refused actions would otherwise spend the whole attempt budget on refusals.

## Scope

Cited: crates/control-plane-runtime/src/fleet.rs, crates/control-plane-runtime/src/engine.rs, crates/control-plane-runtime/src/read_request.rs, crates/control-plane-runtime/src/process.rs (typed timeout and output-limit errors). Inferred: crates/control-plane-runtime/tests/fleet.rs, crates/control-plane-runtime/tests/planner.rs.

## Relation to the round-6 proposal

On 2026-10-06 the operator chose this story over the round-6 correction proposed in story:runtime-boundary ("commit + fix" in answer to options A/B/C, A being story:candidate-process-environment then this story). The round-6 proposal is superseded by it; the next eval round runs after both stories pass.

## Implementation notes (2026-10-06)

- `crates/control-plane-runtime/src/refusal.rs` holds the typed `InputRefusal`, the classification `refusal_of` (input refusals, read-path syntax, typed process limits) and the consecutive budget (`CONSECUTIVE_REFUSAL_LIMIT = 5`, reset by any admitted action). Planner and implementor effect ports both return `EffectOutcome::Refused` for these and stay fatal for everything else.
- Two members never reach the host through the native runtime: Loom refuses a 33-path read against the published tool schema (`maxItems: 32`) and any tool arguments over 65,536 bytes, so a write over 256 KiB is refused by Loom. Both arrive at the model as a failed tool result; `admission_refusals_reach_the_model` asserts Loom's text for these two. The host checks stay as a second layer.
- The timeout member is exercised below the native runtime by `execution_limits_are_typed_and_stop_the_process_group` (process.rs): a fixture command cannot be made slow without also slowing the host's own checks. Output size and encoding run natively in `execution_refusals_reach_the_model`.
- The implementor's instructions and every command refusal carry the admitted command grammar (`COMMAND_GRAMMAR`); `command_grammar_examples_are_admitted` keeps its examples tied to the validator.
- Planner Finish now checks the story selection before it spends a review attempt, and an unchanged revision after a rejected review is a refusal rather than a fatal error.
- An unregistered `workspace:<directory>` read is now a refusal: it reads nothing, as the parent-traversal refusal approved in round 5. Symlink, `.git` and changed-root failures stay fatal.
