---
format: aep.planning-md/3
id: decision-blocker:unattended-run-spending
kind: decision-blocker
status: open
title: How many real-model runs may story:unattended-goal-evidence spend, with which limits?
relations:
- blocks: story:unattended-goal-evidence
revision: 1
---
## Question

story:unattended-goal-evidence needs one real-model run: a goal on a real repository from StartGoal to Satisfied with no operator input. That run spends the operator's model budget through the operator's model login. How many runs, with which limits, are authorised?

## Needs

- Credential: the model login on this machine. With no provider bound, `crates/control-plane-runtime/src/loom_model.rs` builds `b10x_llm_tool_call::codex_model(&request.model)`; the default model for all three roles is `gpt-5.6-sol` (`crates/control-plane-app/src/cli.rs:90-94`, `frontend/src/GoalForm.vue:7`). No new account, key or secret is created.
- Repository: an isolated evaluation repository from `cargo run -p control-plane-xtask -- eval init` (cases go-cli, go-json-http, go-auth-web) with a local origin; no GitHub publication (README "Isolated model evaluations").
- Recorded usage of earlier real rounds (story:runtime-boundary): the round that delivered and published an application used 51 turns, 2,216,026 input and 24,034 output tokens; round 6 used 25 turns, 134,292 input and 7,799 output tokens and stopped before review. Dollar cost was reported by none of them.

## Options

| option | what runs | cost |
|---|---|---|
| A | one run, case go-auth-web, gpt-5.6-sol for every role, one worker, one attempt, 10-minute goal budget; a failed run is kept and the next run waits for a new decision | up to about 2.2M input tokens, by the largest earlier round |
| B | up to three runs under A's limits, each after the fix the previous failure asks for | up to about 6.6M input tokens |
| C | no real run now; wave 8 builds only the report checks (`unattended_run_report_counts_operator_commands`, `unattended_run_report_requires_a_merged_commit_on_target`) | no model spend; G1 evidence stays open |

Recommendation: A. One bounded run gives the evidence or names the next defect, and every further run is a separate spending decision.
