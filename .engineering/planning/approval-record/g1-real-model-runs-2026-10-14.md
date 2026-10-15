---
format: aep.planning-md/3
id: approval-record:g1-real-model-runs-2026-10-14
kind: approval-record
status: draft
title: Up to three real-model runs for G1, 2.2M input tokens each, approved by the operator
relations:
- decides: story:g1-real-model-runs
revision: 1
---
The operator decided on 2026-10-14 how many further real-model runs goal G1 (a goal accepted without operator input) may spend, after the single run of approval-record:unattended-run-spending-2026-10-11 was spent.

## Decision

- Up to three real-model runs in all, each capped at 2.2M input tokens; a run is stopped at the cap.
- The next run starts only after the defect the previous run showed is fixed: spec first, in one wave, one pull request, merged on green CI.
- Each run's input tokens and outcome are reported.
- On the existing codex login on this machine: no new account, key or secret. No GitHub publication by a run: an eval case publishes only to its local origin under the eval root.

## Run bounds kept from the earlier runs

One goal per run, one worker, one attempt, a 10-minute goal budget, one run at a time, failures kept as evidence and no silent retry (story:external-evaluations).

## Units

1. story:g1-real-model-runs: the runs, their records and the fixes their defects need.

## Commits this approval authorises

Store, evidence and fix commits on the wave branch of each run's wave, the merges into it, its push, one pull request into `main` per wave and its merge once its checks pass. No tag, no release (the repository has no release process).
