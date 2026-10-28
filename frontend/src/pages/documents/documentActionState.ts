import type { Document, DocumentListItem } from '@/lib/api'

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

/**
 * 「撤銷」與「沖銷」按鈕的分流：同一張單只會出現其中一顆。
 *
 * - `can_revoke` 由後端判定（與 `POST /documents/{id}/revoke` 同一個判斷函式，已含角色與
 *   「有沒有有效下游單據」），前端**只讀、不自行推導**——自己算會讓按鈕與後端前置條件漂移。
 * - 沖銷維持既有條件，另加一條：可撤銷的單不顯示沖銷（走哪條路由後端決定）。
 * - 已被沖銷的單兩顆都不顯示。
 */
export function resolveReverseRevokeButtons(
  doc: Pick<Document, 'status' | 'can_revoke' | 'reverses_doc_id' | 'reversed_by_doc_id'>,
  canReverseByRole: boolean,
): { showRevoke: boolean; showReverse: boolean } {
  const approved = doc.status === 'approved'
  const showRevoke = approved && doc.can_revoke === true
  const showReverse =
    approved &&
    !doc.reverses_doc_id &&
    !doc.reversed_by_doc_id &&
    canReverseByRole &&
    !showRevoke
  return { showRevoke, showReverse }
}
