import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { UserCheck, Clock, Loader2, UserX } from 'lucide-react'
import api from '@/lib/api'
import type { PaginatedResponse } from '@/types/common'

interface AttendanceWithUser {
    user_name: string
    clock_in_time: string | null
    clock_out_time: string | null
    status: string
}

export function StaffAttendanceWidget() {
    const { t, i18n } = useTranslation()
    const { data: records, isLoading, error } = useQuery({
        queryKey: ['staff-attendance-widget'],
        queryFn: async () => {
            const today = new Date().toISOString().split('T')[0]
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(`/hr/attendance?from=${today}&to=${today}&per_page=50`)
            return res.data.data.map(item => ({
                user_name: item.user_name,
                clock_in: item.clock_in_time,
                clock_out: item.clock_out_time,
                status: (item.status.toLowerCase() === 'normal' ? 'normal' : item.status.toLowerCase() === 'late' ? 'late' : 'absent') as 'normal' | 'late' | 'absent'
            }))
        },
        staleTime: 30_000,
    })

    const formatTime = (timeStr: string | null) => {
        if (!timeStr) return '--:--'
        return new Date(timeStr).toLocaleTimeString(i18n.language === 'en' ? 'en-US' : 'zh-TW', {
            hour: '2-digit',
            minute: '2-digit',
        })
    }

    const getStatusBadge = (status: string) => {
        const lowerStatus = status.toLowerCase()
        switch (lowerStatus) {
            case 'normal':
                return <Badge variant="secondary" className="bg-green-100 text-green-700 hover:bg-green-100 border-none text-[10px]">{t('dashboard.widgets.attendance.normal')}</Badge>
            case 'late':
                return <Badge variant="destructive" className="text-[10px]">{t('dashboard.widgets.attendance.late')}</Badge>
            case 'leave':
                return <Badge variant="outline" className="text-[10px]">{t('dashboard.widgets.attendance.leave')}</Badge>
            default:
                return <Badge variant="secondary" className="text-[10px]">{t('dashboard.widgets.attendance.normal')}</Badge>
        }
    }

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <UserCheck className="h-4 w-4 text-emerald-500" />
                        {t('dashboard.widgets.names.staff_attendance')}
                    </CardTitle>
                </CardHeader>
                <CardContent className="flex items-center justify-center py-10">
                    <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <UserCheck className="h-4 w-4 text-emerald-500" />
                        {t('dashboard.widgets.names.staff_attendance')}
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card className="h-full flex flex-col overflow-hidden">
            <CardHeader className="pb-2 border-b bg-muted/30">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <UserCheck className="h-4 w-4 text-emerald-500" />
                    {t('dashboard.widgets.names.staff_attendance')}
                </CardTitle>
                <CardDescription className="text-xs">{t('dashboard.widgets.descriptions.staff_attendance')}</CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-auto p-0">
                {records && records.length > 0 ? (
                    <div className="divide-y">
                        {records.map((record, idx) => (
                            <div key={idx} className="flex items-center justify-between p-3 hover:bg-muted/50 transition-colors">
                                <div className="flex items-center gap-3">
                                    <div className="w-8 h-8 rounded-full bg-slate-100 flex items-center justify-center text-xs font-semibold text-slate-600">
                                        {record.user_name.charAt(0)}
                                    </div>
                                    <div>
                                        <p className="text-sm font-medium">{record.user_name}</p>
                                        <div className="flex items-center gap-2 mt-0.5">
                                            {getStatusBadge(record.status)}
                                        </div>
                                    </div>
                                </div>
                                <div className="text-right">
                                    <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                        <Clock className="h-3 w-3" />
                                        <span>{record.clock_in ? formatTime(record.clock_in) : '--:--'}</span>
                                    </div>
                                    <p className="text-[10px] text-muted-foreground mt-0.5">{t('dashboard.widgets.attendance.clockInTime')}</p>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-10 text-muted-foreground">
                        <UserX className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-xs">{t('dashboard.widgets.attendance.noRecords')}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}

