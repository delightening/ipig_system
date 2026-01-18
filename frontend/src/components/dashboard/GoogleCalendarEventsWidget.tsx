import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { format, startOfWeek, endOfWeek, isToday, parseISO } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { CalendarDays, Clock, Loader2, Calendar, ExternalLink } from 'lucide-react'
import api from '@/lib/api'

interface CalendarEvent {
    id: string
    summary: string
    start: string
    end: string
    all_day: boolean
    description?: string
    location?: string
    html_link?: string
}

function formatEventTime(dateStr: string, allDay: boolean): string {
    const date = parseISO(dateStr)
    if (allDay) {
        return format(date, 'M/d (EEE)', { locale: zhTW })
    }
    return format(date, 'M/d HH:mm', { locale: zhTW })
}

function formatTimeOnly(dateStr: string): string {
    return format(parseISO(dateStr), 'HH:mm')
}

export function GoogleCalendarEventsWidget() {
    const navigate = useNavigate()

    // 取得本週的日期範圍
    const today = new Date()
    const weekStart = startOfWeek(today, { weekStartsOn: 1 }) // Monday
    const weekEnd = endOfWeek(today, { weekStartsOn: 1 })

    const { data: events, isLoading, error } = useQuery({
        queryKey: ['calendar-events-widget', format(weekStart, 'yyyy-MM-dd')],
        queryFn: async () => {
            const startDate = format(weekStart, 'yyyy-MM-dd')
            const endDate = format(weekEnd, 'yyyy-MM-dd')
            const res = await api.get<CalendarEvent[]>(`/hr/calendar/events?start_date=${startDate}&end_date=${endDate}`)
            return res.data
        },
        staleTime: 5 * 60 * 1000, // 5 minutes
    })

    // 分類事件：今日 vs 本週其他
    const todayEvents = events?.filter(e => isToday(parseISO(e.start))) || []
    const weekEvents = events?.filter(e => !isToday(parseISO(e.start))) || []

    const handleTitleClick = () => {
        navigate('/hr/calendar')
    }

    if (isLoading) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle
                        className="text-sm font-medium flex items-center gap-2 cursor-pointer hover:text-primary transition-colors"
                        onClick={handleTitleClick}
                    >
                        <CalendarDays className="h-4 w-4 text-indigo-500" />
                        日曆事件
                        <ExternalLink className="h-3 w-3 ml-auto text-muted-foreground" />
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
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle
                        className="text-sm font-medium flex items-center gap-2 cursor-pointer hover:text-primary transition-colors"
                        onClick={handleTitleClick}
                    >
                        <CalendarDays className="h-4 w-4 text-indigo-500" />
                        日曆事件
                        <ExternalLink className="h-3 w-3 ml-auto text-muted-foreground" />
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="text-center py-4 text-muted-foreground">
                        <Calendar className="h-8 w-8 mx-auto mb-2 text-muted-foreground/50" />
                        <p className="text-sm">尚未連接 Google Calendar</p>
                        <p className="text-xs mt-1">請至日曆頁面設定</p>
                    </div>
                </CardContent>
            </Card>
        )
    }

    const weekDateRange = `${format(weekStart, 'M/d')} - ${format(weekEnd, 'M/d')}`

    return (
        <Card>
            <CardHeader className="pb-2">
                <CardTitle
                    className="text-sm font-medium flex items-center gap-2 cursor-pointer hover:text-primary transition-colors"
                    onClick={handleTitleClick}
                >
                    <CalendarDays className="h-4 w-4 text-indigo-500" />
                    日曆事件
                    <ExternalLink className="h-3 w-3 ml-auto text-muted-foreground" />
                </CardTitle>
                <CardDescription className="text-xs">
                    本週 {weekDateRange}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
                {/* 今日事件 */}
                <div>
                    <div className="flex items-center gap-2 mb-2">
                        <Clock className="h-3 w-3 text-orange-500" />
                        <span className="text-xs font-medium text-muted-foreground">今日</span>
                        {todayEvents.length > 0 && (
                            <Badge variant="secondary" className="text-xs px-1.5 py-0">
                                {todayEvents.length}
                            </Badge>
                        )}
                    </div>
                    {todayEvents.length > 0 ? (
                        <div className="space-y-1.5">
                            {todayEvents.slice(0, 3).map((event) => (
                                <div
                                    key={event.id}
                                    className="flex items-center gap-2 p-2 bg-orange-50 rounded-md border border-orange-100"
                                >
                                    <div className="flex-1 min-w-0">
                                        <p className="text-sm font-medium text-orange-900 truncate">
                                            {event.summary}
                                        </p>
                                        {!event.all_day && (
                                            <p className="text-xs text-orange-600">
                                                {formatTimeOnly(event.start)} - {formatTimeOnly(event.end)}
                                            </p>
                                        )}
                                    </div>
                                    {event.all_day && (
                                        <Badge variant="outline" className="text-xs text-orange-600 border-orange-200">
                                            全天
                                        </Badge>
                                    )}
                                </div>
                            ))}
                            {todayEvents.length > 3 && (
                                <p className="text-xs text-muted-foreground text-center">
                                    還有 {todayEvents.length - 3} 個事件...
                                </p>
                            )}
                        </div>
                    ) : (
                        <p className="text-xs text-muted-foreground pl-5">無排程事件</p>
                    )}
                </div>

                {/* 本週其他事件 */}
                {weekEvents.length > 0 && (
                    <div>
                        <div className="flex items-center gap-2 mb-2">
                            <CalendarDays className="h-3 w-3 text-indigo-500" />
                            <span className="text-xs font-medium text-muted-foreground">本週</span>
                            <Badge variant="secondary" className="text-xs px-1.5 py-0">
                                {weekEvents.length}
                            </Badge>
                        </div>
                        <div className="space-y-1">
                            {weekEvents.slice(0, 3).map((event) => (
                                <div
                                    key={event.id}
                                    className="flex items-center justify-between text-sm px-2"
                                >
                                    <span className="truncate text-muted-foreground">
                                        {event.summary}
                                    </span>
                                    <span className="text-xs text-indigo-600 whitespace-nowrap ml-2">
                                        {formatEventTime(event.start, event.all_day)}
                                    </span>
                                </div>
                            ))}
                            {weekEvents.length > 3 && (
                                <p className="text-xs text-muted-foreground text-center">
                                    還有 {weekEvents.length - 3} 個事件...
                                </p>
                            )}
                        </div>
                    </div>
                )}

                {/* 空狀態 */}
                {todayEvents.length === 0 && weekEvents.length === 0 && (
                    <div className="text-center py-2">
                        <p className="text-xs text-muted-foreground">本週無排程事件 ✓</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
