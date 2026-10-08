import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, expect, test, vi } from 'vitest'
import Evidence from './Evidence.vue'
import { evidenceGoal, evidencePath } from './evidence.js'

afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals() })

const answer = body => ({ ok: true, status: 200, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
const event = (id, assignment, at, action, detail, extra = {}) => ({ id, assignment_id: assignment, goal_revision: 1, at, action, role: 'host', status: 'running', worktree: 't', detail, ...extra })
const row = (id, story, state, extra = {}) => ({ assignment_id: id, story_id: story, state, candidate: '', test_revision: '', review_revision: '', reviewer_run: '', implementor_run: 'impl', merge_receipt: '', ...extra })
const goal = (activity, extra = {}) => ({ goal_id: 'g1', objective: 'two stories', state: 'Running', satisfaction_receipt: '', activity_history: { activity, fleet: {} }, ...extra })

async function render(body) {
  vi.stubGlobal('fetch', vi.fn(async () => answer(body)))
  const wrapper = mount(Evidence, { props: { goalId: 'g1' }, attachTo: document.body })
  await flushPromises()
  return wrapper.get('[data-test="latest-check"]').text()
}

// Two stories, one after the other (fleet.rs runs checks.run before ReviewAssignment; a failed
// check never reaches ReviewAssignment, so story B's test_revision stays ''). Story A merged at
// a1; story B's checks then ran on b1 and failed. The newest check the JSON records is B's on
// b1, and the panel titled "Latest check" must show it. Evidence.vue suppresses latestCheck
// whenever any assignment has a test_revision, so it shows only A's older check.
test('adversary_latest_check_of_a_second_story_is_not_hidden_by_an_earlier_tested_story', async () => {
  const activity = [
    event('1', 'A', '2026-10-06T09:00:00Z', 'checks.run', { candidate: 'a1', command: 'task check' }),
    event('2', 'A', '2026-10-06T09:05:00Z', 'merge.completed', {}),
    event('3', 'B', '2026-10-06T09:30:00Z', 'checks.run', { candidate: 'b1', command: 'task check' }),
    event('4', 'B', '2026-10-06T09:31:00Z', 'blocked', { reason: 'checks failed' }, { status: 'failed' }),
  ]
  const text = await render({
    goal: goal(activity),
    history: { activity, fleet: {} },
    assignments: [
      row('A', 'story:a', 'Merged', { candidate: 'a1', test_revision: 'a1', review_revision: 'a1', reviewer_run: 'rev-a', merge_receipt: '{"observed_head":"m1"}' }),
      row('B', 'story:b', 'Blocked'),
    ],
    publications: [],
  })
  expect(text, 'the latest recorded check ran on b1').toContain('b1')
})

// A goal over two repositories records one goal.checks per repository in the same acceptance
// pass (fleet.rs: `for repo in repos` ... host.progress("goal.checks")). goalCheck keeps only
// the last, so the first repository's acceptance check and head disappear from the panel.
test('adversary_goal_acceptance_over_two_repositories_shows_both_heads', async () => {
  const activity = [
    event('1', 'A', '2026-10-06T09:00:00Z', 'checks.run', { candidate: 'a1', command: 'task check' }),
    event('2', 'B', '2026-10-06T09:01:00Z', 'checks.run', { candidate: 'b1', command: 'cargo test' }),
    event('3', 'A', '2026-10-06T10:00:00Z', 'goal.checks', { observed_head: 'head-repo-one', worktree: 'w1', command: 'task check' }),
    event('4', 'A', '2026-10-06T10:00:30Z', 'goal.checks', { observed_head: 'head-repo-two', worktree: 'w2', command: 'cargo test' }),
  ]
  const text = await render({
    goal: goal(activity),
    history: { activity, fleet: {} },
    assignments: [
      row('A', 'story:a', 'Merged', { candidate: 'a1', test_revision: 'a1', review_revision: 'a1', reviewer_run: 'r', merge_receipt: '{"observed_head":"head-repo-one"}' }),
      row('B', 'story:b', 'Merged', { candidate: 'b1', test_revision: 'b1', review_revision: 'b1', reviewer_run: 'r', merge_receipt: '{"observed_head":"head-repo-two"}' }),
    ],
    publications: [],
  })
  expect(text, 'first repository acceptance head').toContain('head-repo-one')
  expect(text, 'second repository acceptance head').toContain('head-repo-two')
})

// The decode fallback and the encoder: whatever the browser's location.pathname holds, the
// fetched URL stays one segment under /api/goals/ (dot segments are normalised by the URL parser
// before main.js sees them).
test('adversary_console_path_cannot_steer_the_fetch_outside_the_goal_segment', () => {
  for (const typed of ['/goals/a%2Fb%/evidence', '/goals/%2e%2e/evidence', '/goals/..%2F..%2Fapi%2Fstate/evidence', '/goals/x%3Fq%23h%/evidence', '/goals/%E0%A4%A/evidence']) {
    const pathname = new URL(typed, 'http://127.0.0.1:8787').pathname
    const id = evidenceGoal(pathname)
    if (!id) continue
    const fetched = new URL(evidencePath(id), 'http://127.0.0.1:8787')
    expect(fetched.pathname, typed).toMatch(/^\/api\/goals\/[^/]+\/evidence$/)
    expect(fetched.search + fetched.hash, typed).toBe('')
  }
})
