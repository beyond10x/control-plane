---
format: aep.planning-md/3
id: review-result:contract-verification-review-1
kind: review-result
status: active
title: Independent conformance and generation review
relations:
- reviews: story:contract-verification
revision: 1
---
needs-revision

unit: story:contract-verification at ec71b23a84f8ed0d7a83cb13fbb957a1e0f329ca
verdict: CONFIRMED
cases: executed 5 to 7, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: return generator ownership refusal to implementation

Tests-only addition: crates/control-plane-xtask/tests/generation_ownership.rs, 26 lines, commit 2198e99. Cargo.lock was already modified by the implementor and remained uncommitted; the review did not change implementation code.

The new case invokes the actual ESS generator, places an extra operator evidence file beside generated Rust, then invokes generation again. First isolated execution: FAILED, 0 passed; 1 failed; 0 ignored; 1 filtered out, exit 101. The assertion was `unowned extra files must require an explicit ownership decision`. The second generation returned success after deleting the extra file through generation.rs:80.

Subsequent full suite: five unit tests passed; the integration test binary ran two cases, one passed and this case failed, exit 101. Logs are retained in the review checkout under .scratch/conformance-adversary. The existing real durable conformance suite and its negative mutation continued to pass their assertions.

CONFIRMED / introduced: generation.rs:80 deletes any file missing from the new emitter output without proving the tool owns it. The public `task generate` route reaches this code; an extra retained note or standalone generated-crate build output is enough. Preflight all outputs, refuse unowned extras before any write, and retain the regression. The compiler-owned .ess-output directory must not become a second hand-maintained ownership format.

The review also inspected complete-inventory report admission, lossless integer codecs, actual dispatcher refusal handling, event observation, replay between commands and queries, and production dependency traversal. No other confirmed finding in this bounded pass.

```findings
- file: crates/control-plane-xtask/src/generation.rs
  line: 80
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Generation deletes extra files without ownership proof; refuse them before modifying any output.
```
