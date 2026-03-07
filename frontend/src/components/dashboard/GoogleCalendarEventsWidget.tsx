import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { format, startOfDay, endOfISOWeek } from 'date-fns'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Calendar as CalendarIcon, Clock, Loader2, ExternalLink, CalendarX } from 'lucide-react'
import api from '@/lib/api'
import type { CalendarEvent } from '@/types/hr'

export function GoogleCalendarEventsWidget() {
    const { t, i18n } = useTranslation()
    const { data: events, isLoading, error } = useQuery({
        queryKey: ['google-calendar-events-widget'],
        queryFn: async () => {
            try {
                const now = new Date()
                // 從當天開始顯示日程，到本週末
                const startDate = format(startOfDay(now), 'yyyy-MM-dd')
                const endDate = format(endOfISOWeek(now), 'yyyy-MM-dd')

                const res = await api.get<CalendarEvent[]>('/hr/calendar/events', {
                    params: { start_date: startDate, end_date: endDate }
                })
                return res.data
            } catch (err: unknown) {
                const errObj = err as { response?: { data?: { message?: string; error?: string | { message?: string } }; status?: number } }
                const data = errObj?.response?.data
                const errorMsg = data?.message || (typeof data?.error === 'object' && data?.error?.message) || (typeof data?.error === 'string' ? data.error : '')

                const errResp = errObj?.response
                if (errResp?.status === 400 && errorMsg.includes('Google Calendar')) {
                    return null // Representing not connected
                }
                throw err
            }
        },
        staleTime: 60_000,
    })

    const formatTime = (dateStr: string) => {
        return new Date(dateStr).toLocaleTimeString(i18n.language, {
            hour: '2-digit',
            minute: '2-digit',
            timeZone: 'Asia/Taipei',
        })
    }

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString(i18n.language, {
            timeZone: 'Asia/Taipei',
            month: 'short',
            day: 'numeric',
            weekday: 'short',
        })
    }

    const handleOpenInGoogle = (event: CalendarEvent) => {
        if (!event.html_link) return
        window.open(event.html_link, '_blank', 'noopener,noreferrer')
    }

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <CalendarIcon className="h-4 w-4 text-blue-500" />
                        {t('dashboard.widgets.names.google_calendar_events')}
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
                        <CalendarIcon className="h-4 w-4 text-blue-500" />
                        {t('dashboard.widgets.names.google_calendar_events')}
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
                    <CalendarIcon className="h-4 w-4 text-blue-500" />
                    {t('dashboard.widgets.names.google_calendar_events')}
                </CardTitle>
                <CardDescription className="text-xs">{t('dashboard.widgets.calendar.googleDescription')}</CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-auto p-0">
                {events === null ? (
                    <div className="flex flex-col items-center justify-center py-10 text-muted-foreground bg-muted/5">
                        <CalendarX className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-sm font-medium">{t('dashboard.widgets.calendar.googleNotConnected')}</p>
                        <p className="text-xs mt-1 opacity-70">{t('dashboard.widgets.calendar.googleConnectHint')}</p>
                    </div>
                ) : events && events.length > 0 ? (
                    <div className="divide-y">
                        {events.map((event) => (
                            <button
                                key={event.id}
                                type="button"
                                onClick={() => handleOpenInGoogle(event)}
                                className="w-full p-3 text-left hover:bg-muted/50 transition-colors group focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary"
                            >
                                <div className="flex items-start justify-between gap-2">
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-2 mb-1">
                                            <span className="text-xs font-semibold text-blue-600">
                                                {formatDate(event.start)}
                                            </span>
                                            {event.all_day && (
                                                <Badge variant="outline" className="h-4 px-1 text-[9px] bg-blue-50 text-blue-700 border-blue-200">
                                                    {t('dashboard.widgets.common.allDay')}
                                                </Badge>
                                            )}
                                        </div>
                                        <p className="text-sm font-medium line-clamp-2">{event.summary}</p>
                                        {!event.all_day && (
                                            <div className="flex items-center gap-1 text-[10px] text-muted-foreground mt-1">
                                                <Clock className="h-3 w-3" />
                                                <span>{formatTime(event.start)} - {formatTime(event.end)}</span>
                                            </div>
                                        )}
                                        {event.location && (
                                            <p className="text-[10px] text-muted-foreground mt-1 truncate">
                                                📍 {event.location}
                                            </p>
                                        )}
                                    </div>
                                    {event.html_link && (
                                        <ExternalLink className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
                                    )}
                                </div>
                            </button>
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-10 text-muted-foreground">
                        <CalendarX className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-xs">{t('dashboard.widgets.calendar.noEvents')}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
