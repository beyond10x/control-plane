---
format: aep.planning-md/3
id: upstream-blocker:ess-subject-field-selectors
kind: upstream-blocker
status: open
title: ESS synthesis of subject-field selectors and two selectors per command
relations:
- blocks: story:spec-owned-busy-rules
revision: 2
---
## What is blocked

story:spec-owned-busy-rules cannot declare its two rules: ESS 0.56.0 conformance synthesis refuses a `when_related` selector whose `where` compares a row field with a subject field, and a command with two related selectors.

## Upstream

- ess `story:a-related-selector-compares-a-row-field-with-a-subject-field` (draft)
- ess `story:a-command-with-two-related-selectors-arranges-both` (draft)

Both are planned after ESS 0.57.0 and 0.58.0. ESS holds the reproductions taken from this repository's trial on 2026-10-08.

## Clears when

An ESS release ships both stories, and `ess verify conform synthesize` on the declared selectors reports 0 refusals.

## Checked against ESS 0.59.0 (2026-10-11)

The host-fact trials of the specification README (rows 1 and 2, on the wave-7 specification with their snippets, `requires:` raised to 0.59.0) print the same output as under 0.56.0 apart from the version line: row 1 synthesis refuses `controlplane.host.StartGoal/workspace-busy` with ESS-SYNTH-001 ("reads a subject field the steps leave undetermined"); row 2 with both guards is refused at validation with ESS-COMMAND-009 (one command reads one kind of related row). Still open.
