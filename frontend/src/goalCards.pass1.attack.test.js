import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import Activity from './Activity.vue'
import { activitySentence } from './activity.js'
import recorded from './fixtures/status-frame.json'

// Adversary pass 1 on story:console-goal-cards. Same harness shape as goalCard.test.js.
class Stream {
  static opened = []
  constructor(url) { this.url = url; this.listeners = {}; Stream.opened.push(this) }
  addEventListener(type, listener) { (this.listeners[type] ||= []).push(listener) }
  close() { this.closed = true }
  deliver(type, data) { for (const listener of this.listeners[type] || []) listener({ type, data }) }
}
const answer = body => ({ ok: true, status: 200, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
let wrapper

beforeEach(() => {
  Stream.opened = []
  history.replaceState({}, '', '/')
  vi.stubGlobal('EventSource', Stream)
  vi.stubGlobal('fetch', vi.fn(async path => {
    if (path === '/api/session') return answer({ csrf_token: 'test-token' })
    if (path === '/api/workspaces') return answer({ workspaces: recorded.workspaces })
    return answer({ published: [] })
  }))
})
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; vi.unstubAllGlobals() })

const frame = () => structuredClone(recorded)
const goalOf = view => view.goals[0]
const [alpha] = recorded.assignments.map(a => a.assignment_id)
const assignment = (view, id) => view.assignments.find(a => a.assignment_id === id)
async function show(view, path = '/') {
  history.replaceState({}, '', path)
  wrapper = mount(App, { attachTo: document.body }); await flushPromises()
  Stream.opened.at(-1).deliver('operations', JSON.stringify(view)); await flushPromises()
}
const card = () => wrapper.findAll('article.goal-card')[0]
function visible(element) {
  const copy = element.cloneNode(true)
  for (const details of copy.querySelectorAll('details')) for (const child of [...details.childNodes]) if (child.nodeName !== 'SUMMARY') child.remove()
  return copy.textContent
}
const now = Date.parse('2026-10-06T10:00:00Z')

test('runtime action planner.prepare-branch (engine.rs run, supervisor.rs planner.<kind>) has no sentence and renders as its raw id', () => {
  // engine.rs `run` opens every planning run with progress("prepare-branch"), which supervisor.rs
  // records as action `planner.prepare-branch` with detail "prepare-branch".
  expect(activitySentence('planner.prepare-branch')).not.toBe('planner.prepare-branch')
  const shown = mount(Activity, { props: { event: { action: 'planner.prepare-branch', at: '2026-10-06T09:59:00Z', detail: 'prepare-branch', goal_revision: 1, role: 'planner', status: 'running' }, current: true, now } })
  expect(shown.get('.event-top strong').text()).not.toBe('planner.prepare-branch')
})

test('runtime action waiting (fleet.rs note "waiting") has no sentence and its detail leaks the holder assignment id and story id in the open', () => {
  // fleet.rs records `waiting` on an assignment held behind another one in the same repository;
  // live.rs `observation` turns its `reason` into the detail.
  const holder = '6f1d2c3b-0000-4000-8000-0000000000d9'
  const event = { action: 'waiting', at: '2026-10-06T09:59:00Z', detail: `Waiting for repository alpha: assignment ${holder} (story:status-docs) holds it until that assignment is merged or cancelled`, goal_revision: 1, role: 'host', status: 'running', worktree: '' }
  expect(activitySentence('waiting')).not.toBe('waiting')
  const shown = mount(Activity, { props: { event, current: false, now } })
  const open = visible(shown.element)
  expect.soft(open).not.toContain(holder)
  expect.soft(open).not.toContain('story:status-docs')
})

test('planner.accept-story detail "Accepting story:<id>" (engine.rs accept-story, supervisor.rs label) is shown as prose: the story id is visible on the card', async () => {
  const view = frame()
  goalOf(view).planning_phase = 'Validated'
  goalOf(view).planner_activity = { action: 'planner.accept-story', at: '2026-10-06T09:59:00Z', detail: 'Accepting story:status-endpoint', goal_revision: 1, id: 'entry-accept', role: 'planner', status: 'running', worktree: 'planning-tree' }
  await show(view)
  expect(visible(card().element)).not.toContain('story:status-endpoint')
})

test('a Blocked assignment whose reason names its candidate hash (fleet.rs publisher-exited reason) shows the hash on the card and in the queue', async () => {
  const hash = '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b'
  const view = frame()
  Object.assign(assignment(view, alpha), {
    state: 'Blocked', candidate: hash,
    reason: `The publisher exited and main at 1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e does not contain candidate ${hash}; closed as not published: 300 s passed since the outcome became uncertain`,
  })
  await show(view)
  expect.soft(visible(card().element), 'card').not.toContain(hash)
  expect.soft(visible(wrapper.get('table').element), 'queue').not.toContain(hash)
})
