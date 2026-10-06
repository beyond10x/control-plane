---
format: aep.planning-md/3
id: story:acceptance-traceability
kind: story
status: draft
title: Named acceptance scenarios resolve to tests in the gate
relations:
- serves: vision:autonomous-engineering
- informed_by: review-result:plan-audit-2026-10-06
- depends_on: story:spec-history-gate
- decomposes: epic:bootstrap
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/control-plane-xtask/src
revision: 4
---
## Outcome

A scenario name in the Acceptance section of an active or implemented story is the name of a test in crates/, and `task check` says which names are not. This extends epic:bootstrap's "named story scenarios extend generated lifecycle coverage with real adapter tests".

## Evidence

review-result:plan-audit-2026-10-06 item 1: 21 of 52 named scenarios occur nowhere in crates/; three implemented stories carry unmatched names (story:operator-observability 5 of 6, story:verification 3 of 3, story:workspace-directories 1 of 6).

## Scenario name rule

A scenario name is a token in a story's `## Acceptance` section made of three or more lower-case words joined by `_` or `-`, backticked or not (several stories list names in plain prose). A backticked span that contains a space, such as `ess verify conform mutate --emit`, is a command and is skipped. A `-` name matches a test whose name has `_` in its place.

## Acceptance

- `scenario_names_follow_the_rule`: the gate's extraction over the current store returns the 52 names counted in the audit and no backticked command.
- `unresolved_name_fails_gate`: a story naming a scenario absent from crates/ fails the gate, which prints the story id and the name.
- `renamed_tests_resolve`: every unresolved name belonging to a test that exists under another name is resolved by renaming the test in crates/.
- Remaining unresolved names are rewritten in their stories by the coordinator through `aep plan artifact body`; the gate is switched on in Taskfile.yml only after it passes.

## Scope

Cited: Taskfile.yml. Inferred: crates/control-plane-xtask/src/, test files under crates/ whose names change. Story text changes are planning-store writes owned by the coordinator.
