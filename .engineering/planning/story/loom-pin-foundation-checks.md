---
format: aep.planning-md/3
id: story:loom-pin-foundation-checks
kind: story
status: active
title: Keep Loom pinned to a release and resolved once
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
scope:
- confidence: cited
  path: crates/control-plane-xtask/src/foundation.rs
- confidence: cited
  path: crates/control-plane-xtask/tests/foundation_check.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T02:58:09Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-14T02:58:09Z", actor: "human:timo", revision: 4}
---
# Keep Loom pinned to a release and resolved once

## Outcome

`foundation-check` refuses a tree whose Loom dependency is not a release tag, or whose lock file resolves a Loom crate more than once, so the move to a released Loom cannot drift back to a commit or split into two copies.

## Acceptance

- loom_pin_must_be_a_release_tag: the foundation check fails on a fixture tree whose `b10x-loom-*` dependency is declared with `rev`, `branch` or no `tag`, naming the manifest and the crate; it passes on a fixture with `tag = "<x.y.z>"`.
- loom_crates_resolve_once: the foundation check fails on a fixture `Cargo.lock` holding one `b10x-loom-*` package from two sources or versions, naming the package and both sources; it passes when each Loom package appears once.
- The checks run inside `cargo run -p control-plane-xtask -- foundation-check`, which `task check` already runs, so the PR's CI gate holds them.
- The current tree passes both checks once story:loom-release-pin is merged into the wave branch.

## Out of scope

The repin itself (story:loom-release-pin). Duplicate resolution of non-Loom crates (canon, llm) is reported, not enforced here.
