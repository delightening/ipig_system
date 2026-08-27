import { describe, expect, it } from 'vitest'

import { buildStocktakeScope, scopeForPayload } from '@/pages/documents/stocktakeScope'
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

  it.each(['PO', 'GRN', 'PR', 'SO', 'TR', 'ADJ'] as const)(
    '%s 一律送 null——後端不會讀它，留著只會被誤讀成「這張單當初盤了哪些類別」',
    (docType) => {
      expect(scopeForPayload(docType, scope)).toBeNull()
    },
  )
})
