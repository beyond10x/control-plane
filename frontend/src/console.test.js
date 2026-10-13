import { readFileSync } from 'node:fs'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import GoalCard from './GoalCard.vue'
import { readSnapshot } from './snapshot.js'
import { rules, value } from './stylesheet.js'
// The projection frame of goalCard.test.js: workspace `demo` with its directory, repositories
// `alpha` and `beta`, the Running goal "Add a status endpoint", an Implementing and a Queued
// assignment, and `server_time` 10:00.
import recorded from './fixtures/status-frame.json'

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
  vi.stubGlobal('fetch', vi.fn(async path => path === '/api/session' ? answer({ csrf_token: 'test-token' }) : path === '/api/workspaces' ? answer({ workspaces: recorded.workspaces }) : answer({ published: [] })))
})
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; vi.unstubAllGlobals() })

const workspaceId = recorded.workspaces[0].workspace_id
const frame = () => structuredClone(recorded)
const goalOf = view => view.goals[0]
const button = (scope, label) => scope.findAll('button').find(candidate => candidate.text() === label)

test('app_mounts_from_the_embedded_snapshot_before_sse', async () => {
  const view = frame()
  goalOf(view).objective = 'Ship <b>bold</b> & </script> safely'
  // What web.rs `console_page` writes: the JSON with `<`, `>` and `&` as unicode escapes.
  const element = document.createElement('script')
  element.type = 'application/json'
  element.id = 'console-snapshot'
  element.textContent = JSON.stringify(view).replace(/</g, '\\u003c').replace(/>/g, '\\u003e').replace(/&/g, '\\u0026')
  document.body.append(element)
  expect(element.textContent).not.toContain('<')
  const snapshot = readSnapshot(document)
  expect(goalOf(snapshot).objective).toBe('Ship <b>bold</b> & </script> safely')
  expect(readSnapshot({ getElementById: () => null })).toBeNull()

  history.replaceState({}, '', `/workspaces/${workspaceId}`)
  wrapper = mount(App, { props: { snapshot }, attachTo: document.body })
  // Rendered from the snapshot before any stream frame, and before any request has answered.
  expect(wrapper.findAll('article.goal-card h3').map(heading => heading.text())).toEqual(['Ship <b>bold</b> & </script> safely'])
  expect(wrapper.find('article.goal-card b').exists()).toBe(false)
  expect(wrapper.text()).not.toContain('Waiting for current committed state')
  await flushPromises()
  expect(Stream.opened.map(stream => stream.url)).toEqual([`/workspaces/${encodeURIComponent(workspaceId)}/events`])
  // The settings of the workspace stay once the catalog answers, and the first frame replaces the snapshot.
  expect(wrapper.find('details.directories').exists()).toBe(true)
  const next = frame()
  goalOf(next).objective = 'Live objective'
  Stream.opened[0].deliver('operations', JSON.stringify(next)); await flushPromises()
  expect(wrapper.findAll('article.goal-card h3').map(heading => heading.text())).toEqual(['Live objective'])
})

test('destructive_buttons_are_styled_as_danger', async () => {
  const view = frame()
  const goal = goalOf(view)
  const props = { assignments: view.assignments, publications: [], repositories: view.repositories, workspace: 'demo', now: Date.parse(view.server_time), busy: false, refusals: {}, run: vi.fn() }
  const destructive = [], reversible = []
  const open = async (scope, label, confirm, list) => {
    const trigger = button(scope, label)
    expect(trigger, label).toBeTruthy()
    await trigger.trigger('click')
    const accept = scope.findAll('.confirmation button').find(candidate => candidate.text() === confirm)
    expect(accept, confirm).toBeTruthy()
    list.push([label, trigger], [confirm, accept])
  }

  // Cancel goal, and grant merge authority, on a Running goal with work in flight.
  const running = mount(GoalCard, { props: { ...props, goal: { ...goal, merge_authority: false } }, attachTo: document.body })
  await open(running, 'Cancel', 'Cancel goal', destructive)
  await open(running, 'Grant', 'Grant merge authority', reversible)
  const withheld = mount(GoalCard, { props: { ...props, goal: { ...goal, merge_authority: true } }, attachTo: document.body })
  await open(withheld, 'Withhold', 'Withhold merge authority', reversible)
  // Delete goal, on a Cancelled goal.
  const cancelled = mount(GoalCard, { props: { ...props, goal: { ...goal, state: 'Cancelled' } }, attachTo: document.body })
  await open(cancelled, 'Delete goal', 'Delete goal', destructive)
  // Remove directory and disable/enable repository, in the workspace settings.
  history.replaceState({}, '', `/workspaces/${workspaceId}`)
  const disabled = frame()
  disabled.repositories[1].state = 'Disabled'
  wrapper = mount(App, { props: { snapshot: disabled }, attachTo: document.body }); await flushPromises()
  await open(wrapper.get('details.directories'), 'Remove directory', 'Remove directory', destructive)
  const repositories = wrapper.findAll('section.repository')
  await open(repositories[0], 'Disable', 'Disable repository', reversible)
  reversible.push(['Enable', button(repositories[1], 'Enable')])

  expect(destructive.filter(([, b]) => !b.classes().includes('danger')).map(([label]) => label)).toEqual([])
  expect(reversible.filter(([, b]) => b.classes().includes('danger')).map(([label]) => label)).toEqual([])
  // Every button the console renders is danger exactly when it cancels, deletes or removes.
  const labels = new Set(destructive.map(([label]) => label))
  const everything = [running, withheld, cancelled, wrapper].flatMap(scope => scope.findAll('button'))
  expect(everything.filter(b => labels.has(b.text()) && !b.classes().includes('danger')).map(b => b.text())).toEqual([])
  const others = everything.filter(b => !labels.has(b.text()))
  expect(others.length).toBeGreaterThan(10)
  expect(others.filter(b => b.classes().includes('danger')).map(b => b.text())).toEqual([])
  for (const scope of [running, withheld, cancelled]) scope.unmount()
})

test('queue_items_keep_their_reason', async () => {
  const view = frame()
  view.assignments[0].state = 'Blocked'
  view.assignments[0].reason = 'Repository tests failed on the candidate'
  view.assignments[1].reason = 'Waiting for a worker slot'
  wrapper = mount(App, { props: { snapshot: view }, attachTo: document.body }); await flushPromises()
  expect(wrapper.findAll('table')).toHaveLength(0)
  const items = wrapper.findAll('.queue .queue-item')
  expect(items).toHaveLength(2)
  const reasons = ['Repository tests failed on the candidate', 'Waiting for a worker slot']
  items.forEach((item, index) => expect(item.get('.queue-reason').text()).toContain(reasons[index]))
  // No rule, at any width, hides a reason or anything around it.
  const css = readFileSync(`${process.cwd()}/dist/index.css`, 'utf8')
  const hiding = rules(css).filter(rule => value(rule, 'display') === 'none' || value(rule, 'visibility') === 'hidden')
  expect(hiding.length).toBeGreaterThan(0)
  const hidden = []
  for (const reason of wrapper.findAll('.queue-reason').map(found => found.element)) {
    for (let element = reason; element; element = element.parentElement) {
      for (const rule of hiding) for (const selector of rule.selectors) if (element.matches(selector)) hidden.push(`@media ${rule.media} ${selector}`)
    }
  }
  expect(hidden).toEqual([])
})
