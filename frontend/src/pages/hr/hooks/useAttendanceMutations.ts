import { useMutation, useQueryClient } from '@tanstack/react-query'
import { AxiosError } from 'axios'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { AttendanceWithUser } from '@/types/hr'

/** 取得 GPS 座標（使用者拒絕或不支援時回傳 null） */
function getGpsPosition(): Promise<{ latitude: number; longitude: number } | null> {
    return new Promise((resolve) => {
        if (!navigator.geolocation) {
            resolve(null)
            return
        }
        navigator.geolocation.getCurrentPosition(
            (pos) => resolve({ latitude: pos.coords.latitude, longitude: pos.coords.longitude }),
            () => resolve(null),
            { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 }
        )
    })
}

function handleClockError(error: unknown): string {
    if (error instanceof AxiosError && error.response?.status === 403) {
        return (error.response?.data as { error?: { message?: string } })?.error?.message
            || '請確認您已連接辦公室 WiFi 或允許定位權限'
    }
    return getApiErrorMessage(error, '請稍後再試')
}

interface UseAttendanceMutationsOptions {
    refetchToday: () => void
    canViewAll: boolean
    dateFrom: string
    dateTo: string
    viewAll: boolean
    filterUserId: string
}

export function useAttendanceMutations(opts: UseAttendanceMutationsOptions) {
    const queryClient = useQueryClient()

    const clockInMutation = useMutation({
        mutationFn: async () => {
            const gps = await getGpsPosition()
            return api.post<{ success: boolean; clock_in_time: string }>('/hr/attendance/clock-in', {
                source: 'web',
                ...(gps && { latitude: gps.latitude, longitude: gps.longitude }),
            })
        },
        onSuccess: (res) => {
            const clockInTime = res.data.clock_in_time
            queryClient.setQueryData(queryKeys.hr.todayAttendance, (old: AttendanceWithUser | null) => ({
                ...old,
                clock_in_time: clockInTime,
            } as AttendanceWithUser))
            opts.refetchToday()
            toast({
                title: '打卡成功',
                description: `上班打卡時間：${new Date(clockInTime).toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
            })
        },
        onError: (error: unknown) => {
            toast({ title: '打卡失敗', description: handleClockError(error), variant: 'destructive' })
        },
    })

    const clockOutMutation = useMutation({
        mutationFn: async () => {
            const gps = await getGpsPosition()
            return api.post<{ success: boolean; clock_out_time: string }>('/hr/attendance/clock-out', {
                source: 'web',
                ...(gps && { latitude: gps.latitude, longitude: gps.longitude }),
            })
        },
        onSuccess: (res) => {
            const clockOutTime = res.data.clock_out_time
            queryClient.setQueryData(queryKeys.hr.todayAttendance, (old: AttendanceWithUser | null) => ({
                ...old,
                clock_out_time: clockOutTime,
            } as AttendanceWithUser))
            opts.refetchToday()
            queryClient.invalidateQueries({ queryKey: queryKeys.hr.allAttendanceHistory })
            toast({
                title: '打卡成功',
                description: `下班打卡時間：${new Date(clockOutTime).toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
            })
        },
        onError: (error: unknown) => {
            toast({ title: '打卡失敗', description: handleClockError(error), variant: 'destructive' })
        },
    })

    const exportExcelMutation = useMutation({
        mutationFn: async () => {
            const params = new URLSearchParams()
            if (opts.dateFrom) params.set('from', opts.dateFrom)
            if (opts.dateTo) params.set('to', opts.dateTo)
            if (opts.canViewAll && opts.viewAll) params.set('view_all', 'true')
            if (opts.canViewAll && opts.viewAll && opts.filterUserId) params.set('user_id', opts.filterUserId)
            const res = await api.get(`/hr/attendance/export?${params.toString()}`, { responseType: 'blob' })
            return res.data
        },
        onSuccess: (data) => {
            const blob = new Blob([data], {
                type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
            })
            const url = URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `attendance_records_${new Date().toISOString().slice(0, 10)}.xlsx`
            a.click()
            URL.revokeObjectURL(url)
            toast({ title: '匯出成功', description: '出勤記錄已下載' })
        },
        onError: (error: unknown) => {
            toast({ title: '匯出失敗', description: getApiErrorMessage(error, '請稍後再試'), variant: 'destructive' })
        },
    })

    return { clockInMutation, clockOutMutation, exportExcelMutation }
}
