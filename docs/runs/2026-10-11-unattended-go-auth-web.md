# Unattended run: go-auth-web, 2026-10-11

The one real-model run the operator approved for story:unattended-goal-evidence (approval-record:unattended-run-spending-2026-10-11). The goal ran from StartGoal to Satisfied with no Operator command in between.

## Setup

- Control plane: `control-plane/wave-9` at 6bf3795 (main f0ad6c2 plus store records only), `cargo build -p control-plane-app -p control-plane-xtask` (dev profile). sha256 of the binaries as copied to the eval root: `control-plane` 81807e4792b00fab9873b40383196d9f73635a512974781f1b5ce99cd2795c8c, `control-plane-xtask` 077559197ca35421d56f61587832019a1af613ad7e690a275b4bd0b4b8be4e0c.
- Repository: a fresh seed `repos/go-auth-web-round07` at 0e10459 (the case's seed) with its own local origin `remotes/go-auth-web-round07.git`, both under the eval root `~/control-plane-evals`, outside this repository. Nothing was published to GitHub.
- Service: `control-plane serve --state <eval root>/state-round07/state.sqlite --listen 127.0.0.1:8797 --workspace <repo> --local-eval-root <eval root>`, on the existing codex login (provider binding `codex-subscription`).
- Repository settings: base branch `main`; test command `<eval root>/tools/control-plane-eval-07 eval verify --case go-auth-web` (the trusted verifier from the same build); publish command `git push origin {candidate}:refs/heads/main`.
- Goal 44f885aa-77cc-4da7-a42e-069b364f54a3, created paused: objective = the case's TASK.md; acceptance = the verifier's nine named scenarios, Go tests, README requirements; 1 worker, 1 attempt, 10 minutes; gpt-5.6-sol for planner, implementor and reviewer; merge authority on.
- Limits held outside the service: stop at 600 s after StartGoal or at 2,200,000 recorded input tokens, whichever came first; stop on any blocker. The run ended on Satisfied before either limit.

## Timeline (UTC, from the run's monitor)

| at | state |
|---|---|
| 22:08:43.13Z | StartGoal (the only Operator command after creation) |
| 22:08:44Z | planning |
| 22:10:08Z | plan validated and critiqued; one assignment implementing on the planning base 14ba998 |
| 22:12:15Z | the trusted checks ran |
| 22:12:16Z | reviewing candidate a0f52f8 |
| 22:12:33Z | assignment merged, a0f52f8 on origin/main |
| 22:12:45Z | goal acceptance completed, goal Satisfied |
| 22:12:51.20Z | service stopped by SIGINT (5 s after Satisfied was seen) |

## Report

`control-plane-xtask eval report --state <state.sqlite> --goal 44f885aa-77cc-4da7-a42e-069b364f54a3 --repo <repo>`, exit 0:

```text
goal: 44f885aa-77cc-4da7-a42e-069b364f54a3
state: Satisfied
receipt revision: 1
merged commit: a0f52f8dd098d475f94bf1181202d6c4291edb31 on origin/main: yes
elapsed: 242.132s
operator commands after start: 0
unattended: yes
```

The decision log it read (`state.sqlite`, sha256 96a827a065e898dc498da3b3989252922954ad71a1ef7c6f3fc82673b80af273) and the five Loom sessions stay under the eval root; they hold machine paths and are not committed.

## Usage (from the five filed Loom sessions)

| role | turns | input tokens | cached input | output tokens |
|---|---|---|---|---|
| planner | 14 | 80,259 | 20,224 | 2,887 |
| plan critic | 1 | 6,176 | 0 | 386 |
| implementor | 18 | 111,633 | 30,336 | 6,160 |
| code reviewer | 1 | 8,229 | 0 | 794 |
| goal reviewer | 1 | 7,781 | 0 | 628 |
| **total** | **35** | **214,078** | **50,560** | **10,855** |

Wall time from StartGoal to the stop: 248.07 s; StartGoal to SatisfyGoal: 242.132 s (the report). Input tokens were 9.7 % of the 2.2M limit. Dollar cost was not reported by the provider.

Host load (`/proc/loadavg`, 1/5/15 min): 38.79 38.55 33.79 at the start, 13.70 26.26 30.02 at the stop, on 20 cores (the load-sensitivity finding on the story).

## Independent check

A clean clone of the local origin at a0f52f8, outside the service's trees: `control-plane-xtask eval verify --case go-auth-web` printed `PASS go-auth-web`, exit 0. The published tree holds `cmd/server/main.go`, `internal/authweb/{app,app_test}.go`, `web/` (templates, CSS, assets), `README.md`, an ESS specification (`ess/system.yaml`, `ess/domains/session.yaml`) and an AEP story with its evidence. Nobody edited the application by hand.

## Compared with earlier rounds (story:runtime-boundary)

Round 4, the only earlier round that published: 569 s, 51 turns, 2,216,026 input and 24,034 output tokens, and final acceptance rejected. This round: 242 s, 35 turns, 214,078 input and 10,855 output tokens, accepted.
