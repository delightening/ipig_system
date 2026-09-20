import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { deleteResource } from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_OVERTIME } from '@/lib/guest-demo'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import type { CreateOvertimeData } from '../constants'

/** Hook for fetching my overtime records */
export const useMyOvertime = () => {
    return useGuestQuery(DEMO_OVERTIME, {
        queryKey: queryKeys.hr.myOvertime,
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>('/hr/overtime')
            return res.data
        },
    })
}

/** Hook for fetching pending overtime approvals */
export const usePendingOvertime = () => {
    return useGuestQuery(DEMO_OVERTIME, {
        queryKey: queryKeys.hr.pendingOvertime,
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>(
                '/hr/overtime?pending_approval=true'
            )
            return res.data
        },
    })
}

/** Hook for overtime mutations (CRUD operations) */
export const useOvertimeMutations = () => {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const createOvertime = useMutation({
        mutationFn: async (data: CreateOvertimeData) => {
            return api.post('/hr/overtime', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: t('common.success'), description: t('hrPages.overtime.toast.created') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('hrPages.shared.toast.createFailed')),
                variant: 'destructive',
            })
        },
    })

    const submitOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/submit`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: t('common.success'), description: t('hrPages.shared.toast.submittedForReview') })
        },
    })

    const approveOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/approve`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.pendingOvertime })
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            // 全部加班紀錄分頁用 ['hr-all-overtime', filters]；缺這個 → 該分頁狀態不刷新。
            queryClient.invalidateQueries({ queryKey: ['hr-all-overtime'] })
            toast({ title: t('common.success'), description: t('hrPages.shared.toast.approved') })
        },
    })

    const rejectOvertime = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/overtime/${id}/reject`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.pendingOvertime })
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            queryClient.invalidateQueries({ queryKey: ['hr-all-overtime'] })
            toast({ title: t('hrPages.shared.toast.rejectedTitle'), description: t('hrPages.overtime.toast.rejected') })
        },
    })

    // R86-2：作廢已核准的加班單（負責人專用，理由必填）。
    const voidOvertime = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/overtime/${id}/void`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            queryClient.invalidateQueries({ queryKey: ['hr-all-overtime'] })
            // 作廢會收回 comp_time_balances，餘額摘要（含「即將到期」）必須跟著重抓，
            // 否則畫面上的補休時數還停在作廢前的數字。
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.balanceSummary })
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.balanceSummaryExpiring })
            toast({ title: t('hrPages.overtime.toast.voidedTitle'), description: t('hrPages.overtime.toast.voided') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('hrPages.overtime.toast.voidFailed')),
                variant: 'destructive',
            })
        },
    })

    const deleteOvertime = useMutation({
        mutationFn: async (id: string) => {
            return deleteResource(`/hr/overtime/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: t('common.success'), description: t('hrPages.overtime.toast.deleted') })
        },
    })

    return {
        createOvertime,
        submitOvertime,
        approveOvertime,
        rejectOvertime,
        voidOvertime,
        deleteOvertime,
    }
}
