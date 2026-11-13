/**
 * 批次新增動物的純邏輯。
 *
 * 要釘住的契約：
 * 1. 耳號輸入支援清單與範圍（可混用）、補零成三位、去重、無效片段單獨回報。
 * 2. 加入列表時略過已存在的耳號，並受單次上限（50）限制。
 * 3. 逐隻檢查：體重／品種／出生日期／欄位必填、出生日期不可是未來、批次內耳號不可重複；
 *    欄位容量依列順序佔用，超出空位的列標紅，容量 0 代表無限制。
 * 4. 組請求：共用欄位填進每一筆；「逐隻不同」的欄位取該列的值；pen_location 與 pen_id 一併送出。
 */
import { describe, it, expect } from 'vitest'

import {
  BATCH_CREATE_MAX_ANIMALS,
  addTagsToRows,
  applyBatchResults,
  buildBatchItem,
  checkRows,
  parseEarTagInput,
  type BatchRow,
  type BatchShared,
  type FieldModes,
  type PenInfo,
} from '@/pages/animals/batchAdd/batchAddLogic'

const TODAY = '2026-11-12'

const shared: BatchShared = {
  entryDate: '2026-11-12',
  sourceId: '',
  preExperimentCode: ' PRE-001 ',
  speciesId: 'sp-mini',
  breedOther: '',
  gender: 'male',
  birthDate: '2026-09-15',
  penId: 'pen-a',
}

const allSame: FieldModes = { species: 'same', gender: 'same', birthDate: 'same', pen: 'same' }

function row(earTag: string, overrides: Partial<BatchRow> = {}): BatchRow {
  return {
    key: `k-${earTag}`,
    earTag,
    speciesId: '',
    gender: 'male',
    birthDate: '',
    penId: '',
    weight: '25.5',
    confirmForce: false,
    ...overrides,
  }
}

const pens = (list: PenInfo[]) => new Map(list.map((p) => [p.id, p]))
const penA: PenInfo = { id: 'pen-a', code: 'A01', capacity: 0, current_count: 0 }

describe('parseEarTagInput', () => {
  it('清單：空白、逗號、頓號、換行都可分隔，並補零成三位', () => {
    const r = parseEarTagInput('21 022,23、024\n5')
    expect(r.tags).toEqual(['021', '022', '023', '024', '005'])
    expect(r.invalid).toEqual([])
  })

  it('範圍含頭尾，前後可有空白，接受多種連字符', () => {
    expect(parseEarTagInput('021-024').tags).toEqual(['021', '022', '023', '024'])
    expect(parseEarTagInput('021 - 023').tags).toEqual(['021', '022', '023'])
    expect(parseEarTagInput('021~022').tags).toEqual(['021', '022'])
    expect(parseEarTagInput('021–022').tags).toEqual(['021', '022'])
  })

  it('清單與範圍可混用，順序依輸入', () => {
    expect(parseEarTagInput('030 021-023 040').tags).toEqual(['030', '021', '022', '023', '040'])
  })

  it('重複的耳號只保留第一次，並單獨回報', () => {
    const r = parseEarTagInput('021 022 021 021-022')
    expect(r.tags).toEqual(['021', '022'])
    expect(r.duplicates).toEqual(['021', '022'])
  })

  it('無法解析的片段單獨回報，不影響其他', () => {
    const r = parseEarTagInput('021 abc 5-3 1000 022')
    expect(r.tags).toEqual(['021', '022'])
    expect(r.invalid).toEqual(['abc', '5-3', '1000'])
  })

  it('空白輸入回傳空結果', () => {
    expect(parseEarTagInput('  \n ')).toEqual({ tags: [], invalid: [], duplicates: [] })
  })
})

describe('addTagsToRows', () => {
  const make = (tag: string) => row(tag)

  it('略過已在列表中的耳號並計數', () => {
    const out = addTagsToRows([row('021')], ['021', '022'], make)
    expect(out.rows.map((r) => r.earTag)).toEqual(['021', '022'])
    expect(out.skippedExisting).toBe(1)
    expect(out.skippedOverLimit).toBe(0)
  })

  it(`受單次上限 ${BATCH_CREATE_MAX_ANIMALS} 隻限制`, () => {
    const tags = Array.from({ length: 60 }, (_, i) => String(i + 100))
    const out = addTagsToRows([], tags, make)
    expect(out.rows).toHaveLength(BATCH_CREATE_MAX_ANIMALS)
    expect(out.skippedOverLimit).toBe(10)
  })
})

describe('checkRows', () => {
  it('全部合法 → 沒有任何問題', () => {
    const rows = [row('021'), row('022')]
    const { issuesByKey, hasPenFull } = checkRows(rows, shared, allSame, pens([penA]), TODAY)
    expect(Object.values(issuesByKey).flat()).toEqual([])
    expect(hasPenFull).toBe(false)
  })

  it('體重必填且須為正數', () => {
    const rows = [row('021', { weight: '' }), row('022', { weight: '0' }), row('023', { weight: 'abc' })]
    const { issuesByKey } = checkRows(rows, shared, allSame, pens([penA]), TODAY)
    for (const r of rows) expect(issuesByKey[r.key].map((i) => i.code)).toEqual(['weightInvalid'])
  })

  it('整批相同模式：共用的出生日期不可是未來', () => {
    const future = { ...shared, birthDate: '2026-11-13' }
    const { issuesByKey } = checkRows([row('021')], future, allSame, pens([penA]), TODAY)
    expect(issuesByKey['k-021'].map((i) => i.code)).toEqual(['birthFuture'])
  })

  it('逐隻不同模式：缺的欄位在該列標出，其他列不受影響', () => {
    const modes: FieldModes = { species: 'each', gender: 'each', birthDate: 'each', pen: 'each' }
    const rows = [
      row('021', { speciesId: 'sp-mini', birthDate: '2026-09-15', penId: 'pen-a' }),
      row('022'),
    ]
    const { issuesByKey } = checkRows(rows, shared, modes, pens([penA]), TODAY)
    expect(issuesByKey['k-021']).toEqual([])
    expect(issuesByKey['k-022'].map((i) => i.code)).toEqual(['speciesRequired', 'birthRequired', 'penRequired'])
  })

  it('批次內耳號重複的每一列都標紅', () => {
    const rows = [row('021', { key: 'a' }), row('021', { key: 'b' })]
    const { issuesByKey } = checkRows(rows, shared, allSame, pens([penA]), TODAY)
    expect(issuesByKey.a.map((i) => i.code)).toEqual(['duplicateInBatch'])
    expect(issuesByKey.b.map((i) => i.code)).toEqual(['duplicateInBatch'])
  })

  it('欄位容量：依列順序佔用，超出空位的列標紅（容量 2、已有 1 → 只放得下 1 隻）', () => {
    const tight: PenInfo = { id: 'pen-a', code: 'A01', capacity: 2, current_count: 1 }
    const rows = [row('021'), row('022'), row('023')]
    const { issuesByKey, hasPenFull } = checkRows(rows, shared, allSame, pens([tight]), TODAY)
    expect(issuesByKey['k-021']).toEqual([])
    expect(issuesByKey['k-022'].map((i) => i.code)).toEqual(['penFull'])
    expect(issuesByKey['k-022'][0].params).toEqual({ pen: 'A01', capacity: 2, current: 1 })
    expect(issuesByKey['k-023'].map((i) => i.code)).toEqual(['penFull'])
    expect(hasPenFull).toBe(true)
  })

  it('欄位容量 0 代表無限制', () => {
    const rows = Array.from({ length: 20 }, (_, i) => row(String(100 + i)))
    const { hasPenFull } = checkRows(rows, shared, allSame, pens([penA]), TODAY)
    expect(hasPenFull).toBe(false)
  })

  it('不同欄位各自計數：A 欄放不下的列改到 B 欄就通過', () => {
    const penAFull: PenInfo = { id: 'pen-a', code: 'A01', capacity: 1, current_count: 0 }
    const penB: PenInfo = { id: 'pen-b', code: 'A02', capacity: 1, current_count: 0 }
    const modes: FieldModes = { ...allSame, pen: 'each' }
    const rows = [
      row('021', { penId: 'pen-a' }),
      row('022', { penId: 'pen-a' }),
      row('023', { penId: 'pen-b' }),
    ]
    const { issuesByKey } = checkRows(rows, shared, modes, pens([penAFull, penB]), TODAY)
    expect(issuesByKey['k-021']).toEqual([])
    expect(issuesByKey['k-022'].map((i) => i.code)).toEqual(['penFull'])
    expect(issuesByKey['k-023']).toEqual([])
  })
})

describe('applyBatchResults', () => {
  const sent = ['k-021', 'k-022', 'k-023']

  it('已建立的列移除；重複與失敗的留下並標上原因', () => {
    const rows = [row('021'), row('022'), row('023')]
    const out = applyBatchResults(rows, sent, [
      { index: 0, ear_tag: '021', outcome: 'created', animal_id: 'a1' },
      { index: 1, ear_tag: '022', outcome: 'duplicate_warning', message: '耳號 022 已存在其他存活動物' },
      { index: 2, ear_tag: '023', outcome: 'failed', message: '欄位已滿' },
    ])
    expect(out.createdCount).toBe(1)
    expect(out.rows.map((r) => r.earTag)).toEqual(['022', '023'])
    expect(out.rows[0]).toMatchObject({ serverOutcome: 'duplicate_warning', confirmForce: false })
    expect(out.rows[0].serverMessage).toContain('已存在')
    expect(out.rows[1]).toMatchObject({ serverOutcome: 'failed', serverMessage: '欄位已滿' })
  })

  it('沒送出的列原樣保留；重送時以先前的位置對回正確的列', () => {
    const rows = [row('021'), row('022')]
    // 只送第二列（index 0 對應 k-022）
    const out = applyBatchResults(rows, ['k-022'], [{ index: 0, ear_tag: '022', outcome: 'created' }])
    expect(out.createdCount).toBe(1)
    expect(out.rows.map((r) => r.earTag)).toEqual(['021'])
  })

  it('重送時先前確認過的重複標記會被重設，再次失敗須重新確認', () => {
    const rows = [row('022', { confirmForce: true, serverOutcome: 'duplicate_warning' })]
    const out = applyBatchResults(rows, ['k-022'], [
      { index: 0, ear_tag: '022', outcome: 'failed', message: '耳號已存在同出生日期的存活動物' },
    ])
    expect(out.rows[0]).toMatchObject({ serverOutcome: 'failed', confirmForce: false })
  })
})

describe('buildBatchItem', () => {
  const pensById = pens([{ id: 'pen-a', code: 'A01', capacity: 0, current_count: 0 }])

  it('整批相同：共用欄位填進每一筆，pen_location 與 pen_id 一併送出', () => {
    const item = buildBatchItem(row('021', { weight: '25.5' }), shared, allSame, pensById)
    expect(item).toEqual({
      ear_tag: '021',
      species_id: 'sp-mini',
      gender: 'male',
      entry_date: '2026-11-12',
      birth_date: '2026-09-15',
      entry_weight: 25.5,
      pen_location: 'A01',
      pen_id: 'pen-a',
      pre_experiment_code: 'PRE-001',
    })
  })

  it('逐隻不同：取該列的值，不用共用值', () => {
    const modes: FieldModes = { species: 'each', gender: 'each', birthDate: 'each', pen: 'each' }
    const item = buildBatchItem(
      row('022', { speciesId: 'sp-white', gender: 'female', birthDate: '2026-09-01', penId: 'pen-a' }),
      shared,
      modes,
      pensById,
    )
    expect(item.species_id).toBe('sp-white')
    expect(item.gender).toBe('female')
    expect(item.birth_date).toBe('2026-09-01')
  })

  it('有來源才帶 source_id；確認過的重複耳號才帶 force_create', () => {
    const withSource = { ...shared, sourceId: 'src-1' }
    const item = buildBatchItem(row('021', { confirmForce: true }), withSource, allSame, pensById)
    expect(item.source_id).toBe('src-1')
    expect(item.force_create).toBe(true)

    const plain = buildBatchItem(row('021'), shared, allSame, pensById)
    expect('source_id' in plain).toBe(false)
    expect('force_create' in plain).toBe(false)
  })
})
