import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import App from './App.vue'
// The first `operations` frame of `GET /events`, recorded byte for byte from `control-plane serve
// --state <copy of crates/control-plane-core/tests/fixtures/recorded-history.db> --workspace
// <empty directory>`. It holds only that scripted history and the empty startup workspace.
import recording from './fixtures/overview-operations.sse?raw'

/** Read one SSE frame the way the browser's EventSource does. */
function frame(text) {
  const fields = { data: [] }
  for (const line of text.split('\n')) {
    const separator = line.indexOf(':')
    if (separator <= 0) continue
    const name = line.slice(0, separator), value = line.slice(separator + 1).replace(/^ /, '')
    if (name === 'data') fields.data.push(value)
    else fields[name] = value
  }
  return { ...fields, data: fields.data.join('\n') }
}

class RecordedStream {
  static opened = []
  constructor(url) { this.url = url; this.listeners = {}; RecordedStream.opened.push(this) }
  addEventListener(type, listener) { (this.listeners[type] ||= []).push(listener) }
  close() { this.closed = true }
  deliver(type, data) { for (const listener of this.listeners[type] || []) listener({ type, data }) }
}

const answer = body => ({ ok: true, headers: new Headers({ 'content-type': 'application/json' }), json: async () => body })

beforeEach(() => {
  RecordedStream.opened = []
  history.replaceState({}, '', '/')
  vi.stubGlobal('EventSource', RecordedStream)
  // The catalog request answers empty, so everything rendered below comes from the recorded frame.
  vi.stubGlobal('fetch', vi.fn(async path => path === '/api/session' ? answer({ csrf_token: 'test-token' }) : answer({ workspaces: [] })))
})
afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals() })

test('app_renders_a_snapshot_payload', async () => {
  const wrapper = mount(App, { attachTo: document.body })
  await flushPromises()
  expect(RecordedStream.opened.map(stream => stream.url)).toEqual(['/events'])
  expect(wrapper.text()).not.toContain('Deliver the recorded change')

  const recorded = frame(recording)
  expect(recorded.event).toBe('operations')
  RecordedStream.opened[0].deliver(recorded.event, recorded.data)
  await flushPromises()

  expect(wrapper.get('#connection-status').text()).toBe('Live · committed observations')
  expect(wrapper.findAll('nav .workspace-link').map(link => link.text())).toContain('recorded')
  expect(wrapper.findAll('.operation h3').map(heading => heading.text())).toContain('Deliver the recorded change')
  const queue = wrapper.findAll('tbody tr').map(row => row.findAll('td').map(cell => cell.text()))
  expect(queue).toContainEqual([expect.stringContaining('story:first'), 'Merged', '2', 'review requested changes'])
  expect(queue.find(row => row[0].includes('story:first'))[0]).toContain('alpha')
})
