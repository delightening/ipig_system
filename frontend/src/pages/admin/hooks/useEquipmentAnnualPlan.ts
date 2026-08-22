/**
 * 年度計畫分頁的完整資料層：年度 state、計畫/執行摘要查詢、產生與 CRUD mutations。
 */
import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_ANNUAL_PLANS } from '@/lib/guest-demo'

import type { AnnualPlanExecutionSummary, AnnualPlanWithEquipment } from '../types'

const PLAN_KEYS = {
  list: ['equipment-annual-plans'],
  summary: ['equipment-annual-plans-summary'],
} as const

export function useEquipmentAnnualPlan() {
  const queryClient = useQueryClient()

  const [planYear, setPlanYear] = useState(new Date().getFullYear())

  const { data: plans = [] } = useGuestQuery(
    DEMO_ANNUAL_PLANS as unknown as AnnualPlanWithEquipment[],
    {
      queryKey: ['equipment-annual-plans', planYear],
      queryFn: async () =>
        (
          await api.get<AnnualPlanWithEquipment[]>('/equipment-annual-plans', {
            params: { year: planYear },
          })
        ).data,
    },
  )

  const { data: executionSummary = null } = useGuestQuery<AnnualPlanExecutionSummary | null>(
    null,
    {
      queryKey: ['equipment-annual-plans-summary', planYear],
      queryFn: async () =>
        (
          await api.get<AnnualPlanExecutionSummary>('/equipment-annual-plans/execution-summary', {
            params: { year: planYear },
          })
        ).data,
    },
  )

  const invalidatePlans = () => {
    queryClient.invalidateQueries({ queryKey: PLAN_KEYS.list })
    queryClient.invalidateQueries({ queryKey: PLAN_KEYS.summary })
  }

  // year 走 mutation 變數而非閉包：使用者若在請求進行中切換年度，
  // 閉包版的成功訊息會報出切換後的年度，與實際產生的那一年不符。
  const generatePlanMutation = useMutation({
    mutationFn: (year: number) => api.post('/equipment-annual-plans/generate', { year }),
    onSuccess: (_data, year) => {
      invalidatePlans()
      toast({ title: '成功', description: `已產生 ${year} 年度計畫` })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '產生失敗'), variant: 'destructive' })
    },
  })

  const createPlanMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => api.post('/equipment-annual-plans', data),
    onSuccess: () => {
      invalidatePlans()
      toast({ title: '成功', description: '已新增年度計畫項目' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  // 逐月切換會高頻觸發，故成功時刻意不跳 toast（僅失敗時提示）
  const updatePlanMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Record<string, unknown> }) =>
      api.put(`/equipment-annual-plans/${id}`, data),
    onSuccess: () => {
      invalidatePlans()
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deletePlanMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-annual-plans/${id}`),
    onSuccess: () => {
      invalidatePlans()
      toast({ title: '成功', description: '已刪除年度計畫項目' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  return {
    plans,
    executionSummary,
    planYear,
    setPlanYear,
    generatePlanMutation,
    createPlanMutation,
    updatePlanMutation,
    deletePlanMutation,
  }
}
