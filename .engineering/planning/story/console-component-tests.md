---
format: aep.planning-md/3
id: story:console-component-tests
kind: story
status: implemented
title: Vue component tests run in the gate
relations:
- decomposes: epic:console-clarity
- serves: vision:autonomous-engineering
- depends_on: story:spec-history-gate
scope:
- confidence: cited
  path: crates/control-plane-xtask/src/frontend.rs
- confidence: inferred
  path: frontend/package-lock.json
- confidence: inferred
  path: frontend/package.json
- confidence: inferred
  path: frontend/src
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T05:17:10Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-wave-coordinator"}
- {from: "proposed", to: "active", at: "2026-10-06T05:17:11Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-wave-coordinator"}
- {from: "active", to: "implemented", at: "2026-10-06T06:33:37Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}, executor: "agent:claude-wave-coordinator"}
---
## Outcome

The Vue components have tests that run in `task check`, so later console stories can state acceptance a check observes.

## Evidence

frontend/package.json declares only `vite build`; the repository has no component, browser or WebDriver test (review-result:console-ux-review-2026-10-06, "Not settled"). vision.md:40 requires the live behaviour to be observed.

## Acceptance

- `component_tests_run_in_the_gate`: `cargo run -p control-plane-xtask -- frontend-check` runs the frontend test suite after the drift check.
- `failing_component_test_fails_the_gate`: a committed xtask test runs the frontend suite against a fixture test that fails and asserts `frontend-check` exits non-zero.
- `app_renders_a_snapshot_payload`: a component test mounts the console with a recorded SSE payload and finds the workspace, goal and assignment it contains.

## Scope

Cited: crates/control-plane-xtask/src/frontend.rs. Inferred: frontend/package.json, frontend/package-lock.json, frontend/src/**/*.test.js. Frontend test code is frontend code (AGENTS.md: the Rust rule does not restrict the frontend); the runner stays Rust.
