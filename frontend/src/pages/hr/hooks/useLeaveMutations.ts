import { useMutation, useQueryClient } from '@tanstack/react-query'

import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

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
    const queryClient = useQueryClient()

    const invalidateAll = (keys: string[]) => {
        for (const key of keys) {
            queryClient.invalidateQueries({ queryKey: [key] })
        }
        queryClient.invalidateQueries({ queryKey: ['hr-balance-summary'] })
    }

    const createLeaveMutation = useMutation({
        mutationFn: async (data: CreateLeavePayload) => {
            return api.post('/hr/leaves', data)
        },
        onSuccess: () => {
            invalidateAll(['hr-my-leaves'])
            options?.onCreateSuccess?.()
            toast({ title: '成功', description: '已建立請假申請' })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '建立失敗'),
                variant: 'destructive',
            })
        },
    })

    const submitLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/submit`)
        },
        onSuccess: () => {
            invalidateAll(['hr-my-leaves'])
            toast({ title: '成功', description: '已送出審核' })
        },
    })

    const approveLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/approve`, {})
        },
        onSuccess: () => {
            invalidateAll(['hr-pending-leaves', 'hr-my-leaves'])
            toast({ title: '成功', description: '已核准' })
        },
    })

    const rejectLeaveMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/leaves/${id}/reject`, { reason })
        },
        onSuccess: () => {
            invalidateAll(['hr-pending-leaves'])
            toast({ title: '已駁回', description: '請假已被駁回' })
        },
    })

    const cancelLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/cancel`, {})
        },
        onSuccess: () => {
            invalidateAll(['hr-my-leaves'])
            toast({ title: '成功', description: '已取消請假' })
        },
    })

    return {
        createLeaveMutation,
        submitLeaveMutation,
        approveLeaveMutation,
        rejectLeaveMutation,
        cancelLeaveMutation,
    }
}
