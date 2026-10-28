import { describe, it, expect } from 'vitest'

import {
  detailReversalState,
  listReversalState,
  receiptProgress,
  resolveReverseRevokeButtons,
} from '../documentActionState'

/**
 * 撤銷／沖銷按鈕分流與「已沖銷」衍生顯示。
 *
 * 規格的四種詳情頁狀況各一條；`can_revoke` 由後端判定，前端只讀、不自行推導，
 * 所以這裡的測試只餵 `can_revoke`，不模擬「有沒有有效入庫單」。
 */
describe('resolveReverseRevokeButtons', () => {
  const approvedPo = {
    status: 'approved' as const,
    can_revoke: false,
    reverses_doc_id: undefined,
    reversed_by_doc_id: undefined,
  }

  it('已核准、無有效下游單據（can_revoke）→ 只顯示撤銷', () => {
    expect(resolveReverseRevokeButtons({ ...approvedPo, can_revoke: true }, true)).toEqual({
      showRevoke: true,
      showReverse: false,
    })
  })

  it('已核准、有有效入庫單（can_revoke 為假）→ 只顯示沖銷', () => {
    expect(resolveReverseRevokeButtons(approvedPo, true)).toEqual({
      showRevoke: false,
      showReverse: true,
    })
  })

  it('已被沖銷 → 兩顆都不顯示', () => {
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, reversed_by_doc_id: 'rev-1' }, true),
    ).toEqual({ showRevoke: false, showReverse: false })
  })

  it('草稿／待核准 → 兩顆都不顯示（維持既有「作廢」）', () => {
    for (const status of ['draft', 'submitted'] as const) {
      expect(resolveReverseRevokeButtons({ ...approvedPo, status }, true)).toEqual({
        showRevoke: false,
        showReverse: false,
      })
    }
  })

  it('沖銷單本身不可再沖銷', () => {
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, reverses_doc_id: 'orig-1' }, true).showReverse,
    ).toBe(false)
  })

  it('沒有沖銷角色 → 不顯示沖銷；撤銷的角色判斷在後端 can_revoke', () => {
    expect(resolveReverseRevokeButtons(approvedPo, false).showReverse).toBe(false)
    expect(resolveReverseRevokeButtons({ ...approvedPo, can_revoke: true }, false).showRevoke).toBe(
      true,
    )
  })

  it('can_revoke 缺欄位（舊後端）→ 視為假，不顯示撤銷', () => {
    const legacy = {
      status: 'approved' as const,
      reverses_doc_id: undefined,
      reversed_by_doc_id: undefined,
    }
    expect(resolveReverseRevokeButtons(legacy, true).showRevoke).toBe(false)
  })
})

describe('沖銷衍生狀態', () => {
  it('詳情：沖銷單已核准 → 已沖銷；待核准 → 沖銷中；沒有沖銷單 → 無', () => {
    expect(
      detailReversalState({ reversed_by_doc_id: 'r1', reversed_at: '2026-10-28T08:00:00Z' }),
    ).toBe('reversed')
    expect(detailReversalState({ reversed_by_doc_id: 'r1', reversed_at: undefined })).toBe(
      'reversing',
    )
    expect(detailReversalState({ reversed_by_doc_id: undefined, reversed_at: undefined })).toBeNull()
  })

  it('列表：只認 reversed / reversing，其餘（含 null、未知值）一律無', () => {
    expect(listReversalState({ reversal_state: 'reversed' })).toBe('reversed')
    expect(listReversalState({ reversal_state: 'reversing' })).toBe('reversing')
    expect(listReversalState({ reversal_state: null })).toBeNull()
    expect(listReversalState({ reversal_state: undefined })).toBeNull()
    expect(
      listReversalState({ reversal_state: 'cancelled' as unknown as 'reversed' }),
    ).toBeNull()
  })
})

describe('receiptProgress：入庫進度欄的優先序', () => {
  const po = {
    doc_type: 'PO' as const,
    status: 'approved' as const,
    receipt_status: 'pending',
    reversal_state: null,
  }

  it('一般已核准 PO 顯示入庫進度', () => {
    expect(receiptProgress(po)?.labelKey).toBe('erpDocs.documents.table.receipt.pending')
    expect(receiptProgress({ ...po, receipt_status: 'partial' })?.labelKey).toBe(
      'erpDocs.documents.table.receipt.partial',
    )
  })

  it('已撤銷（作廢且 receipt_status 殘留）→ 已撤銷，不是「未入庫」', () => {
    expect(receiptProgress({ ...po, status: 'cancelled' })?.labelKey).toBe(
      'erpDocs.documents.table.receipt.revoked',
    )
  })

  it('草稿／待核准時被作廢（receipt_status 從未寫入）→ 不是撤銷，不顯示', () => {
    expect(receiptProgress({ ...po, status: 'cancelled', receipt_status: undefined })).toBeNull()
  })

  it('已沖銷／沖銷中優先於入庫進度', () => {
    expect(receiptProgress({ ...po, reversal_state: 'reversed' })?.labelKey).toBe(
      'erpDocs.documents.reversal.badgeReversed',
    )
    expect(receiptProgress({ ...po, reversal_state: 'reversing' })?.labelKey).toBe(
      'erpDocs.documents.reversal.badgeReversing',
    )
  })

  it('非 PO 不顯示入庫進度', () => {
    expect(receiptProgress({ ...po, doc_type: 'GRN' })).toBeNull()
  })
})
