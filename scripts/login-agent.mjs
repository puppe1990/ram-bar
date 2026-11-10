export const LOGIN_AGENT_LABEL = 'com.gestaobem.ram-bar'
export const BUNDLED_APP_PROGRAM = '/Applications/RAM Bar.app/Contents/MacOS/ram-bar'

/**
 * LaunchAgent plist so RAM Bar starts in the Aqua session at login.
 * KeepAlive stays false: Sair must stick until the next login.
 * @param {string} program absolute path of the MacOS executable
 * @returns {string}
 * @example
 * loginAgentPlist('/Applications/RAM Bar.app/Contents/MacOS/ram-bar')
 */
export function loginAgentPlist(program) {
  const exe = xmlEscape(String(program ?? ''))
  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${LOGIN_AGENT_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>${exe}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
  <key>LimitLoadToSessionType</key>
  <string>Aqua</string>
</dict>
</plist>
`
}

/**
 * Only the binary inside a .app bundle is a stable login target.
 * `tauri dev` debug bins stay out so they do not overwrite the agent.
 * @param {string} exePath
 * @returns {string | null}
 */
export function bundleMacosExecutable(exePath) {
  const path = String(exePath ?? '').replace(/\/+$/, '')
  const marker = '.app/Contents/MacOS/'
  const idx = path.lastIndexOf(marker)
  if (idx <= 0) return null
  const after = path.slice(idx + marker.length)
  if (!after || after.includes('/')) return null
  return path
}

function xmlEscape(value) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
}
