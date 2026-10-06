/** Milliseconds since the epoch of an RFC 3339 time, or null. Digits past milliseconds are cut. */
export function parseTime(text) {
  if (typeof text !== 'string' || !text) return null
  const value = Date.parse(text.replace(/(\.\d{3})\d+/, '$1'))
  return Number.isFinite(value) ? value : null
}
