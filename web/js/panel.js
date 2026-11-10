import {
  diskCopy,
  formatDisk,
  formatGb,
  menuCopy,
  rankAppSpenders,
  rankDiskVolumes,
  usagePercent,
} from './format.mjs'

export function paintSnapshot(snap) {
  const used = snap.usedBytes
  const total = snap.totalBytes
  const copy = menuCopy({ used, total })
  document.getElementById('plan').textContent = formatGb(total)
  document.getElementById('used').textContent = formatGb(used)
  document.getElementById('free').textContent = formatGb(Math.max(0, total - used))
  document.getElementById('total').textContent = formatGb(total)
  document.getElementById('usage').textContent = copy.usage
  document.getElementById('meter')?.style.setProperty('--used', `${usagePercent(used, total)}%`)
  fillAppSpendList(document.getElementById('apps'), rankAppSpenders(snap.apps ?? []))
}

export function paintDisk(snap) {
  const used = snap.usedBytes
  const total = snap.totalBytes
  const available = snap.availableBytes
  const copy = diskCopy({ used, total, available })
  document.getElementById('disk-title').textContent = snap.name || 'Disco'
  document.getElementById('disk-plan').textContent = formatDisk(total)
  document.getElementById('disk-used').textContent = formatDisk(used)
  document.getElementById('disk-free').textContent = formatDisk(available)
  document.getElementById('disk-total').textContent = formatDisk(total)
  document.getElementById('disk-usage').textContent = copy.usage
  document
    .getElementById('disk-meter')
    ?.style.setProperty('--used', `${usagePercent(used, total)}%`)
  fillVolumeList(document.getElementById('volumes'), rankDiskVolumes(snap.volumes ?? []))
}

function fillAppSpendList(listEl, apps) {
  if (!listEl) return
  listEl.replaceChildren()
  if (apps.length === 0) {
    const empty = document.createElement('li')
    empty.className = 'empty'
    empty.textContent = 'Nenhum app medido'
    listEl.append(empty)
    return
  }
  const peak = apps[0].bytes
  for (const app of apps) {
    listEl.append(appSpendItem(app, peak))
  }
}

function appSpendItem(app, peakBytes) {
  const item = document.createElement('li')
  item.append(appSpendRow(app), appShareTrack(app.bytes, peakBytes))
  return item
}

function appSpendRow(app) {
  const row = document.createElement('div')
  row.className = 'app-row'
  const name = document.createElement('span')
  name.className = 'app-name'
  name.textContent = app.name
  const spend = document.createElement('span')
  spend.className = 'app-spend'
  spend.textContent = app.spend
  row.append(name, spend)
  return row
}

function appShareTrack(bytes, peakBytes) {
  const track = document.createElement('span')
  track.className = 'app-track'
  const fill = document.createElement('span')
  fill.className = 'app-fill'
  fill.style.width = `${usagePercent(bytes, peakBytes)}%`
  track.append(fill)
  return track
}

function fillVolumeList(listEl, volumes) {
  if (!listEl) return
  listEl.replaceChildren()
  if (volumes.length === 0) {
    const empty = document.createElement('li')
    empty.className = 'empty'
    empty.textContent = 'Nenhum volume medido'
    listEl.append(empty)
    return
  }
  for (const volume of volumes) {
    listEl.append(volumeItem(volume))
  }
}

function volumeItem(volume) {
  const item = document.createElement('li')
  item.append(
    appSpendRow({ name: volume.name, spend: volume.spend }),
    appShareTrack(volume.usedBytes, volume.totalBytes),
  )
  return item
}

function showTab(tab) {
  for (const button of document.querySelectorAll('[data-tab]')) {
    button.classList.toggle('is-on', button.dataset.tab === tab)
  }
  for (const pane of document.querySelectorAll('.pane')) {
    pane.classList.toggle('is-on', pane.id === `pane-${tab}`)
  }
}

async function bindTauri() {
  const tauri = window.__TAURI__
  if (!tauri) return
  const snap = await tauri.core.invoke('memory_snapshot')
  paintSnapshot(snap)
  const disk = await tauri.core.invoke('disk_snapshot')
  paintDisk(disk)
  await tauri.event.listen('memory', (event) => paintSnapshot(event.payload))
  await tauri.event.listen('disk', (event) => paintDisk(event.payload))
  bindPanelChrome(tauri)
}

function bindPanelChrome(tauri) {
  document.getElementById('quit')?.addEventListener('click', () => {
    tauri.core.invoke('quit_app')
  })
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') tauri.core.invoke('hide_panel')
  })
  document.querySelector('.tabs')?.addEventListener('click', (event) => {
    const tab = event.target.closest('[data-tab]')?.dataset.tab
    if (tab) showTab(tab)
  })
}

bindTauri()
