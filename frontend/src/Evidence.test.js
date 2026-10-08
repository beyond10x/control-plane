import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import Evidence from './Evidence.vue'
// The `GET /api/goals/{id}/evidence` shape (dashboard.rs `evidence`): the goal row with its
// attached history, that history, the goal's assignment rows and their publication rows.
import merged from './fixtures/evidence-merged.json'
import cancelled from './fixtures/evidence-cancelled.json'

const answer = body => ({ ok: true, status: 200, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })
let requested
function serve(body) { requested = []; vi.stubGlobal('fetch', vi.fn(async path => { requested.push(path); return answer(body) })) }

beforeEach(() => { requested = [] })
afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals() })

/** Position of `text` in the rendered page, failing when it is absent. */
function at(html, text) { const index = html.indexOf(text); expect(index, `${text} is rendered`).toBeGreaterThanOrEqual(0); return index }

test('evidence_view_summarises_before_raw', async () => {
  serve(merged)
  const wrapper = mount(Evidence, { props: { goalId: merged.goal.goal_id }, attachTo: document.body })
  await flushPromises()
  expect(requested).toEqual([`/api/goals/${merged.goal.goal_id}/evidence`])

  const download = wrapper.get('a.download')
  expect(download.attributes('href')).toBe(`/api/goals/${merged.goal.goal_id}/evidence`)
  expect(download.attributes('download')).toBe(`evidence-${merged.goal.goal_id}.json`)

  const summary = wrapper.get('[data-test="summary"]').text()
  expect(summary).toContain('Satisfied')
  expect(summary).toContain('Acceptance recorded')
  const check = wrapper.get('[data-test="latest-check"]').text()
  expect(check).toContain('task check')
  expect(check).toContain('2222222bbbbbbb')
  expect(check).not.toContain('cargo test --old')
  const assignment = wrapper.get('[data-test="assignment"]').text()
  expect(assignment).toContain('reviewer-run-alpha')
  expect(assignment).toContain('3333333ccccccc')
  expect(wrapper.text()).not.toContain('Nothing was merged')

  const html = wrapper.html(), raw = at(html, 'class="download"')
  for (const text of ['Satisfied', 'task check', '2222222bbbbbbb', 'reviewer-run-alpha', '3333333ccccccc']) expect(at(html, text)).toBeLessThan(raw)
  // The view summarises; it does not inline the raw JSON.
  expect(html).not.toContain('git_merge_observation')
})

test('evidence_view_handles_a_goal_without_merges', async () => {
  serve(cancelled)
  const wrapper = mount(Evidence, { props: { goalId: cancelled.goal.goal_id }, attachTo: document.body })
  await flushPromises()

  const summary = wrapper.get('[data-test="summary"]').text()
  expect(summary).toContain('Cancelled')
  expect(summary).toContain('No acceptance recorded')
  expect(wrapper.findAll('[data-test="assignment"]')).toHaveLength(0)
  expect(wrapper.get('[data-test="merges"]').text()).toContain('Nothing was merged')
  expect(wrapper.get('[data-test="latest-check"]').text()).toContain('No check recorded')
  const timeline = wrapper.findAll('[data-test="timeline"] li').map(item => item.text())
  expect(timeline).toHaveLength(2)
  expect(timeline[0]).toContain('Planning stopped')
  expect(timeline[0]).toContain('Cancelled by the operator')
  expect(timeline[1]).toContain('Planning started')
  expect(wrapper.get('a.download').attributes('href')).toBe(`/api/goals/${cancelled.goal.goal_id}/evidence`)
})
