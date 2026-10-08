---
format: aep.planning-md/3
id: story:spec-owned-busy-rules
kind: story
status: draft
title: The two busy admission rules are declared in ESS once synthesis supports subject-field selectors
relations:
- decomposes: epic:unattended-operation
- serves: vision:autonomous-engineering
- informed_by: decision-blocker:admission-selectors-not-synthesized
- depends_on: story:admission-conformance-target
revision: 1
---
## Outcome

The two busy rules that stay host facts after story:spec-owned-admission (one Running goal per workspace; one active change per Git common directory) are declared in the specification and exercised by synthesized conformance scenarios, and their host guards in crates/control-plane-core/src/guards.rs are removed.

## Evidence

- decision-blocker:admission-selectors-not-synthesized: ESS 0.56.0 validates both selectors, but synthesis refuses them ("reads a subject field the steps leave undetermined"; 180 scenarios fell to 141 and 159), and a second selector on one command is refused ("a scenario arranges the rows of one selector per command in this cut").
- upstream-blocker:ess-subject-field-selectors names the two ESS stories that lift both refusals.

## Acceptance

- `second_running_goal_is_refused_in_conformance`: the synthesized scenario that sends StartGoal for a workspace that already has a Running goal passes against the conformance target and observes the declared refusal.
- `second_change_in_one_common_dir_is_refused_in_conformance`: the synthesized scenario that claims an assignment while another assignment on a repository with the same Git common directory is active observes the declared refusal.
- Synthesis reports 0 refusals, and the host-facts section of the specification README no longer lists the two rules.

## Out of scope

Worker limit and repository-disabled rows (other refusals: `count` takes no subject field; ESS-SYNTH-003).

## Scope

Inferred: ess/domains/host.yaml, ess/README.md, ess/spec-acknowledgements.json, generated, crates/control-plane-core/src/guards.rs.
