import { useQuery } from '@tanstack/react-query'

import api, { ProtocolListItem } from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'

/**
 * 可指派計畫（動物 IACUC No. 下拉用）的單一來源。
 *
 * 後端 `GET /protocols?assignable=true` 已將清單過濾為「已核准
 * （APPROVED / APPROVED_WITH_CONDITIONS）且具 iacuc_no」的計畫，故前端不再逐處
 * 抓全部計畫再 client-side filter（效能 P1：減 over-fetch + DRY，原 5 處重複）。
 * 可見授權範圍由後端 view_all / 我的計劃兩條路徑各自界定，與舊行為一致。
 *
 * `sdOnly`（SO 銷貨計畫下拉用，2026-07-22 裁定）：帶 `sd_only=true` 讓後端只回
 * 「自己是該計畫 SD」的計畫（admin 後端豁免不過濾），
 * 避免列出選了也會被 `authorize_sales_document` 擋下的計畫。
 *
 * 已核准計畫變動少，沿用 10 分鐘 staleTime（多數呼叫端原本即為此值）。
 */
export function useAssignableProtocols(options?: { enabled?: boolean; sdOnly?: boolean }) {
  const sdOnly = options?.sdOnly ?? false
  return useQuery({
    queryKey: [...queryKeys.protocols.approvedList, { sdOnly }],
    queryFn: async () => {
      const res = await api.get<ProtocolListItem[]>(
        `/protocols?assignable=true${sdOnly ? '&sd_only=true' : ''}`,
      )
      return res.data
    },
    staleTime: 600_000,
    enabled: options?.enabled ?? true,
  })
}
