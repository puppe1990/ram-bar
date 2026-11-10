import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'
import { BUNDLED_APP_PROGRAM, LOGIN_AGENT_LABEL, loginAgentPlist } from './login-agent.mjs'

const SRC = join(
  import.meta.dirname,
  '..',
  'src-tauri',
  'target',
  'release',
  'bundle',
  'macos',
  'RAM Bar.app',
)

function destApp() {
  const system = '/Applications/RAM Bar.app'
  const user = join(homedir(), 'Applications', 'RAM Bar.app')
  return { system, user }
}

function programInside(appDir) {
  return join(appDir, 'Contents', 'MacOS', 'ram-bar')
}

function writeLaunchAgent(program) {
  const agentsDir = join(homedir(), 'Library', 'LaunchAgents')
  mkdirSync(agentsDir, { recursive: true })
  const plistPath = join(agentsDir, `${LOGIN_AGENT_LABEL}.plist`)
  writeFileSync(plistPath, loginAgentPlist(program))
  return plistPath
}

function copyApp(dst) {
  rmSync(dst, { recursive: true, force: true })
  mkdirSync(join(dst, '..'), { recursive: true })
  cpSync(SRC, dst, { recursive: true })
}

if (!existsSync(SRC)) {
  console.error(`missing bundle: ${SRC} (run npm run bundle)`)
  process.exit(1)
}

const { system, user } = destApp()
let dst = system
try {
  copyApp(system)
} catch (err) {
  console.error(`copy to ${system} failed (${err.message}); trying ${user}`)
  copyApp(user)
  dst = user
}

const program = dst === system ? BUNDLED_APP_PROGRAM : programInside(dst)
const plistPath = writeLaunchAgent(program)
console.log(`installed ${dst}`)
console.log(`login agent ${plistPath}`)
console.log('abre no próximo login do Mac')
