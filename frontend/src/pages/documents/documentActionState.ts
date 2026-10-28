import type { Document, DocumentListItem, RevokeBlockedBy } from '@/lib/api'

/** 沖銷衍生狀態：原單 DB 狀態仍是 approved，「已沖銷」「沖銷中」只是顯示。 */
export type ReversalDisplayState = 'reversed' | 'reversing' | null

/**
 * 撤銷原因的字數下限。與後端 `MIN_REVOKE_REASON_CHARS`（services/document/revoke.rs）同值，
 * 改一邊要改另一邊；後端才是最後一道閘，這裡只是讓按鈕在明顯不合格時不可按。
 */
export const MIN_REVOKE_REASON_CHARS = 5

/**
 * 詳情頁的沖銷衍生狀態。
 *
 * 沖銷單核准後 `reversed_at` 才有值；已建立但尚未核准的沖銷單只有 `reversed_by_doc_id`。
 * 沖銷單已作廢者後端不回 `reversed_by_doc_id`（原單回到普通狀態），這裡不需要特別處理。
 */
export function detailReversalState(
  doc: Pick<Document, 'reversed_by_doc_id' | 'reversed_at'>,
): ReversalDisplayState {
  if (!doc.reversed_by_doc_id) return null
  return doc.reversed_at ? 'reversed' : 'reversing'
}

/** 列表的沖銷衍生狀態：後端直接帶 `reversal_state`，只做型別收斂。 */
export function listReversalState(doc: Pick<DocumentListItem, 'reversal_state'>): ReversalDisplayState {
  return doc.reversal_state === 'reversed' || doc.reversal_state === 'reversing'
    ? doc.reversal_state
    : null
}

export interface ReceiptProgress {
  /** i18n 鍵（渲染時才 t()，語言切換才會即時更新） */
  labelKey: string
  variant: 'warning' | 'info' | 'success' | 'error' | 'neutral'
}

const RECEIPT_STATUS_CONFIG: Record<string, ReceiptProgress> = {
  pending: { labelKey: 'erpDocs.documents.table.receipt.pending', variant: 'warning' },
  partial: { labelKey: 'erpDocs.documents.table.receipt.partial', variant: 'info' },
  complete: { labelKey: 'erpDocs.documents.table.receipt.complete', variant: 'success' },
}

/**
 * PO「入庫進度」欄的顯示。已沖銷／沖銷中／已撤銷優先於入庫進度——否則撤銷後仍殘留
 * `receipt_status = 'pending'` 的 PO 會被讀成「未入庫」，使用者以為還有貨要收。
 *
 * 「已撤銷」＝作廢且 `receipt_status` 有值：`receipt_status` 只在核准時寫入，所以草稿／待核准時
 * 被作廢的 PO 沒有值（那不是撤銷），撤銷則保留核准當時的值。
 */
export function receiptProgress(
  doc: Pick<DocumentListItem, 'doc_type' | 'status' | 'receipt_status' | 'reversal_state'>,
): ReceiptProgress | null {
  if (doc.doc_type !== 'PO') return null

  const reversal = listReversalState(doc)
  if (reversal === 'reversed') {
    return { labelKey: 'erpDocs.documents.reversal.badgeReversed', variant: 'error' }
  }
  if (reversal === 'reversing') {
    return { labelKey: 'erpDocs.documents.reversal.badgeReversing', variant: 'warning' }
  }

  if (doc.status === 'cancelled') {
    return doc.receipt_status
      ? { labelKey: 'erpDocs.documents.table.receipt.revoked', variant: 'neutral' }
      : null
  }
  if (doc.status !== 'approved') return null
  return doc.receipt_status ? (RECEIPT_STATUS_CONFIG[doc.receipt_status] ?? null) : null
}

export interface ReverseRevokeButtons {
  showRevoke: boolean
  showReverse: boolean
  /**
   * 「撤銷」顯示但停用時，擋下它的下游單據（單號與狀態）；`null` 表示不需要顯示停用的撤銷。
   * 與 `showRevoke` 互斥：能撤銷就不會有擋下者。
   */
  revokeBlockedBy: RevokeBlockedBy | null
}

/**
 * 「撤銷」與「沖銷」按鈕的分流。
 *
 * 後端判定的欄位**只讀、不自行推導**——自己算會讓按鈕與後端前置條件漂移：
 * - `can_revoke`：與 `POST /documents/{id}/revoke` 同一個判斷函式，已含角色與「有沒有下游單據」。
 * - `has_effective_downstream`：採購單有沒有有效下游單據（已核准且未被沖銷）。
 * - `revoke_blocked_by`：擋住撤銷的草稿／待核准下游單據。
 *
 * 採購單的四種情況（皆為已核准、非沖銷單、尚未被沖銷）：
 * 1. 沒有任何下游單據 → 只顯示「撤銷」
 * 2. 有有效下游單據（入庫單）→ 只顯示「沖銷」（撤銷會被後端擋、也不顯示）
 * 3. 只有草稿／待核准的下游單據 → 「撤銷」顯示但停用並點名單號；**不顯示沖銷**
 *    （沖銷在沒有有效入庫單時不適用，後端 `create_reversal` 也會拒絕）
 * 4. 已被沖銷 → 兩顆都不顯示
 *
 * 其他單據類型沒有「撤銷」，沖銷維持既有條件（已核准、非沖銷單、尚未被沖銷、角色允許）。
 */
export function resolveReverseRevokeButtons(
  doc: Pick<
    Document,
    | 'status'
    | 'doc_type'
    | 'can_revoke'
    | 'revoke_blocked_by'
    | 'has_effective_downstream'
    | 'reverses_doc_id'
    | 'reversed_by_doc_id'
  >,
  canReverseByRole: boolean,
): ReverseRevokeButtons {
  const none: ReverseRevokeButtons = { showRevoke: false, showReverse: false, revokeBlockedBy: null }
  const eligible = doc.status === 'approved' && !doc.reverses_doc_id && !doc.reversed_by_doc_id
  if (!eligible) return none

  if (doc.can_revoke === true) return { ...none, showRevoke: true }
  if (!canReverseByRole) return none

  if (doc.doc_type !== 'PO') return { ...none, showReverse: true }

  if (doc.has_effective_downstream === true) return { ...none, showReverse: true }
  return { ...none, revokeBlockedBy: doc.revoke_blocked_by ?? null }
}
