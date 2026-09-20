import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

interface CreateLeavePayload {
    leave_type: string
    start_date: string
    end_date: string
    total_hours: number
    total_days: number
    reason?: string
    supporting_documents?: string[]
    proxy_user_id?: string
}

export function useLeaveMutations(options?: { onCreateSuccess?: () => void }) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const invalidateLeaveQueries = (keys: readonly (readonly string[])[]) => {
        for (const key of keys) {
            queryClient.invalidateQueries({ queryKey: [...key] })
        }
        queryClient.invalidateQueries({ queryKey: [...queryKeys.hr.balanceSummary] })
    }

    const createLeaveMutation = useMutation({
        mutationFn: async (data: CreateLeavePayload) => {
            return api.post('/hr/leaves', data)
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.myLeaves])
            options?.onCreateSuccess?.()
            toast({ title: t('common.success'), description: t('hrPages.leaves.toast.created') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('hrPages.shared.toast.createFailed')),
                variant: 'destructive',
            })
        },
    })

    const submitLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/submit`)
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.myLeaves])
            toast({ title: t('common.success'), description: t('hrPages.shared.toast.submittedForReview') })
        },
    })

    const approveLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/approve`, {})
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.pendingLeaves, queryKeys.hr.myLeaves])
            toast({ title: t('common.success'), description: t('hrPages.shared.toast.approved') })
        },
    })

    const rejectLeaveMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/leaves/${id}/reject`, { reason })
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.pendingLeaves])
            toast({ title: t('hrPages.shared.toast.rejectedTitle'), description: t('hrPages.leaves.toast.rejected') })
        },
    })

    const cancelLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/cancel`, {})
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.myLeaves])
            toast({ title: t('common.success'), description: t('hrPages.leaves.toast.cancelled') })
        },
    })

    // 代理人確認：待代理確認 → 進入審核關（單位主管 / 負責人）
    const proxyConfirmLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/proxy-confirm`, {})
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.pendingLeaves, queryKeys.hr.myLeaves])
            toast({ title: t('common.success'), description: t('hrPages.leaves.toast.delegateConfirmed') })
        },
    })

    // 代理人退回：待代理確認 → 退回草稿（申請人重新指定代理人）
    const proxyRejectLeaveMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason?: string }) => {
            return api.post(`/hr/leaves/${id}/proxy-reject`, { reason })
        },
        onSuccess: () => {
            invalidateLeaveQueries([queryKeys.hr.pendingLeaves, queryKeys.hr.myLeaves])
            toast({ title: t('hrPages.leaves.toast.returnedTitle'), description: t('hrPages.leaves.toast.returned') })
        },
    })

    return {
        createLeaveMutation,
        submitLeaveMutation,
        approveLeaveMutation,
        rejectLeaveMutation,
        cancelLeaveMutation,
        proxyConfirmLeaveMutation,
        proxyRejectLeaveMutation,
    }
}
