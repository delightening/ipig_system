import type { QueryKey } from '@tanstack/react-query'

/**
 * 單據核准會套用庫存異動（ADJ 盤點差異調整、GRN 入庫、SO 出庫等）。
 * 下列 query 顯示「倉庫現況」，核准後若不失效，使用者需手動重新整理才看得到新庫存。
 * 以 key 前綴失效，可一併涵蓋帶 filter 的子 key（如 ['inventory', locationFilter, ...]）。
 */
export const STOCK_QUERY_KEY_PREFIXES: readonly QueryKey[] = [
  ['inventory'], // 庫存查詢頁 + 批號明細 / 未分配 / 在庫
  ['stock-ledger'], // 庫存異動流水
  ['stock-balances'], // 品項揀貨對話框的儲位庫存
  ['storage-location-inventory'], // 倉庫貨架配置的貨架庫存
  ['warehouse-report'], // 倉庫庫存報表
  ['low-stock-alerts'], // 儀表板低庫存警示
]

/**
 * 單據狀態變更（送審 / 核准 / 駁回 / 作廢）後應失效的 query keys。
 *
 * - 一律失效單據清單 (`['documents']`)；`id` 有值時一併失效該單據詳情 (`['document', id]`)。
 *   `id` 為 undefined 時不放入 `['document', undefined]`（避免無意義的失效 key）。
 * - `affectsStock` 為 true（核准會套用庫存）時，額外失效庫存相關查詢，
 *   讓庫存頁面自動更新、毋須手動重新整理。
 */
export function documentChangeQueryKeys(
  id: string | undefined,
  { affectsStock }: { affectsStock: boolean },
): QueryKey[] {
  const keys: QueryKey[] = []
  if (id) keys.push(['document', id])
  keys.push(['documents'])
  if (affectsStock) keys.push(...STOCK_QUERY_KEY_PREFIXES)
  return keys
}
