import { menuCopy, trayTitle } from './format.mjs'

function paint(snap) {
  const copy = menuCopy({
    used: snap.usedBytes,
    available: snap.availableBytes,
    total: snap.totalBytes,
  })
  document.getElementById('tray').textContent = trayTitle(snap.usedBytes, snap.totalBytes)
  document.getElementById('used').textContent = copy.used
  document.getElementById('free').textContent = copy.free
  document.getElementById('total').textContent = copy.total
  document.getElementById('usage').textContent = copy.usage
}

async function bindTauri() {
  const tauri = window.__TAURI__
  if (!tauri) return
  const snap = await tauri.core.invoke('memory_snapshot')
  paint(snap)
  await tauri.event.listen('memory', (event) => paint(event.payload))
}

bindTauri()
