import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Calendar, UserCheck, Clock, Loader2 } from 'lucide-react'
import api from '@/lib/api'

interface CalendarData {
    today_leaves: Array<{
        user_name: string
        leave_type: string
        start_time: string
        end_time: string
        is_all_day: boolean
    }>
    today_events: Array<{
        id: string
        summary: string
        start: string
        end: string
        all_day: boolean
        location?: string
    }>
    upcoming_leaves: Array<{
        user_name: string
        start_date: string
        end_date: string
    }>
}

export function CalendarWidget() {
    const { t, i18n } = useTranslation()
    const { data, isLoading, error } = useQuery({
        queryKey: ['dashboard-calendar'],
        queryFn: async () => {
            const res = await api.get<CalendarData>('/hr/dashboard/calendar')
            return res.data
        },
        staleTime: 30_000,
    })

    const formatTime = (timeStr?: string) => {
        if (!timeStr) return ''
        try {
            return new Date(timeStr).toLocaleTimeString(i18n.language, {
                hour: '2-digit',
                minute: '2-digit',
            })
        } catch {
            return timeStr
        }
    }

    const formatDateShort = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString(i18n.language, {
            timeZone: 'Asia/Taipei',
            month: 'short',
            day: 'numeric',
        })
    }

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-indigo-500" />
                        {t('dashboard.widgets.names.calendar_widget')}
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
                        <Calendar className="h-4 w-4 text-indigo-500" />
                        {t('dashboard.widgets.names.calendar_widget')}
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card className="h-full overflow-hidden flex flex-col">
            <CardHeader className="pb-2 border-b bg-muted/30">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-indigo-500" />
                    {t('dashboard.widgets.names.calendar_widget')}
                </CardTitle>
                <CardDescription className="text-xs">
                    {t('dashboard.widgets.common.today')} {new Date().toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei', weekday: 'long', month: 'long', day: 'numeric' })}
                </CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-auto p-4 space-y-4">
                {/* 今日請假 */}
                <div>
                    <h4 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1">
                        <UserCheck className="h-3 w-3" />
                        {t('dashboard.widgets.calendar.todayLeave')}
                    </h4>
                    <div className="space-y-1">
                        {data?.today_leaves && data.today_leaves.length > 0 ? (
                            data.today_leaves.map((leave, idx) => (
                                <div key={idx} className="flex items-center justify-between text-sm p-1.5 bg-status-warning-bg/50 rounded-md border border-status-warning-border/50">
                                    <span className="font-medium">{leave.user_name}</span>
                                    <div className="flex items-center gap-2 text-xs text-status-warning-text">
                                        <Badge variant="outline" className="h-5 px-1 bg-white text-[10px] border-status-warning-border">
                                            {leave.leave_type}
                                        </Badge>
                                        <span>
                                            {leave.is_all_day ? t('dashboard.widgets.common.allDay') : `${formatTime(leave.start_time)} - ${formatTime(leave.end_time)}`}
                                        </span>
                                    </div>
                                </div>
                            ))
                        ) : (
                            <p className="text-xs text-muted-foreground py-1 px-2 italic">{t('dashboard.widgets.calendar.noOneLeave')}</p>
                        )}
                    </div>
                </div>

                {/* 今日日程 */}
                <div>
                    <h4 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {t('dashboard.widgets.calendar.todayEvents')}
                    </h4>
                    <div className="space-y-1">
                        {data?.today_events && data.today_events.length > 0 ? (
                            data.today_events.map((event) => (
                                <div key={event.id} className="flex flex-col gap-0.5 p-1.5 bg-status-info-bg/50 rounded-md border border-indigo-100/50">
                                    <div className="flex items-center justify-between">
                                        <span className="text-sm font-medium">{event.summary}</span>
                                        {event.location && (
                                            <Badge variant="secondary" className="h-4 px-1 text-[10px]">
                                                {event.location}
                                            </Badge>
                                        )}
                                    </div>
                                    <span className="text-[10px] text-status-info-text font-medium">
                                        {event.all_day ? t('dashboard.widgets.common.allDay') : `${formatTime(event.start)} - ${formatTime(event.end)}`}
                                    </span>
                                </div>
                            ))
                        ) : (
                            <p className="text-xs text-muted-foreground py-1 px-2 italic">{t('dashboard.widgets.calendar.noEvents')}</p>
                        )}
                    </div>
                </div>

                {/* 近期請假 */}
                {data?.upcoming_leaves && data.upcoming_leaves.length > 0 && (
                    <div>
                        <h4 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1">
                            <Calendar className="h-3 w-3" />
                            {t('dashboard.widgets.calendar.upcomingLeave')}
                        </h4>
                        <div className="space-y-1">
                            {data.upcoming_leaves.map((leave, idx) => (
                                <div key={idx} className="flex items-center justify-between text-xs p-1.5 bg-muted/50 rounded-md">
                                    <span>{leave.user_name}</span>
                                    <span className="text-muted-foreground">
                                        {formatDateShort(leave.start_date)} - {formatDateShort(leave.end_date)}
                                    </span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
