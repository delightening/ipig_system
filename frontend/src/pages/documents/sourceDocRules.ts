/**
 * 「來源單據」（source_doc_id）的送出規則，抽出 Hook 之外以便單獨測試。
 *
 * 只有入庫單（GRN）會指向來源採購單；其餘單據類型沒有來源單據這個概念。
 * 表單狀態裡的 `source_doc_id` 卻不會隨單據類型切換而清掉：
 * 先選入庫單、挑了來源採購單，再把類型改成採購退貨，殘留值會被原樣送出，
 * 造成「非入庫單卻指向採購單」的單據——核准後撤銷／反轉的按鈕都會消失（issue #189）。
 *
 * 後端並未限制 source_doc_id 的類型，所以由前端在送出前把關。
 */
import type { DocType } from '@/lib/api'

/** 會帶來源單據的單據類型。 */
export const SOURCE_DOC_TYPES: readonly string[] = ['GRN']

/** 該單據類型是否允許帶來源單據。 */
export function carriesSourceDoc(docType: DocType | string | undefined): boolean {
  return !!docType && SOURCE_DOC_TYPES.includes(docType)
}

/**
 * 送出時的 source_doc_id：不允許帶來源的類型一律送 `null`，其餘空白視為 `null`。
 *
 * @param docType 單據類型
 * @param sourceDocId 表單裡目前的來源單據 id（可能是殘留值）
 */
export function sourceDocIdForPayload(
  docType: DocType | string | undefined,
  sourceDocId: string | undefined,
): string | null {
  if (!carriesSourceDoc(docType)) return null
  return sourceDocId?.trim() ? sourceDocId : null
}
