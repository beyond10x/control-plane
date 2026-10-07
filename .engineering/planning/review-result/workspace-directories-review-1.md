---
format: aep.planning-md/3
id: review-result:workspace-directories-review-1
kind: review-result
status: active
title: Workspace directory startup adversarial regression
relations:
- reviews: story:workspace-directories
revision: 1
---
unit: workspace-directories, ca0bdba54ae0d7eaf62eb3bd9e9da30139e0f47b
verdict: CONFIRMED
cases: executed 24→26, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: repair legacy migration before qualifying directory support

Review-authored diff: crates/control-plane-core/src/tests.rs, two added tests only. Candidate implementation was imported into the integration tree before this review; it was not changed during the review.

Added missing_legacy_workspace_does_not_block_healthy_directory_backfill before execution. Register two workspaces through the public operator command, disconnect one directory, then invoke the startup backfill used by serve. Assert the healthy workspace migrates, the missing record survives, and reconnecting permits a later migration. First isolated execution returned exit 101:

```text
running 1 test
test tests::missing_legacy_workspace_does_not_block_healthy_directory_backfill ... FAILED
called `Result::unwrap()` on an `Err` value: directory does not exist
Caused by:
    No such file or directory (os error 2)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.04s
```

Also added startup_backfill_does_not_restore_explicitly_removed_membership. First isolated execution passed: 1 passed, 0 failed. The case removes initial membership, reopens the actual SQLite store, runs backfill and startup registration, and observes the original Removed record with no replacement.

Then ran cargo test -p control-plane-core. Exit 101: 26 executed, 25 passed, 1 failed. Full first-run and suite output retained in .scratch/directory-review-first.log, .scratch/directory-review-removal.log and .scratch/directory-review-suite.log.

The failing workflow is reachable after upgrading an existing store whose registered directory has been moved or disconnected. initialize_workspaces calls backfill_workspace_directories before any startup root is registered. A filesystem absence propagates through the entire server startup, preventing access to healthy workspaces. This call and migration were introduced by the candidate. No storage error was injected; storage failures must continue to stop effects.

```findings
- file: crates/control-plane-core/src/directories.rs
  line: 188
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: An unavailable legacy workspace directory aborts server startup and prevents healthy workspaces from loading.
```
