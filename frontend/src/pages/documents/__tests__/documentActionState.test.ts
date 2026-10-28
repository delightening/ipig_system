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
 * 後端判定的欄位（`can_revoke`、`has_effective_downstream`、`revoke_blocked_by`）前端只讀、
 * 不自行推導，所以這裡的測試直接餵這三個欄位，不模擬「有沒有有效入庫單」。
 */
describe('resolveReverseRevokeButtons', () => {
  const NONE = { showRevoke: false, showReverse: false, revokeBlockedBy: null }

  /** 已核准、非沖銷單、尚未被沖銷的採購單；各情境在此之上覆寫後端欄位。 */
  const approvedPo = {
    status: 'approved' as const,
    doc_type: 'PO' as const,
    can_revoke: false,
    revoke_blocked_by: undefined,
    has_effective_downstream: false,
    reverses_doc_id: undefined,
    reversed_by_doc_id: undefined,
  }
  const draftGrn = { doc_no: 'GRN-261028-01', doc_type: 'GRN' as const, status: 'draft' as const }

  it('沒有任何下游單據（can_revoke）→ 只顯示撤銷', () => {
    expect(resolveReverseRevokeButtons({ ...approvedPo, can_revoke: true }, true)).toEqual({
      ...NONE,
      showRevoke: true,
    })
  })

  it('有有效入庫單 → 只顯示沖銷', () => {
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, has_effective_downstream: true }, true),
    ).toEqual({ ...NONE, showReverse: true })
  })

  it('已被沖銷 → 兩顆都不顯示', () => {
    expect(
      resolveReverseRevokeButtons(
        { ...approvedPo, reversed_by_doc_id: 'rev-1', has_effective_downstream: true },
        true,
      ),
    ).toEqual(NONE)
  })

  it('QA 退件(a)：只有草稿入庫單 → 撤銷停用並點名單號，沖銷不出現', () => {
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, revoke_blocked_by: draftGrn }, true),
    ).toEqual({ ...NONE, revokeBlockedBy: draftGrn })
  })

  it('QA 退件(b)：只有待核准入庫單 → 同上，狀態跟著帶出', () => {
    const submittedGrn = { ...draftGrn, status: 'submitted' as const }
    const result = resolveReverseRevokeButtons({ ...approvedPo, revoke_blocked_by: submittedGrn }, true)
    expect(result).toEqual({ ...NONE, revokeBlockedBy: submittedGrn })
    expect(result.showReverse).toBe(false)
  })

  it('有有效入庫單、同時還有草稿入庫單 → 走沖銷，不顯示停用的撤銷', () => {
    expect(
      resolveReverseRevokeButtons(
        { ...approvedPo, has_effective_downstream: true, revoke_blocked_by: draftGrn },
        true,
      ),
    ).toEqual({ ...NONE, showReverse: true })
  })

  it('沒有擋下者、也沒有有效入庫單，且不可撤銷（例如 PO 有待核准的沖銷單）→ 什麼都不顯示', () => {
    expect(resolveReverseRevokeButtons(approvedPo, true)).toEqual(NONE)
  })

  it('草稿／待核准 → 兩顆都不顯示（維持既有「作廢」）', () => {
    for (const status of ['draft', 'submitted'] as const) {
      expect(
        resolveReverseRevokeButtons({ ...approvedPo, status, has_effective_downstream: true }, true),
      ).toEqual(NONE)
    }
  })

  it('沖銷單本身不可再沖銷', () => {
    expect(
      resolveReverseRevokeButtons(
        { ...approvedPo, reverses_doc_id: 'orig-1', has_effective_downstream: true },
        true,
      ).showReverse,
    ).toBe(false)
  })

  it('沒有沖銷角色 → 不顯示沖銷、也不顯示停用的撤銷；撤銷的角色判斷在後端 can_revoke', () => {
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, has_effective_downstream: true }, false),
    ).toEqual(NONE)
    expect(
      resolveReverseRevokeButtons({ ...approvedPo, revoke_blocked_by: draftGrn }, false),
    ).toEqual(NONE)
    expect(resolveReverseRevokeButtons({ ...approvedPo, can_revoke: true }, false).showRevoke).toBe(
      true,
    )
  })

  it('後端欄位缺（舊後端）→ 視為假：不顯示撤銷，採購單也不顯示沖銷', () => {
    const legacy = {
      status: 'approved' as const,
      doc_type: 'PO' as const,
      reverses_doc_id: undefined,
      reversed_by_doc_id: undefined,
    }
    expect(resolveReverseRevokeButtons(legacy, true)).toEqual(NONE)
  })

  it('非採購單：沒有「撤銷」，沖銷維持既有條件（已核准、非沖銷單、未被沖銷、角色允許）', () => {
    const approvedGrn = { ...approvedPo, doc_type: 'GRN' as const }
    expect(resolveReverseRevokeButtons(approvedGrn, true)).toEqual({ ...NONE, showReverse: true })
    expect(resolveReverseRevokeButtons(approvedGrn, false)).toEqual(NONE)
    expect(
      resolveReverseRevokeButtons({ ...approvedGrn, reversed_by_doc_id: 'rev-1' }, true),
    ).toEqual(NONE)
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
