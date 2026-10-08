import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, expect, test, vi } from 'vitest'
import Evidence from './Evidence.vue'
import { assignments, evidenceGoal, latestCheck } from './evidence.js'

afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals() })

const event = (id, at, action, detail, extra = {}) => ({ id, at, action, role: 'host', status: 'running', worktree: 't', detail, ...extra })

// main.js runs evidenceGoal(location.pathname) before mounting anything; a throw there leaves a
// blank page. The server serves the console shell for `/goals/50%/evidence` (Rust attack case
// adversary_console_is_served_for_a_bare_percent_id).
test('adversary_bare_percent_in_the_console_path_does_not_throw', () => {
  expect(() => evidenceGoal('/goals/50%/evidence')).not.toThrow()
})

// dashboard.rs `evidence` answers 409 {"error":"goal not found"} through api_answer.
test('adversary_missing_goal_shows_the_server_error', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false, status: 409, headers: new Headers({ 'content-type': 'application/json' }), json: async () => ({ error: 'goal not found' }) })))
  const wrapper = mount(Evidence, { props: { goalId: 'nope' }, attachTo: document.body })
  await flushPromises()
  expect(wrapper.get('[role="alert"]').text()).toContain('goal not found')
})

// fleet.rs:2673 records `goal.checks` with observed_head after the merges; it is the newest check.
test('adversary_goal_checks_after_checks_run_is_the_latest', () => {
  const body = { history: { activity: [
    event('a', '2026-10-06T09:00:00Z', 'checks.run', { candidate: 'c1', command: 'task check' }),
    event('b', '2026-10-06T09:10:00Z', 'goal.checks', { observed_head: 'h9', worktree: 'w', command: 'task check' }),
  ] } }
  expect(latestCheck(body)).toEqual({ command: 'task check', candidate: 'h9', at: '2026-10-06T09:10:00Z' })
})

// now() is second-precision: two checks in one second keep recording order.
test('adversary_same_second_checks_keep_recording_order', () => {
  const body = { history: { activity: [
    event('a', '2026-10-06T09:00:00Z', 'checks.run', { candidate: 'c1', command: 'first' }),
    event('b', '2026-10-06T09:00:00Z', 'checks.run', { candidate: 'c2', command: 'second' }),
  ] } }
  expect(latestCheck(body).candidate).toBe('c2')
})

// QueueAssignment carries reviewer_run as input; only ReadyAssignment sets review_revision.
test('adversary_reviewer_without_review_revision_is_not_an_approval', () => {
  const [item] = assignments({ assignments: [{ assignment_id: 'x', reviewer_run: 'reviewer-run', review_revision: '', state: 'Reviewing', merge_receipt: '' }] })
  expect(item.approvedBy).toBe('')
  expect(item.merge).toBeNull()
})
