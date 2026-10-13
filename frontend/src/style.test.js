// @vitest-environment node
import { readdirSync, readFileSync } from 'node:fs'
import { expect, test } from 'vitest'
import { background, contrast, fontSizes, rgb, rules, value } from './stylesheet.js'

// The compiled stylesheet the server embeds (crates/control-plane-app/src/web.rs `stylesheet`).
// `xtask frontend-check` rebuilds it and refuses a difference, so this is what the browser gets.
const css = readFileSync(new URL('../dist/index.css', import.meta.url), 'utf8')
const all = rules(css)
const where = rule => `${rule.media ? `[${rule.media}] ` : ''}${rule.selectors.join(',')}`

test('the stylesheet reader sees the rules inside media queries', () => {
  expect(all.length).toBeGreaterThan(100)
  expect(all.some(rule => rule.media.includes('600px'))).toBe(true)
  expect(rules('a{color:#000}@media(max-width:1px){b{font:9px x}}')).toEqual([
    { media: '', selectors: ['a'], declarations: [['color', '#000']] },
    { media: '(max-width:1px)', selectors: ['b'], declarations: [['font', '9px x']] },
  ])
})

test('the stylesheet reader reads every at-rule block that holds style rules', () => {
  expect(rules('@supports (display:grid){@media (max-width:1px){b{color:#111}}}@layer x{c{color:#222}}')).toEqual([
    { media: '@supports (display:grid) and (max-width:1px)', selectors: ['b'], declarations: [['color', '#111']] },
    { media: '@layer x', selectors: ['c'], declarations: [['color', '#222']] },
  ])
  // Keyframe selectors and descriptors are not style rules; statement at-rules end at `;`.
  expect(rules('@charset "utf-8";@layer a, b;@keyframes k{from{color:#eee}to{color:#fff}}@font-face{font-family:x}a{color:#000}')).toEqual([
    { media: '', selectors: ['a'], declarations: [['color', '#000']] },
  ])
  // A style rule nested inside another is refused, never silently unmeasured.
  expect(() => rules('a{color:#000;&.b{color:#eee}}')).toThrow(/nested style rule/)
})

// The colour a rule's text inherits when the rule sets none: the newest unconditional rule that
// sets `color` for the element the selector's last compound names (its tag or one of its
// classes, as `button` for `button.danger`), else for an ancestor compound, nearest first, else
// `body`'s, else `:root`'s.
function inheritedColour(selector) {
  const plain = all.filter(rule => !rule.media && value(rule, 'color') !== undefined)
  const compounds = selector.trim().split(/\s*[\s>+~]\s*/).filter(Boolean).reverse()
  for (const compound of [...compounds, 'body', ':root']) {
    const simple = [compound, ...(compound.match(/^[a-z][a-z0-9-]*|\.[\w-]+/gi) || [])]
    const owner = plain.filter(rule => rule.selectors.some(own => own !== selector && simple.includes(own))).pop()
    if (owner) return value(owner, 'color')
  }
}

// An element whose box is shorter or narrower than the minimum text size (a status dot) holds
// no text: some rule for its last compound's class sets `width` and `height` both under 12px.
function holdsNoText(selector) {
  const classes = selector.trim().split(/\s*[\s>+~]\s*/).pop().match(/\.[\w-]+/g) || []
  return all.some(rule => rule.selectors.some(own => classes.includes(own)) &&
    ['width', 'height'].every(side => { const size = value(rule, side)?.match(/^([\d.]+)px$/); return size && Number(size[1]) < 12 }))
}

test('text_meets_contrast', () => {
  const page = background(all.find(rule => rule.selectors.includes(':root')))
  expect(rgb(page)).toBeInstanceOf(Array)
  const failures = []
  const measured = { colour: 0, inherited: 0 }
  for (const rule of all) {
    const colour = value(rule, 'color')
    const own = background(rule)
    if (colour === undefined) {
      // A rule that sets only a background: measure it against the text colour it inherits.
      if (own === undefined) continue
      const behind = rgb(own)
      if (behind === null) continue
      if (behind instanceof Error) { failures.push(`${where(rule)}: ${behind.message}`); continue }
      for (const selector of rule.selectors) {
        if (holdsNoText(selector)) continue
        const inherited = inheritedColour(selector)
        const text = rgb(inherited)
        if (text === null) continue
        if (text instanceof Error) { failures.push(`${where(rule)}: ${text.message}`); continue }
        measured.inherited++
        const ratio = contrast(text, behind)
        if (ratio < 4.5) failures.push(`${where(rule)}: ${selector} inherits ${inherited} on ${own} at ${ratio.toFixed(2)}:1`)
      }
      continue
    }
    const text = rgb(colour)
    if (text === null) continue
    if (text instanceof Error) { failures.push(`${where(rule)}: ${text.message}`); continue }
    measured.colour++
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
  // Both kinds of rule are measured: a reader that stopped seeing either fails here.
  expect(measured.colour).toBeGreaterThan(20)
  expect(measured.inherited).toBeGreaterThan(5)
  // The control: the destructive button inherits the base button's white text.
  expect(inheritedColour('button.danger')).toBe('#fff')
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
