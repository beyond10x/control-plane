import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import { deriveGoalState } from './goalState.js'
// One projection frame in the shape `compact` sends (crates/control-plane-app/src/live.rs): a
// Running goal in phase Queued whose assignment in `alpha` is Implementing with no open model
// call, a queued assignment in `beta`, the planner's last entry at 09:50 and a newer worker entry
// at 09:55, and `server_time` 10:00. Each test changes only the fields its scenario names.
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
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; vi.unstubAllGlobals(); vi.useRealTimers() })

const frame = () => structuredClone(recorded)
const goalOf = view => view.goals[0]
const [alpha, beta] = recorded.assignments.map(a => a.assignment_id)
const assignment = (view, id) => view.assignments.find(a => a.assignment_id === id)
const entry = (at, action, role, detail) => ({ action, at, detail, goal_revision: 1, id: `entry-${action}-${at}`, role, status: 'running', worktree: '' })

async function open() { wrapper = mount(App, { attachTo: document.body }); await flushPromises(); return Stream.opened.at(-1) }
async function deliver(stream, view) { stream.deliver('operations', JSON.stringify(view)); await flushPromises() }
async function show(view) { await deliver(await open(), view) }
const text = selector => wrapper.get(selector).text()
const counts = () => wrapper.findAll('.metric strong').map(count => count.text())
const card = objective => wrapper.findAll('article.operation:not(.worker)').find(article => article.get('h3').text() === objective)
const rows = () => wrapper.findAll('.attention-row')
// An error identifier: a Rust variant or a debug wrapper such as `ExternalAvailability(Text("…"))`.
const identifier = /\b[A-Z][a-z]+[A-Z]\w*\b|\b[A-Z]\w*\(/

test('disconnected_is_not_idle', async () => {
  const stream = await open()
  stream.deliver('error'); await flushPromises()
  expect(text('#system-status')).toBe('Disconnected')
  expect(counts()).toEqual(['—', '—', '—', '—'])

  const settled = frame()
  goalOf(settled).state = 'Satisfied'
  for (const a of settled.assignments) a.state = 'Merged'
  await deliver(stream, settled)
  expect(text('#system-status')).toBe('Idle')
  expect(counts()).toEqual(['0', '0', '0', '0'])

  stream.deliver('error'); await flushPromises()
  expect(text('#system-status')).toBe('Disconnected')
  expect(wrapper.get('.status-header').text()).not.toMatch(/idle/i)
  expect(counts()).toEqual(['—', '—', '—', '—'])
})

test('paused_is_shown_as_paused', async () => {
  const view = frame()
  const second = { ...structuredClone(goalOf(view)), goal_id: '6f1d2c3b-0000-4000-8000-0000000000c2', objective: 'Document the status endpoint', planning_phase: 'Blocked', planning_reason: 'tool refused' }
  view.goals.push(second)
  for (const goal of view.goals) goal.state = 'Paused'
  for (const a of view.assignments) a.state = 'Queued'
  await show(view)
  expect(text('#system-status')).toBe('Paused')
})

test('working_is_shown_with_its_activity', async () => {
  await show(frame())
  expect(text('#system-status')).toBe('Working')
  expect(text('#system-detail')).toContain('implementor')
  expect(text('#system-detail')).toContain('alpha')
})

test('blocked_is_shown_as_blocked', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Blocked', reason: 'Tests still fail after 2 attempts' })
  await show(view)
  expect(text('#system-status')).toBe('Blocked')
  expect(text('#system-detail')).toContain('Tests still fail after 2 attempts')
})

test('idle_and_waiting_are_distinct', async () => {
  const stream = await open()
  const implementing = frame()
  assignment(implementing, alpha).waiting = { role: 'implementor', model: 'fixture-implementor', since: '2026-10-06T09:59:20Z' }
  await deliver(stream, implementing)
  expect(text('#system-status').toLowerCase()).toBe('waiting for fixture-implementor')
  expect(text('#system-detail')).toContain('alpha')

  const planning = frame()
  Object.assign(goalOf(planning), { planning_phase: 'Planning', waiting: { role: 'planner', model: 'fixture-planner', since: '2026-10-06T09:59:50Z' } })
  for (const a of planning.assignments) a.state = 'Cancelled'
  await deliver(stream, planning)
  expect(text('#system-status').toLowerCase()).toBe('waiting for fixture-planner')

  const settled = frame()
  goalOf(settled).state = 'Satisfied'
  for (const a of settled.assignments) a.state = 'Merged'
  await deliver(stream, settled)
  expect(text('#system-status')).toBe('Idle')
  expect(wrapper.get('.status-header').text()).not.toMatch(/waiting for/i)
})

test('last_activity_age_is_shown', async () => {
  // The newest entry is the worker's at 09:55, five minutes before the server clock; the browser's
  // own clock, set years away from the fixture, must not enter the age.
  vi.useFakeTimers({ toFake: ['Date'] })
  vi.setSystemTime(new Date('2031-01-01T00:00:00Z'))
  await show(frame())
  expect(text('#last-activity')).toBe('Last activity 5 min ago')

  const later = frame()
  later.server_time = '2026-10-06T11:02:30.000Z'
  await deliver(Stream.opened.at(-1), later)
  expect(text('#last-activity')).toBe('Last activity 1 h 7 min ago')
})

test('derived_goal_state_combines_lifecycle_and_phase', () => {
  const phases = ['Idle', 'Provisioning', 'Planning', 'Validated', 'Queued', 'Blocked']
  const derived = state => phases.map(planning_phase => deriveGoalState({ state, planning_phase }))
  expect(deriveGoalState({ state: 'Running', planning_phase: 'Blocked' })).toBe('blocked')
  expect(deriveGoalState({ state: 'Running', planning_phase: 'Planning' })).toBe('planning')
  expect(derived('Cancelled')).toEqual(phases.map(() => 'cancelled'))
  expect(derived('Satisfied')).toEqual(phases.map(() => 'satisfied'))
  expect(derived('Paused')).toEqual(phases.map(() => 'paused'))
  expect(derived('Running')).toEqual(['starting', 'planning', 'planning', 'planning', 'executing', 'blocked'])
  expect(deriveGoalState({ state: 'Running', planning_phase: 'Unheard' })).toBe('unknown')
  expect(deriveGoalState({ state: 'Unheard', planning_phase: 'Queued' })).toBe('unknown')
})

test('blocked_goal_is_not_green', async () => {
  const view = frame()
  Object.assign(goalOf(view), { planning_phase: 'Blocked', planning_reason: 'ExternalAvailability(Text("effect failed: model command is not admitted"))', planner_activity: entry('2026-10-06T09:58:00Z', 'planning.Blocked', 'planner', 'effect failed') })
  view.assignments = []
  await show(view)
  const badge = card('Add a status endpoint').get('.badge')
  expect(badge.text()).toBe('Blocked')
  expect(badge.classes()).toContain('blocked')
  expect(badge.classes()).not.toContain('active')
  expect(rows()).toHaveLength(1)
  expect(rows()[0].text()).toContain('Add a status endpoint')
  expect(rows()[0].get('.attention-reason').text()).toMatch(/model command is not admitted/i)
  expect(rows()[0].text()).not.toMatch(identifier)
  expect(rows()[0].get('.attention-age').text()).toBe('2 min ago')
})

test('attention_lists_each_blocked_item', async () => {
  const view = frame()
  const goal = goalOf(view)
  Object.assign(assignment(view, alpha), { state: 'Blocked', reason: 'ExternalAvailability(Text("effect failed: model command is not admitted"))' })
  Object.assign(assignment(view, beta), { state: 'Blocked', reason: 'Publication outcome remains unresolved; no duplicate effect will be invoked' })
  goal.fleet = { [alpha]: entry('2026-10-06T09:57:00Z', 'blocked', 'host', 'model command is not admitted'), [beta]: entry('2026-10-06T09:52:00Z', 'blocked', 'host', 'publication unresolved') }
  view.publications = [{ assignment_id: beta, candidate: 'candidate-beta', expected_base: 'base-beta', publication_id: '6f1d2c3b-0000-4000-8000-0000000000e1', state: 'Uncertain', target: 'main' }]
  await show(view)

  expect(rows()).toHaveLength(3)
  const workspace = '/workspaces/6f1d2c3b-0000-4000-8000-000000000001#goal-6f1d2c3b-0000-4000-8000-0000000000c1'
  const evidence = '/goals/6f1d2c3b-0000-4000-8000-0000000000c1/evidence'
  const shown = rows().map(row => ({ subject: row.get('.attention-subject').text(), reason: row.get('.attention-reason').text(), age: row.get('.attention-age').text(), link: row.get('a.resolve').attributes('href'), control: row.get('a.resolve').text(), all: row.text() }))
  for (const row of shown) {
    expect(row.subject).toContain('Add a status endpoint')
    expect(row.all).not.toMatch(identifier)
    expect(row.reason.length).toBeGreaterThan(0)
    expect(row.control.length).toBeGreaterThan(0)
  }
  const inAlpha = shown.filter(row => row.subject.includes('alpha'))
  const inBeta = shown.filter(row => row.subject.includes('beta'))
  expect(inAlpha).toHaveLength(1)
  expect(inBeta).toHaveLength(2)
  expect(inAlpha[0]).toMatchObject({ age: '3 min ago', link: workspace })
  expect(inAlpha[0].reason).toMatch(/model command is not admitted/i)
  const [blocked, uncertain] = [inBeta.find(row => row.link === workspace), inBeta.find(row => row.link === evidence)]
  expect(blocked).toMatchObject({ age: '8 min ago' })
  expect(blocked.reason).toContain('Publication outcome remains unresolved')
  expect(uncertain).toMatchObject({ age: '8 min ago' })
  expect(uncertain.reason).toMatch(/main/)
  expect(counts()[3]).toBe('3')
})

test('planner_status_uses_planner_activity', async () => {
  // The fixture's worker entry (09:55, `cargo test --locked`) is newer than the planner's (09:50).
  await show(frame())
  expect(text('#planner-status')).toContain('Planning phase: Queued')
  expect(text('#planner-status')).toContain('10 min ago')
  expect(text('#planner-status')).not.toContain('cargo test')
  const planner = card('Add a status endpoint')
  expect(planner.text()).toContain('Planning phase: Queued')
  expect(planner.text()).not.toContain('cargo test')
})

test('satisfied_goal_shows_acceptance_recorded', async () => {
  const view = frame()
  const satisfied = goalOf(view)
  Object.assign(satisfied, { state: 'Satisfied', acceptance_recorded: true })
  const running = { ...structuredClone(satisfied), goal_id: '6f1d2c3b-0000-4000-8000-0000000000c2', objective: 'Document the status endpoint', state: 'Running', acceptance_recorded: false }
  const unrecorded = { ...structuredClone(satisfied), goal_id: '6f1d2c3b-0000-4000-8000-0000000000c3', objective: 'Retire the old endpoint', acceptance_recorded: false }
  view.goals.push(running, unrecorded)
  view.assignments = []
  await show(view)
  expect(card('Add a status endpoint').text()).toMatch(/acceptance recorded/i)
  expect(card('Document the status endpoint').text()).not.toMatch(/acceptance recorded/i)
  expect(card('Retire the old endpoint').text()).not.toMatch(/acceptance recorded/i)
})
