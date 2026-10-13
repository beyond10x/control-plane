// @vitest-environment node
import { readdirSync, readFileSync } from 'node:fs'
import { expect, test } from 'vitest'
import { background, contrast, fontSizes, rgb, rules, value } from './stylesheet.js'

// The compiled stylesheet the server embeds (crates/control-plane-app/src/web.rs `stylesheet`).
// `xtask frontend-check` rebuilds it and refuses a difference, so this is what the browser gets.
const css = readFileSync(new URL('../dist/index.css', import.meta.url), 'utf8')
const all = rules(css)
const where = rule => `${rule.media ? `@media ${rule.media} ` : ''}${rule.selectors.join(',')}`

test('the stylesheet reader sees the rules inside media queries', () => {
  expect(all.length).toBeGreaterThan(100)
  expect(all.some(rule => rule.media.includes('600px'))).toBe(true)
  expect(rules('a{color:#000}@media(max-width:1px){b{font:9px x}}')).toEqual([
    { media: '', selectors: ['a'], declarations: [['color', '#000']] },
    { media: '(max-width:1px)', selectors: ['b'], declarations: [['font', '9px x']] },
  ])
})

test('text_meets_contrast', () => {
  const page = background(all.find(rule => rule.selectors.includes(':root')))
  expect(rgb(page)).toBeInstanceOf(Array)
  const failures = []
  for (const rule of all) {
    const colour = value(rule, 'color')
    if (colour === undefined) continue
    const text = rgb(colour)
    if (text === null) continue
    if (text instanceof Error) { failures.push(`${where(rule)}: ${text.message}`); continue }
    const own = background(rule)
    const against = own === undefined ? ['#fff', page] : [own]
    for (const surface of against) {
      const behind = rgb(surface)
      if (behind === null) continue
      if (behind instanceof Error) { failures.push(`${where(rule)}: ${behind.message}`); continue }
      const ratio = contrast(text, behind)
      if (ratio < 4.5) failures.push(`${where(rule)}: ${colour} on ${surface} is ${ratio.toFixed(2)}:1`)
    }
  }
  expect(failures).toEqual([])
})

test('text_meets_minimum_size', () => {
  const failures = []
  for (const rule of all) {
    for (const size of fontSizes(rule)) {
      if (size instanceof Error) failures.push(`${where(rule)}: ${size.message}`)
      else if (size < 12) failures.push(`${where(rule)}: ${size}px`)
    }
  }
  expect(failures).toEqual([])
})

test('each_derived_state_has_its_own_colour', async () => {
  const { stateBadge } = await import('./goalState.js')
  expect(typeof stateBadge).toBe('function')
  const plain = all.filter(rule => !rule.media)
  const colour = (selector, property) => {
    const set = plain.filter(rule => rule.selectors.includes(selector)).map(rule => property === 'color' ? value(rule, 'color') : background(rule)).filter(Boolean).pop()
    expect(set, `${selector} sets no ${property}`).toBeTruthy()
    return rgb(set).join(',')
  }
  // Each derived goal state has its own badge background, and merged assignments one of their own.
  const derived = ['starting', 'planning', 'executing', 'stalled', 'blocked', 'paused', 'cancelled', 'satisfied']
  const backgrounds = derived.map(state => colour(`.badge.${stateBadge(state)}`, 'background'))
  const merged = colour(`.badge.${stateBadge('Merged')}`, 'background')
  expect(new Set([...backgrounds, merged]).size).toBe(derived.length + 1)
  // The status header's six states have six colours.
  const states = ['idle', 'waiting', 'working', 'paused', 'blocked', 'disconnected']
  expect(new Set(states.map(state => colour(`.status-header.${state}`, 'color'))).size).toBe(states.length)
  // One badge mapper: every badge a component renders takes its class from `stateBadge`.
  const sources = readdirSync(new URL('.', import.meta.url)).filter(name => name.endsWith('.vue'))
  const offenders = []
  for (const name of sources) {
    const source = readFileSync(new URL(name, import.meta.url), 'utf8')
    if (/const badge\b|goalStateBadge/.test(source)) offenders.push(`${name}: defines its own badge mapper`)
    for (const tag of source.match(/<span[^>]*class="badge[^"]*"[^>]*>/g) || []) if (!tag.includes(':class="stateBadge(')) offenders.push(`${name}: ${tag}`)
  }
  expect(sources.length).toBeGreaterThan(5)
  expect(offenders).toEqual([])
})
