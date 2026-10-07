---
format: aep.planning-md/3
id: decision-blocker:stale-blocked-reason
kind: decision-blocker
status: cleared
title: How is a Blocked assignment's stale reason replaced?
relations:
- blocks: story:publication-exit
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-07T01:44:09Z", actor: "human:timo", revision: 3}
---
## Question

How should the stale "Current reason" of a Blocked assignment be fixed, now that a publication intent can be closed as not published?

## State

- review-result:adversary-publication-exit-pass-2 finding 4: after an intent becomes NotPublished, the console's "Current reason" (frontend/src/App.vue:66) still reads "Publication outcome unresolved…". The fleet sends BlockAssignment only when the assignment is not already Blocked (crates/control-plane-runtime/src/fleet.rs:254), and the specification declares no way to replace a Blocked assignment's reason.
- The assignment's activity history does record the close, so the fact is visible one level down.

## Options

| option | what it does | cost |
|---|---|---|
| A | wave 4 closes without it; a draft story declares in ESS how a Blocked assignment's reason is replaced (a Blocked to Blocked transition of BlockAssignment, or a reason update command), then the fleet uses it | the stale reason stays until that story ships |
| B | hold wave 4 and add that specification change now in story:publication-exit | one more implementor round, a spec-history acknowledgement and conformance rerun; wave 4 closes later |
| C | change nothing; the activity history is the record | the attention strip and goal card keep showing a reason that is no longer true |

Recommendation: A. The defect is a display of an older true reason, the fix is a specification change that deserves its own acceptance, and wave 4 is already late.

## Decided

Option A, 2026-10-07. Wave 4 closes without this fix, and its pull request names the stale reason as a known limit. story:blocked-reason-update declares in ESS how a Blocked assignment's reason is replaced, and the fleet then uses it; it is planned for wave 5.
