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
// revision, and their fleet entries:
//
// | current work                                                                          | derived   |
// |---------------------------------------------------------------------------------------|-----------|
// | an assignment Queued, Implementing, Reviewing, ReadyToMerge or Merging, or an open     | executing |
// | goal-level model call (`waiting`, such as the final goal review)                       |           |
// | otherwise, an assignment Blocked                                                       | blocked   |
// | otherwise, the goal's acceptance is running ({@link acceptanceStep}): every current    | executing |
// | assignment Merged or Cancelled, and the newest fleet entry of the current assignments  |           |
// | is `goal.checks` or `goal.review` with status `running`                                |           |
// | none of these                                                                          | stalled   |
//
// `stalled` means the plan is queued but nothing of it is waiting, running or blocked; after a
// rejected final goal review ({@link acceptanceRejection}) the goal stays stalled.
// `assignments` is the projection's assignment list (any goal's; only this goal's current ones
// count); omitted, the goal has none, so a caller that does not pass them sees `stalled`, never a
// false `executing`.
//
// Anything outside the table (a state or phase the specification does not define) is `unknown`.
import { parseTime } from './time.js'

const running = { Idle: 'starting', Provisioning: 'planning', Planning: 'planning', Validated: 'planning', Queued: 'executing', Blocked: 'blocked' }
const settled = { Paused: 'paused', Satisfied: 'satisfied', Cancelled: 'cancelled' }
const active = ['Queued', 'Implementing', 'Reviewing', 'ReadyToMerge', 'Merging']
// The steps fleet.rs `satisfy_goals` records while it accepts a goal: `goal.checks` before each
// repository's test command, `goal.review` before the final goal review call.
const accepting = ['goal.checks', 'goal.review']

/** The goal's assignments at its revision. */
const currentOf = (goal, assignments) => (Array.isArray(assignments) ? assignments : []).filter(assignment => assignment?.goal_id === goal?.goal_id && assignment.goal_revision === goal.revision)
/** Newest first; an entry whose time does not parse sorts last. */
const newestFirst = (first, second) => (parseTime(second.at) ?? -Infinity) - (parseTime(first.at) ?? -Infinity)

/**
 * The goal acceptance step in progress, as `{ assignment, entry }`, or null. Acceptance runs once
 * every current assignment is Merged or Cancelled (at least one Merged, fleet.rs `satisfy_goals`);
 * its step is the newest fleet entry of the current assignments at the goal's revision, and it is
 * in progress while that entry is `goal.checks` or `goal.review` with status `running`. The
 * newest entry decides, so a step left behind by an earlier, rejected attempt on another
 * assignment does not count.
 */
export function acceptanceStep(goal, assignments = []) {
  const current = currentOf(goal, assignments)
  if (!current.some(assignment => assignment.state === 'Merged') || !current.every(assignment => ['Merged', 'Cancelled'].includes(assignment.state))) return null
  const [newest] = current
    .map(assignment => ({ assignment, entry: goal.fleet?.[assignment.assignment_id] }))
    .filter(({ entry }) => entry && entry.goal_revision === goal.revision)
    .sort((first, second) => newestFirst(first.entry, second.entry))
  return newest && accepting.includes(newest.entry.action) && newest.entry.status === 'running' ? newest : null
}

/** The newest `blocked` fleet entry the final goal review recorded at the goal's revision (its
 * `detail` is the rejection), or null. */
export function acceptanceRejection(goal) {
  const [newest] = Object.values(goal?.fleet || {})
    .filter(entry => entry?.action === 'blocked' && entry.role === 'goal_reviewer' && entry.goal_revision === goal.revision)
    .sort(newestFirst)
  return newest || null
}

export function deriveGoalState(goal, assignments = []) {
  const table = goal?.state === 'Running' ? running : settled
  const key = goal?.state === 'Running' ? goal.planning_phase : goal?.state
  if (typeof key !== 'string' || !Object.hasOwn(table, key)) return 'unknown'
  if (table[key] !== 'executing') return table[key]
  const current = currentOf(goal, assignments)
  if (goal.waiting || current.some(assignment => active.includes(assignment.state))) return 'executing'
  if (current.some(assignment => assignment.state === 'Blocked')) return 'blocked'
  return acceptanceStep(goal, current) ? 'executing' : 'stalled'
}

export const goalStateLabels = { starting: 'Starting', planning: 'Planning', executing: 'Executing', stalled: 'Stalled', blocked: 'Blocked', paused: 'Paused', satisfied: 'Satisfied', cancelled: 'Cancelled', unknown: 'Unknown' }

/** The badge class for a derived state: green only while the goal is moving. */
export const goalStateBadge = derived => ({ starting: 'active', planning: 'active', executing: 'active', blocked: 'blocked', stalled: 'blocked', satisfied: 'done' })[derived] || 'neutral'
