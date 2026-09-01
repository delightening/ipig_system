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
 * 只有 STK **建立**時後端才會讀它（`generate_stocktake_lines` 產底稿、INSERT 寫欄位）；
 * 其餘情況帶了也不會被讀，卻會在單據上留下一個事後容易被誤讀成「這張單當初盤了哪些
 * 類別」的欄位。因此非 STK 一律回 null。
 *
 * `isEdit` 同理（CodeRabbit 於 PR #37 指出，Minor）：編輯走 PUT，後端**沒有任何
 * UPDATE 路徑**會寫 `stocktake_scope`，所以送過去只是被忽略——但表單在編輯時帶的是
 * 預設值 `{full, []}`，不是這張單真正的範圍，送出它等於在 payload 裡放一個看起來像
 * 事實、其實是預設值的東西。不送最誠實。
 */
export function scopeForPayload(
  docType: DocType,
  scope: StocktakeScope | undefined,
  isEdit = false,
): StocktakeScope | null {
  if (docType !== 'STK' || isEdit || !scope) return null
  return scope
}

/**
 * 品類清單沒成功載入時，擋下建單並說明原因；可以建單時回 `undefined`。
 *
 * 為什麼要擋，而不是讓他送出去：`useSkuCategories` 在「載入中」與「載入失敗」兩種
 * 情況都回空陣列，而空清單在畫面上跟「這個系統沒有任何品類」完全無法區分。使用者
 * 會以為沒得篩選而直接建單，後端收到空的 `category_codes` 即視為不限範圍——想要的
 * 部分盤點就這樣變成全盤，而且**沒有任何跡象**，盤完才會發現。
 *
 * 這與後端那側「解析失敗不可 `.ok()` 靜默降級」是同一條原則，只是發生在前端。
 * （CodeRabbit 於 PR #37 指出，Major。）
 */
export function stocktakeBlockReason(state: {
  /** 只有「建立盤點單」需要品類清單；改單不重產底稿，其餘單別根本不用 */
  needed: boolean
  loading: boolean
  error: boolean
}): string | undefined {
  if (!state.needed) return undefined
  if (state.loading) return '盤點品類清單載入中，請稍候再建立盤點單。'
  if (state.error) return '盤點品類清單載入失敗，現在建單會盤到全部品項。請重試後再建立。'
  return undefined
}
