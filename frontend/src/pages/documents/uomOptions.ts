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

/**
 * 單位欄要不要渲染成唯讀文字（而非下拉）。
 *
 * 「只有一個選項就顯示文字」這個直覺**不夠**：舊單據的 `uom` 可能是本 PR 之前
 * 留下的自由字串（例如 base_uom 是「雙」而該行寫「打」）。那種行的選項只有
 * 一個（base_uom），若照選項數就渲染成文字，畫面會顯示那個**無效的**「打」、
 * 使用者卻改不動它——而後端 `assert_lines_uom_defined` 更新時必定回 400，
 * 等於把人卡死在一張永遠存不了的單上，沒有任何補救途徑。
 *
 * 所以判準是「唯一的選項就是它現在的值」，不是「只有一個選項」：
 * - 無選項（新增行 `uom` 還是空的）→ 文字，維持既有版面。
 * - 一個選項且等於現值 → 文字，一個選項的下拉只是噪音。
 * - 一個選項但不等於現值 → **下拉**，讓使用者把無效單位改回合法值。
 * - 多個選項 → 下拉。
 */
export function isUomReadOnly(line: UomOptionSource): boolean {
  const options = buildUomOptions(line)
  return options.length === 0 || (options.length === 1 && options[0] === line.uom)
}

/**
 * 餵給 Radix `<Select value>` 的值。現值不在選項內（舊資料的無效單位）時回空字串。
 *
 * 為什麼不能直接把 `line.uom` 丟進去：Radix 的
 * `shouldShowPlaceholder(value) { return value === "" || value === void 0 }`
 * ——**只有空值才顯示 placeholder**；而 trigger 上的文字是由「被選中的那個
 * `SelectItem`」透過 portal 注入的。給一個不在選項內的非空值（例如舊單據的
 * 「打」），placeholder 不顯示、也沒有 Item 會注入文字，trigger 就是**一片空白**：
 * 使用者看不出這行原本是什麼、也看不出為什麼要他重選。
 *
 * 回空字串則兩者都成立：placeholder 顯示得出原值，選單照樣能選。
 */
export function selectedUomValue(line: UomOptionSource): string {
  return buildUomOptions(line).includes(line.uom) ? line.uom : ''
}
