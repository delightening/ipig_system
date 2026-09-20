import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

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
  const { t } = useTranslation()
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
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(e, t('animalPages.reservation.toast.remarkSaveFailed')),
        variant: 'destructive',
      }),
  })
}

/** 預約 / 解除預約 / 正式分配 mutations（成功後 invalidate 規劃 + 動物清單） */
export function useReservationMutations() {
  const { t } = useTranslation()
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
      toast({
        title: t('animalPages.reservation.status.reserved'),
        description: t('animalPages.reservation.toast.reservedCount', { count: vars.animal_ids.length }),
      })
    },
    onError: (e: unknown) =>
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(e, t('animalPages.reservation.toast.reserveFailed')),
        variant: 'destructive',
      }),
  })

  const unreserve = useMutation({
    mutationFn: (animalIds: string[]) => reservationPlanningApi.unreserve(animalIds),
    onSuccess: () => {
      invalidate()
      toast({
        title: t('animalPages.reservation.toast.unreservedTitle'),
        description: t('animalPages.reservation.toast.unreservedDescription'),
      })
    },
    onError: (e: unknown) =>
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(e, t('animalPages.reservation.toast.unreserveFailed')),
        variant: 'destructive',
      }),
  })

  const assign = useMutation({
    mutationFn: (v: { animalIds: string[]; iacucNo: string }) =>
      reservationPlanningApi.assign(v.animalIds, v.iacucNo),
    onSuccess: () => {
      invalidate()
      toast({
        title: t('animalPages.reservation.toast.assignedTitle'),
        description: t('animalPages.reservation.toast.assignedDescription'),
      })
    },
    onError: (e: unknown) =>
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(e, t('animalPages.reservation.toast.assignFailed')),
        variant: 'destructive',
      }),
  })

  return { reserve, unreserve, assign }
}
