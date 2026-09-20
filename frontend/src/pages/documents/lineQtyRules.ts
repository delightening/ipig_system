/**
 * 明細數量（qty）的驗證規則，抽出 Hook 之外以便單獨測試。
 *
 * 後端對應：`DocumentService::validate_line_qty_price`（`backend/src/services/document/crud.rs`）。
 * 兩邊必須同進退——前端比後端寬會讓錯誤拖到 API 才浮現，比後端嚴則會擋掉合法操作
 * （2026-09-14 的 STK／ADJ 故障就是後者）。
 */
import i18n from '@/lib/i18n'
import type { DocType } from '@/lib/api'

/**
 * 檢查單一明細的數量，不合法時回錯誤訊息，合法時回 `undefined`。
 *
 * 收的是使用者輸入的**原始字串**而非數字，因為 `<input type="number">` 在值不成數
 * （空白、只打了一個減號）時回的是空字串，那一段的判定也屬於本規則的一部分，
 * 轉成數字再傳進來就測不到了。
 *
 * 規則依單據類型分流（2026-09-14 使用者裁定）：
 *
 * - **STK 盤點**：`>= 0`。0 合法且必要——「系統有、現場沒有」＝全數短少，是盤點最該
 *   登記的結果之一；負數在盤點語意下不存在，實盤數不可能是負的。
 * - **ADJ 調整**：可正可負（調增／調減），但不得為 0——沒有調整就不該開調整單。
 * - **其餘**（PO／GRN／PR／SO／TR）：`> 0`。
 *
 * @param docType 單據類型
 * @param rawQty 使用者輸入的數量字串
 * @param lineNo 行號（1 起算，只用於錯誤訊息）
 */
export function lineQtyError(
  docType: DocType,
  rawQty: string | undefined,
  lineNo: number,
): string | undefined {
  // 訊息＝「第 N 行：」前綴＋規則本體；前綴永遠在句首，兩種語言語序一致。
  const withLine = (message: string) =>
    `${i18n.t('erpDocs.documents.validation.linePrefix', { lineNo })}${message}`

  const qty = parseFloat(rawQty ?? '')
  if (Number.isNaN(qty)) return withLine(i18n.t('erpDocs.documents.validation.qtyNotNumber'))

  if (docType === 'STK') {
    return qty < 0
      ? withLine(i18n.t('erpDocs.documents.validation.stocktakeQtyNegative'))
      : undefined
  }
  if (docType === 'ADJ') {
    return qty === 0
      ? withLine(i18n.t('erpDocs.documents.validation.adjustQtyZero'))
      : undefined
  }
  return qty <= 0 ? withLine(i18n.t('validation.quantityPositive')) : undefined
}
