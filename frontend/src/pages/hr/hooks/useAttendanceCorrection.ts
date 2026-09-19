import { useMutation, useQueryClient } from '@tanstack/react-query'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { AttendanceBackfillRequest, AttendanceCorrectionRequest } from '@/types/hr'

/**
 * 補卡：補登缺漏日（POST）與更正既有紀錄（PUT）。
 *
 * 兩條路徑分開的原因在後端：整天沒打卡的日子資料庫沒有 row，
 * `PUT /hr/attendance/{id}` 會回 404，補不了——那才是補卡最常見的情境。
 *
 * 「不得補自己的卡」由後端判定（403），前端不重複實作那條規則，
 * 只把錯誤訊息原樣呈現；兩邊各判一次遲早會分歧。
 */
export function useAttendanceCorrection() {
    const queryClient = useQueryClient()

    const invalidateAttendance = () => {
        queryClient.invalidateQueries({ queryKey: queryKeys.hr.allAttendanceHistory })
        queryClient.invalidateQueries({ queryKey: queryKeys.hr.allMonthlyReport })
        queryClient.invalidateQueries({ queryKey: queryKeys.hr.todayAttendance })
    }

    const backfillMutation = useMutation({
        mutationFn: async (payload: AttendanceBackfillRequest) => {
            const res = await api.post<{ success: boolean; id: string }>('/hr/attendance', payload)
            return res.data
        },
        onSuccess: () => {
            invalidateAttendance()
            toast({ title: '補卡成功', description: '已補登出勤記錄' })
        },
        onError: (error: unknown) => {
            toast({
                title: '補卡失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    const correctMutation = useMutation({
        mutationFn: async ({ id, ...payload }: AttendanceCorrectionRequest & { id: string }) => {
            const res = await api.put<{ success: boolean }>(`/hr/attendance/${id}`, payload)
            return res.data
        },
        onSuccess: () => {
            invalidateAttendance()
            toast({ title: '更正成功', description: '已更正出勤記錄' })
        },
        onError: (error: unknown) => {
            toast({
                title: '更正失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    return { backfillMutation, correctMutation }
}
