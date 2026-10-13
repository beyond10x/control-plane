// @vitest-environment node
// Adversary cases against the visual-baseline tests (style.test.js): rules the contrast and size
// checks do not measure.
import { readFileSync } from 'node:fs'
import { expect, test } from 'vitest'
import { background, contrast, fontSizes, rgb, rules, value } from './stylesheet.js'

const css = readFileSync(new URL('../dist/index.css', import.meta.url), 'utf8')
const all = rules(css)

// A conditional group rule other than `@media` (`@supports`, `@container`, `@layer`) holds style
// rules the browser applies like any other. The reader drops them whole, so `text_meets_contrast`
// and `text_meets_minimum_size` pass a 9 px or 1.1:1 rule written inside one.
test('stylesheet_reader_sees_rules_inside_supports_container_and_layer', () => {
  const found = rules('@supports (display:grid){b{font-size:9px}}@container (min-width:1px){c{color:#eee}}@layer base{d{font:8px x}}')
  const selectors = found.flatMap(rule => rule.selectors)
  expect(selectors).toEqual(['b', 'c', 'd'])
  expect(found.flatMap(fontSizes)).toEqual([9, 8])
})

// `text_meets_contrast` measures only rules that set `color`. A rule that sets a background and
// leaves the text colour to another rule (`button.danger` over `button`'s white) is never measured:
// `button.danger{background:#e8806a}` (white text at 2.6:1) passes it. Here the text colour of such
// a rule is the one its element's base rule sets (`button` for `button.danger`), else `:root`'s.
test('text_meets_contrast_where_a_rule_sets_only_the_background', () => {
  const root = all.find(rule => rule.selectors.includes(':root'))
  const base = selector => {
    const tag = selector.trim().split(/[\s>+~]+/).pop().match(/^[a-z][a-z0-9-]*/)?.[0]
    const owner = tag && all.find(rule => !rule.media && rule.selectors.includes(tag) && value(rule, 'color') !== undefined)
    return value(owner || root, 'color')
  }
  const failures = []
  for (const rule of all) {
    if (value(rule, 'color') !== undefined) continue
    const surface = background(rule)
    if (surface === undefined || surface instanceof Error) continue
    const behind = rgb(surface)
    if (!(behind instanceof Array)) continue
    // A rule for an element that holds no text (a dot, a frame) has a size and no text.
    if (/dot|frame/.test(rule.selectors.join(','))) continue
    for (const selector of rule.selectors) {
      const text = rgb(base(selector))
      if (!(text instanceof Array)) continue
      const ratio = contrast(text, behind)
      if (ratio < 4.5) failures.push(`${rule.media ? `@media ${rule.media} ` : ''}${selector}: ${base(selector)} on ${surface} is ${ratio.toFixed(2)}:1`)
    }
  }
  expect(failures).toEqual([])
})
