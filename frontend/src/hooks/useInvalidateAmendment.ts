import { useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { queryKeys } from '@/lib/queryKeys'

/**
 * 集中失效單一變更申請的三個快取（詳情 / 委員指派 / 狀態歷程）。
 * 投票、分類、狀態變更、開始審查等 mutation 成功後皆需一次失效這三個，故抽為共用 hook（DRY）。
 */
export function useInvalidateAmendment(amendmentId: string) {
  const queryClient = useQueryClient()
  return useCallback(() => {
    queryClient.invalidateQueries({ queryKey: queryKeys.amendments.detail(amendmentId) })
    queryClient.invalidateQueries({ queryKey: queryKeys.amendments.assignments(amendmentId) })
    queryClient.invalidateQueries({ queryKey: queryKeys.amendments.history(amendmentId) })
  }, [queryClient, amendmentId])
}
