// Every way a Vitest test or suite can end without running as written, copied into a temporary
// frontend by `tests/frontend_check.rs`. Vitest exits 0 for all of them; `frontend-check` must not.
import { describe, test } from 'vitest'

describe.skip('fixture skipped suite', () => {
  test('fixture test in a skipped suite', () => {})
})
describe.skip('fixture empty skipped suite', () => {})
describe.todo('fixture todo suite')
test.skip('fixture skipped test', () => {})
test.skipIf(true)('fixture skipIf test', () => {})
test.todo('fixture todo test')
test('fixture test that skips itself', (context) => {
  context.skip()
})
test.fails('fixture expected-fail test', () => {
  throw new Error('inverted')
})
test('fixture expected-fail option', { fails: true }, () => {
  throw new Error('inverted')
})
const inverted = test.fails
inverted('fixture aliased expected-fail test', () => {
  throw new Error('inverted')
})
