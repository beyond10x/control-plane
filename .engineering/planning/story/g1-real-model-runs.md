---
format: aep.planning-md/3
id: story:g1-real-model-runs
kind: story
status: active
title: Real goals run unattended on the released Loom foundation, within three bounded runs
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: story:unattended-goal-evidence
- informed_by: story:external-evaluations
- informed_by: story:loom-release-pin
scope:
- confidence: cited
  path: crates/control-plane-xtask/src/eval.rs
- confidence: cited
  path: crates/control-plane-xtask/src/eval_tests.rs
- confidence: cited
  path: docs/runs/
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T06:48:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-15T06:48:03Z", actor: "human:timo", revision: 3}
---
## Outcome

The control plane, on its released Loom foundation (tag 0.20.0, story:loom-release-pin), runs real goals unattended: each run takes one goal from StartGoal to Satisfied with no Operator command in between, and a run that does not get there names the defect, which is fixed before the next run. This is G1's evidence on the current foundation; the one earlier accepted run (story:unattended-goal-evidence, 2026-10-11) ran on Loom commit 1b25fe89.

## Evidence

- story:unattended-goal-evidence: one go-auth-web run reached Satisfied on 2026-10-11, 214,078 input tokens, 242 s, before the Loom repin.
- story:loom-release-pin moved every model call and Commission admission to Loom 0.20.0 across six CHANGELOG breaks; no real-model run has exercised it.
- story:external-evaluations keeps two lines open that need a model: real runs of the Go CLI case and of the JSON HTTP case.
- approval-record:g1-real-model-runs-2026-10-14 sets the budget: up to three runs, at most 2.2M input tokens each.

## Acceptance

- Run 1 uses the Go CLI eval case on a fresh seed with its own local origin, a control plane built from `main` with no source change, one worker, one attempt, a 10-minute goal budget, and is stopped at 2.2M recorded input tokens.
- Each run's record under `docs/runs/` states the goal, the control-plane commit, input, cached and output tokens per Loom session, the wall time, the host load, the outcome, and whether the goal ran unattended to the end; `control-plane-xtask eval report` printed it, so `unattended_run_report_counts_operator_commands` and `unattended_run_report_requires_a_merged_commit_on_target` hold for the report it reads.
- A Satisfied run is checked independently: the trusted verifier on a clean clone of the published local origin passes.
- A run that stops short names its defect from the run's own sessions and decision log, reproduces it without a model, and the fix lands spec first in one wave before the next run starts. At most three runs.
- Run 1 defect (the trusted check printed only its verdict): `passing_verify_reports_each_executed_check`, the verifier lists the Go tests it saw pass, `go vet ./...` and each named scenario before its PASS line.

## Out of scope

More than three runs, runs in parallel, repositories outside the eval root, GitHub publication by a run, and the stories that wait on an ESS release.

## Scope

Cited: `docs/runs/` (the run records). Inferred: crates/control-plane-runtime/ for a fix a run's defect needs. The eval harness is story:external-evaluations' (crates/control-plane-xtask/src/eval.rs); the report is story:unattended-run-report's.

## Run 1 (2026-10-15)

Record: docs/runs/2026-10-15-g1-run-1-go-cli.md.

- Outcome: not Satisfied. Goal 3ec923b5-bf66-46a3-b64f-6dfee02c33b3 on a fresh Go CLI seed was planned, critiqued, implemented, reviewed and merged (0c7794c on the local origin) at 230 s; the final goal review rejected it at 07:20:42Z because the checks output was only `PASS go-cli` and showed no run of `go test ./...` or `go vet ./...`. The goal stayed Running and idle; the monitor stopped the service at the 600 s budget.
- Ran unattended to the end: no Operator command after StartGoal (report: 0); the end was the rejection, not acceptance.
- Tokens: 219,693 input (72,448 cached) and 7,011 output in 42 turns over five Loom sessions; 10.0 % of the 2.2M cap. No model call after the rejection.
- Wall time: 600.75 s from StartGoal to the stop. Host load 6.26 at the start, 11.97 at the stop.
- Independent check: the trusted verifier on a clean clone of the origin at 0c7794c printed PASS go-cli.
- Defect: the trusted verifier ran go test and go vet but printed only its verdict, so the reviewer had no execution evidence. Fixed in wave 14 before run 2.
