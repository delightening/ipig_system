/**
 * 請假行事曆分頁（原生資料）
 *
 * 與「日曆」分頁的差別：那一頁畫的是 Google 已同步的**已核准**事件，
 * 本頁直接讀 leave_requests，含**審核中**的假單，且可見範圍由後端
 * 依觀看者的部門與代理關係決定（見 services/hr/leave_calendar.rs）。
 */
import { lazy, Suspense, useState, useMemo } from 'react'
import { AlertTriangle, CalendarOff, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ErrorBoundary } from '@/components/ui/error-boundary'
import { LEAVE_TYPE_COLORS, getLeaveType } from '@/hooks/useCalendarEvents'
import { useLeaveCalendarEvents } from '@/hooks/useLeaveCalendarEvents'
import type { LeaveCalendarFilter } from '@/hooks/useLeaveCalendarEvents'

const CalendarView = lazy(() =>
    import('../CalendarView').then((mod) => ({ default: mod.CalendarView })),
)

interface LeaveCalendarTabProps {
    /** 分頁是否為當前分頁；false 時不發查詢 */
    isActive: boolean
}

export function LeaveCalendarTab({ isActive }: LeaveCalendarTabProps) {
    const [scopeFilter, setScopeFilter] = useState<LeaveCalendarFilter>('all')
    const [activeType, setActiveType] = useState<string | null>(null)

    const { events, scope, isLoading, isFetching, isError, refetch, handleDatesSet } =
        useLeaveCalendarEvents(isActive, scopeFilter)

    // 假別 chips 依 LEAVE_TYPE_COLORS 的順序排列，與「日曆」分頁一致
    const presentTypes = useMemo(() => {
        const found = new Set<string>()
        for (const e of events) {
            const type = getLeaveType(e.title)
            if (type) found.add(type)
        }
        return Object.keys(LEAVE_TYPE_COLORS).filter(t => found.has(t))
    }, [events])

    const visibleEvents = useMemo(
        () => (activeType ? events.filter(e => getLeaveType(e.title) === activeType) : events),
        [events, activeType],
    )

    // 從實際畫出來的那一組算，不是從原始資料算——圖例說「N 筆」就必須等於
    // 畫面上數得到的虛線色塊數，否則假別 chip 一按數字就對不上。
    const pendingCount = useMemo(
        () => visibleEvents.filter(e => e.extendedProps?.pendingStage).length,
        [visibleEvents],
    )

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                <div className="text-sm text-muted-foreground">載入請假資料中...</div>
            </div>
        )
    }

    // 失敗必須擋在空狀態之前。若讓它掉進下面的 events.length === 0 分支，
    // 後端掛掉會顯示成「這段期間沒有請假紀錄」——把不可用的排班資料
    // 冒充成可信的空結果，比直接報錯危險得多。
    if (isError) {
        return (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
                <AlertTriangle className="h-12 w-12 text-destructive" />
                <div className="text-center space-y-1">
                    <div className="font-medium">無法載入請假資料</div>
                    <div className="text-sm text-muted-foreground max-w-md">
                        這不代表這段期間沒有人請假——請重試，若持續失敗請聯繫系統管理員。
                    </div>
                </div>
                <Button variant="outline" onClick={() => void refetch()}>
                    重試
                </Button>
            </div>
        )
    }

    return (
        <div className="space-y-3">
            {/* 範圍切換 + 圖例 */}
            <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                    {(
                        [
                            { value: 'all', label: '全場' },
                            { value: 'mine', label: '與我相關' },
                        ] as const
                    ).map(opt => (
                        <button
                            key={opt.value}
                            type="button"
                            onClick={() => setScopeFilter(opt.value)}
                            aria-pressed={scopeFilter === opt.value}
                            className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                                scopeFilter === opt.value
                                    ? 'bg-foreground text-background border-foreground'
                                    : 'bg-background text-muted-foreground border-border hover:border-foreground'
                            }`}
                        >
                            {opt.label}
                        </button>
                    ))}
                    {scope === 'department' && (
                        <span className="text-xs text-muted-foreground">
                            僅顯示您所屬部門與相鄰層級
                        </span>
                    )}
                </div>

                {/* 審核中的圖例。沒有審核中的假單時不顯示，避免無謂的視覺雜訊 */}
                {pendingCount > 0 && (
                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                        <span
                            className="inline-block h-3 w-5 rounded-sm border-[1.5px] border-dashed border-muted-foreground"
                            aria-hidden="true"
                        />
                        虛線 = 審核中（{pendingCount} 筆）
                    </div>
                )}
            </div>

            {/* 假別篩選 chips */}
            {presentTypes.length > 0 && (
                <div className="flex flex-wrap items-center gap-2">
                    <button
                        type="button"
                        onClick={() => setActiveType(null)}
                        className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                            activeType === null
                                ? 'bg-foreground text-background border-foreground'
                                : 'bg-background text-muted-foreground border-border hover:border-foreground'
                        }`}
                    >
                        全部
                    </button>
                    {presentTypes.map(type => {
                        const color = LEAVE_TYPE_COLORS[type]
                        const isActive = activeType === type
                        return (
                            <button
                                key={type}
                                type="button"
                                onClick={() => setActiveType(isActive ? null : type)}
                                className="px-3 py-1 rounded-full text-xs font-medium border transition-colors"
                                style={
                                    isActive
                                        ? { backgroundColor: color, borderColor: color, color: '#fff' }
                                        : { borderColor: color, color, backgroundColor: 'transparent' }
                                }
                            >
                                {type}
                                {isActive && (
                                    <span className="ml-1 opacity-75">({visibleEvents.length})</span>
                                )}
                            </button>
                        )
                    })}
                </div>
            )}

            {/* 判斷用 visibleEvents 而非 events：假別 chip 把資料篩光時，
                events 仍非空但畫面上一格都沒有，該顯示空狀態而不是一張空日曆 */}
            {visibleEvents.length === 0 ? (
                <div className="text-center py-12 space-y-3">
                    <CalendarOff className="h-14 w-14 mx-auto text-muted-foreground" />
                    <div className="text-sm text-muted-foreground">
                        {activeType
                            ? `這段期間沒有${activeType}的紀錄`
                            : scopeFilter === 'mine'
                              ? '這段期間沒有您請的假或需要您代理的假'
                              : '這段期間沒有請假紀錄'}
                    </div>
                </div>
            ) : (
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
                            events={visibleEvents}
                            onDatesSet={handleDatesSet}
                            isFetching={isFetching}
                        />
                    </Suspense>
                </ErrorBoundary>
            )}
        </div>
    )
}
