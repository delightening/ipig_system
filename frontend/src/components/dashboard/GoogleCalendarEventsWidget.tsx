import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { format, startOfISOWeek, endOfISOWeek } from 'date-fns'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Calendar as CalendarIcon, Loader2, CalendarX, List, CalendarRange } from 'lucide-react'
import { isAxiosError } from 'axios'
import { cn } from '@/lib/utils'
import api from '@/lib/api'
import type { CalendarEvent } from '@/types/hr'
import { CalendarEventList } from './CalendarEventList'
import { CalendarWeekGrid } from './CalendarWeekGrid'

type CalendarView = 'list' | 'week'

/** 列表 / 週曆視圖切換鈕（segmented control） */
function ViewToggle({ view, onChange }: { view: CalendarView; onChange: (v: CalendarView) => void }) {
    const { t } = useTranslation()
    const base = 'flex h-6 w-6 items-center justify-center rounded-sm transition-colors focus:outline-hidden focus-visible:ring-1 focus-visible:ring-primary'
    const stateClass = (active: boolean) =>
        active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'
    return (
        <div className="flex shrink-0 items-center gap-0.5 rounded-md border bg-background p-0.5">
            <button
                type="button"
                aria-label={t('dashboard.widgets.calendar.viewList')}
                aria-pressed={view === 'list'}
                onClick={() => onChange('list')}
                className={cn(base, stateClass(view === 'list'))}
            >
                <List className="h-3.5 w-3.5" />
            </button>
            <button
                type="button"
                aria-label={t('dashboard.widgets.calendar.viewWeek')}
                aria-pressed={view === 'week'}
                onClick={() => onChange('week')}
                className={cn(base, stateClass(view === 'week'))}
            >
                <CalendarRange className="h-3.5 w-3.5" />
            </button>
        </div>
    )
}

export function GoogleCalendarEventsWidget() {
    const { t } = useTranslation()
    const [view, setView] = useState<CalendarView>('list')
    const { data: events, isLoading, error } = useQuery({
        queryKey: ['google-calendar-events-widget'],
        queryFn: async () => {
            try {
                const now = new Date()
                // 顯示整週（週一～週日），讓列表與週曆視圖共用同一份資料
                const startDate = format(startOfISOWeek(now), 'yyyy-MM-dd')
                const endDate = format(endOfISOWeek(now), 'yyyy-MM-dd')

                const res = await api.get<CalendarEvent[]>('/hr/calendar/events', {
                    params: { start_date: startDate, end_date: endDate }
                })
                return res.data
            } catch (err: unknown) {
                // 未連接 Google Calendar 時回傳 null，視為正常狀態而非錯誤
                if (isAxiosError(err) && err.response?.status === 400) {
                    const msg = err.response.data?.message || err.response.data?.error || ''
                    if (typeof msg === 'string' && msg.includes('Google Calendar')) {
                        return null
                    }
                }
                throw err
            }
        },
        staleTime: 60_000,
    })

    const handleOpenInGoogle = (event: CalendarEvent) => {
        if (!event.html_link) return
        window.open(event.html_link, '_blank', 'noopener,noreferrer')
    }

    const titleNode = (
        <CardTitle className="text-sm font-medium flex items-center gap-2">
            <CalendarIcon className="h-4 w-4 text-status-info-solid" />
            {t('dashboard.widgets.names.google_calendar_events')}
        </CardTitle>
    )

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pt-3 pb-2">{titleNode}</CardHeader>
                <CardContent className="flex items-center justify-center py-10">
                    <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card className="h-full">
                <CardHeader className="pt-3 pb-2">{titleNode}</CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    const hasEvents = events != null && events.length > 0

    return (
        <Card className="h-full flex flex-col overflow-hidden">
            <CardHeader className="pt-3 pb-2 border-b bg-muted/30">
                <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0">
                        {titleNode}
                        <CardDescription className="text-xs">{t('dashboard.widgets.calendar.googleDescription')}</CardDescription>
                    </div>
                    {hasEvents && <ViewToggle view={view} onChange={setView} />}
                </div>
            </CardHeader>
            <CardContent className={cn('flex-1 overflow-auto p-0', view === 'week' && hasEvents && 'overflow-hidden')}>
                {events === null ? (
                    <div className="flex flex-col items-center justify-center py-10 text-muted-foreground bg-muted/5">
                        <CalendarX className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-sm font-medium">{t('dashboard.widgets.calendar.googleNotConnected')}</p>
                        <p className="text-xs mt-1 opacity-70">{t('dashboard.widgets.calendar.googleConnectHint')}</p>
                    </div>
                ) : hasEvents ? (
                    view === 'week' ? (
                        <CalendarWeekGrid events={events} onOpenEvent={handleOpenInGoogle} />
                    ) : (
                        <CalendarEventList events={events} onOpenEvent={handleOpenInGoogle} />
                    )
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
