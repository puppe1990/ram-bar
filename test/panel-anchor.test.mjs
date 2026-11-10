import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  PANEL_GAP,
  TRAY_REOPEN_MS,
  panelOrigin,
  trayClickOpens,
  blurHides,
  scaleForTray,
} from '../web/js/panel-anchor.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

function fakeRect() {
  return {
    x: faker.number.int({ min: 0, max: 800 }),
    y: faker.number.int({ min: 0, max: 40 }),
    width: faker.number.int({ min: 40, max: 160 }),
    height: faker.number.int({ min: 22, max: 32 }),
  }
}

test('painel centra no extra e fica PANEL_GAP abaixo', () => {
  const tray = { x: 1000, y: 0, width: 80, height: 24 }
  const panel = { width: 360, height: 520 }
  const work = { x: 0, y: 0, width: 1440, height: 900 }
  const origin = panelOrigin(tray, panel, work)
  assert.equal(origin.x, 1000 + 80 / 2 - 360 / 2)
  assert.equal(origin.y, 24 + PANEL_GAP)
})

test('painel não vaza da direita da área útil', () => {
  const tray = { x: 1400, y: 0, width: 80, height: 24 }
  const panel = { width: 360, height: 520 }
  const work = { x: 0, y: 0, width: 1440, height: 900 }
  const origin = panelOrigin(tray, panel, work)
  assert.equal(origin.x, 1440 - 360)
  assert.ok(origin.x + panel.width <= work.width)
})

test('painel não vaza da esquerda da área útil', () => {
  const tray = { x: 0, y: 0, width: 40, height: 24 }
  const panel = { width: 360, height: 520 }
  const work = { x: 0, y: 0, width: 1440, height: 900 }
  const origin = panelOrigin(tray, panel, work)
  assert.equal(origin.x, 0)
})

test('origem aleatória cabe na área útil', () => {
  const tray = fakeRect()
  const panel = {
    width: faker.number.int({ min: 280, max: 400 }),
    height: faker.number.int({ min: 400, max: 600 }),
  }
  const work = { x: 0, y: 0, width: 1440, height: 900 }
  const origin = panelOrigin(tray, panel, work)
  assert.ok(origin.x >= work.x)
  assert.ok(origin.y >= work.y)
  assert.ok(origin.x + panel.width <= work.x + work.width + 1e-9)
  assert.ok(origin.y + panel.height <= work.y + work.height + 1e-9)
})

test('clique com o painel aberto fecha', () => {
  assert.equal(trayClickOpens(true, null), false)
})

test('clique logo após o blur do extra não reabre', () => {
  const ago = faker.number.int({ min: 0, max: TRAY_REOPEN_MS - 1 })
  assert.equal(trayClickOpens(false, ago), false)
})

test('clique depois do limiar abre de novo', () => {
  assert.equal(trayClickOpens(false, TRAY_REOPEN_MS), true)
  assert.equal(trayClickOpens(false, null), true)
})

test('blur do mouseUp que abre o painel não fecha', () => {
  const ago = faker.number.int({ min: 0, max: TRAY_REOPEN_MS - 1 })
  assert.equal(blurHides(ago), false)
})

test('blur depois do clique de abrir fecha', () => {
  assert.equal(blurHides(TRAY_REOPEN_MS), true)
  assert.equal(blurHides(null), true)
})

test('escala do extra segue o monitor 2x mesmo com o painel no 1x', () => {
  const laptop = { x: 0, y: 0, width: 3420, height: 2214, scale: 2 }
  const dell = { x: 3420, y: 0, width: 1920, height: 1080, scale: 1 }
  const tray = { x: 2238, y: 0, width: 222, height: 76 }
  assert.equal(scaleForTray(tray, [dell, laptop]), 2)
})

test('escala do extra no monitor 1x é 1', () => {
  const laptop = { x: 0, y: 0, width: 3420, height: 2214, scale: 2 }
  const dell = { x: 3420, y: 0, width: 1920, height: 1080, scale: 1 }
  const tray = { x: 4747, y: 0, width: 111, height: 24 }
  assert.equal(scaleForTray(tray, [laptop, dell]), 1)
})

test('sem monitores a escala cai em 1', () => {
  const tray = fakeRect()
  assert.equal(scaleForTray(tray, []), 1)
})
