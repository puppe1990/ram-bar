import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import { formatSpend, rankAppSpenders } from '../web/js/format.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

function fakeApp(overrides = {}) {
  return {
    name: faker.commerce.productName(),
    bytes: faker.number.int({ min: 5_000_000, max: 4_000_000_000 }),
    ...overrides,
  }
}

test('gasto em GB SI com uma casa quando passa de 1 GB', () => {
  assert.equal(formatSpend(2_400_000_000), '2,4 GB')
})

test('gasto em MB inteiro entre 10 MB e 1 GB', () => {
  assert.equal(formatSpend(320_000_000), '320 MB')
})

test('gasto em MB com vírgula abaixo de 10 MB', () => {
  assert.equal(formatSpend(5_400_000), '5,4 MB')
})

test('lista junta processos do mesmo app e ordena pelo mais gastão', () => {
  const hog = faker.commerce.productName()
  const mid = faker.commerce.productName()
  const low = faker.commerce.productName()
  const ranked = rankAppSpenders([
    { name: mid, bytes: 200_000_000 },
    { name: hog, bytes: 800_000_000 },
    { name: hog, bytes: 400_000_000 },
    { name: low, bytes: 50_000_000 },
  ])
  assert.deepEqual(
    ranked.map((row) => row.name),
    [hog, mid, low],
  )
  assert.equal(ranked[0].bytes, 1_200_000_000)
  assert.equal(ranked[0].spend, '1,2 GB')
})

test('lista ignora nome vazio e gasto zero', () => {
  const keep = fakeApp({ bytes: 80_000_000 })
  const ranked = rankAppSpenders([
    { name: '  ', bytes: 500_000_000 },
    { name: keep.name, bytes: 0 },
    keep,
  ])
  assert.deepEqual(ranked, [{ name: keep.name, bytes: 80_000_000, spend: '80 MB' }])
})

test('lista corta nos 25 mais gastões', () => {
  const rows = Array.from({ length: 30 }, (_, i) => ({
    name: `${faker.word.noun()}-${i}`,
    bytes: (30 - i) * 10_000_000,
  }))
  const ranked = rankAppSpenders(rows)
  assert.equal(ranked.length, 25)
  assert.ok(ranked[0].bytes > ranked[24].bytes)
})

test('empate de bytes ordena pelo nome', () => {
  const ranked = rankAppSpenders([
    { name: 'Safari', bytes: 100_000_000 },
    { name: 'Chrome', bytes: 100_000_000 },
  ])
  assert.deepEqual(
    ranked.map((row) => row.name),
    ['Chrome', 'Safari'],
  )
})
