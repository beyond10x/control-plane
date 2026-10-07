---
format: aep.planning-md/3
id: decision-blocker:merge-bootstrap-into-main
kind: decision-blocker
status: open
title: Merge pull request 1 (bootstrap into main) before wave 4?
relations:
- blocks: epic:bootstrap
revision: 1
---
## Question

Should pull request 1 (control-plane/bootstrap into main, waves 1 to 3 at e77e533) be merged into main now, ahead of wave 4?

## State

- Draft; GitHub reports it mergeable with state CLEAN.
- Both `control-plane / task check` runs on e77e533 passed (runs 37526310767 and 37526316203).
- The approval for waves 3 to 5 names merges into control-plane/bootstrap and its push after a green gate; it does not name a merge into main.

## Options

| option | what it does | cost |
|---|---|---|
| A | mark it ready and merge it through the bot route now; wave 4 and later waves reach main through their own pull request from control-plane/bootstrap | a second pull request for wave 4 |
| B | keep it open until wave 4 is on control-plane/bootstrap and its gate is green, then merge once | main stays empty until the build hold lifts and wave 4 closes |

Recommendation: A. Waves 1 to 3 are green and main has had nothing but the initial commit since the repository started.
