import { useEffect } from 'react'
import { BarChart3, Calendar, Clock } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { useAuthHasPermission } from '@/stores/auth'
import { DEMO_ATTENDANCE } from '@/lib/guest-demo'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import type { AttendanceWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { TodayClockTab } from './components/TodayClockTab'
import { AttendanceHistoryTab } from './components/AttendanceHistoryTab'
import { MonthlyReportTab } from './components/MonthlyReportTab'
import { useAttendanceMutations, prewarmGpsPosition } from './hooks/useAttendanceMutations'

export function HrAttendancePage() {
    // 進頁面即預熱定位，使用者按打卡時可重用快取、近乎瞬間送出（打卡延遲修復）
    useEffect(() => {
        prewarmGpsPosition()
    }, [])

    // 與後端 get_monthly_report/export_monthly_report 的權限門檻對齊
    // （CodeRabbit PR #35 指出）：沒有 hr.attendance.view 的角色（PI/VET/CLIENT 等，
    // 見 backend/src/startup/permissions.rs）打這兩支端點會拿到 403，但分頁一直都
    // 看得到、選得到——只隱藏分頁按鈕不夠，`?tab=monthly` 深連結仍可能讓內容掛載
    // （PageTabs 的 `hidden` 只濾掉觸發按鈕，`activeTab` 仍可能解析成該值），
    // 所以分頁清單與內容渲染都要各自擋一次。跟 HrLeavePage.tsx 的既有作法一致。
    const hasPermission = useAuthHasPermission()
    const canViewMonthly = hasPermission('hr.attendance.view')

    // 今日打卡狀態
    const { data: todayAttendance, refetch: refetchToday } = useGuestQuery(
        DEMO_ATTENDANCE.data[0] ?? null,
        {
            queryKey: queryKeys.hr.todayAttendance,
            queryFn: async () => {
                const today = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })
                const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                    `/hr/attendance?from=${today}&to=${today}`
                )
                return res.data.data[0] || null
            },
        },
    )

    const { clockInMutation, clockOutMutation } = useAttendanceMutations({
        refetchToday,
        canViewAll: false,
        dateFrom: '',
        dateTo: '',
        viewAll: false,
        filterUserId: '',
    })

    return (
        <div className="space-y-6">
            <PageHeader title="出勤管理" description="打卡、補卡與工時月報" />

            <PageTabs
                tabs={[
                    { value: 'today', label: '今日打卡', icon: Clock },
                    { value: 'history', label: '出勤記錄', icon: Calendar },
                    { value: 'monthly', label: '工時月報', icon: BarChart3, hidden: !canViewMonthly },
                ]}
                defaultTab="today"
            >
                <PageTabContent value="today" className="space-y-4">
                    {/* R49 follow-up：guest 看完整頁面 + 按鈕（disabled），TodayClockTab 內讀 isGuest 鎖按鈕 */}
                    <TodayClockTab
                        todayAttendance={todayAttendance}
                        clockInPending={clockInMutation.isPending}
                        clockOutPending={clockOutMutation.isPending}
                        onClockIn={() => clockInMutation.mutate()}
                        onClockOut={() => clockOutMutation.mutate()}
                    />
                </PageTabContent>

                <PageTabContent value="history" className="space-y-4">
                    <AttendanceHistoryTab />
                </PageTabContent>

                {canViewMonthly && (
                    <PageTabContent value="monthly" className="space-y-4">
                        <MonthlyReportTab />
                    </PageTabContent>
                )}
            </PageTabs>
        </div>
    )
}

export default HrAttendancePage
