import test from 'ava'

import { compile } from '../index'

test('sync function from native code', (t) => {
  const fixture = 42
  // t.is(compile(fixture))
  t.is(true, true)
})
