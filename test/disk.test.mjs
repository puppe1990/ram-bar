import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  diskCopy,
  formatDisk,
  keepDiskMount,
  pickPrimaryDisk,
  rankDiskVolumes,
} from '../web/js/format.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

function fakeBytes() {
  return faker.number.int({ min: 100_000_000, max: 2_000_000_000_000 })
}

test('1 TB SI vira 1,0 TB', () => {
  assert.equal(formatDisk(1_000_000_000_000), '1,0 TB')
})

test('abaixo de 1 TB continua em GB SI', () => {
  const bytes = faker.number.int({ min: 1_000_000_000, max: 999_000_000_000 })
  const expected = `${(bytes / 1_000_000_000).toFixed(1).replace('.', ',')} GB`
  assert.equal(formatDisk(bytes), expected)
})

test('meio terabyte fica em GB', () => {
  assert.equal(formatDisk(500_000_000_000), '500,0 GB')
})

test('raiz e Volumes passam no filtro do HD', () => {
  assert.equal(keepDiskMount('/'), true)
  const volume = `/Volumes/${faker.word.noun()}`
  assert.equal(keepDiskMount(volume), true)
})

test('fatias de sistema do macOS ficam de fora', () => {
  assert.equal(keepDiskMount('/System/Volumes/Preboot'), false)
  assert.equal(keepDiskMount('/System/Volumes/VM'), false)
  assert.equal(keepDiskMount('/System/Volumes/Update'), false)
  assert.equal(keepDiskMount('/System/Volumes/Data'), false)
  assert.equal(keepDiskMount('/dev'), false)
  assert.equal(keepDiskMount(''), false)
})

test('lista junta volumes, ignora fatia de sistema e ordena pelo mais cheio', () => {
  const hog = faker.commerce.productName()
  const mid = faker.commerce.productName()
  const ranked = rankDiskVolumes([
    {
      name: mid,
      mount: '/Volumes/Mid',
      totalBytes: 400_000_000_000,
      availableBytes: 200_000_000_000,
    },
    {
      name: hog,
      mount: '/',
      totalBytes: 1_000_000_000_000,
      availableBytes: 200_000_000_000,
    },
    {
      name: 'Preboot',
      mount: '/System/Volumes/Preboot',
      totalBytes: 1_000_000_000,
      availableBytes: 100_000_000,
    },
  ])
  assert.equal(ranked[0].name, hog)
  assert.equal(ranked[0].usedBytes, 800_000_000_000)
  assert.equal(ranked[1].name, mid)
  assert.equal(
    ranked.some((row) => row.mount === '/System/Volumes/Preboot'),
    false,
  )
})

test('primário é a raiz quando ela existe', () => {
  const rootName = faker.commerce.productName()
  const other = faker.commerce.productName()
  const primary = pickPrimaryDisk([
    {
      name: other,
      mount: '/Volumes/Extra',
      totalBytes: 4_000_000_000_000,
      availableBytes: 1,
      usedBytes: 4_000_000_000_000 - 1,
    },
    {
      name: rootName,
      mount: '/',
      totalBytes: 500_000_000_000,
      availableBytes: 100_000_000_000,
      usedBytes: 400_000_000_000,
    },
  ])
  assert.equal(primary.mount, '/')
  assert.equal(primary.name, rootName)
})

test('sem raiz o primário é o maior total', () => {
  const big = faker.commerce.productName()
  const primary = pickPrimaryDisk([
    {
      name: faker.commerce.productName(),
      mount: '/Volumes/A',
      totalBytes: 200_000_000_000,
      availableBytes: 10,
      usedBytes: 200_000_000_000 - 10,
    },
    {
      name: big,
      mount: '/Volumes/B',
      totalBytes: 800_000_000_000,
      availableBytes: 10,
      usedBytes: 800_000_000_000 - 10,
    },
  ])
  assert.equal(primary.name, big)
})

test('copy do disco usa TB/GB e livre = available', () => {
  const total = 1_000_000_000_000
  const used = 250_000_000_000
  const available = total - used
  const copy = diskCopy({ used, total, available })
  assert.equal(copy.used, `Usado ${formatDisk(used)}`)
  assert.equal(copy.free, `Livre ${formatDisk(available)}`)
  assert.equal(copy.total, `Total ${formatDisk(total)}`)
  assert.equal(copy.usage, '25% em uso')
})

test('gêmeos APFS com o mesmo nome e tamanho viram uma linha', () => {
  const name = faker.commerce.productName()
  const totalBytes = 245_000_000_000
  const availableBytes = 5_000_000_000
  const ranked = rankDiskVolumes([
    { name, mount: '/System/Volumes/Data', totalBytes, availableBytes },
    { name, mount: '/', totalBytes, availableBytes },
  ])
  assert.equal(ranked.length, 1)
  assert.equal(ranked[0].mount, '/')
  assert.equal(ranked[0].name, name)
})

test('volumes de tamanho diferente com o mesmo nome ficam os dois', () => {
  const name = faker.commerce.productName()
  const ranked = rankDiskVolumes([
    { name, mount: '/', totalBytes: 245_000_000_000, availableBytes: 5_000_000_000 },
    {
      name,
      mount: '/Volumes/Extra',
      totalBytes: 1_000_000_000_000,
      availableBytes: 100_000_000_000,
    },
  ])
  assert.equal(ranked.length, 2)
})

test('bytes aleatórios formatam igual o ouro SI', () => {
  const bytes = fakeBytes()
  if (bytes >= 1_000_000_000_000) {
    assert.equal(
      formatDisk(bytes),
      `${(bytes / 1_000_000_000_000).toFixed(1).replace('.', ',')} TB`,
    )
  } else {
    assert.equal(formatDisk(bytes), `${(bytes / 1_000_000_000).toFixed(1).replace('.', ',')} GB`)
  }
})
