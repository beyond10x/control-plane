---
format: aep.planning-md/3
id: dependency-blocker:aep-imports-ess-053-suites
kind: dependency-blocker
status: cleared
title: AEP 0.68.0 refuses ESS 0.53 conformance suites (scenario_initial_state)
relations:
- blocks: executable-system-specification:control-plane
- blocks: story:conformance-evidence-record
withholds: ess_conformance_v2
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T14:17:27Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"review_outcome":1}}}
---
## What is blocked

executable-system-specification:control-plane cannot record its conformance report, so it stays validated; story:conformance-evidence-record cannot start.

## The refusal

`aep plan artifact evidence executable-system-specification:control-plane --from .scratch/conformance/report.json --suite generated/conformance.json` with AEP 0.68.0 and an ESS 0.53.0 suite answers:

```text
Error: UnknownField at $suite.provenance.scenario_initial_state: closed count-stage vocabulary
```

Re-run 2026-10-06 on a throwaway copy of this store. The refusal was first recorded in the specification artifact's body.

## What clears it

An AEP release that admits ESS 0.53 suite provenance, or an ESS release whose suites AEP admits. No beyond10x/aep issue matching "scenario_initial_state" or "count-stage" was found on 2026-10-06; beyond10x/aep#88 is a related refusal of grant scenario ids.
