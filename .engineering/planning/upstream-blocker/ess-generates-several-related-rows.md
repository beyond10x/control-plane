---
format: aep.planning-md/3
id: upstream-blocker:ess-generates-several-related-rows
kind: upstream-blocker
status: open
title: ESS generation of a command reading several related rows
relations:
- blocks: story:admission-conformance-target
revision: 2
---
## What is blocked

story:admission-conformance-target: ess 0.56.0 generation keeps a command whose `when_related:` reads related rows through two or more input fields as an obligation, so QueueAssignment cannot declare its repository guard beside its goal guards.

## Upstream

- ess `story:generated-behaviour-reads-several-related-rows` (draft), planned for ESS 0.58.0. ESS holds the reproduction from tree cp-wave7-spec-owned-admission, `.scratch/repro-two-related-generation/`.

## Clears when

An ESS release generates such a command in Rust with no obligation; the pin moves to it.

## Checked against ESS 0.59.0 (2026-10-11)

The recorded reproduction (`repro-two-related-generation`, its `requires:` raised to 0.59.0) still generates `136 capabilities: 135 generated, 1 obligation(s), 0 refused`; plan.json keeps controlplane.host.QueueAssignment an obligation, class undetermined, construct "`when_related:` reading several related rows in one command". Still open.
