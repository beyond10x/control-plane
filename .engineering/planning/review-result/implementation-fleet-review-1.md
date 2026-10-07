---
format: aep.planning-md/3
id: review-result:implementation-fleet-review-1
kind: review-result
status: active
title: 'Independent fleet review: two reproduced defects'
relations:
- reviews: story:implementation-fleet
revision: 1
---
unit: fleet candidate `5e13769`
verdict: CONFIRMED — two blockers
cases: executed 26→29, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route repairs before integration

Tests-only commit: `0f3e0229e5f51573b3d5011f0fc27223658e2bf5`. Lease released; bot identities verified.

Diff: `crates/control-plane-runtime/tests/fleet.rs | 160 insertions`. Existing coordinator-owned Cargo.lock remains unstaged.

1. **Inspection command escapes confinement**, `fleet.rs:871`: real model action `git diff --output=tests/external.txt` follows a tracked symlink and truncates an external fixture file. Isolated test: **0 passed, 1 failed**, observed `""` instead of `"operator-owned data"`. Ordinary tracked test-file modification is caught before publication; external effects escape that final check.

2. **Exhausted interrupted attempt stays falsely active**, `fleet.rs:312`: after real Claim→Block→Repair persistence and Store reopen, the scheduler skips the final consumed attempt without recording a blocker. Isolated test: **0 passed, 1 failed**, observed `Implementing` instead of `Blocked`. The repository slot remains occupied.

Validation: runtime unit **2 passed**; fleet **8 passed, 2 failed**; planner separately **17 passed**. Formatting and clippy passed. Logs: `target/fleet-adversary-*.log`.

```findings
- file: crates/control-plane-runtime/src/fleet.rs
  line: 871
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Admitted git inspection arguments can write through tracked symlinks outside the managed worktree.
- file: crates/control-plane-runtime/src/fleet.rs
  line: 312
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Restart skips exhausted in-flight attempts without recording a blocker, leaving assignments falsely active and retaining repository slots.
```
