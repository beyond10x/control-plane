// What the header and the attention strip say, derived from one projection frame (the fields
// `compact` in crates/control-plane-app/src/live.rs documents). Nothing here reads the browser
// clock: ages take `now`, the server clock the caller derives from the frame's `server_time`.
import { acceptanceRejection, acceptanceStep, deriveGoalState } from './goalState.js'
import { parseTime } from './time.js'

export { parseTime }

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

// Runtime reasons are anyhow error chains (links joined by `: `), and some links are Rust `{:?}`
// output of a value: `ExternalAvailability(Text("…"))`, Loom's `NoAdmissibleAction(Unit(true))`
// (engine.rs) or `MaxTurns { limit: 1 }` (loom_model.rs). Every runtime site that formats a
// value with `{:?}` puts it as a whole link (`Loom stopped: {:?}`, `… proposal: {:?}`,
// `{e:?}`). Such a value is parsed strictly (text that is not Debug syntax stays as written) and
// rendered as words; text inside it, such as Loom's outage message, is rendered the same way.

const unescape = body => body.replace(/\\(?:u\{([0-9a-fA-F]{1,6})\}|(.))/gs, (_, hex, character) => hex ? String.fromCodePoint(parseInt(hex, 16)) : ({ n: '\n', r: '\r', t: '\t', 0: '' })[character] ?? character)
/** An identifier as words: `NoAdmissibleAction` → `no admissible action`, `asked_again` → `asked again`. */
const nameWords = name => name.replace(/_/g, ' ').replace(/([a-z0-9])([A-Z])/g, '$1 $2').replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2').trim().toLowerCase()
const skip = (text, at) => { while (at < text.length && /\s/.test(text[at])) at++; return at }
const identifier = /^[A-Za-z_][A-Za-z0-9_]*/

/**
 * One Debug value at `at`, as `{ node, end }`, or null when the text there is not one. Nodes:
 * `string`, `number` (with a unit suffix such as `600s`), `bool`, `list` (`[…]`, also
 * `Array […]`), `tuple` (`Name(…)`, or an unnamed `(…)` such as `()`), `struct` (`Name { field: value }`, or a map
 * `{"key": value}` / `Object {…}`) and `unit` (`Name`, `None`). A value name starts upper-case.
 */
function parseValue(text, at) {
  at = skip(text, at)
  const first = text[at]
  if (first === '"' || first === "'") {
    let end = at + 1
    for (; end < text.length && text[end] !== first; end++) if (text[end] === '\\') end++
    return end < text.length ? { node: { kind: 'string', value: unescape(text.slice(at + 1, end)) }, end: end + 1 } : null
  }
  const number = /^-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?[a-zµ]*/.exec(text.slice(at))
  if (number) return { node: { kind: 'number', value: number[0] }, end: at + number[0].length }
  if (first === '[') return sequence(text, at + 1, ']', items => ({ kind: 'list', items }))
  if (first === '(') return sequence(text, at + 1, ')', items => ({ kind: 'tuple', name: '', items }))
  if (first === '{') return fields(text, at + 1, '')
  const name = identifier.exec(text.slice(at))?.[0]
  if (!name) return null
  const end = at + name.length
  if (name === 'true' || name === 'false') return { node: { kind: 'bool', value: name === 'true' }, end }
  if (!/^[A-Z]/.test(name)) return null
  if (text[end] === '(') return sequence(text, end + 1, ')', items => ({ kind: 'tuple', name, items }))
  const next = skip(text, end)
  if (text[next] === '{') return fields(text, next + 1, name)
  if (text[next] === '[' && next > end) return sequence(text, next + 1, ']', items => ({ kind: 'list', items }))
  return { node: { kind: 'unit', name }, end }
}

/** Comma-separated values up to `close`; a trailing comma is allowed. */
function sequence(text, at, close, make) {
  const items = []
  for (;;) {
    at = skip(text, at)
    if (text[at] === close) return { node: make(items), end: at + 1 }
    const item = parseValue(text, at)
    if (!item) return null
    items.push(item.node)
    at = skip(text, item.end)
    if (text[at] === ',') at++
    else if (text[at] !== close) return null
  }
}

/** `key: value` pairs up to `}`; a key is a field name, a string or a number, and `..` (a
 * non-exhaustive struct) is skipped. */
function fields(text, at, name) {
  const pairs = []
  for (;;) {
    at = skip(text, at)
    if (text[at] === '}') return { node: { kind: 'struct', name, fields: pairs }, end: at + 1 }
    if (text.startsWith('..', at)) { at = skip(text, at + 2); if (text[at] === ',') at++; continue }
    let key
    if (text[at] === '"' || /[-\d]/.test(text[at] ?? '')) {
      const literal = parseValue(text, at)
      if (!literal || !['string', 'number'].includes(literal.node.kind)) return null
      key = literal.node.value; at = literal.end
    } else {
      key = identifier.exec(text.slice(at))?.[0]
      if (!key) return null
      at += key.length
    }
    at = skip(text, at)
    if (text[at] !== ':') return null
    const value = parseValue(text, at + 1)
    if (!value) return null
    pairs.push([key, value.node])
    at = skip(text, value.end)
    if (text[at] === ',') at++
    else if (text[at] !== '}') return null
  }
}

// Wrappers that add no meaning of their own; and struct names that are not a subject: maps, and
// the forms std::io::Error prints (`Os { code, kind, message }`, `Custom { kind, error }`).
const transparent = new Set(['', 'Some', 'Ok', 'Err', 'Box', 'Rc', 'Arc', 'String', 'Number', 'Bool', 'Text'])
const anonymous = new Set(['', 'Object', 'Map', 'HashMap', 'BTreeMap', 'IndexMap', 'Os', 'Custom', 'Simple', 'SimpleMessage'])
const duration = milliseconds => milliseconds >= 1000 ? `${Number((milliseconds / 1000).toFixed(1))} s` : `${milliseconds} ms`

/**
 * Loom's JSON value (`json::Value`: `Null`, `Bool(bool)`, `Number(String)`, `Text(String)`,
 * `Array(Vec<Value>)`, `Object(Vec<(String, Value)>)`) as the value it holds: `Object([("key",
 * value)])` is a map, `Array([…])` a list, `Number("8")` a number and `Null` nothing. `Bool` and
 * `Text` are plain wrappers already (`transparent`). Any other node is returned as it is.
 */
function plain(node) {
  if (node.kind === 'unit' && node.name === 'Null') return { kind: 'unit', name: 'None' }
  if (node.kind !== 'tuple' || node.items.length !== 1) return node
  const [item] = node.items
  if (node.name === 'Array' && item.kind === 'list') return item
  if (node.name === 'Number' && item.kind === 'string' && /^-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?$/.test(item.value)) return { kind: 'number', value: item.value }
  const pair = entry => entry.kind === 'tuple' && !entry.name && entry.items.length === 2 && entry.items[0].kind === 'string'
  if (node.name === 'Object' && item.kind === 'list' && item.items.every(pair)) return { kind: 'struct', name: '', fields: item.items.map(entry => [entry.items[0].value, entry.items[1]]) }
  return node
}

/** A struct field as words; units carried by the field name (`_ms`, `_micro_usd`) are applied.
 * A field with nothing to say (`None`, an empty list) is left out. */
function fieldWords(key, node, shown) {
  if (!shown.text) return ''
  const plain = node.kind === 'number' && /^-?\d+(?:\.\d+)?$/.test(node.value) ? Number(node.value) : null
  if (plain !== null && /_ms$/.test(key)) return `${nameWords(key.slice(0, -3))} ${duration(plain)}`
  if (plain !== null && /_micro_usd$/.test(key)) return `${nameWords(key.slice(0, -10))} $${(plain / 1e6).toFixed(2)}`
  return `${nameWords(String(key))} ${shown.text}`
}

/**
 * A parsed value as `{ text, carried, detail }`. `carried` is true when the text is words the
 * value carries (its strings); `detail` when it holds data worth naming beside a name (numbers,
 * fields), not only names and flags. Rendering:
 *
 * - Loom's JSON values are first read as the value they hold (see `plain`);
 * - a string is its text, with the Debug values in it rendered too (see `words`), and a string
 *   that is one identifier (`"max_output_tokens"`) its words; a bool says nothing beside a name;
 *   `None` is empty;
 * - a tuple variant carrying strings is `name: strings` (`Text` and other plain wrappers add no
 *   name); otherwise its name in words, followed by its details: `NoAdmissibleAction(Unit(true))`
 *   is `no admissible action`;
 * - a struct carrying strings is `name: strings` (`Cancelled { reason: "stopping" }` is
 *   `cancelled: stopping`), without a name for a map, an io error form (`Os { code: 2, …,
 *   message: "No such file" }` is `No such file`) or the only item of a tuple variant, which
 *   already names it; otherwise its name in words and its fields: `MaxTurns { limit: 1 }` is
 *   `max turns (limit 1)` and `Deadline { limit_ms: 600000 }` is `deadline (limit 600 s)`.
 *
 * `options.unnamed` is the only-item-of-a-tuple case. It is an option, not a positional flag, so
 * an array method's index can never be read as it.
 */
function render(node, options = {}) {
  const unnamed = options?.unnamed === true
  node = plain(node)
  if (node.kind === 'string') {
    const value = node.value.trim()
    const token = /^(?:[a-z][a-z0-9]*(?:_[a-z0-9]+)+|[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]*)+)$/.test(value)
    return { text: token ? nameWords(value) : words(node.value), carried: value !== '', detail: value !== '' }
  }
  if (node.kind === 'number') return { text: node.value, carried: false, detail: true }
  if (node.kind === 'bool') return { text: node.value ? 'yes' : 'no', carried: false, detail: false }
  if (node.kind === 'unit') return { text: node.name === 'None' ? '' : nameWords(node.name), carried: false, detail: false }
  if (node.kind === 'list') {
    const items = node.items.map(item => render(item)), carried = items.filter(item => item.carried)
    if (carried.length) return { text: carried.map(item => item.text).join(', '), carried: true, detail: true }
    const shown = items.filter(item => item.text)
    return { text: shown.map(item => item.text).join(', '), carried: false, detail: shown.some(item => item.detail) }
  }
  if (node.kind === 'tuple') {
    const items = node.items.map(item => render(item, { unnamed: node.items.length === 1 && !transparent.has(node.name) }))
    if (transparent.has(node.name) && items.length === 1) return items[0]
    const carried = items.filter(item => item.carried)
    if (carried.length) return { text: [nameWords(node.name), ...carried.map(item => item.text)].filter(Boolean).join(': '), carried: true, detail: true }
    const shown = items.filter(item => item.detail && item.text).map(item => item.text)
    if (!node.name) return { text: shown.join(', '), carried: false, detail: shown.length > 0 }
    return { text: shown.length ? `${nameWords(node.name)} (${shown.join(', ')})` : nameWords(node.name), carried: false, detail: shown.length > 0 }
  }
  const values = node.fields.map(([key, value]) => [key, plain(value), render(value)]), carried = values.filter(([, , shown]) => shown.carried)
  const name = unnamed || anonymous.has(node.name) ? '' : nameWords(node.name)
  if (carried.length) return { text: [name, ...carried.map(([, , shown]) => shown.text)].filter(Boolean).join(': '), carried: true, detail: true }
  const shown = values.map(([key, value, rendered]) => fieldWords(key, value, rendered)).filter(Boolean), inner = shown.join(', ')
  return { text: name && inner ? `${name} (${inner})` : name || inner, carried: false, detail: shown.length > 0 }
}

// A reason link starts at the start of the text or after `:` and white space, and ends at the
// end of the text or before them.
const startsLink = (text, at) => /(?:^|:\s)\s*$/.test(text.slice(0, at))
const endsLink = (text, at) => /^\s*(?::\s|:?$)/.test(text.slice(at))

/** Replace each Debug value in a reason by its words. A value starts at an upper-case name that
 * is not all capitals (`HTTP(S)` is prose) followed by `(`, `{` or ` [`, and is replaced only
 * when it is a whole reason link and renders as some words: prose that quotes code (`returns
 * Ok(()) even when`, `returns Some(0)`) stays as written. */
function words(text) {
  const start = /\b[A-Z][A-Za-z0-9_]*/g
  let result = '', at = 0, match
  while ((match = start.exec(text))) {
    if (!/[a-z]/.test(match[0]) || !/^\s*[({]|^ \[/.test(text.slice(match.index + match[0].length))) continue
    const value = parseValue(text, match.index)
    if (!value || value.node.kind === 'unit' || !startsLink(text, match.index) || !endsLink(text, value.end)) continue
    const shown = render(value.node).text
    if (!shown.trim()) continue
    result += text.slice(at, match.index) + shown
    at = value.end
    start.lastIndex = value.end
  }
  result += text.slice(at)
  const whole = parseValue(result.trim(), 0)
  return whole && whole.node.kind === 'string' && skip(result.trim(), whole.end) === result.trim().length ? whole.node.value : result
}

/**
 * A recorded reason as words for the operator: Debug values are rendered as words (see
 * `render`), quoting goes, and the sentence starts upper-case. Empty gives ''.
 */
export function plainReason(text) {
  const plain = words(String(text ?? '')).replace(/\s+/g, ' ').trim()
  if (!plain) return ''
  const sentence = plain[0].toUpperCase() + plain.slice(1)
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
 * - a goal Running in planning phase Blocked, or whose derived state is `stalled` (Running in
 *   phase Queued with no current work, see goalState.js), with its planning reason, aged from
 *   its newest planner entry, resolved through the goal's controls (a goal held only by blocked
 *   assignments derives `blocked` but gets no row: the assignments' rows carry the reasons); a
 *   stalled goal whose final goal review rejected it (`acceptanceRejection`) states that
 *   rejection instead, aged from it;
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
    const derived = deriveGoalState(goal, rows.assignments)
    // A goal blocked only by its blocked assignments gets no row of its own: their rows say why.
    const stalled = derived === 'stalled'
    if (!stalled && !(derived === 'blocked' && goal.planning_phase === 'Blocked')) continue
    // A rejected goal review is newer than the plan it followed and is why the goal stopped.
    const rejection = stalled ? acceptanceRejection(goal) : null, rejected = rejection ? plainReason(rejection.detail) : ''
    items.push({ key: `goal:${goal.goal_id}`, kind: stalled ? 'Stalled goal' : 'Blocked goal', goal: shortGoal(goal), repository: rows.repository(goal.planning_repository), reason: rejected || plainReason(goal.planning_reason) || (stalled ? 'The plan is queued, but no assignment of this goal is queued or running.' : 'Planning stopped without a recorded reason.'), at: (rejected ? rejection : goal.planner_activity || goal.last_activity)?.at, ...controlsOf(goal) })
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
 * | `waiting` | a model call is open anywhere (`waiting` on a Running goal or on an assignment of its current revision); the label names the model of the oldest open call |
 * | `working` | no call is open and a lane of a Running goal is busy: the planner while the goal is planning, the goal's acceptance checks or goal review while they run (`acceptanceStep`), an assignment of its current revision while Implementing, Reviewing or Merging |
 * | `paused` | no goal is Running and at least one is Paused |
 * | `blocked` | an item needs the operator ({@link attentionItems}) |
 * | `idle` | otherwise |
 *
 * `detail` names the lanes (role and repository), the reason, or what is known. For `waiting`,
 * `since` is the oldest open call's request time and `also` names the lanes working meanwhile.
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
    else if (deriveGoalState(goal, rows.assignments) === 'planning') working.push(`planner in ${where(goal, goal.planning_repository)}`)
    else {
      // The checks run in the repository whose assignment carries the step; the goal review,
      // like its open call above, belongs to the goal.
      const step = acceptanceStep(goal, rows.assignments)
      if (step) working.push(step.entry.action === 'goal.checks' ? `acceptance checks in ${where(goal, step.assignment.repository_id)}` : `goal review in ${where(goal, goal.planning_repository)}`)
    }
  }
  for (const assignment of rows.assignments) {
    const goal = rows.goal(assignment.goal_id)
    if (goal?.state !== 'Running' || goal.revision !== assignment.goal_revision) continue
    if (assignment.waiting) waiting.push({ ...assignment.waiting, lane: `${waitingRoles[assignment.waiting.role] || assignment.waiting.role} in ${where(goal, assignment.repository_id)}` })
    else if (assignmentRoles[assignment.state]) working.push(`${assignmentRoles[assignment.state]} in ${where(goal, assignment.repository_id)}`)
  }
  if (waiting.length) {
    const age = call => parseTime(call.since) ?? Infinity
    waiting.sort((first, second) => age(first) - age(second))
    return { state: 'waiting', label: `Waiting for ${waiting[0].model || 'a model'}`, detail: listed(waiting.map(call => call.lane)), since: waiting[0].since, also: working.length ? `${working.length} also working: ${listed(working)}` : '' }
  }
  if (working.length) return { state: 'working', label: 'Working', detail: listed(working) }
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
