---
format: aep.planning-md/3
id: decision-blocker:wave-10-approval
kind: decision-blocker
status: cleared
title: Approve wave 10 (ESS pin move and blocker re-checks)?
relations:
- blocks: story:ess-059-upgrade
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-11T23:03:13Z", actor: "human:timo", revision: 2}
---
## Question

Approve wave 10: move control-plane from ESS 0.56.0 to the newest release (story:ess-059-upgrade), then re-run the three open ESS upstream-blockers against it, on `control-plane/wave-10` with one pull request and no release?

## Options

| option | what runs | cost |
|---|---|---|
| A | both units as proposed | one workspace build (about 3 GiB), package gates, CI; no model spend |
| B | the pin move only | the same build; the blockers stay unchecked |
| C | defer | the host ess (0.59.0) keeps failing 5 xtask tests against the 0.56.0 pin |

Recommendation: A.

## Answer

Option A, decided on 2026-10-11 under the operator's delegated decision authority, with one condition: the pin target is the newest ESS release listed when unit 1 starts (0.59.0, or 0.60.0 if it has been released by then), one move and no second pin wave. Recorded as approval-record:wave-10-2026-10-11.
