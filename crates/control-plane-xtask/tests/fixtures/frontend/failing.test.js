// Negative control for `frontend-check`, copied into a temporary frontend by
// `tests/frontend_check.rs`. It never sits in `frontend/src`, so the real suite never runs it.
import { mount } from '@vue/test-utils'
import { expect, test } from 'vitest'
import Activity from './Activity.vue'

test('fixture component assertion that does not hold', () => {
  const event = { at: '2026-10-06T10:00:00Z', action: 'tool.run', role: 'planner', status: 'running', detail: 'rendered detail' }
  const wrapper = mount(Activity, { props: { event, current: true } })
  expect(wrapper.text()).toContain('text the component never renders')
})
