// Every way a Vitest test can end without running, copied into a temporary frontend by
// `tests/frontend_check.rs`. Vitest exits 0 for all of them; `frontend-check` must not.
import { describe, test } from 'vitest'

describe.skip('fixture skipped suite', () => {
  test('fixture test in a skipped suite', () => {})
})
test.skip('fixture skipped test', () => {})
test.skipIf(true)('fixture skipIf test', () => {})
test.todo('fixture todo test')
test('fixture test that skips itself', (context) => {
  context.skip()
})
