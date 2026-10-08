import { withoutIds } from './identifiers.js'

// Activity entries as sentences. Every action id the runtime records has one: the planner's model
// calls and Loom stream (engine.rs), its progress hooks `planner.<kind>` (engine.rs `progress`
// kinds and governance.rs `HostCases` kinds, labelled by supervisor.rs) and phases
// `planning.<phase>` (supervisor.rs), and the fleet's worker and acceptance steps (fleet.rs
// `progress`, `note` and `block`, and the governance kinds its source-delivery cases record). An
// id not listed here is shown as recorded, so a new runtime action is visible rather than
// mislabelled.
const sentences = {
  'planner.prepare-branch': 'The planner prepared its planning branch',
  'planner.governor-state': 'Recorded the planning governance state',
  'planner.loom-observation': 'Recorded a planning observation',
  'governor-state': 'Recorded the delivery governance state',
  'loom-observation': 'Recorded a delivery observation',
  'waiting': 'Waiting for the repository to be free',
  'model.requested': 'Sent a model request',
  'model.completed': 'Received the model response',
  'model.failed': 'The model call failed',
  'loom.event': 'Receiving model output',
  'aep.syntax_rejected': 'Refused a malformed planning command',
  'plan.review_rejected': 'The plan review rejected the plan',
  'planner.intent': 'The planner chose its next step',
  'planner.observation': 'The planner completed a repository operation',
  'planner.plan-approved': 'The independent plan review passed',
  'planner.accept-story': 'The planner accepted a story',
  'planner.adopt': 'The planner initialized the planning store',
  'planning.Idle': 'Planning is idle',
  'planning.Provisioning': 'Planning is preparing its worktree',
  'planning.Planning': 'Planning started',
  'planning.Validated': 'The plan was validated',
  'planning.Queued': 'The plan was queued for implementation',
  'planning.Blocked': 'Planning stopped',
  'worktree.prepare': 'Prepared a worktree',
  'model.request': 'Asked the implementation model',
  'review.request': 'Asked the review model',
  'tool.run': 'Started a command',
  'tool.run.completed': 'Finished a command',
  'file.write.completed': 'Wrote a file',
  'file.delete.completed': 'Deleted a file',
  'input.refused': 'Refused a model request',
  'attempt.repair': 'Started a repair attempt',
  'checks.run': 'Ran the repository checks',
  'blocked': 'Stopped and needs attention',
  'goal.checks': 'Started the goal acceptance checks',
  'goal.review': 'Started the final goal review',
  'goal.acceptance.completed': 'Completed the goal acceptance',
  'merge.completed': 'Merged the change',
  'publication.invoke': 'Started publishing the change',
}

/** The sentence for an action id, or the id unchanged when it is not a known action. */
export const activitySentence = action => Object.hasOwn(sentences, action) ? sentences[action] : action

// Actions whose `detail` (live.rs `observation`: command · program · path · reason · candidate)
// is a path, a hash or a bare hook name (supervisor.rs labels an unnamed planner kind with the
// kind itself). Their detail is shown inside the entry's details only.
const rawDetail = new Set(['checks.run', 'publication.invoke', 'file.write.completed', 'file.delete.completed', 'merge.completed', 'worktree.prepare', 'governor-state', 'loom-observation', 'planner.prepare-branch', 'planner.governor-state', 'planner.loom-observation'])

/** Whether an entry's detail is prose that may be shown in the open. */
export const detailIsProse = action => !rawDetail.has(action)

/** The part of an entry's detail shown in the open: prose without its identifiers, or ''. */
export const openDetail = event => detailIsProse(event?.action) ? withoutIds(event?.detail) : ''
