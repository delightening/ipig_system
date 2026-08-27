/**
 * 盤點範圍（STK）的純邏輯，抽出 JSX 之外以便單獨測試。
 *
 * 後端對應：`StocktakeScope { scope_type, category_codes, warehouse_ids, product_ids }`
 * 與 `DocumentService::generate_stocktake_lines`。
 */
import type { DocType } from '@/lib/api'
import type { StocktakeScope } from './types'

/**
 * 由使用者勾選的品類代碼組出範圍設定。
 *
 * 一個都沒選＝全盤（`scope_type: 'full'`），與本功能存在之前的行為相同；
 * 有選才是循環盤點（`partial`）。`scope_type` 是後端必填欄位，缺了會被
 * `generate_stocktake_lines` 判為格式錯誤而回 400，所以兩條路徑都要給值。
 */
export function buildStocktakeScope(categoryCodes: string[]): StocktakeScope {
  return {
    scope_type: categoryCodes.length > 0 ? 'partial' : 'full',
    category_codes: categoryCodes,
  }
}

/**
 * 決定送出時 payload 要不要帶盤點範圍。
 *
 * 只有 STK **建立**時後端才會呼叫 `generate_stocktake_lines`；其餘單據型別帶了
 * 也不會被讀，卻會在單據上留下一個事後容易被誤讀成「這張單當初盤了哪些類別」
 * 的欄位。因此非 STK 一律回 null。
 */
export function scopeForPayload(
  docType: DocType,
  scope: StocktakeScope | undefined,
): StocktakeScope | null {
  if (docType !== 'STK' || !scope) return null
  return scope
}
