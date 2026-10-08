// What a goal card shows besides the derived state (goalState.js): its assignments at the goal's
// revision, each one's current step, and which of them a goal save would cancel. Everything is
// read from the projection fields live.rs `compact` documents; nothing here reads the browser
// clock.
import { activitySentence } from './activity.js'

/** The fields an `UpdateGoal` carries besides `goal_id` (ess/domains/host.yaml). */
export const goalFields = ['objective', 'acceptance', 'planner_model', 'implementor_model', 'reviewer_model', 'max_workers', 'max_attempts', 'max_minutes', 'merge_authority']

/** The goal's assignments at its revision. */
export const currentAssignments = (goal, assignments) => (assignments || []).filter(assignment => assignment.goal_id === goal.goal_id && assignment.goal_revision === goal.revision)

/**
 * The current assignments a goal save cancels. Every save increments the goal's revision
 * (UpdateGoal `revision: {increment: 1}`), and fleet.rs `retire_stale` then cancels each
 * assignment of an older revision unless it is Merged, Cancelled or Merging, or a publication
 * intent that is not `NotPublished` still holds it (`held`).
 */
export function inFlight(goal, assignments, publications) {
  const held = assignment => (publications || []).some(publication => publication.assignment_id === assignment.assignment_id && publication.state !== 'NotPublished')
  return currentAssignments(goal, assignments).filter(assignment => !['Merged', 'Cancelled', 'Merging'].includes(assignment.state) && !held(assignment))
}

/**
 * The step an assignment is on, as `{ label, at }`; `at` is when the step started, or null when
 * the projection does not say. A step is the lane's newest entry other than a streamed
 * `loom.event` (live.rs). While a model call is open the step is that call, which started at
 * `waiting.since`. Otherwise the assignment's fleet entry is its newest entry: when it is not a
 * `loom.event` it is the step itself; when it is one, the step's own entry is not projected, so
 * no start time is claimed.
 */
export function assignmentStep(goal, assignment) {
  if (assignment.waiting?.since) return { label: `Waiting for ${assignment.waiting.model || 'the model'}`, at: assignment.waiting.since }
  const entry = goal.fleet?.[assignment.assignment_id]
  if (!entry || entry.goal_revision !== goal.revision) return { label: '', at: null }
  return { label: activitySentence(entry.action), at: entry.action === 'loom.event' ? null : entry.at }
}

export { storyName } from './identifiers.js'

/**
 * The assignments in `repository` that a change of its settings stops. The store admits a
 * repository configuration with each claim and repair (guards.rs
 * `assignment_configuration_changed`); once the settings differ it refuses the assignment's
 * repair, review, readiness, merge and publication ("repository configuration changed;
 * assignment evidence is stale"), so it is blocked and can only be cancelled or re-planned, and
 * fleet.rs `guard` stops a running execution ("repository configuration changed or disabled").
 * That is every claimed assignment still open: Implementing, Reviewing, ReadyToMerge (fleet.rs
 * still delivers it) and Merging, and a Blocked one that was claimed (its worktree or
 * implementation run is recorded).
 */
export const stoppedBySettings = (repository, assignments) => (assignments || []).filter(assignment => assignment.repository_id === repository
  && (['Implementing', 'Reviewing', 'ReadyToMerge', 'Merging'].includes(assignment.state) || (assignment.state === 'Blocked' && Boolean(assignment.worktree_id || assignment.implementor_run))))

// Planning phases in which a planner run is in progress (supervisor.rs `tick`).
const planningPhases = ['Provisioning', 'Planning', 'Validated']
/**
 * The goals whose running planner stops when `repository`'s settings change: supervisor.rs
 * refuses each further planning step with "repository configuration changed during planning"
 * once the planning repository's row differs.
 */
export const planningIn = (repository, goals) => (goals || []).filter(goal => goal.state === 'Running' && planningPhases.includes(goal.planning_phase) && goal.planning_repository === repository)
