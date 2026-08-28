import { describe, expect, it } from 'vitest'

import { buildUomOptions, type UomOptionSource } from '@/pages/documents/uomOptions'

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

  it('選項數決定 UomSelect 顯示下拉還是純文字', () => {
    // 這是元件唯一的分支條件，把它的兩側都釘住
    expect(buildUomOptions(glove).length).toBeGreaterThan(1)
    expect(buildUomOptions({ base_uom: '盒', uom: '盒' }).length).toBe(1)
  })
})
