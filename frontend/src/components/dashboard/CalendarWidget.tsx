import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Calendar, Clock, Loader2, Users, CalendarDays } from 'lucide-react'
import api from '@/lib/api'

interface TodayLeaveInfo {
    user_id: string
    user_name: string
    leave_type: string
    leave_type_display: string
    is_all_day: boolean
    start_date: string
    end_date: string
}

interface CalendarEvent {
    id: string
    summary: string
    start: string
    end: string
    all_day: boolean
    description?: string
    location?: string
}

interface DashboardCalendarData {
    today: string
    today_leaves: TodayLeaveInfo[]
    today_events: CalendarEvent[]
    upcoming_leaves: TodayLeaveInfo[]
}

function formatTime(dateStr: string): string {
    const date = new Date(dateStr)
    return date.toLocaleTimeString('zh-TW', { hour: '2-digit', minute: '2-digit' })
}

function formatDate(dateStr: string): string {
    const date = new Date(dateStr)
    return date.toLocaleDateString('zh-TW', { month: 'short', day: 'numeric' })
}

export function CalendarWidget() {
    const { data, isLoading, error } = useQuery({
        queryKey: ['dashboard-calendar'],
        queryFn: async () => {
            const res = await api.get<DashboardCalendarData>('/hr/dashboard/calendar')
            return res.data
        },
    })

    if (isLoading) {
        return (
            <Card className="col-span-2">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-blue-500" />
                        今日日曆
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="flex items-center justify-center py-6">
                        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    </div>
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card className="col-span-2">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-blue-500" />
                        今日日曆
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    const todayDate = new Date().toLocaleDateString('zh-TW', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        weekday: 'long'
    })

    const hasEvents = (data?.today_events?.length ?? 0) > 0
    const hasLeaves = (data?.today_leaves?.length ?? 0) > 0
    const hasUpcoming = (data?.upcoming_leaves?.length ?? 0) > 0

    return (
        <Card className="col-span-2">
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-blue-500" />
                    今日日曆
                </CardTitle>
                <CardDescription>{todayDate}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* 今日請假 */}
                <div>
                    <div className="flex items-center gap-2 mb-2">
                        <Users className="h-4 w-4 text-orange-500" />
                        <span className="text-sm font-medium">今日請假</span>
                        {hasLeaves && (
                            <Badge variant="secondary" className="text-xs">
                                {data?.today_leaves.length} 人
                            </Badge>
                        )}
                    </div>
                    {hasLeaves ? (
                        <div className="flex flex-wrap gap-2">
                            {data?.today_leaves.map((leave) => (
                                <Badge
                                    key={leave.user_id}
                                    variant="outline"
                                    className="text-xs bg-orange-50 border-orange-200 text-orange-700"
                                >
                                    {leave.user_name}
                                    <span className="ml-1 text-orange-500">({leave.leave_type_display})</span>
                                </Badge>
                            ))}
                        </div>
                    ) : (
                        <p className="text-sm text-muted-foreground">今日無人請假 ✓</p>
                    )}
                </div>

                {/* 今日日程 */}
                <div>
                    <div className="flex items-center gap-2 mb-2">
                        <Clock className="h-4 w-4 text-indigo-500" />
                        <span className="text-sm font-medium">今日日程</span>
                    </div>
                    {hasEvents ? (
                        <div className="space-y-2">
                            {data?.today_events.slice(0, 5).map((event) => (
                                <div
                                    key={event.id}
                                    className="flex items-center gap-2 p-2 bg-indigo-50 rounded-lg border border-indigo-100"
                                >
                                    <div className="flex-1 min-w-0">
                                        <p className="text-sm font-medium text-indigo-900 truncate">
                                            {event.summary}
                                        </p>
                                        {!event.all_day && (
                                            <p className="text-xs text-indigo-600">
                                                {formatTime(event.start)} - {formatTime(event.end)}
                                            </p>
                                        )}
                                    </div>
                                    {event.all_day && (
                                        <Badge variant="secondary" className="text-xs bg-indigo-100 text-indigo-700">
                                            全天
                                        </Badge>
                                    )}
                                </div>
                            ))}
                            {(data?.today_events?.length ?? 0) > 5 && (
                                <p className="text-xs text-muted-foreground text-center">
                                    還有 {(data?.today_events?.length ?? 0) - 5} 個事件...
                                </p>
                            )}
                        </div>
                    ) : (
                        <p className="text-sm text-muted-foreground">今日無排程事件</p>
                    )}
                </div>

                {/* 即將請假 */}
                {hasUpcoming && (
                    <div>
                        <div className="flex items-center gap-2 mb-2">
                            <CalendarDays className="h-4 w-4 text-purple-500" />
                            <span className="text-sm font-medium">近期請假</span>
                        </div>
                        <div className="space-y-1">
                            {data?.upcoming_leaves.slice(0, 3).map((leave, idx) => (
                                <div
                                    key={`${leave.user_id}-${idx}`}
                                    className="flex items-center justify-between text-sm"
                                >
                                    <span className="text-muted-foreground">
                                        {formatDate(leave.start_date)} - {leave.user_name}
                                    </span>
                                    <Badge variant="outline" className="text-xs">
                                        {leave.leave_type_display}
                                    </Badge>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
