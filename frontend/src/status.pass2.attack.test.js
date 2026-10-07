import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import { plainReason } from './status.js'
// Adversary pass 2 for story:console-status-and-attention (head 343561b). Each case mounts the
// console on the unit's own frame (a Running goal in phase Queued, assignment `alpha` Implementing
// with no open call, `beta` Queued, server_time 10:00) and changes only what its scenario names.
import recorded from './fixtures/status-frame.json'

class Stream {
  static opened = []
  constructor(url) { this.url = url; this.listeners = {}; Stream.opened.push(this) }
  addEventListener(type, listener) { (this.listeners[type] ||= []).push(listener) }
  close() { this.closed = true }
  deliver(type, data) { for (const listener of this.listeners[type] || []) listener({ type, data }) }
}
const answer = body => ({ ok: true, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
let wrapper

beforeEach(() => {
  Stream.opened = []
  history.replaceState({}, '', '/')
  vi.stubGlobal('EventSource', Stream)
  vi.stubGlobal('fetch', vi.fn(async path => path === '/api/session' ? answer({ csrf_token: 'test-token' }) : answer({ workspaces: [] })))
})
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; vi.unstubAllGlobals() })

const frame = () => structuredClone(recorded)
const goalOf = view => view.goals[0]
const [alpha, beta] = recorded.assignments.map(a => a.assignment_id)
const assignment = (view, id) => view.assignments.find(a => a.assignment_id === id)
// A fleet entry as live.rs `observation` projects it: `detail` is the entry's command or reason.
const entry = (at, action, role, status, detail, worktree = 'tree-alpha') => ({ action, at, detail, goal_revision: 1, id: `entry-${action}-${at}`, role, status, worktree })
async function show(view) {
  wrapper = mount(App, { attachTo: document.body }); await flushPromises()
  Stream.opened.at(-1).deliver('operations', JSON.stringify(view)); await flushPromises()
}
const rows = () => wrapper.findAll('.attention-row')
const reasons = () => wrapper.findAll('.attention-row .attention-reason').map(reason => reason.text())
const card = objective => wrapper.findAll('article.operation:not(.worker)').find(article => article.get('h3').text() === objective)
// The unit's own definition of an error identifier (status.test.js:43).
const identifier = /\b[A-Z][a-z]+[A-Z]\w*\b|\b[A-Z]\w*\(/

// A planner model call that stops (loom_model.rs:144-148, `Loom stopped: {:?}`) fails the
// planner's selector (engine.rs:764, SelectorError::Unavailable(e.to_string())); Loom's executor
// turns that into `Suspended` with `ExternalAvailability(Object([("error", Text(<message>))]))`
// (loom-executor lib.rs:173-180, 221-222; Loom's json::Value::Object is a tuple of a Vec of
// pairs), and engine.rs:192-195 records the run outcome's Debug form, which supervisor.rs:160
// and :186 store as the planning reason with phase Blocked.
test('attack2_planner_model_stop_inside_an_outage_is_rendered_as_words', async () => {
  const view = frame()
  Object.assign(goalOf(view), { planning_phase: 'Blocked', planning_reason: 'planner ended without a validated plan: Suspended(RunOutcomeSuspended { reason: ExternalAvailability(Object([("error", Text("Loom stopped: Deadline { limit_ms: 600000 }"))])) })' })
  view.assignments = []
  await show(view)
  expect(reasons()).toHaveLength(1)
  const [reason] = reasons()
  expect.soft(reason).toMatch(/deadline/i)
  expect.soft(reason).not.toMatch(identifier)
  expect.soft(reason).not.toMatch(/[{}]|\b[a-z]+_[a-z]+\b/)
  // The render doc comment: a map is rendered without a name.
  expect.soft(reason).not.toMatch(/\bobject\b/i)
})

// The implementor's selector fails the same way (fleet.rs:1236); the run ends suspended before a
// candidate proposal and fleet.rs:1153-1157 records `{:?}` of the outcome, which fleet.rs:497-499
// stores as the assignment's reason.
test('attack2_implementor_model_stop_inside_an_outage_is_rendered_as_words', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Blocked', reason: 'implementation ended before a candidate proposal: Suspended(RunOutcomeSuspended { reason: ExternalAvailability(Object([("error", Text("Loom stopped: MaxTurns { limit: 40 }"))])) })' })
  await show(view)
  expect(reasons()).toHaveLength(1)
  const [reason] = reasons()
  expect.soft(reason).toMatch(/max turns/i)
  expect.soft(reason).not.toMatch(identifier)
  expect.soft(reason).not.toMatch(/[{}]/)
})

// A rejected review blocks the assignment with `independent review rejected candidate: {review}`
// (fleet.rs:955-961, the critique JSON of model.rs:62-63), and fleet.rs:497-499 stores it as the
// reason. The reviewer's prose quotes code; the parser must not delete or rewrite it.
test('attack2_reviewer_prose_keeps_its_code_references', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Blocked', reason: 'independent review rejected candidate: {"approved":false,"reason":"publish() returns Ok(()) even when the push fails, and count() returns Some(0) for an empty queue"}' })
  await show(view)
  expect(reasons()).toHaveLength(1)
  const [reason] = reasons()
  expect.soft(reason).toContain('returns Ok(()) even when the push fails')
  expect.soft(reason).toContain('returns Some(0) for an empty queue')
})

// Once every current assignment is Merged, the same fleet tick runs goal acceptance
// (fleet.rs:2001-2006, called from `run` at :513): it records `goal.checks` on the assignment and runs
// the repository's test command (fleet.rs:2104-2111) before the goal review call opens a wait.
// The goal is Running in phase Queued with no open call while those checks run; it is working,
// not stalled, and nothing needs the operator.
test('attack2_goal_acceptance_checks_are_not_a_stall', async () => {
  const view = frame()
  const goal = goalOf(view)
  Object.assign(assignment(view, alpha), { state: 'Merged', merged_at: '2026-10-06T09:57:00Z' })
  Object.assign(assignment(view, beta), { state: 'Merged', merged_at: '2026-10-06T09:58:00Z', attempt: 1, worktree_id: 'tree-beta' })
  const checks = entry('2026-10-06T09:59:00Z', 'goal.checks', 'host', 'running', 'cargo test --locked')
  goal.fleet = { [alpha]: checks, [beta]: entry('2026-10-06T09:58:00Z', 'merge.observed', 'host', 'running', 'merged', 'tree-beta') }
  goal.last_activity = checks
  await show(view)
  expect.soft(card('Add a status endpoint').get('.badge').text()).not.toBe('Stalled')
  expect.soft(rows()).toHaveLength(0)
  expect.soft(wrapper.get('#system-status').text()).not.toBe('Blocked')
})

// When the final goal review rejects, satisfy_goals latches the failure (fleet.rs:2238-2262): it
// records a `blocked` entry with the reason on the assignment, and neither the supervisor
// (supervisor.rs:100-112) nor the fleet (fleet.rs:405-414, :2037-2039) plans or checks that goal again. The
// planning reason stays as the last plan left it (empty: it queued work). The stall row must say
// why the goal stopped.
test('attack2_stalled_goal_after_rejected_acceptance_states_the_rejection', async () => {
  const view = frame()
  const goal = goalOf(view)
  Object.assign(assignment(view, alpha), { state: 'Merged', merged_at: '2026-10-06T09:57:00Z' })
  Object.assign(assignment(view, beta), { state: 'Merged', merged_at: '2026-10-06T09:58:00Z', attempt: 1, worktree_id: 'tree-beta' })
  const rejected = entry('2026-10-06T09:59:30Z', 'blocked', 'goal_reviewer', 'failed', 'Goal acceptance blocked: final goal review rejected: {"approved":false,"reason":"GET /status answers 500 on the observed target"}')
  goal.fleet = { [alpha]: rejected, [beta]: entry('2026-10-06T09:58:00Z', 'merge.observed', 'host', 'running', 'merged', 'tree-beta') }
  goal.last_activity = rejected
  await show(view)
  expect(rows()).toHaveLength(1)
  expect(reasons()[0]).toMatch(/goal review rejected/i)
})

// status.js:157 maps list items with `node.items.map(render)`, so `render` receives each item's
// index as its `unnamed` flag: every struct after the first in a list loses its name. No runtime
// reason found carries a list of structs (built, not observed).
test('attack2_every_struct_in_a_list_keeps_its_name', () => {
  const reason = plainReason('budgets exceeded: Stops([MaxTurns { limit: 1 }, Deadline { limit_ms: 600000 }])')
  expect(reason).toMatch(/max turns/i)
  expect(reason).toMatch(/deadline/i)
})
