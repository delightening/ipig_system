import { useMutation, useQueryClient } from '@tanstack/react-query'
import { AxiosError } from 'axios'

import api from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { queryKeys } from '@/lib/queryKeys'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { AttendanceWithUser } from '@/types/hr'

// 打卡定位選項。先前為了「打卡延遲」改用 enableHighAccuracy:false + maximumAge:60000，
// 但對辦公室 200m 半徑而言，網路（WiFi/基地台）定位精度常達數百公尺～數公里，行動網路
// 使用者（無辦公室 IP，只能靠 GPS）第一次打卡常拿到「範圍外」的粗略定位而失敗，且
// maximumAge:60000 會把這個錯誤定位鎖住重用 60 秒 → 使用者必須連點/等待才會成功
// （即「打卡要打兩次」的成因）。改為：
//   enableHighAccuracy:true → 取真實 GPS 精度，落在 200m 半徑內才可靠
//   maximumAge:0 → 每次打卡都取「當下」定位，不重用可能過期/範圍外的舊值（修「第一次必失敗」）
//   timeout 10s → 給 GPS 鎖定時間；拿不到時回 null，辦公室網段使用者仍由 IP 白名單通過
// 桌機/筆電無 GPS 晶片時靠 WiFi 定位（多在 1～3s 回來，最差 timeout 後 fallback IP），
// 取捨：以一天一兩次的打卡換取「不再需要點兩次」的正確性。prewarmGpsPosition 仍保留以
// 預熱 GPS 模組、降低首次取得延遲。
const GEO_OPTIONS: PositionOptions = {
    enableHighAccuracy: true,
    timeout: 10000,
    maximumAge: 0,
}

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
            GEO_OPTIONS
        )
    })
}

/**
 * 頁面載入時預熱定位：先觸發一次定位以填入瀏覽器位置快取（fire-and-forget），
 * 使用者實際按下打卡時 getGpsPosition 可透過 GEO_OPTIONS.maximumAge 重用該快取，
 * 省去當下取定位的等待，打卡幾乎瞬間送出。
 */
export function prewarmGpsPosition(): void {
    if (!navigator.geolocation) return
    navigator.geolocation.getCurrentPosition(
        () => {},
        () => {},
        GEO_OPTIONS
    )
}

function handleClockError(error: unknown): string {
    // 地理圍籬失敗後端回 422（BusinessRule）；保留 403 以相容部署過渡期的舊後端。
    if (error instanceof AxiosError && (error.response?.status === 422 || error.response?.status === 403)) {
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
                description: `上班打卡時間：${new Date(clockInTime).toLocaleTimeString(uiLocale(), { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
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
                description: `下班打卡時間：${new Date(clockOutTime).toLocaleTimeString(uiLocale(), { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
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
