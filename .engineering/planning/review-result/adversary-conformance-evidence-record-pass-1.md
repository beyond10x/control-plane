---
format: aep.planning-md/3
id: review-result:adversary-conformance-evidence-record-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: story:conformance-evidence-record at c5f4fa1'
relations:
- reviews: story:conformance-evidence-record
- reviews: executable-system-specification:control-plane
revision: 1
---
unit: story:conformance-evidence-record @ c5f4fa1 (worktree cp-wave8-conformance-evidence-record)
verdict: green
cases: executed 141→144, red 0
origin: introduced 0, pre-existing 1, undecided 0
wrote-outside-worktree: none
needs-coordinator: yes

The clock change holds up: I wrote 3 cases and none went red. One acceptance item still fails, because the planning store's `model_digest` is stale. That is the coordinator's to fix, not the implementor's.

**1. Diff scope** (`git --no-pager diff --stat` is empty because the file is untracked; `git status --short`):
```
?? crates/control-plane-xtask/tests/conformance_evidence_record_attack.rs
```
That is the only path. It is a test file, and no implementation file was touched.

**2. Cases added**, in `crates/control-plane-xtask/tests/conformance_evidence_record_attack.rs`. Each case runs `xtask conformance` through a shared mutex. All are green now, and were green on their first run (`--test conformance_evidence_record_attack`, EXIT=0, 3 passed, 408.52s):

| case | asserts |
|---|---|
| `two_runs_agree_on_the_count_report_apart_from_completed_at` | two runs give byte-equal `report.json` once `completed_at` is masked |
| `two_runs_agree_on_the_diagnostics_apart_from_time` | two runs give equal `diagnostics.json` once `started_at`, `completed_at` and each `duration_ms` are masked |
| `the_stated_run_interval_is_coherent_and_inside_the_run` | before ≤ `started_at` ≤ `completed_at` ≤ after; the report's `completed_at` equals the diagnostics' value; the scenario durations add up to no more than the interval (the ESS `counts.rs:539-546` rule) |

**3. Suite run:** `cargo test --locked -p control-plane-xtask` gave EXIT=0, 144 passed across 19 binaries, with nothing failed or ignored. That includes `guard_mutants_are_killed` and `conformance_report_carries_run_instant`. The "before" count of 141 is this same run minus my file's 3 cases.

**4. Findings**
- `.engineering/planning/executable-system-specification/control-plane.md:7` — the acceptance item `conformance_report_imports_as_evidence` is not met. Running aep 0.69.1 on a copy of the store under `.scratch/adv-store-probe`:
  - `evidence --from report.json --suite generated/conformance.json` returns EXIT=0 and records `ess_conformance_coverage_v1` with an observed time of 2026-10-08T14:01:38Z, which is the real run time.
  - `move --to conforming` returns EXIT=1: "not counted: the ess_conformance_coverage_v1 record observed at 2026-10-08T14:01:38Z: it was run against 067305d07e71…, and this specification is at 528a7c48088b…".
  - The store's `model_digest` is the same at base d39f2db, so this is pre-existing.
  - In the copy, after replacing `model_digest` with `067305d07e71dad22be3826b880d520f1f1c41ed0bdd99b54d385d0d95f58dad`, the move succeeds: "moved validated -> conforming (revision 10)". The stale digest is the only blocker.
  - What reaches it: the coordinator's own integration-branch `move`, which the story requires.
  - Fix: the coordinator refreshes the artifact's `model_digest` before the move.
- `crates/control-plane-xtask/src/target.rs:16-17` — the doc comment says a bounded assertion "still ends at its budget" because the machine clock keeps moving. Under a backward clock step that is false. `last` holds, so a retry loop busy-polls the target for the size of the step plus 5 s. The ESS `Clock` contract (`runner.rs:113-115`, "advanced by this read") is also not met: two reads in the same millisecond return the same instant.
  - What reaches it: nothing. The suite has no retry steps (no `Eventually*`, ordered-scan halt or bounded retry), and `DurableTarget` ignores deadlines.
  - I could not write a failing case without editing the implementation, so this is a judgement finding. Rated INFEASIBLE, severity note.

**5. Attacked and not broken**
- **Report and import fields:** conformance ids and correlation ids are seeded from the suite, and the masked reports are equal. The count report carries only `completed_at`.
- **Time ordering:** `started_at` ≤ `completed_at` holds, and the scenario durations fit inside the interval.
- **Mutation fixture:** `guard-mutation-report.json` has no time fields, and its digest check compares `spec_digest` only.
- **Time-reading steps:** the suite has no `now_offset`, no elapsed-bound steps and no eventual steps, so no scenario status can depend on the clock.
- **Full mutation run:** I did not run the 20-minute `xtask mutate`. The point above settles it more cheaply, and `guard_mutants_are_killed` passes.
- **Checksum:** `check.yml` pins `39eb89d6…a4c4a3f0`. It matches the release `SHA256SUMS` and the local tarball's sha256.
- **Base:** the base run of `run_admitted` has no monotonicity problem, because the WallClock starts at 0 and takes the max of each machine reading.

**6. Paths written outside the worktree:** none. The scratch is in `.scratch/adv-sums/`, `.scratch/adv-store-probe/`, `.scratch/adv-attack-alone.log` and `.scratch/adv-suite.log`, all inside the tree. The harness's own task-output files went under `~/.cache/claude-tmp/`. I removed `frontend/node_modules` after the gate.

```findings
- file: .engineering/planning/executable-system-specification/control-plane.md
  line: 7
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the stale model_digest 528a7c… makes `move --to conforming` refuse the imported report (spec 067305…), so conformance_report_imports_as_evidence fails until the coordinator refreshes the digest; with it refreshed in a copy the move succeeds
- file: crates/control-plane-xtask/src/target.rs
  line: 16
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: WallClock holds its last reading under a backward machine-clock step and does not advance on every read as the ESS Clock contract requires, so a retry loop would busy-poll for the step plus 5 s; no step in the current suite retries
```
