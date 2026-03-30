import { useQuery } from '@tanstack/react-query'
import { Calendar, Clock } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import type { AttendanceWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { TodayClockTab } from './components/TodayClockTab'
import { AttendanceHistoryTab } from './components/AttendanceHistoryTab'
import { useAttendanceMutations } from './hooks/useAttendanceMutations'

export function HrAttendancePage() {
    // 今日打卡狀態
    const { data: todayAttendance, refetch: refetchToday } = useQuery({
        queryKey: queryKeys.hr.todayAttendance,
        queryFn: async () => {
            const today = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                `/hr/attendance?from=${today}&to=${today}`
            )
            return res.data.data[0] || null
        },
    })

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
            <PageHeader title="出勤管理" description="打卡與出勤記錄" />

            <PageTabs
                tabs={[
                    { value: 'today', label: '今日打卡', icon: Clock },
                    { value: 'history', label: '出勤記錄', icon: Calendar },
                ]}
                defaultTab="today"
            >
                <PageTabContent value="today" className="space-y-4">
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
            </PageTabs>
        </div>
    )
}

export default HrAttendancePage
