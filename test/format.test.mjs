import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import { formatGb, menuCopy, trayTitle, usagePercent } from '../web/js/format.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

function fakeBytes() {
  return faker.number.int({ min: 100_000_000, max: 128_000_000_000 })
}

function fakeSnapshot() {
  const total = faker.number.int({ min: 4_000_000_000, max: 128_000_000_000 })
  const used = faker.number.int({ min: 0, max: total })
  return { total, used, available: total - used }
}

test('formata gigabytes SI com uma casa e vírgula', () => {
  const bytes = fakeBytes()
  const expected = `${(bytes / 1_000_000_000).toFixed(1).replace('.', ',')} GB`
  assert.equal(formatGb(bytes), expected)
})

test('12,4 GB é o ouro do Activity Monitor (SI)', () => {
  assert.equal(formatGb(12_400_000_000), '12,4 GB')
})

test('zero vira 0,0 GB', () => {
  assert.equal(formatGb(0), '0,0 GB')
})

test('percentual é inteiro 0–100 a partir do usado sobre o total', () => {
  const { total, used } = fakeSnapshot()
  const expected = Math.min(100, Math.round((used / total) * 100))
  assert.equal(usagePercent(used, total), expected)
})

test('metade da RAM é 50%', () => {
  const total = faker.number.int({ min: 8, max: 64 }) * 1_000_000_000
  assert.equal(usagePercent(total / 2, total), 50)
})

test('total zero não divide e devolve 0%', () => {
  const used = fakeBytes()
  assert.equal(usagePercent(used, 0), 0)
})

test('usado acima do total satura em 100%', () => {
  const total = faker.number.int({ min: 1_000_000_000, max: 8_000_000_000 })
  const extra = faker.number.int({ min: 1, max: 4_000_000_000 })
  assert.equal(usagePercent(total + extra, total), 100)
})

test('título do menu bar junta usado e percentual com dois espaços', () => {
  const { total, used } = fakeSnapshot()
  assert.equal(trayTitle(used, total), `${formatGb(used)}  ${usagePercent(used, total)}%`)
})

test('título ouro 8,0 GB  50%', () => {
  assert.equal(trayTitle(8_000_000_000, 16_000_000_000), '8,0 GB  50%')
})

test('menu traz usado, livre, total, uso e sair em português', () => {
  const { total, used, available } = fakeSnapshot()
  const copy = menuCopy({ used, available, total })
  assert.equal(copy.used, `Usado ${formatGb(used)}`)
  assert.equal(copy.free, `Livre ${formatGb(Math.max(0, total - used))}`)
  assert.equal(copy.total, `Total ${formatGb(total)}`)
  assert.equal(copy.usage, `${usagePercent(used, total)}% em uso`)
  assert.equal(copy.details, 'Detalhes')
  assert.equal(copy.quit, 'Sair')
})

test('livre usa total menos usado mesmo quando available vem zero', () => {
  const copy = menuCopy({
    used: 7_200_000_000,
    available: 0,
    total: 8_600_000_000,
  })
  assert.equal(copy.free, 'Livre 1,4 GB')
})
