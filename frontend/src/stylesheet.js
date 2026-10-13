// Reading the compiled stylesheet (`frontend/dist/index.css`) for the visual-baseline tests:
// its rules with the at-rule conditions they sit in, colours as RGB and contrast ratios (WCAG 2.x).

// At-rules whose block holds no style rules: keyframe selectors, descriptors or page margins.
// Every other at-rule block (`@media`, `@supports`, `@container`, `@layer`, `@scope`,
// `@starting-style`, `@document` and any not yet known) is read for the style rules inside it.
const notStyle = /^@(-[a-z]+-)?(keyframes|font-face|font-feature-values|font-palette-values|counter-style|property|page|color-profile|position-try|view-transition)\b/i

/** Every style rule as `{ media, selectors, declarations }`. `media` is the conditions of the
 * at-rule blocks the rule sits in, outermost first, joined by ` and `: a `@media` query as
 * written after `@media`, any other at-rule with its name (`@supports (display:grid)`,
 * `@layer base`); empty for a rule outside every at-rule. A style rule nested inside another
 * style rule is not read, so it throws rather than going unmeasured. */
export function rules(css) {
  const out = []
  const text = css.replace(/\/\*[\s\S]*?\*\//g, '')
  let at = 0
  const walk = (conditions, end) => {
    while (at < end) {
      const open = text.indexOf('{', at)
      const semicolon = text.indexOf(';', at)
      // A statement at-rule (`@import`, `@charset`, `@layer a, b;`) ends at its semicolon.
      if (semicolon >= 0 && semicolon < end && (open < 0 || semicolon < open) && text.slice(at, semicolon).trim().startsWith('@')) {
        at = semicolon + 1
        continue
      }
      if (open < 0 || open >= end) { at = end; return }
      const prelude = text.slice(at, open).trim()
      const close = matching(text, open)
      if (prelude.startsWith('@')) {
        if (!notStyle.test(prelude)) {
          const condition = /^@media\b/i.test(prelude) ? prelude.slice(6).trim() : prelude
          at = open + 1
          walk([...conditions, condition], close)
        }
        at = close + 1
        continue
      }
      const body = text.slice(open + 1, close)
      if (body.includes('{')) throw Error(`nested style rule not read: ${prelude}{${body}}`)
      out.push({ media: conditions.join(' and '), selectors: split(prelude, ','), declarations: declarations(body) })
      at = close + 1
    }
  }
  walk([], text.length)
  return out
}

function matching(text, open) {
  let depth = 0
  for (let index = open; index < text.length; index++) {
    if (text[index] === '{') depth++
    else if (text[index] === '}' && --depth === 0) return index
  }
  throw Error('unbalanced stylesheet')
}

/** Split on `separator` outside parentheses, brackets and quotes. */
function split(text, separator) {
  const parts = []
  let depth = 0, quote = '', start = 0
  for (let index = 0; index < text.length; index++) {
    const c = text[index]
    if (quote) { if (c === quote) quote = ''; continue }
    if (c === '"' || c === "'") quote = c
    else if (c === '(' || c === '[') depth++
    else if (c === ')' || c === ']') depth--
    else if (c === separator && depth === 0) { parts.push(text.slice(start, index).trim()); start = index + 1 }
  }
  parts.push(text.slice(start).trim())
  return parts.filter(Boolean)
}

/** The declarations of a rule body as `[property, value]` pairs, in order. */
function declarations(body) {
  return split(body, ';').map(item => {
    const colon = item.indexOf(':')
    return [item.slice(0, colon).trim().toLowerCase(), item.slice(colon + 1).replace(/!important$/, '').trim()]
  }).filter(([property]) => property)
}

/** The last value a rule gives `property`, or undefined. */
export const value = (rule, property) => rule.declarations.filter(([name]) => name === property).map(([, v]) => v).pop()

const named = { white: [255, 255, 255], black: [0, 0, 0] }
/** `[r, g, b]` of a colour, null for a colour that names no fixed value (`inherit`,
 * `currentColor`, `transparent`), and an Error for a colour this reader does not know. */
export function rgb(colour) {
  const text = String(colour).trim().toLowerCase()
  if (['inherit', 'currentcolor', 'transparent', 'initial', 'unset'].includes(text)) return null
  if (Object.hasOwn(named, text)) return named[text]
  const hex = text.match(/^#([0-9a-f]{3,8})$/)
  if (hex) {
    let digits = hex[1]
    if (digits.length === 3 || digits.length === 4) digits = [...digits].map(d => d + d).join('')
    if (digits.length === 6 || digits.length === 8) return [0, 2, 4].map(i => parseInt(digits.slice(i, i + 2), 16))
  }
  return Error(`unknown colour ${colour}`)
}

/** The background colour a rule sets (`background-color`, or a `background` that is one colour). */
export function background(rule) {
  const set = value(rule, 'background-color') ?? value(rule, 'background')
  if (set === undefined) return undefined
  return set.split(/\s+/).length === 1 ? set : Error(`background is not one colour: ${set}`)
}

const channel = c => { const v = c / 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }
const luminance = ([r, g, b]) => 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
/** The WCAG contrast ratio of two RGB colours. */
export function contrast(first, second) {
  const [light, dark] = [luminance(first), luminance(second)].sort((a, b) => b - a)
  return (light + 0.05) / (dark + 0.05)
}

/** The font sizes in px a rule sets through `font-size` or the `font` shorthand, with an Error
 * for a size this reader cannot turn into px. */
export function fontSizes(rule) {
  const sizes = []
  for (const [property, v] of rule.declarations) {
    if (property === 'font-size' && !['inherit', 'initial', 'unset'].includes(v)) sizes.push(px(v))
    if (property === 'font') {
      const size = v.split(/\s+/).find(token => /^[\d.]+(px|em|rem|%|pt)(\/.*)?$/.test(token))
      if (size) sizes.push(px(size.split('/')[0]))
      else if (!['inherit', 'initial', 'unset'].includes(v)) sizes.push(Error(`font shorthand without a size: ${v}`))
    }
  }
  return sizes
}
function px(size) {
  const match = String(size).match(/^([\d.]+)px$/)
  return match ? Number(match[1]) : Error(`font size not in px: ${size}`)
}
