import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
import Activity from './Activity.vue'
import { activitySentence } from './activity.js'
import { deriveGoalState, goalStateLabels } from './goalState.js'
// One projection frame in the shape `compact` sends (crates/control-plane-app/src/live.rs): the
// Running goal "Add a status endpoint" in workspace `demo`, planned in `alpha`, with an
// Implementing assignment in `alpha` (newest entry `tool.run` at 09:55), a Queued one in `beta`,
// and `server_time` 10:00.
import recorded from './fixtures/status-frame.json'

class Stream {
  static opened = []
  constructor(url) { this.url = url; this.listeners = {}; Stream.opened.push(this) }
  addEventListener(type, listener) { (this.listeners[type] ||= []).push(listener) }
  close() { this.closed = true }
  deliver(type, data) { for (const listener of this.listeners[type] || []) listener({ type, data }) }
}
const answer = body => ({ ok: true, status: 200, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
const refuse = (status, body) => ({ ok: false, status, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
let wrapper, refusals

beforeEach(() => {
  Stream.opened = []
  refusals = {}
  history.replaceState({}, '', '/')
  vi.stubGlobal('EventSource', Stream)
  vi.stubGlobal('fetch', vi.fn(async path => {
    if (path === '/api/session') return answer({ csrf_token: 'test-token' })
    if (path === '/api/workspaces') return answer({ workspaces: recorded.workspaces })
    return refusals[path] || answer({ published: [] })
  }))
})
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; vi.unstubAllGlobals() })

// Story ids in the runtime's shape: supervisor.rs queues each story as `<repository_id>::<story>`.
const frame = () => {
  const view = structuredClone(recorded)
  for (const row of view.assignments) row.story_id = `${row.repository_id}::${row.story_id}`
  return view
}
const goalOf = view => view.goals[0]
const [alpha, beta] = recorded.assignments.map(a => a.assignment_id)
const workspaceId = recorded.workspaces[0].workspace_id
const assignment = (view, id) => view.assignments.find(a => a.assignment_id === id)
const entry = (at, action, role, status, detail, worktree = 'tree-alpha') => ({ action, at, detail, goal_revision: 1, id: `entry-${action}-${at}`, role, status, worktree })

async function show(view, path = '/') {
  history.replaceState({}, '', path)
  wrapper = mount(App, { attachTo: document.body }); await flushPromises()
  Stream.opened.at(-1).deliver('operations', JSON.stringify(view)); await flushPromises()
}
const cards = () => wrapper.findAll('article.goal-card')
const card = () => cards()[0]
const row = id => card().find(`[data-assignment="${id}"]`)
const commands = () => fetch.mock.calls.map(([path]) => path).filter(path => path.startsWith('/api/commands/') || path.startsWith('/api/workspaces/') || path.startsWith('/workspaces/'))
const button = (scope, label) => scope.findAll('button').find(candidate => candidate.text() === label)
/** The text a reader sees without opening anything: each `<details>` contributes its summary only. */
function visible(element) {
  const copy = element.cloneNode(true)
  for (const details of copy.querySelectorAll('details')) for (const child of [...details.childNodes]) if (child.nodeName !== 'SUMMARY') child.remove()
  return copy.textContent
}
const disclosed = element => [...element.querySelectorAll('details')].map(details => details.textContent).join('\n')

test('goal_card_title_names_workspace_repository_and_goal', async () => {
  const view = frame()
  goalOf(view).objective = 'Add a status endpoint\nIt answers 200 with the build version and the uptime.'
  await show(view)
  expect(cards()).toHaveLength(1)
  expect(card().get('.card-title').text()).toBe('demo · alpha · Add a status endpoint')

  wrapper.unmount()
  const long = frame()
  goalOf(long).objective = `Rewrite the ${'very '.repeat(30)}long objective`
  await show(long)
  const title = card().get('.card-title').text()
  expect(title.startsWith('demo · alpha · Rewrite the very')).toBe(true)
  expect(title.endsWith('…')).toBe(true)
  expect(title).not.toContain('\n')
})

test('goal_card_shows_the_derived_state', async () => {
  const cases = [
    ['Running', 'Queued', 'Executing'],
    ['Running', 'Blocked', 'Blocked'],
    ['Paused', 'Queued', 'Paused'],
    ['Cancelled', 'Queued', 'Cancelled'],
  ]
  for (const [state, planning_phase, label] of cases) {
    const view = frame()
    Object.assign(goalOf(view), { state, planning_phase })
    await show(view)
    expect(goalStateLabels[deriveGoalState(goalOf(view), view.assignments)]).toBe(label)
    expect(card().get('.card-head .badge').text()).toBe(label)
    wrapper.unmount(); wrapper = undefined
  }
})

test('goal_card_shows_step_and_elapsed_time', async () => {
  // Between model calls the step is the assignment's newest entry: `tool.run` at 09:55.
  await show(frame())
  const step = row(alpha).get('.assignment-step').text()
  expect(step).toContain('Implementing')
  expect(step).toContain(activitySentence('tool.run'))
  expect(step).toContain('started 5 min ago')
  wrapper.unmount(); wrapper = undefined

  // While a model call is open the step started with the call's request, not with the newest
  // streamed entry (live.rs: a step is the lane's newest entry other than a `loom.event`).
  const streaming = frame()
  assignment(streaming, alpha).waiting = { role: 'implementor', model: 'fixture-implementor', since: '2026-10-06T09:59:20Z' }
  goalOf(streaming).fleet[alpha] = entry('2026-10-06T09:59:50Z', 'loom.event', 'runtime', 'running', 'Receiving model response (4 streamed events)')
  await show(streaming)
  const waiting = row(alpha).get('.assignment-step').text()
  expect(waiting).toContain('Implementing')
  expect(waiting).toMatch(/fixture-implementor/)
  expect(waiting).toContain('started 40 s ago')
  wrapper.unmount(); wrapper = undefined

  // A streamed entry without an open call does not say when its step started: no time is claimed.
  const stale = frame()
  goalOf(stale).fleet[alpha] = entry('2026-10-06T09:59:50Z', 'loom.event', 'runtime', 'running', 'Receiving model response (4 streamed events)')
  await show(stale)
  const unknown = row(alpha).get('.assignment-step').text()
  expect(unknown).toContain('Implementing')
  expect(unknown).not.toMatch(/started \d/)
})

test('goal_controls_are_on_the_card', async () => {
  for (const path of ['/', `/workspaces/${workspaceId}`]) {
    const view = frame()
    await show(view, path)
    const head = card().get('.card-head')
    for (const label of ['Pause', 'Cancel', 'Edit']) expect(button(head, label), `${path} ${label}`).toBeTruthy()
    expect(head.get('.merge-authority').text()).toMatch(/merge authority/i)
    wrapper.unmount(); wrapper = undefined

    const paused = frame()
    goalOf(paused).state = 'Paused'
    await show(paused, path)
    for (const label of ['Start', 'Cancel', 'Edit']) expect(button(card().get('.card-head'), label), `${path} ${label}`).toBeTruthy()
    expect(card().get('.card-head .merge-authority').exists()).toBe(true)
    wrapper.unmount(); wrapper = undefined
  }
})

test('destructive_actions_confirm_with_their_subject', async () => {
  await show(frame(), `/workspaces/${workspaceId}`)
  // Cancel goal.
  await button(card().get('.card-head'), 'Cancel').trigger('click')
  expect(commands()).toEqual([])
  let confirmation = card().get('[role="alertdialog"]').text()
  expect(confirmation).toContain('Add a status endpoint')
  expect(confirmation).toMatch(/no further work starts/i)
  expect(confirmation).toMatch(/in-flight assignments are cancelled/i)
  await button(card().get('[role="alertdialog"]'), 'Cancel goal').trigger('click'); await flushPromises()
  expect(commands()).toEqual(['/api/commands/CancelGoal'])
  expect(JSON.parse(fetch.mock.calls.find(([path]) => path === '/api/commands/CancelGoal')[1].body)).toEqual({ goal_id: goalOf(recorded).goal_id })

  // Remove directory.
  const directories = wrapper.get('.directories')
  await button(directories, 'Remove directory').trigger('click')
  expect(commands()).toHaveLength(1)
  confirmation = directories.get('[role="alertdialog"]').text()
  expect(confirmation).toContain('/srv/console-fixture/demo')
  expect(confirmation).toMatch(/its repositories leave the workspace unless another directory covers them/i)
  // Correction 2, F8: supervisor.rs stops a planner whose workspace directories changed.
  expect(confirmation).toMatch(/a planner running for this workspace is stopped/i)
  // fleet.rs `guard`: "workspace directory membership changed during execution".
  expect(confirmation).toMatch(/every in-flight assignment in this workspace is stopped and blocked/i)

  // Disable repository.
  const repository = wrapper.findAll('.repository').find(section => section.get('h3').text() === 'alpha')
  await button(repository, 'Disable').trigger('click')
  expect(commands()).toHaveLength(1)
  confirmation = repository.get('[role="alertdialog"]').text()
  expect(confirmation).toContain('alpha')
  // fleet.rs `guard`: "repository configuration changed or disabled".
  expect(confirmation).toMatch(/no new work starts in alpha, and its in-flight assignments are stopped and blocked/i)
  wrapper.unmount(); wrapper = undefined

  // Delete goal.
  const cancelled = frame()
  goalOf(cancelled).state = 'Cancelled'
  fetch.mockClear()
  await show(cancelled, `/workspaces/${workspaceId}`)
  await button(card().get('.card-head'), 'Delete goal').trigger('click')
  expect(commands()).toEqual([])
  confirmation = card().get('[role="alertdialog"]').text()
  expect(confirmation).toContain('Add a status endpoint')
  expect(confirmation).toMatch(/cannot be undone/i)
  await button(card().get('[role="alertdialog"]'), 'Delete goal').trigger('click'); await flushPromises()
  expect(commands()).toEqual(['/api/commands/DeleteGoal'])
})

test('merged_assignment_shows_merge_time', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Merged', merged_at: '2026-10-06T09:57:00Z' })
  await show(view)
  const merged = row(alpha)
  expect(merged.get('.assignment-step').text()).toContain('Merged 3 min ago')
  expect(merged.get('time').attributes('datetime')).toBe('2026-10-06T09:57:00Z')
  // An assignment that is not merged claims no merge time.
  expect(row(beta).text()).not.toMatch(/merged/i)
})

test('revision_bump_is_announced', async () => {
  // Two current assignments (Implementing, Queued) are in flight.
  await show(frame())
  await button(card().get('.card-head'), 'Edit').trigger('click')
  const form = card().get('form.goal-form')
  const warning = form.get('.revision-warning').text()
  expect(warning).toMatch(/2 in-flight assignments/)
  expect(warning).toMatch(/will be cancelled/i)
  expect(commands()).toEqual([])

  // Changing merge authority is a goal save too: it is announced before it is sent.
  await card().get('.card-head .merge-authority button').trigger('click')
  expect(commands()).toEqual([])
  expect(card().get('.card-head [role="alertdialog"]').text()).toMatch(/2 in-flight assignments will be cancelled/i)
  wrapper.unmount(); wrapper = undefined

  // Nothing in flight: nothing to announce.
  const settled = frame()
  Object.assign(assignment(settled, alpha), { state: 'Merged', merged_at: '2026-10-06T09:57:00Z' })
  Object.assign(assignment(settled, beta), { state: 'Cancelled' })
  await show(settled)
  await button(card().get('.card-head'), 'Edit').trigger('click')
  expect(card().get('form.goal-form').find('.revision-warning').exists()).toBe(false)
})

test('activity_reads_as_sentences', async () => {
  // Every action id the runtime records, read off its call sites (crates/control-plane-runtime/src).
  const known = [
    // engine.rs: actions passed through the `activity` hook kind.
    'model.requested', 'model.completed', 'model.failed', 'loom.event', 'aep.syntax_rejected', 'plan.review_rejected',
    // supervisor.rs `planner.{kind}` for each other hook kind: engine.rs `progress` (prepare-branch,
    // adopt, observation, intent, plan-approved, accept-story) and governance.rs `HostCases`
    // (governor-state, loom-observation) of the planning case.
    'planner.prepare-branch', 'planner.adopt', 'planner.observation', 'planner.intent', 'planner.plan-approved', 'planner.accept-story',
    'planner.governor-state', 'planner.loom-observation',
    // supervisor.rs `planning.{phase}` for each planning phase.
    'planning.Idle', 'planning.Provisioning', 'planning.Planning', 'planning.Validated', 'planning.Queued', 'planning.Blocked',
    // fleet.rs `progress`, `note` (`waiting`) and `block` (`blocked`), and governance.rs kinds of
    // the source-delivery cases (role `runtime`).
    'loom.event', 'blocked', 'waiting', 'worktree.prepare', 'attempt.repair', 'checks.run', 'review.request', 'model.request',
    'input.refused', 'file.write.completed', 'file.delete.completed', 'tool.run', 'tool.run.completed', 'publication.invoke',
    'merge.completed', 'goal.checks', 'goal.review', 'goal.acceptance.completed', 'governor-state', 'loom-observation']
  for (const action of known) {
    const sentence = activitySentence(action)
    expect(sentence, action).not.toContain(action)
    expect(sentence, action).toMatch(/^[A-Z][a-z]* [a-z]/)
  }
  const now = Date.parse('2026-10-06T10:00:00Z')
  const shown = mount(Activity, { props: { event: entry('2026-10-06T09:55:00Z', 'loom.event', 'runtime', 'running', 'Receiving model response'), current: true, now } })
  expect(shown.get('.event-top strong').text()).toBe(activitySentence('loom.event'))
  expect(shown.text()).toContain('5 min ago')
  const unknown = mount(Activity, { props: { event: entry('2026-10-06T09:58:00Z', 'mystery.action', 'host', 'running', ''), current: false, now } })
  expect(activitySentence('mystery.action')).toBe('mystery.action')
  expect(unknown.get('.event-top strong').text()).toBe('mystery.action')
  expect(unknown.text()).toContain('2 min ago')
})

test('refusal_appears_under_its_form', async () => {
  refusals['/api/commands/UpdateGoal'] = refuse(409, { error: 'Goal revision changed; reload the goal' })
  await show(frame())
  await button(card().get('.card-head'), 'Edit').trigger('click')
  await card().get('form.goal-form').trigger('submit'); await flushPromises()
  expect(commands()).toEqual(['/api/commands/UpdateGoal'])
  expect(card().get('form.goal-form .refusal').text()).toContain('Goal revision changed; reload the goal')
  expect(wrapper.find('main > section.error').exists()).toBe(false)
})

test('raw_ids_are_behind_details', async () => {
  const view = frame()
  Object.assign(assignment(view, alpha), { candidate: 'c0ffee1234abcd', reviewer_run: 'reviewer-alpha-1' })
  goalOf(view).fleet[alpha] = entry('2026-10-06T09:58:00Z', 'checks.run', 'host', 'running', 'cargo test --locked · c0ffee1234abcd')
  await show(view)
  const raw = ['story:status-endpoint', 'story:status-docs', 'c0ffee1234abcd', 'implementor-alpha-1', 'reviewer-alpha-1', 'tree-alpha', '/srv/console-fixture/planning', alpha,
    // Correction 2, F1: the repository UUID and `::` of the runtime story id.
    ...view.assignments.map(row => row.repository_id), '::']
  const shownCard = visible(card().element), hiddenCard = disclosed(card().element)
  const queue = wrapper.get('table').element, shownQueue = visible(queue), hiddenQueue = disclosed(queue)
  for (const id of raw) {
    expect.soft(shownCard, id).not.toContain(id)
    expect.soft(shownQueue, id).not.toContain(id)
  }
  for (const id of ['story:status-endpoint', 'c0ffee1234abcd', 'implementor-alpha-1', 'reviewer-alpha-1', 'tree-alpha', '/srv/console-fixture/planning']) expect.soft(hiddenCard, id).toContain(id)
  for (const id of ['story:status-endpoint', 'story:status-docs', 'c0ffee1234abcd', 'implementor-alpha-1', 'reviewer-alpha-1']) expect.soft(hiddenQueue, id).toContain(id)
})

// Correction 1, F5: fleet.rs `guard` stops an execution whose repository registration changed
// ("repository configuration changed or disabled"), so a settings save is announced first.
test('repository_settings_save_is_announced', async () => {
  // `alpha` has one Implementing assignment; `beta` only a Queued one, which no execution holds.
  await show(frame(), `/workspaces/${workspaceId}`)
  const section = name => wrapper.findAll('.repository').find(candidate => candidate.get('h3').text() === name)
  const warning = section('alpha').get('form.repository-form .repository-warning').text()
  expect(warning).toContain('alpha')
  expect(warning).toMatch(/its 1 in-flight assignment is stopped and blocked/i)
  expect(section('beta').find('.repository-warning').exists()).toBe(false)
  expect(commands()).toEqual([])
  // Correction 2, F7 (guards.rs `assignment_configuration_changed`): afterwards only cancelling or re-planning ends it.
  expect(warning).toMatch(/can then only be cancelled or re-planned/i)
  expect(warning).not.toMatch(/planner/i)
  wrapper.unmount(); wrapper = undefined

  // Correction 2, F6 and F7: ReadyToMerge and a claimed Blocked assignment are stopped too; an
  // unclaimed Blocked one holds no admitted configuration. F8: a planner running in the
  // repository is stopped (supervisor.rs "repository configuration changed during planning").
  const view = frame()
  assignment(view, alpha).state = 'ReadyToMerge'
  Object.assign(assignment(view, beta), { state: 'Blocked', repository_id: assignment(view, alpha).repository_id, worktree_id: 'tree-beta', implementor_run: 'implementor-beta-1' })
  view.assignments.push({ ...structuredClone(assignment(view, beta)), assignment_id: '6f1d2c3b-0000-4000-8000-0000000000d3', state: 'Blocked', worktree_id: '', implementor_run: '' })
  goalOf(view).planning_phase = 'Planning'
  await show(view, `/workspaces/${workspaceId}`)
  const both = section('alpha').get('.repository-warning').text()
  expect(both).toMatch(/its 2 in-flight assignments are stopped and blocked/i)
  expect(both).toMatch(/a running planner for this workspace in alpha is stopped/i)
})

// Correction 1, F3 and F4: a reason that names a hash, a revision, a UUID or a path reads without
// them on the card and in the queue; the reason as recorded is inside the details disclosure.
test('blocked_reason_reads_without_ids', async () => {
  const candidate = '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b', head = '1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e'
  const reason = `The publisher exited and main at ${head} does not contain candidate ${candidate}; closed as not published: 300 s passed since the outcome became uncertain`
  const view = frame()
  Object.assign(assignment(view, alpha), { state: 'Blocked', candidate, reason })
  await show(view)
  const expected = 'The publisher exited and main does not contain candidate; closed as not published: 300 s passed since the outcome became uncertain'
  expect(row(alpha).get('.assignment-step').text()).toBe(`Blocked · ${expected}`)
  expect(disclosed(row(alpha).element)).toContain(reason)
  const queued = wrapper.findAll('tbody tr').find(tr => tr.text().includes('Status endpoint')).findAll('td')[3]
  expect(visible(queued.element).trim()).toBe(`${expected}Full reason`)
  expect(disclosed(queued.element)).toContain(reason)
})
