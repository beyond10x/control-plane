---
format: aep.planning-md/3
id: story:gates-audit-findings
kind: story
status: implemented
title: Clear the Gates audit findings in tracked sources
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: cited
  path: crates/control-plane-app/src/cli.rs
- confidence: cited
  path: crates/control-plane-runtime/src/process.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T03:35:10Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-14T03:35:10Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-14T05:13:09Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Clear the Gates audit findings in tracked sources

## Outcome

The repository passes the Gates baseline audit, so its Gates hooks can be installed: no tracked file holds a path under a personal home directory, and no literal reads to the secrets scanner as a credential.

## Basis

An audit of the tree at f9dcbd6 found four findings: personal-paths in `crates/control-plane-runtime/src/process.rs` (the child-environment test used a home directory under `/home` three times) and one `generic-api-key` match in `crates/control-plane-app/src/cli.rs:305`. The match is not a credential: the scanner read the environment-variable names in the gates settings list, `B10X_GATES_KEY` followed by the next name, as a key and its value.

## Acceptance

- `gitleaks dir .` on the merged tree reports no finding.
- `git grep` finds no `/home/<name>/` path in tracked sources.
- child_environment_keeps_only_allowlisted_and_configured_variables stays green with the new fixture paths, and the gates settings are still kept out of a check's environment (the list holds the same three names).

## Out of scope

Enrolment of the repository in the Gates policy, which happens in the policy repository once this is on `main`.
