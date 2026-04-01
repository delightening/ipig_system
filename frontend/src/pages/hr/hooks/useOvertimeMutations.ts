import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, { deleteResource } from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_OVERTIME } from '@/lib/guest-demo'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
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
    const queryClient = useQueryClient()

    const createOvertime = useMutation({
        mutationFn: async (data: CreateOvertimeData) => {
            return api.post('/hr/overtime', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: '成功', description: '已建立加班申請' })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '建立失敗'),
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
            toast({ title: '成功', description: '已送出審核' })
        },
    })

    const approveOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/approve`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.pendingOvertime })
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: '成功', description: '已核准' })
        },
    })

    const rejectOvertime = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/overtime/${id}/reject`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.pendingOvertime })
            toast({ title: '已駁回', description: '加班已被駁回' })
        },
    })

    const deleteOvertime = useMutation({
        mutationFn: async (id: string) => {
            return deleteResource(`/hr/overtime/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.myOvertime })
            toast({ title: '成功', description: '已刪除加班申請' })
        },
    })

    return {
        createOvertime,
        submitOvertime,
        approveOvertime,
        rejectOvertime,
        deleteOvertime,
    }
}
