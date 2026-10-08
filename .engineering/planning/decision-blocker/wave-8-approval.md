---
format: aep.planning-md/3
id: decision-blocker:wave-8-approval
kind: decision-blocker
status: cleared
title: Approve wave 8 (three units, no real-model run)?
relations:
- blocks: story:unattended-run-report
- blocks: story:conformance-evidence-record
- blocks: story:console-evidence-page
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T13:14:19Z", actor: "human:timo", revision: 2}
---
## Question

Approve wave 8 as .engineering/waves/2026-10-08-wave-8.md proposes: story:unattended-run-report, story:conformance-evidence-record and story:console-evidence-page in parallel on control-plane/wave-8, one pull request into main, no release, no real-model run.

## Options

| option | what it does | cost |
|---|---|---|
| A | the three units as proposed | 3 trees (about 3 GiB each); 2 adversary passes |
| B | story:unattended-run-report alone | 1 tree; the conformance and console stories wait a wave |
| C | hold the wave | nothing ships until the spending decision |

Recommended: A. None of the three needs a model run; together they leave only the real run for G1.
