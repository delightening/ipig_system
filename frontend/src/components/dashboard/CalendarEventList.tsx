import { useTranslation } from 'react-i18next'
import { Clock, ExternalLink } from 'lucide-react'

import { formatTimeShort } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import type { CalendarEvent } from '@/types/hr'

interface CalendarEventListProps {
    events: CalendarEvent[]
    /** 點擊事件時觸發（開啟 Google Calendar 連結） */
    onOpenEvent: (event: CalendarEvent) => void
}

/** 日曆事件條列視圖：依日期由近到遠垂直列出本週事件。 */
export function CalendarEventList({ events, onOpenEvent }: CalendarEventListProps) {
    const { t, i18n } = useTranslation()

    const formatDate = (dateStr: string) =>
        new Date(dateStr).toLocaleDateString(i18n.language, {
            timeZone: 'Asia/Taipei',
            month: 'short',
            day: 'numeric',
            weekday: 'short',
        })

    return (
        <div className="divide-y">
            {events.map((event) => (
                <button
                    key={event.id}
                    type="button"
                    onClick={() => onOpenEvent(event)}
                    className="w-full p-3 text-left hover:bg-muted/50 transition-colors group focus:outline-hidden focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary"
                >
                    <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                                <span className="text-xs font-semibold text-status-info-text">
                                    {formatDate(event.start)}
                                </span>
                                {event.all_day && (
                                    <Badge variant="outline" className="px-1 text-xs bg-status-info-bg text-status-info-text border-status-info-border">
                                        {t('dashboard.widgets.common.allDay')}
                                    </Badge>
                                )}
                            </div>
                            <p className="text-sm font-medium line-clamp-2">{event.summary}</p>
                            {!event.all_day && (
                                <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                                    <Clock className="h-3 w-3" />
                                    <span>{formatTimeShort(event.start)} - {formatTimeShort(event.end)}</span>
                                </div>
                            )}
                            {event.location && (
                                <p className="text-xs text-muted-foreground mt-1 truncate">
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
    )
}
