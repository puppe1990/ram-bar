/** Gap in logical pixels between the extra and the hanging panel. */
export const PANEL_GAP = 4

/** Ignore a tray click that is the same gesture as the panel blur. */
export const TRAY_REOPEN_MS = 400

/**
 * Top-left origin of a panel hung under a menu-bar extra (y down).
 * Centers on the extra, then clamps into the work area.
 * Mirrored in `src-tauri/src/panel_anchor.rs`.
 *
 * @param {{ x: number, y: number, width: number, height: number }} tray
 * @param {{ width: number, height: number }} panel
 * @param {{ x: number, y: number, width: number, height: number }} work
 * @returns {{ x: number, y: number }}
 * @example
 * panelOrigin({ x: 1000, y: 0, width: 80, height: 24 }, { width: 360, height: 520 }, { x: 0, y: 0, width: 1440, height: 900 })
 * // { x: 860, y: 28 }
 */
export function panelOrigin(tray, panel, work) {
  const maxX = work.x + work.width - panel.width
  const maxY = work.y + work.height - panel.height
  const centered = tray.x + tray.width / 2 - panel.width / 2
  const x = clamp(centered, work.x, maxX)
  const below = tray.y + tray.height + PANEL_GAP
  const y = clamp(below, work.y, maxY)
  return { x, y }
}

/**
 * Left-click on the extra opens the panel unless it is already open
 * or this click is what just dismissed it (blur).
 * @param {boolean} panelVisible
 * @param {number | null} hiddenAgoMs
 * @returns {boolean}
 * @example
 * trayClickOpens(false, 50) // false
 */
export function trayClickOpens(panelVisible, hiddenAgoMs) {
  if (panelVisible) return false
  if (hiddenAgoMs != null && hiddenAgoMs < TRAY_REOPEN_MS) return false
  return true
}

/**
 * The mouseUp of the opening click lands on the extra, outside the panel,
 * and would otherwise blur-dismiss immediately.
 * @param {number | null} shownAgoMs
 * @returns {boolean}
 * @example
 * blurHides(50) // false
 */
export function blurHides(shownAgoMs) {
  if (shownAgoMs != null && shownAgoMs < TRAY_REOPEN_MS) return false
  return true
}

function clamp(value, min, max) {
  if (max < min) return min
  if (value < min) return min
  if (value > max) return max
  return value
}

/**
 * Scale of the monitor that contains the extra. The panel window may
 * sit on another display (1x Dell vs 2x laptop); using that window's
 * scale misplaces the hang-under origin.
 * Mirrored in `src-tauri/src/panel_anchor.rs`.
 *
 * @param {{ x: number, y: number, width: number, height: number }} tray
 * @param {{ x: number, y: number, width: number, height: number, scale: number }[]} monitors
 * @returns {number}
 * @example
 * scaleForTray({ x: 2238, y: 0, width: 222, height: 76 }, [
 *   { x: 0, y: 0, width: 3420, height: 2214, scale: 2 },
 * ])
 * // 2
 */
export function scaleForTray(tray, monitors) {
  return monitorForTray(tray, monitors)?.scale ?? 1
}

function monitorForTray(tray, monitors) {
  const cx = tray.x + tray.width / 2
  const cy = tray.y + tray.height / 2
  return monitors.find((m) => containsPoint(m, cx, cy)) ?? null
}

function containsPoint(rect, x, y) {
  return x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
