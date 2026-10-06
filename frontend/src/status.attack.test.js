import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
// Adversary pass 1 for story:console-status-and-attention. Each case mounts the console on the
// unit's own frame (a Running goal in phase Queued, assignment `alpha` Implementing with no open
// call, `beta` Queued, server_time 10:00) and changes only what its scenario names.
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
async function show(view) {
  wrapper = mount(App, { attachTo: document.body }); await flushPromises()
  Stream.opened.at(-1).deliver('operations', JSON.stringify(view)); await flushPromises()
}
const reasons = () => wrapper.findAll('.attention-row .attention-reason').map(reason => reason.text())
// The unit's own definition of an error identifier (status.test.js:43).
const identifier = /\b[A-Z][a-z]+[A-Z]\w*\b|\b[A-Z]\w*\(/

// engine.rs:192-195 records `planner ended without a validated plan: {:?}` of the Loom run
// outcome, and supervisor.rs:160 stores it as the planning reason with phase Blocked. Loom ends a
// run whose model proposes nothing admissible twice with `NoAdmissibleAction(Unit(true))`
// (loom-commission runtime.rs:555; `Unit` is `struct Unit(pub bool)`).
test('attack_blocked_goal_reason_keeps_the_words_of_the_outcome', async () => {
  const view = frame()
  Object.assign(goalOf(view), { planning_phase: 'Blocked', planning_reason: 'planner ended without a validated plan: NoAdmissibleAction(Unit(true))' })
  view.assignments = []
  await show(view)
  expect(reasons()).toHaveLength(1)
  const [reason] = reasons()
  expect(reason).toMatch(/admissible/i)
  expect(reason).not.toMatch(/\btrue\b/)
  expect(reason).not.toMatch(identifier)
})

// loom_model.rs:144-148 fails a model call with `Loom stopped: {:?}` of Loom's `LoopStop`, whose
// budget variants are structs (`MaxTurns { limit }`, `Deadline { limit_ms }`); the call's budget
// is set at loom_model.rs:113 from the turn budget and the call's timeout. fleet.rs:498 blocks
// the assignment with `format!("{error:#}")` as its reason.
test('attack_blocked_assignment_reason_is_not_a_debug_struct', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Blocked', reason: 'Loom stopped: MaxTurns { limit: 1 }' })
  Object.assign(assignment(view, beta), { state: 'Blocked', reason: 'Loom stopped: Deadline { limit_ms: 600000 }' })
  await show(view)
  expect(reasons()).toHaveLength(2)
  for (const reason of reasons()) {
    expect.soft(reason).not.toMatch(identifier)
    expect.soft(reason).not.toMatch(/[{}]|\b[a-z]+_[a-z]+\b/)
  }
})

// Acceptance `idle_and_waiting_are_distinct`: "a projection with `waiting` set renders
// 'waiting for <model>'"; the brief: "an assignment waiting on a model makes the header say
// 'waiting for <model>'". Two workers at once (max_workers 2): `alpha` has an open implementor
// call, `beta` is Implementing between calls (running its tests).
test('attack_open_model_call_is_named_while_another_worker_runs', async () => {
  const view = frame()
  assignment(view, alpha).waiting = { role: 'implementor', model: 'fixture-implementor', since: '2026-10-06T09:59:20Z' }
  Object.assign(assignment(view, beta), { state: 'Implementing', attempt: 1, waiting: null })
  await show(view)
  expect(wrapper.get('.status-header').text()).toMatch(/waiting for fixture-implementor/i)
})
