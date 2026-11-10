import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  LOGIN_AGENT_LABEL,
  bundleMacosExecutable,
  loginAgentPlist,
} from '../scripts/login-agent.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

function fakeBundleProgram() {
  const stem = faker.word.noun()
  return `/Applications/${stem}.app/Contents/MacOS/ram-bar`
}

test('plist usa o identifier, RunAtLoad e sessão Aqua', () => {
  const program = fakeBundleProgram()
  const plist = loginAgentPlist(program)
  assert.match(plist, new RegExp(`<string>${LOGIN_AGENT_LABEL}</string>`))
  assert.match(plist, /<key>RunAtLoad<\/key>\s*<true\/>/)
  assert.match(plist, /<key>KeepAlive<\/key>\s*<false\/>/)
  assert.match(plist, /<key>LimitLoadToSessionType<\/key>\s*<string>Aqua<\/string>/)
  assert.match(plist, new RegExp(`<string>${program}</string>`))
})

test('plist escapa &, < e > no caminho do binário', () => {
  const program = `/tmp/${faker.word.noun()}&<>.app/Contents/MacOS/ram-bar`
  const plist = loginAgentPlist(program)
  assert.equal(plist.includes(program), false)
  assert.match(plist, /&amp;&lt;&gt;\.app/)
})

test('só registra executável dentro de um .app/Contents/MacOS', () => {
  const program = fakeBundleProgram()
  assert.equal(bundleMacosExecutable(program), program)
  assert.equal(
    bundleMacosExecutable(`/Users/${faker.person.firstName()}/target/debug/ram-bar`),
    null,
  )
})
