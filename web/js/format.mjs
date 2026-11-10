/** SI gigabyte (10^9). Activity Monitor uses this, not 1024^3. */
const SI_GB = 1_000_000_000

/**
 * Format bytes as Brazilian SI GB (one decimal, comma).
 * Mirrored in `src-tauri/src/memory.rs` — keep goldens in both suites.
 * @param {number} bytes
 * @returns {string}
 * @example
 * formatGb(12_400_000_000) // "12,4 GB"
 */
export function formatGb(bytes) {
  const gb = Number(bytes) / SI_GB
  return `${gb.toFixed(1).replace('.', ',')} GB`
}

/**
 * Used/total as a clamped integer percent.
 * @param {number} used
 * @param {number} total
 * @returns {number}
 * @example
 * usagePercent(8_000_000_000, 16_000_000_000) // 50
 */
export function usagePercent(used, total) {
  if (total <= 0) return 0
  const pct = Math.round((used / total) * 100)
  if (pct < 0) return 0
  if (pct > 100) return 100
  return pct
}

/**
 * Menu-bar title: used GB, two spaces, integer percent.
 * @param {number} used
 * @param {number} total
 * @returns {string}
 * @example
 * trayTitle(8_000_000_000, 16_000_000_000) // "8,0 GB  50%"
 */
export function trayTitle(used, total) {
  return `${formatGb(used)}  ${usagePercent(used, total)}%`
}

/**
 * Portuguese tray menu copy. Livre is total−used: sysinfo `available`
 * can be 0 on Apple when compressor pages exceed free+inactive+purgeable.
 * @param {{ used: number, total: number }} snapshot
 * @returns {{ used: string, free: string, total: string, usage: string, details: string, quit: string }}
 * @example
 * menuCopy({ used: 7_200_000_000, total: 8_600_000_000 }).free // "Livre 1,4 GB"
 */
export function menuCopy({ used, total }) {
  const freeBytes = Math.max(0, total - used)
  return {
    used: `Usado ${formatGb(used)}`,
    free: `Livre ${formatGb(freeBytes)}`,
    total: `Total ${formatGb(total)}`,
    usage: `${usagePercent(used, total)}% em uso`,
    details: 'Detalhes',
    quit: 'Sair',
  }
}

const SI_MB = 1_000_000
const APP_SPEND_LIMIT = 25

/**
 * Per-app RAM for the details list. GB from 1e9 up; MB below.
 * @param {number} bytes
 * @returns {string}
 * @example
 * formatSpend(320_000_000) // "320 MB"
 */
export function formatSpend(bytes) {
  if (Number(bytes) >= SI_GB) return formatGb(bytes)
  const mb = Number(bytes) / SI_MB
  if (mb >= 10) return `${Math.round(mb)} MB`
  return `${mb.toFixed(1).replace('.', ',')} MB`
}

/**
 * Group by name, drop empty/zero, sort biggest first, cap at 25.
 * @param {{ name: string, bytes: number }[]} rows
 * @returns {{ name: string, bytes: number, spend: string }[]}
 * @example
 * rankAppSpenders([{ name: 'Chrome', bytes: 800e6 }, { name: 'Chrome', bytes: 400e6 }])[0].bytes
 * // 1200000000
 */
export function rankAppSpenders(rows) {
  const totals = new Map()
  for (const row of rows) {
    const name = String(row.name ?? '').trim()
    const bytes = Number(row.bytes)
    if (!name || bytes <= 0) continue
    totals.set(name, (totals.get(name) || 0) + bytes)
  }
  return [...totals.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
    .slice(0, APP_SPEND_LIMIT)
    .map(([name, bytes]) => ({ name, bytes, spend: formatSpend(bytes) }))
}

const SI_TB = 1_000_000_000_000
const SKIP_MOUNTS = [
  '/dev',
  '/System/Volumes/Preboot',
  '/System/Volumes/VM',
  '/System/Volumes/Update',
  '/System/Volumes/iSCPreboot',
  '/System/Volumes/Hardware',
  '/System/Volumes/Data',
]

/**
 * Disk size: SI TB from 1e12, otherwise GB. Mirrored in `src-tauri/src/disk.rs`.
 * @param {number} bytes
 * @returns {string}
 * @example
 * formatDisk(1_000_000_000_000) // "1,0 TB"
 */
export function formatDisk(bytes) {
  if (Number(bytes) >= SI_TB) {
    return `${(Number(bytes) / SI_TB).toFixed(1).replace('.', ',')} TB`
  }
  return formatGb(bytes)
}

/**
 * Keep the boot volume and /Volumes. Drop macOS system slices (incl. Data).
 * @param {string} mount
 * @returns {boolean}
 * @example
 * keepDiskMount('/System/Volumes/Preboot') // false
 */
export function keepDiskMount(mount) {
  const path = String(mount ?? '').trim()
  if (!path) return false
  return !SKIP_MOUNTS.some((skip) => path === skip || path.startsWith(`${skip}/`))
}

/**
 * Drop system slices and empty disks, add usedBytes, sort fullest first.
 * @param {{ name: string, mount: string, totalBytes: number, availableBytes: number }[]} rows
 * @returns {{ name: string, mount: string, totalBytes: number, availableBytes: number, usedBytes: number, spend: string }[]}
 */
export function rankDiskVolumes(rows) {
  const ranked = []
  for (const row of rows) {
    const name = String(row.name ?? '').trim()
    const mount = String(row.mount ?? '').trim()
    const totalBytes = Number(row.totalBytes)
    const availableBytes = Math.max(0, Number(row.availableBytes))
    if (!name || !keepDiskMount(mount) || totalBytes <= 0) continue
    const usedBytes = Math.max(0, totalBytes - availableBytes)
    ranked.push({
      name,
      mount,
      totalBytes,
      availableBytes,
      usedBytes,
      spend: formatDisk(usedBytes),
    })
  }
  ranked.sort(
    (left, right) => right.usedBytes - left.usedBytes || left.name.localeCompare(right.name),
  )
  return collapseDiskTwins(ranked)
}

function collapseDiskTwins(volumes) {
  const kept = []
  for (const volume of volumes) {
    const twinAt = kept.findIndex((row) => diskTwin(row, volume))
    if (twinAt < 0) {
      kept.push(volume)
      continue
    }
    if (volume.mount === '/') kept[twinAt] = volume
  }
  return kept
}

function diskTwin(left, right) {
  return (
    left.name === right.name &&
    left.totalBytes === right.totalBytes &&
    left.usedBytes === right.usedBytes
  )
}

/**
 * Boot volume (`/`) when present, otherwise the largest disk.
 * @param {{ name: string, mount: string, totalBytes: number, availableBytes: number, usedBytes: number }[]} volumes
 */
export function pickPrimaryDisk(volumes) {
  if (!volumes.length) {
    return { name: '', mount: '', totalBytes: 0, availableBytes: 0, usedBytes: 0 }
  }
  return volumes.find((row) => row.mount === '/') ?? biggestDisk(volumes)
}

function biggestDisk(volumes) {
  return volumes.reduce((best, row) => (row.totalBytes > best.totalBytes ? row : best))
}

/**
 * Portuguese disk card copy. Livre is the volume's available bytes.
 * @param {{ used: number, total: number, available: number }} snapshot
 * @example
 * diskCopy({ used: 250e9, total: 1e12, available: 750e9 }).total // "Total 1,0 TB"
 */
export function diskCopy({ used, total, available }) {
  const freeBytes = Math.max(0, Number(available))
  return {
    used: `Usado ${formatDisk(used)}`,
    free: `Livre ${formatDisk(freeBytes)}`,
    total: `Total ${formatDisk(total)}`,
    usage: `${usagePercent(used, total)}% em uso`,
  }
}
