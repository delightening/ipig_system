import { describe, expect, it } from 'vitest'

import {
  buildUomOptions,
  isUomReadOnly,
  selectedUomValue,
  type UomOptionSource,
} from '@/pages/documents/uomOptions'

/** 乳膠手套：base_uom = 雙，換算表有盒(50)、箱(500) */
const glove: UomOptionSource = {
  base_uom: '雙',
  uom: '雙',
  alt_uoms: ['盒', '箱'],
}

describe('buildUomOptions', () => {
  it('base_uom 排第一，其餘沿用後端依 factor_to_base 遞增的順序', () => {
    expect(buildUomOptions(glove)).toEqual(['雙', '盒', '箱'])
  })

  it('沒建任何換算率時只有一個選項——UomSelect 會據此退回純文字', () => {
    expect(buildUomOptions({ base_uom: '盒', uom: '盒', alt_uoms: [] })).toEqual(['盒'])
  })

  it('alt_uoms 未定義時等同空陣列，不應拋錯', () => {
    expect(buildUomOptions({ base_uom: '盒', uom: '盒' })).toEqual(['盒'])
  })

  it('base_uom 缺席時退回該行當前的 uom，不留空白', () => {
    // 舊單據載入、或品項尚未帶出換算表的情況
    expect(buildUomOptions({ uom: '瓶', alt_uoms: [] })).toEqual(['瓶'])
  })

  it('去重：舊資料若在換算表塞了與 base_uom 同名的列，不產生重複選項', () => {
    expect(buildUomOptions({ base_uom: '雙', uom: '雙', alt_uoms: ['雙', '盒'] })).toEqual([
      '雙',
      '盒',
    ])
  })

  it('濾掉空字串：新增行的初始 uom 是空的，不該變成一個空白選項', () => {
    expect(buildUomOptions({ uom: '', alt_uoms: [] })).toEqual([])
    expect(buildUomOptions({ base_uom: '雙', uom: '', alt_uoms: ['', '盒'] })).toEqual([
      '雙',
      '盒',
    ])
  })

  it('多換算率的品項選項數 > 1，沒建換算率的只有一個', () => {
    expect(buildUomOptions(glove).length).toBeGreaterThan(1)
    expect(buildUomOptions({ base_uom: '盒', uom: '盒' }).length).toBe(1)
  })
})

describe('isUomReadOnly', () => {
  // 這是 UomSelect 唯一的分支條件，四個分支全部釘住

  it('唯一選項就是現值 → 唯讀文字（下拉只會是噪音）', () => {
    expect(isUomReadOnly({ base_uom: '盒', uom: '盒', alt_uoms: [] })).toBe(true)
  })

  it('沒有任何選項（新增行 uom 還是空的）→ 唯讀文字，維持既有版面', () => {
    expect(isUomReadOnly({ uom: '', alt_uoms: [] })).toBe(true)
    expect(isUomReadOnly({ uom: '' })).toBe(true)
  })

  it('多個選項 → 下拉', () => {
    expect(isUomReadOnly(glove)).toBe(false)
  })

  it('🔴 唯一選項不等於現值 → 必須是下拉，否則舊資料的無效單位改不掉', () => {
    // 舊單據殘留自由字串「打」，而該品項的合法單位只有 base_uom「雙」。
    // 若照選項數判斷會渲染成唯讀文字，使用者看得到「打」卻改不動，
    // 後端 assert_lines_uom_defined 又必定回 400 —— 這張單就永遠存不了。
    expect(isUomReadOnly({ base_uom: '雙', uom: '打', alt_uoms: [] })).toBe(false)
  })

  it('現值不在多選項清單內時同樣是下拉', () => {
    expect(isUomReadOnly({ base_uom: '雙', uom: '打', alt_uoms: ['盒'] })).toBe(false)
  })
})

describe('selectedUomValue', () => {
  it('現值在選項內就原樣傳給 Radix', () => {
    expect(selectedUomValue(glove)).toBe('雙')
    expect(selectedUomValue({ base_uom: '雙', uom: '盒', alt_uoms: ['盒'] })).toBe('盒')
  })

  it('🔴 現值不在選項內 → 空字串，否則 Radix trigger 會一片空白', () => {
    // Radix: shouldShowPlaceholder(value) => value === '' || value === undefined
    // 傳「打」這種不在選項內的非空值，placeholder 不顯示、也沒有 SelectItem
    // 會把文字 portal 進 trigger，使用者只會看到空白。
    expect(selectedUomValue({ base_uom: '雙', uom: '打', alt_uoms: ['盒'] })).toBe('')
  })

  it('新增行的空 uom 本來就是空字串', () => {
    expect(selectedUomValue({ uom: '', alt_uoms: [] })).toBe('')
  })
})
