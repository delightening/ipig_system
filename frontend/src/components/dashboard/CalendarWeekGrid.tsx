import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Clock } from 'lucide-react'

import { cn, formatTimeShort } from '@/lib/utils'
import type { CalendarEvent } from '@/types/hr'

import { buildWeekDays } from './calendarWeek'

interface CalendarWeekGridProps {
    events: CalendarEvent[]
    /** 點擊事件時觸發（開啟 Google Calendar 連結） */
    onOpenEvent: (event: CalendarEvent) => void
}

/**
 * 輕量 7 天週曆格狀視圖：一眼看完整週（週一～週日），每日事件以色塊 chip 呈現。
 * 為 dashboard widget 尺寸量身，事件多時各日獨立縱向捲動，不需整體上下捲。
 */
export function CalendarWeekGrid({ events, onOpenEvent }: CalendarWeekGridProps) {
    const { i18n } = useTranslation()
    const days = useMemo(() => buildWeekDays(events, new Date()), [events])

    const weekdayLabel = (date: Date) =>
        date.toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei', weekday: 'short' })
    const dayNumber = (date: Date) =>
        date.toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei', day: 'numeric' })
    return (
        <div className="grid h-full grid-cols-7 gap-px bg-border">
            {days.map((day) => (
                <div key={day.dateKey} className="flex min-w-0 flex-col bg-card">
                    <div
                        className={cn(
                            'border-b px-1 py-1 text-center',
                            day.isToday ? 'bg-status-info-bg' : 'bg-muted/30',
                        )}
                    >
                        <div className="text-xs leading-none text-muted-foreground">{weekdayLabel(day.date)}</div>
                        <div
                            className={cn(
                                'mt-0.5 text-xs font-semibold leading-tight',
                                day.isToday ? 'text-status-info-text' : 'text-foreground',
                            )}
                        >
                            {dayNumber(day.date)}
                        </div>
                    </div>
                    <div className="flex-1 space-y-0.5 overflow-y-auto p-0.5">
                        {day.events.map((event) => (
                            <button
                                key={event.id}
                                type="button"
                                onClick={() => onOpenEvent(event)}
                                title={event.summary}
                                className="w-full rounded-sm bg-status-info-bg/60 px-1 py-0.5 text-left transition-colors hover:bg-status-info-bg focus:outline-hidden focus-visible:ring-1 focus-visible:ring-primary"
                            >
                                {!event.all_day && (
                                    <span className="flex items-center gap-0.5 text-xs font-medium leading-none text-status-info-text">
                                        <Clock className="h-2.5 w-2.5 shrink-0" />
                                        {formatTimeShort(event.start)}
                                    </span>
                                )}
                                <span className="mt-0.5 block text-xs leading-tight text-foreground line-clamp-2">
                                    {event.summary}
                                </span>
                            </button>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    )
}
