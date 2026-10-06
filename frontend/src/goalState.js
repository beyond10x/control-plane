// A goal's one displayed state, from its lifecycle `state` and its `planning_phase` together, and
// for a Running goal in phase Queued also from its current assignments. A terminal or paused
// lifecycle state wins whatever the phase says: a Cancelled goal is cancelled even when its last
// recorded phase is Queued. A Running goal takes its state from the phase.
//
// | state \ planning_phase | Idle     | Provisioning | Planning | Validated | Queued                | Blocked |
// |------------------------|----------|--------------|----------|-----------|-----------------------|---------|
// | Paused                 | paused   | paused       | paused   | paused    | paused                | paused  |
// | Running                | starting | planning     | planning | planning  | (see below)           | blocked |
// | Satisfied              | satisfied (every phase)                                                        |
// | Cancelled              | cancelled (every phase)                                                        |
//
// Running in phase Queued depends on the goal's current work, its assignments at the goal's
// revision:
//
// | current work                                                                          | derived   |
// |---------------------------------------------------------------------------------------|-----------|
// | an assignment Queued, Implementing, Reviewing, ReadyToMerge or Merging, or an open     | executing |
// | goal-level model call (`waiting`, such as the final goal review)                       |           |
// | otherwise, an assignment Blocked                                                       | blocked   |
// | none of these                                                                          | stalled   |
//
// `stalled` means the plan is queued but nothing of it is waiting, running or blocked.
// `assignments` is the projection's assignment list (any goal's; only this goal's current ones
// count); omitted, the goal has none, so a caller that does not pass them sees `stalled`, never a
// false `executing`.
//
// Anything outside the table (a state or phase the specification does not define) is `unknown`.
const running = { Idle: 'starting', Provisioning: 'planning', Planning: 'planning', Validated: 'planning', Queued: 'executing', Blocked: 'blocked' }
const settled = { Paused: 'paused', Satisfied: 'satisfied', Cancelled: 'cancelled' }
const active = ['Queued', 'Implementing', 'Reviewing', 'ReadyToMerge', 'Merging']

export function deriveGoalState(goal, assignments = []) {
  const table = goal?.state === 'Running' ? running : settled
  const key = goal?.state === 'Running' ? goal.planning_phase : goal?.state
  if (typeof key !== 'string' || !Object.hasOwn(table, key)) return 'unknown'
  if (table[key] !== 'executing') return table[key]
  const current = (Array.isArray(assignments) ? assignments : []).filter(assignment => assignment?.goal_id === goal.goal_id && assignment.goal_revision === goal.revision)
  if (goal.waiting || current.some(assignment => active.includes(assignment.state))) return 'executing'
  return current.some(assignment => assignment.state === 'Blocked') ? 'blocked' : 'stalled'
}

export const goalStateLabels = { starting: 'Starting', planning: 'Planning', executing: 'Executing', stalled: 'Stalled', blocked: 'Blocked', paused: 'Paused', satisfied: 'Satisfied', cancelled: 'Cancelled', unknown: 'Unknown' }

/** The badge class for a derived state: green only while the goal is moving. */
export const goalStateBadge = derived => ({ starting: 'active', planning: 'active', executing: 'active', blocked: 'blocked', stalled: 'blocked', satisfied: 'done' })[derived] || 'neutral'
