/**
 * 閒置管理分頁的完整資料層：分頁 state、申請清單查詢、申請/核准 mutations。
 */
import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { PaginatedResponse } from '@/types/common'

import type { IdleRequestWithDetails } from '../components/IdleTabContent'

const IDLE_KEYS = {
  list: ['equipment-idle-requests'],
} as const

const EQUIP_KEYS = {
  list: ['equipment'],
  all: ['equipment-all'],
} as const

export function useEquipmentIdle() {
  const queryClient = useQueryClient()

  const [idlePage, setIdlePage] = useState(1)

  const { data: idleData, isLoading: idleLoading } = useQuery({
    queryKey: ['equipment-idle-requests', idlePage],
    queryFn: async () =>
      (
        await api.get<PaginatedResponse<IdleRequestWithDetails>>('/equipment-idle-requests', {
          params: { page: idlePage, per_page: 20 },
        })
      ).data,
  })

  const createIdleMutation = useMutation({
    mutationFn: (data: { equipment_id: string; request_type: string; reason: string }) =>
      api.post('/equipment-idle-requests', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: IDLE_KEYS.list })
      toast({ title: '成功', description: '已提交閒置申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '申請失敗'), variant: 'destructive' })
    },
  })

  // 核准/駁回會改動設備狀態，故一併失效設備列表（提交申請本身則只影響 IDLE_KEYS）
  const approveIdleMutation = useMutation({
    mutationFn: ({ id, approved, reason }: { id: string; approved: boolean; reason?: string }) =>
      api.post(`/equipment-idle-requests/${id}/approve`, {
        approved,
        // R71-10：駁回原因由審核者填寫（取代原固定『駁回』）；留空時退回沿用『駁回』。
        rejection_reason: approved ? null : (reason?.trim() || '駁回'),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: IDLE_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
      queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
      toast({ title: '成功', description: '已處理閒置申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '操作失敗'), variant: 'destructive' })
    },
  })

  return {
    idleData,
    idleLoading,
    idlePage,
    setIdlePage,
    createIdleMutation,
    approveIdleMutation,
  }
}
