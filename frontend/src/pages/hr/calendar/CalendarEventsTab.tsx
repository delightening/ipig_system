/**
 * 日曆「事件」分頁元件
 * 顯示 FullCalendar 視圖或未連接提示，包含假別篩選 chips 與顏色圖例
 */
import { lazy, Suspense, useState, useMemo } from 'react'
import { AlertTriangle, Calendar, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ErrorBoundary } from '@/components/ui/error-boundary'
import { LEAVE_TYPE_COLORS, getLeaveType } from '@/hooks/useCalendarEvents'
import type { FullCalendarEvent } from '@/hooks/useCalendarEvents'

const CalendarView = lazy(() =>
    import('../CalendarView').then((mod) => ({ default: mod.CalendarView })),
)

interface CalendarEventsTabProps {
    isConfigured: boolean
    isAdmin: boolean
    loadingStatus: boolean
    isLoading: boolean
    isFetching: boolean
    fullCalendarEvents: FullCalendarEvent[]
    onDatesSet: (dateInfo: { start: Date; end: Date }) => void
}

export function CalendarEventsTab({
    isConfigured,
    isAdmin,
    loadingStatus,
    isLoading,
    isFetching,
    fullCalendarEvents,
    onDatesSet,
}: CalendarEventsTabProps) {
    // 篩選狀態：null = 全部顯示，否則只顯示選取假別
    const [activeFilter, setActiveFilter] = useState<string | null>(null)

    // 從現有事件中提取出現過的假別（依 LEAVE_TYPE_COLORS 順序排列）
    const presentLeaveTypes = useMemo(() => {
        const found = new Set<string>()
        for (const e of fullCalendarEvents) {
            const type = getLeaveType(e.title)
            if (type) found.add(type)
        }
        // 維持對照表中定義的順序
        return Object.keys(LEAVE_TYPE_COLORS).filter(t => found.has(t))
    }, [fullCalendarEvents])

    // 套用篩選
    const filteredEvents = useMemo(() => {
        if (!activeFilter) return fullCalendarEvents
        return fullCalendarEvents.filter(e => getLeaveType(e.title) === activeFilter)
    }, [fullCalendarEvents, activeFilter])

    // --- 狀態判斷 ---
    if (loadingStatus) {
        return (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                <div className="text-sm text-muted-foreground">載入中...</div>
            </div>
        )
    }

    if (!isConfigured) {
        return (
            <div className="text-center py-8 space-y-4">
                <Calendar className="h-16 w-16 mx-auto text-muted-foreground" />
                <div>
                    <div className="font-medium">尚未連接 Google Calendar</div>
                    {isAdmin ? (
                        <div className="text-sm text-muted-foreground">
                            請先在「連線狀態」分頁連接 Google Calendar
                        </div>
                    ) : (
                        <div className="text-sm text-muted-foreground">
                            請聯繫系統管理員設定 Google Calendar 連接
                        </div>
                    )}
                </div>
            </div>
        )
    }

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                <div className="text-sm text-muted-foreground">載入事件中...</div>
            </div>
        )
    }

    return (
        <div className="space-y-3">
            {/* 假別篩選 chips（有事件時才顯示） */}
            {presentLeaveTypes.length > 0 && (
                <div className="flex flex-wrap items-center gap-2">
                    <button
                        type="button"
                        onClick={() => setActiveFilter(null)}
                        className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                            activeFilter === null
                                ? 'bg-foreground text-background border-foreground'
                                : 'bg-background text-muted-foreground border-border hover:border-foreground'
                        }`}
                    >
                        全部
                    </button>
                    {presentLeaveTypes.map(type => {
                        const color = LEAVE_TYPE_COLORS[type]
                        const isActive = activeFilter === type
                        return (
                            <button
                                key={type}
                                type="button"
                                onClick={() => setActiveFilter(isActive ? null : type)}
                                className="px-3 py-1 rounded-full text-xs font-medium border transition-colors"
                                style={
                                    isActive
                                        ? { backgroundColor: color, borderColor: color, color: '#fff' }
                                        : { borderColor: color, color: color, backgroundColor: 'transparent' }
                                }
                            >
                                {type}
                                {activeFilter === type && (
                                    <span className="ml-1 opacity-75">
                                        ({filteredEvents.length})
                                    </span>
                                )}
                            </button>
                        )
                    })}
                </div>
            )}

            {/* FullCalendar */}
            <ErrorBoundary
                fallback={
                    <div className="flex flex-col items-center justify-center py-12 gap-4">
                        <AlertTriangle className="h-12 w-12 text-destructive" />
                        <div className="text-center space-y-2">
                            <div className="font-medium">日曆載入失敗</div>
                            <div className="text-sm text-muted-foreground max-w-md">
                                可能是瀏覽器環境限制，請重新整理頁面或聯繫系統管理員
                            </div>
                        </div>
                        <Button variant="outline" onClick={() => window.location.reload()}>
                            重新整理
                        </Button>
                    </div>
                }
            >
                <Suspense
                    fallback={
                        <div className="flex flex-col items-center justify-center py-12 gap-4">
                            <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                            <div className="text-sm text-muted-foreground">載入日曆中...</div>
                        </div>
                    }
                >
                    <CalendarView
                        events={filteredEvents}
                        onDatesSet={onDatesSet}
                        isFetching={isFetching}
                    />
                </Suspense>
            </ErrorBoundary>
        </div>
    )
}
