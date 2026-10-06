// What the header and the attention strip say, derived from one projection frame (the fields
// `compact` in crates/control-plane-app/src/live.rs documents). Nothing here reads the browser
// clock: ages take `now`, the server clock the caller derives from the frame's `server_time`.
import { deriveGoalState } from './goalState.js'

/** Milliseconds since the epoch of an RFC 3339 time, or null. Digits past milliseconds are cut. */
export function parseTime(text) {
  if (typeof text !== 'string' || !text) return null
  const value = Date.parse(text.replace(/(\.\d{3})\d+/, '$1'))
  return Number.isFinite(value) ? value : null
}

/** An elapsed time in words: `45 s ago`, `5 min ago`, `1 h 7 min ago`, `3 d ago`. */
export function formatAge(milliseconds) {
  const seconds = Math.max(0, Math.floor(milliseconds / 1000))
  if (seconds < 60) return `${seconds} s ago`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes} min ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 48) return minutes % 60 ? `${hours} h ${minutes % 60} min ago` : `${hours} h ago`
  return `${Math.floor(hours / 24)} d ago`
}

/** The age of an RFC 3339 time at `now`, or '' when either is unknown. */
export function ageSince(at, now) {
  const then = parseTime(at)
  return then === null || typeof now !== 'number' ? '' : formatAge(now - then)
}

/** The index just past the bracket closing the `(` or `{` at `open`, skipping quoted text; -1 if
 * none. */
function closing(text, open) {
  const [start, end] = text[open] === '{' ? ['{', '}'] : ['(', ')']
  let depth = 0
  for (let at = open; at < text.length; at++) {
    const character = text[at]
    if (character === '"') {
      for (at++; at < text.length && text[at] !== '"'; at++) if (text[at] === '\\') at++
    } else if (character === start) depth++
    else if (character === end && --depth === 0) return at + 1
  }
  return -1
}

/** The text of a Rust string literal (`"…"` with Debug escapes), or the input unchanged. */
function unquote(text) {
  const literal = text.trim().match(/^"((?:[^"\\]|\\.)*)"$/s)
  if (!literal) return text
  return literal[1].replace(/\\(?:u\{([0-9a-fA-F]{1,6})\}|(.))/gs, (_, hex, character) => hex ? String.fromCodePoint(parseInt(hex, 16)) : ({ n: '\n', r: '\r', t: '\t' })[character] ?? character)
}

/**
 * Replace each debug wrapper by the words it carries: a tuple variant `Variant(…)` by its
 * contents, a struct `Name { field: value, … }` by the text of its string fields (kept unchanged
 * when it has none). An all-capitals word such as `HTTP(S)` is not a variant and keeps its text.
 */
function unwrap(text) {
  const wrapper = /\b[A-Z][a-z][A-Za-z0-9]*(?:\(| \{)/g
  let result = '', at = 0, match
  while ((match = wrapper.exec(text))) {
    const open = match.index + match[0].length - 1, end = closing(text, open)
    if (end < 0) break
    const inner = text.slice(open + 1, end - 1)
    const strings = [...inner.matchAll(/"(?:[^"\\]|\\.)*"/gs)].map(literal => unquote(literal[0]))
    const words = text[open] === '(' ? unquote(unwrap(inner)) : strings.length ? strings.join(': ') : text.slice(match.index, end)
    result += text.slice(at, match.index) + words
    at = end
    wrapper.lastIndex = end
  }
  return unquote(result + text.slice(at))
}

/**
 * A recorded reason as words for the operator. Runtime reasons are error chains that may carry
 * Rust debug wrappers (`ExternalAvailability(Text("effect failed: model command is not
 * admitted"))`); the wrappers and quoting go, the words they carry stay. Empty gives ''.
 */
export function plainReason(text) {
  const words = unwrap(String(text ?? '')).replace(/\s+/g, ' ').trim()
  if (!words) return ''
  const sentence = words[0].toUpperCase() + words.slice(1)
  return sentence.length > 400 ? `${sentence.slice(0, 399)}…` : sentence
}

/** A goal's objective cut to one short line. */
export function shortGoal(goal) {
  const line = String(goal?.objective ?? '').split('\n')[0].trim() || 'Untitled goal'
  return line.length > 80 ? `${line.slice(0, 79)}…` : line
}

function index(view) {
  const goals = view?.goals || [], assignments = view?.assignments || [], repositories = view?.repositories || []
  return {
    goals, assignments, publications: view?.publications || [],
    goal: id => goals.find(goal => goal.goal_id === id),
    assignment: id => assignments.find(assignment => assignment.assignment_id === id),
    repository: id => repositories.find(repository => repository.repository_id === id)?.name || '',
  }
}

const controlsOf = goal => ({ control: 'Open goal controls', href: `/workspaces/${goal.workspace_id}#goal-${goal.goal_id}`, workspace_id: goal.workspace_id, anchor: `goal-${goal.goal_id}` })

/**
 * One row per item that needs the operator, goals first:
 *
 * - a goal whose derived state is `blocked` (Running in planning phase Blocked), aged from its
 *   newest planner entry, resolved through the goal's controls;
 * - a Blocked assignment of a Running or Paused goal at that goal's revision (an older
 *   revision's assignment is retired by the fleet, not by the operator), aged from the
 *   assignment's newest entry, resolved through its goal's controls;
 * - an Uncertain publication, aged from its assignment's newest entry, with its evidence.
 *
 * `at` is the recorded time the row's age is measured from, or undefined when none is recorded.
 */
export function attentionItems(view) {
  const rows = index(view), items = []
  for (const goal of rows.goals) {
    if (deriveGoalState(goal) !== 'blocked') continue
    items.push({ key: `goal:${goal.goal_id}`, kind: 'Blocked goal', goal: shortGoal(goal), repository: rows.repository(goal.planning_repository), reason: plainReason(goal.planning_reason) || 'Planning stopped without a recorded reason.', at: (goal.planner_activity || goal.last_activity)?.at, ...controlsOf(goal) })
  }
  for (const assignment of rows.assignments) {
    const goal = rows.goal(assignment.goal_id)
    if (assignment.state !== 'Blocked' || !goal || !['Running', 'Paused'].includes(goal.state) || assignment.goal_revision !== goal.revision) continue
    items.push({ key: `assignment:${assignment.assignment_id}`, kind: 'Blocked assignment', goal: shortGoal(goal), repository: rows.repository(assignment.repository_id), reason: plainReason(assignment.reason) || 'The assignment stopped without a recorded reason.', at: goal.fleet?.[assignment.assignment_id]?.at, ...controlsOf(goal) })
  }
  for (const publication of rows.publications) {
    if (publication.state !== 'Uncertain') continue
    const assignment = rows.assignment(publication.assignment_id), goal = assignment && rows.goal(assignment.goal_id)
    const target = publication.target || 'its target branch'
    items.push({
      key: `publication:${publication.publication_id}`, kind: 'Uncertain publication', goal: goal ? shortGoal(goal) : 'Unknown goal', repository: assignment ? rows.repository(assignment.repository_id) : '',
      reason: `Publishing to ${target} was not confirmed, so the change may or may not be on ${target}. Check ${target}: the service confirms the publication once the change is there and never publishes it twice.`,
      at: goal?.fleet?.[assignment.assignment_id]?.at,
      ...(goal ? { control: 'Inspect evidence', href: `/goals/${goal.goal_id}/evidence`, external: true } : { control: '', href: undefined }),
    })
  }
  return items
}

const assignmentRoles = { Implementing: 'implementor', Reviewing: 'reviewer', Merging: 'merging' }
const waitingRoles = { planner: 'planner', critic: 'plan critic', goal_reviewer: 'goal review', implementor: 'implementor', reviewer: 'reviewer' }
const listed = lanes => lanes.length > 2 ? `${lanes[0]} and ${lanes.length - 1} more` : lanes.join(', ')

/**
 * What the system is doing, first match wins:
 *
 * | state | when |
 * |---|---|
 * | `disconnected` | the stream is closed or failed (`link` is `disconnected`) |
 * | `connecting` | no frame has arrived on the current connection |
 * | `blocked` | the runtime stopped (`runtime_error`) |
 * | `working` | a lane of a Running goal is busy with no open model call: the planner while the goal is planning, an assignment of its current revision while Implementing, Reviewing or Merging |
 * | `waiting` | a model call is open (`waiting` on a goal or an assignment) |
 * | `paused` | no goal is Running and at least one is Paused |
 * | `blocked` | an item needs the operator ({@link attentionItems}) |
 * | `idle` | otherwise |
 *
 * `detail` names the lanes (role and repository), the reason, or what is known; `since` is the
 * open call's request time for `waiting`.
 */
export function systemStatus(view, link) {
  if (link === 'disconnected') return { state: 'disconnected', label: 'Disconnected', detail: 'What the system is doing is unknown until the live stream reconnects.' }
  if (link !== 'live' || !view) return { state: 'connecting', label: 'Connecting', detail: 'Waiting for the current committed state.' }
  if (typeof view.runtime_error === 'string' && view.runtime_error) return { state: 'blocked', label: 'Blocked', detail: `Autonomous processing stopped: ${plainReason(view.runtime_error)}` }
  const rows = index(view), working = [], waiting = []
  const where = (goal, repository) => rows.repository(repository) || shortGoal(goal)
  for (const goal of rows.goals) {
    if (goal.state !== 'Running') continue
    if (goal.waiting) waiting.push({ ...goal.waiting, lane: `${waitingRoles[goal.waiting.role] || goal.waiting.role} in ${where(goal, goal.planning_repository)}` })
    else if (deriveGoalState(goal) === 'planning') working.push(`planner in ${where(goal, goal.planning_repository)}`)
  }
  for (const assignment of rows.assignments) {
    const goal = rows.goal(assignment.goal_id)
    if (goal?.state !== 'Running' || goal.revision !== assignment.goal_revision) continue
    if (assignment.waiting) waiting.push({ ...assignment.waiting, lane: `${waitingRoles[assignment.waiting.role] || assignment.waiting.role} in ${where(goal, assignment.repository_id)}` })
    else if (assignmentRoles[assignment.state]) working.push(`${assignmentRoles[assignment.state]} in ${where(goal, assignment.repository_id)}`)
  }
  if (working.length) return { state: 'working', label: 'Working', detail: listed(working) }
  if (waiting.length) return { state: 'waiting', label: `Waiting for ${waiting[0].model || 'a model'}`, detail: listed(waiting.map(call => call.lane)), since: waiting[0].since }
  const items = attentionItems(view), need = items.length === 1 ? '1 item needs you' : `${items.length} items need you`
  const running = rows.goals.filter(goal => goal.state === 'Running').length, paused = rows.goals.filter(goal => goal.state === 'Paused').length
  if (!running && paused) return { state: 'paused', label: 'Paused', detail: `${paused === 1 ? 'The goal is' : `All ${paused} goals are`} paused${items.length ? `; ${need}` : ''}.` }
  if (items.length) return { state: 'blocked', label: 'Blocked', detail: items.length === 1 ? items[0].reason : `${need}; see below.` }
  return { state: 'idle', label: 'Idle', detail: running ? `${running === 1 ? '1 goal is' : `${running} goals are`} running; no step is in progress.` : 'No goal is running.' }
}

/** The newest recorded activity time of any goal, role or assignment, or null. */
export function newestActivity(view) {
  let newest = null
  for (const goal of view?.goals || []) {
    for (const entry of [goal.last_activity, goal.planner_activity, ...(goal.activity || []), ...Object.values(goal.fleet || {})]) {
      const at = parseTime(entry?.at)
      if (at !== null && (newest === null || at > newest)) newest = at
    }
  }
  return newest
}

/** The newest planner entry (`planner_activity`), Running goals first, or null. */
export function plannerLine(view) {
  const goals = (view?.goals || []).filter(goal => goal.planner_activity && parseTime(goal.planner_activity.at) !== null)
  const pool = goals.some(goal => goal.state === 'Running') ? goals.filter(goal => goal.state === 'Running') : goals
  const goal = pool.reduce((newest, goal) => !newest || parseTime(goal.planner_activity.at) > parseTime(newest.planner_activity.at) ? goal : newest, null)
  if (!goal) return null
  const entry = goal.planner_activity
  return { goal: shortGoal(goal), detail: plainReason(entry.detail) || entry.action, at: entry.at }
}
