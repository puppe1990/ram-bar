const SI_GB = 1_000_000_000

export function formatGb(bytes) {
  const gb = Number(bytes) / SI_GB
  return `${gb.toFixed(1).replace('.', ',')} GB`
}

export function usagePercent(used, total) {
  if (total <= 0) return 0
  const pct = Math.round((used / total) * 100)
  if (pct < 0) return 0
  if (pct > 100) return 100
  return pct
}

export function trayTitle(used, total) {
  return `${formatGb(used)}  ${usagePercent(used, total)}%`
}

export function menuCopy({ used, available: _available, total }) {
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
