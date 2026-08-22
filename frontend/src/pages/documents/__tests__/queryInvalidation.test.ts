import { describe, it, expect } from 'vitest'

import { documentChangeQueryKeys, STOCK_QUERY_KEY_PREFIXES } from '../queryInvalidation'

describe('documentChangeQueryKeys', () => {
  it('一律失效該單據詳情與單據清單', () => {
    const keys = documentChangeQueryKeys('doc-1', { affectsStock: false })
    expect(keys).toContainEqual(['document', 'doc-1'])
    expect(keys).toContainEqual(['documents'])
  })

  it('id 為 undefined 時不放入單據詳情 key，仍失效單據清單', () => {
    const keys = documentChangeQueryKeys(undefined, { affectsStock: false })
    expect(keys).not.toContainEqual(['document', undefined])
    expect(keys).toContainEqual(['documents'])
    expect(keys).toHaveLength(1)
  })

  it('非庫存異動（送審 / 駁回 / 作廢）不失效庫存查詢', () => {
    const keys = documentChangeQueryKeys('doc-1', { affectsStock: false })
    expect(keys).toHaveLength(2)
    for (const stockKey of STOCK_QUERY_KEY_PREFIXES) {
      expect(keys).not.toContainEqual(stockKey)
    }
  })

  // 回歸：核准調整單(ADJ)會改庫存，核准後庫存頁面必須自動更新、毋須手動重新整理。
  it('核准會套用庫存時，額外失效全部庫存相關查詢', () => {
    const keys = documentChangeQueryKeys('doc-1', { affectsStock: true })
    expect(keys).toContainEqual(['document', 'doc-1'])
    expect(keys).toContainEqual(['documents'])
    for (const stockKey of STOCK_QUERY_KEY_PREFIXES) {
      expect(keys).toContainEqual(stockKey)
    }
    expect(keys).toHaveLength(2 + STOCK_QUERY_KEY_PREFIXES.length)
  })

  it('庫存 key 前綴涵蓋庫存查詢頁與庫存流水', () => {
    expect(STOCK_QUERY_KEY_PREFIXES).toContainEqual(['inventory'])
    expect(STOCK_QUERY_KEY_PREFIXES).toContainEqual(['stock-ledger'])
  })
})
