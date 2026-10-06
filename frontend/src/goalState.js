// A goal's one displayed state, from its lifecycle `state` and its `planning_phase` together.
// A terminal or paused lifecycle state wins whatever the phase says: a Cancelled goal is cancelled
// even when its last recorded phase is Queued. A Running goal takes its state from the phase.
//
// | state \ planning_phase | Idle     | Provisioning | Planning | Validated | Queued    | Blocked |
// |------------------------|----------|--------------|----------|-----------|-----------|---------|
// | Paused                 | paused   | paused       | paused   | paused    | paused    | paused  |
// | Running                | starting | planning     | planning | planning  | executing | blocked |
// | Satisfied              | satisfied (every phase)                                             |
// | Cancelled              | cancelled (every phase)                                             |
//
// Anything outside the table (a state or phase the specification does not define) is `unknown`.
const running = { Idle: 'starting', Provisioning: 'planning', Planning: 'planning', Validated: 'planning', Queued: 'executing', Blocked: 'blocked' }
const settled = { Paused: 'paused', Satisfied: 'satisfied', Cancelled: 'cancelled' }

export function deriveGoalState(goal) {
  const table = goal?.state === 'Running' ? running : settled
  const key = goal?.state === 'Running' ? goal.planning_phase : goal?.state
  return typeof key === 'string' && Object.hasOwn(table, key) ? table[key] : 'unknown'
}

export const goalStateLabels = { starting: 'Starting', planning: 'Planning', executing: 'Executing', blocked: 'Blocked', paused: 'Paused', satisfied: 'Satisfied', cancelled: 'Cancelled', unknown: 'Unknown' }

/** The badge class for a derived state: green only while the goal is moving. */
export const goalStateBadge = derived => ({ starting: 'active', planning: 'active', executing: 'active', blocked: 'blocked', satisfied: 'done' })[derived] || 'neutral'
