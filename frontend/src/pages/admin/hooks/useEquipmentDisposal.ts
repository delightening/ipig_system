/**
 * 報廢分頁的完整資料層：分頁 state、申請清單查詢、申請/核准/恢復 mutations。
 */
import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { PaginatedResponse } from '@/types/common'

import type { DisposalWithDetails } from '../types'
import { emptyDisposalForm, type DisposalFormData } from '../components/DisposalFormDialog'

const DISPOSAL_KEYS = {
  list: ['equipment-disposals'],
} as const

const EQUIP_KEYS = {
  list: ['equipment'],
  all: ['equipment-all'],
} as const

interface UseEquipmentDisposalOptions {
  closeDisposalCreate: () => void
}

export function useEquipmentDisposal(options: UseEquipmentDisposalOptions) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const [disposalPage, setDisposalPage] = useState(1)
  const [disposalForm, setDisposalForm] = useState<DisposalFormData>(emptyDisposalForm())

  const { data: disposalData, isLoading: disposalLoading } = useQuery({
    queryKey: ['equipment-disposals', disposalPage],
    queryFn: async () =>
      (
        await api.get<PaginatedResponse<DisposalWithDetails>>('/equipment-disposals', {
          params: { page: disposalPage, per_page: 20 },
        })
      ).data,
  })

  const approveDisposalMutation = useMutation({
    mutationFn: ({ id, approved }: { id: string; approved: boolean }) =>
      api.post(`/equipment-disposals/${id}/approve`, {
        approved,
        rejection_reason: approved ? null : '駁回',
      }),
    // 核准報廢會把設備轉為 decommissioned（backend `services/equipment/disposal.rs`），
    // 故必須連分頁列表 key 一起失效，否則設備分頁會停在舊狀態。
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DISPOSAL_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.disposalProcessed') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.operationFailed')), variant: 'destructive' })
    },
  })

  // 恢復同樣會改設備狀態（decommissioned → active），失效範圍與 approve 一致。
  const restoreEquipmentMutation = useMutation({
    mutationFn: (disposalId: string) =>
      api.post(`/equipment-disposals/${disposalId}/restore`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DISPOSAL_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.equipmentRestored') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.restoreFailed')), variant: 'destructive' })
    },
  })

  const createDisposalMutation = useMutation({
    mutationFn: (data: DisposalFormData) =>
      api.post('/equipment-disposals', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DISPOSAL_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
      options.closeDisposalCreate()
      setDisposalForm(emptyDisposalForm())
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.disposalSubmitted') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.requestFailed')), variant: 'destructive' })
    },
  })

  return {
    disposalData,
    disposalLoading,
    disposalPage,
    setDisposalPage,
    disposalForm,
    setDisposalForm,
    approveDisposalMutation,
    restoreEquipmentMutation,
    createDisposalMutation,
  }
}
