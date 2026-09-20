/**
 * 年度計畫分頁的完整資料層：年度 state、計畫/執行摘要查詢、產生與 CRUD mutations。
 */
import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.planGenerated', { year }) })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.generateFailed')), variant: 'destructive' })
    },
  })

  const createPlanMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => api.post('/equipment-annual-plans', data),
    onSuccess: () => {
      invalidatePlans()
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.planItemCreated') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.createFailed')), variant: 'destructive' })
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
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.updateFailed')), variant: 'destructive' })
    },
  })

  const deletePlanMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-annual-plans/${id}`),
    onSuccess: () => {
      invalidatePlans()
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.planItemDeleted') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.deleteFailed')), variant: 'destructive' })
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
