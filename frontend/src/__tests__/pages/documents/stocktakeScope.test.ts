import { describe, expect, it } from 'vitest'

import {
  buildStocktakeScope,
  scopeForPayload,
  stocktakeBlockReason,
} from '@/pages/documents/stocktakeScope'
import type { StocktakeScope } from '@/pages/documents/types'

describe('buildStocktakeScope', () => {
  it('一個品類都沒選＝全盤，行為與本功能存在之前相同', () => {
    expect(buildStocktakeScope([])).toEqual({
      scope_type: 'full',
      category_codes: [],
    })
  })

  it('選了品類就是循環盤點', () => {
    expect(buildStocktakeScope(['DRG'])).toEqual({
      scope_type: 'partial',
      category_codes: ['DRG'],
    })
  })

  it('多選時全部保留，順序不變', () => {
    expect(buildStocktakeScope(['DRG', 'CHM'])).toEqual({
      scope_type: 'partial',
      category_codes: ['DRG', 'CHM'],
    })
  })

  it('scope_type 永遠有值——後端缺它會判為格式錯誤而回 400', () => {
    for (const codes of [[], ['DRG'], ['DRG', 'CHM', 'CON']]) {
      expect(buildStocktakeScope(codes).scope_type).toBeTruthy()
    }
  })
})

describe('scopeForPayload', () => {
  const scope: StocktakeScope = { scope_type: 'partial', category_codes: ['DRG'] }

  it('STK 帶著範圍時原樣送出', () => {
    expect(scopeForPayload('STK', scope)).toEqual(scope)
  })

  it('STK 但沒有範圍時送 null', () => {
    expect(scopeForPayload('STK', undefined)).toBeNull()
  })

  it('編輯 STK 時送 null——後端沒有 UPDATE 路徑會讀它，而表單帶的是預設值不是該單的真實範圍', () => {
    expect(scopeForPayload('STK', scope, true)).toBeNull()
  })

  it.each(['PO', 'GRN', 'PR', 'SO', 'TR', 'ADJ'] as const)(
    '%s 一律送 null——後端不會讀它，留著只會被誤讀成「這張單當初盤了哪些類別」',
    (docType) => {
      expect(scopeForPayload(docType, scope)).toBeNull()
    },
  )
})

describe('stocktakeBlockReason', () => {
  it('品類清單載入完成才准建單', () => {
    expect(
      stocktakeBlockReason({ needed: true, loading: false, error: false }),
    ).toBeUndefined()
  })

  it('載入中要擋——空清單看起來就像「沒有品類可選」', () => {
    expect(stocktakeBlockReason({ needed: true, loading: true, error: false })).toContain(
      '載入中',
    )
  })

  it('載入失敗要擋，且說明後果是會盤到全部品項', () => {
    expect(stocktakeBlockReason({ needed: true, loading: false, error: true })).toContain(
      '全部品項',
    )
  })

  it('loading 與 error 同時為真時以 loading 的訊息為準（重試中就是這個狀態）', () => {
    expect(stocktakeBlockReason({ needed: true, loading: true, error: true })).toContain(
      '載入中',
    )
  })

  it.each([
    ['載入中', true, false],
    ['載入失敗', false, true],
  ] as [string, boolean, boolean][])('不需要品類清單時一律不擋（%s 也一樣）', (_label, loading, error) => {
    // 改單與其他單別不會重新產生底稿，品類清單的狀態與它們無關；
    // 在這裡擋下去只會讓不相干的單據也送不出去。
    expect(stocktakeBlockReason({ needed: false, loading, error })).toBeUndefined()
  })
})
