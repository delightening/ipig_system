/**
 * 單據明細「單位」下拉的選項邏輯，抽出 JSX 之外以便單獨測試。
 *
 * 後端對應：`products.base_uom` + `product_uom_conversions`（經 `alt_uoms` 帶到前端），
 * 驗證端為 `DocumentService::assert_lines_uom_defined`——**兩邊必須是同一組值**，
 * 下拉列得出來、後端就收；列不出來的填了會被擋成 400。
 */
import type { DocumentLine } from './types'

/** `buildUomOptions` 只需要這三個欄位，收窄型別讓測試不必造整個 DocumentLine。 */
export type UomOptionSource = Pick<DocumentLine, 'base_uom' | 'uom' | 'alt_uoms'>

/**
 * 組出該明細行可選的單位清單。
 *
 * - 第一個永遠是 base_uom。`base_uom` 缺席時退回該行當前的 `uom`——舊單據載入
 *   或品項尚未帶出換算表時，至少要能顯示自己現在是什麼單位，而不是空白。
 * - 其餘為換算表定義的單位（後端已依 factor_to_base 遞增排序，此處不重排）。
 * - 去重：換算表理論上不存基本單位，但舊資料若手動塞過一筆同名的換算列，
 *   重複的 key 會讓 Radix Select 出現重複節點。
 * - 濾掉空字串：`uom` 在新增行的初始狀態是 `''`，不該變成一個空白選項。
 */
export function buildUomOptions(line: UomOptionSource): string[] {
  const base = line.base_uom || line.uom
  return Array.from(new Set([base, ...(line.alt_uoms ?? [])].filter(Boolean)))
}
