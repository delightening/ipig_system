import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import {
  reservationPlanningApi,
  type ReservableQuery,
  type ReserveBody,
} from '@/lib/api/reservationPlanning'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'

const PLANNING_KEY = ['reservation-planning'] as const
const RESERVABLE_KEY = ['reservable'] as const

/** 規劃分組查詢 */
export function useReservationPlanning() {
  return useQuery({
    queryKey: PLANNING_KEY,
    queryFn: () => reservationPlanningApi.getPlanning().then((r) => r.data),
    staleTime: 30_000,
  })
}

/** 備用池：搜尋未分配未預約動物（含條件篩選）。 */
export function useReservable(query: ReservableQuery) {
  return useQuery({
    queryKey: [...RESERVABLE_KEY, query],
    queryFn: () => reservationPlanningApi.searchReservable(query).then((r) => r.data),
    staleTime: 30_000,
  })
}

/** 備註 inline 編輯 mutation（Phase 5）。成功後刷新規劃 + 動物清單；失敗 toast（呼叫端負責還原）。 */
export function useUpdateRemark() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { animalId: string; remark: string }) =>
      reservationPlanningApi.updateRemark(v.animalId, v.remark),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: PLANNING_KEY })
      qc.invalidateQueries({ queryKey: RESERVABLE_KEY })
      qc.invalidateQueries({ queryKey: ['animals'] })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '備註儲存失敗'), variant: 'destructive' }),
  })
}

/** 預約 / 解除預約 / 正式分配 mutations（成功後 invalidate 規劃 + 動物清單） */
export function useReservationMutations() {
  const qc = useQueryClient()
  const invalidate = () => {
    qc.invalidateQueries({ queryKey: PLANNING_KEY })
    qc.invalidateQueries({ queryKey: RESERVABLE_KEY })
    qc.invalidateQueries({ queryKey: ['animals'] })
    qc.invalidateQueries({ queryKey: ['animals-stats'] })
  }

  const reserve = useMutation({
    mutationFn: (body: ReserveBody) => reservationPlanningApi.reserve(body),
    onSuccess: (_d, vars) => {
      invalidate()
      toast({ title: '已預約', description: `已預約 ${vars.animal_ids.length} 隻動物` })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '預約失敗'), variant: 'destructive' }),
  })

  const unreserve = useMutation({
    mutationFn: (animalIds: string[]) => reservationPlanningApi.unreserve(animalIds),
    onSuccess: () => {
      invalidate()
      toast({ title: '已解除', description: '已解除預約' })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '解除預約失敗'), variant: 'destructive' }),
  })

  const assign = useMutation({
    mutationFn: (v: { animalIds: string[]; iacucNo: string }) =>
      reservationPlanningApi.assign(v.animalIds, v.iacucNo),
    onSuccess: () => {
      invalidate()
      toast({ title: '已分配', description: '動物已正式分配進實驗中' })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '正式分配失敗'), variant: 'destructive' }),
  })

  return { reserve, unreserve, assign }
}
