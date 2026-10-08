import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import Activity from './Activity.vue'
import { withoutIds } from './identifiers.js'
import recorded from './fixtures/status-frame.json'

// Adversary pass 2 on story:console-goal-cards. Same harness shape as goalCard.test.js.
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
const [alpha] = recorded.assignments.map(a => a.assignment_id)
const workspaceId = recorded.workspaces[0].workspace_id
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
// A repository id as the store generates it (memory.rs `try_generate_uuid`).
const repositoryUuid = 'bdceb31a-273b-4248-8e69-f88446681e4e'

test('a runtime story id "<repository_id>::story:<slug>" (supervisor.rs QueueAssignment) shows the repository UUID on the card and in the queue', async () => {
  // supervisor.rs queues every story as `format!("{repository_id}::{story}")`
  // (tests/planner.rs:336 asserts that shape); the fixture's bare `story:status-endpoint` is not
  // what the runtime records.
  const view = frame()
  assignment(view, alpha).story_id = `${repositoryUuid}::story:status-endpoint`
  await show(view)
  for (const [where, text] of [['card', visible(card().element)], ['queue', visible(wrapper.get('table').element)]]) {
    expect.soft(text.toLowerCase(), where).toContain('status endpoint')
    expect.soft(text, where).not.toContain('bdceb31a')
    expect.soft(text, where).not.toContain('f88446681e4e')
    expect.soft(text, where).not.toContain('::')
  }
})

test('the waiting detail with a runtime story id (fleet.rs:810 reason) leaves "(::…)" debris in the open', () => {
  const holder = '6f1d2c3b-0000-4000-8000-0000000000d9'
  const detail = `Waiting for repository alpha: assignment ${holder} (${repositoryUuid}::story:status-docs) holds it until that assignment is merged or cancelled`
  const shown = mount(Activity, { props: { event: { action: 'waiting', at: '2026-10-06T09:59:00Z', detail, goal_revision: 1, role: 'host', status: 'running', worktree: '' }, current: false, now } })
  const open = visible(shown.element)
  expect.soft(open.toLowerCase()).toContain('status docs')
  expect.soft(open).not.toContain('::')
  expect.soft(open).not.toContain('()')
})

test('planner.intent "Reading workspace:<directory_id>/…" (engine.rs prompt, supervisor.rs intent_label) shows UUID segments as a quoted name', () => {
  // engine.rs tells the planner to read `workspace:<directory_id>/<relative-file>`; intent_label
  // records "Reading <paths>" and planner.intent is shown as prose.
  const detail = `Reading workspace:${repositoryUuid}/README.md, ess/system.yaml`
  const shown = mount(Activity, { props: { event: { action: 'planner.intent', at: '2026-10-06T09:59:00Z', detail, goal_revision: 1, role: 'planner', status: 'running', worktree: 'planning-tree' }, current: true, now } })
  const open = visible(shown.element)
  expect.soft(open).toContain('README.md')
  expect.soft(open).not.toContain('273b')
  expect.soft(open).not.toContain('8e69')
})

test('withoutIds turns a line number or a port into a quoted plan artifact name', () => {
  // A failing test command blocks with process.rs `{program} {args:?} exited …stderr…`, which
  // carries compiler locations; a provider error can carry a URL with a port.
  const location = withoutIds('error[E0425]: cannot find value `x` in this scope --> src/lib.rs:12:5')
  expect.soft(location).toContain('src/lib.rs:12:5')
  const url = withoutIds('planner: error sending request for url (http://localhost:11434/api/chat)')
  expect.soft(url).toContain('localhost:11434')
  for (const text of [location, url]) expect.soft(text).not.toMatch(/“\d+”/)
})

test('withoutIds glues a word to a following relative path ("run ./x" becomes "run./x")', () => {
  expect(withoutIds('run ./scripts/check failed in .git/worktrees')).toBe('run ./scripts/check failed in .git/worktrees')
})

test('repository settings save warns nothing for a ReadyToMerge assignment, which the store then refuses to merge (guards.rs same_configuration)', async () => {
  // fleet.rs schedules ReadyToMerge assignments for delivery; after a settings change the store
  // refuses MergeAssignment and PreparePublication ("repository configuration changed;
  // assignment evidence is stale") and deliver blocks it ("cancel or re-plan it").
  const view = frame()
  assignment(view, alpha).state = 'ReadyToMerge'
  await show(view, `/workspaces/${workspaceId}`)
  const section = wrapper.findAll('.repository').find(candidate => candidate.get('h3').text() === 'alpha')
  expect(section.find('.repository-warning').exists()).toBe(true)
})
