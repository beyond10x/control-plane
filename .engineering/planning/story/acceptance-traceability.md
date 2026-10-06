---
format: aep.planning-md/3
id: story:acceptance-traceability
kind: story
status: implemented
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
  path: crates/control-plane-app/tests
- confidence: inferred
  path: crates/control-plane-runtime/tests
- confidence: inferred
  path: crates/control-plane-xtask/src
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T05:17:10Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T05:17:10Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T06:33:37Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":4,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

A scenario name in the Acceptance section of an active or implemented story is the name of a test in the repository, and `task check` says which names are not. This extends epic:bootstrap's "named story scenarios extend generated lifecycle coverage with real adapter tests".

## Evidence

review-result:plan-audit-2026-10-06 item 1: 21 of 52 named scenarios occur nowhere in crates/; three implemented stories carry unmatched names (story:operator-observability 5 of 6, story:verification 3 of 3, story:workspace-directories 1 of 6). The audit count predates wave 1; the store at the wave-2 base has more active and implemented stories.

## Scenario name rule

A scenario name is a token in a story's `## Acceptance` section made of three or more lower-case words joined by `_` or `-`, backticked or not (several stories list names in plain prose). A backticked span that contains a space, such as `ess verify conform mutate --emit`, is a command and is skipped. An artifact id (`<kind>:<slug>`) and a path (a token containing `/` or `.`) are not names. A `-` name matches a test whose name has `_` in its place.

A name resolves when a Rust test function under `crates/` has that name, or when a frontend component test under `frontend/src/` (a `*.test.js` file) has that title in its `test(...)` or `it(...)` call.

## Acceptance

- `scenario_names_follow_the_rule`: extraction over a committed fixture of story bodies returns exactly the names they list, plain and backticked, and no backticked command, artifact id or path.
- `unresolved_name_fails_gate`: a story naming a scenario absent from the repository fails the check, which prints the story id and the name.
- `frontend_test_titles_resolve`: a name that is the title of a frontend component test resolves.
- Every unresolved name that belongs to a test existing under another name is resolved by renaming the test.
- Remaining unresolved names are rewritten in their stories by the coordinator through `aep plan artifact body`; the check is switched on in Taskfile.yml only after it passes over the current store.

## Scope

Cited: Taskfile.yml (switched on by the coordinator at integration). Inferred: crates/control-plane-xtask/src/, test files under crates/control-plane-app and crates/control-plane-runtime whose names change. Story text changes are planning-store writes owned by the coordinator.
