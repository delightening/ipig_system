/**
 * 請假行事曆分頁（原生資料）
 *
 * 與「日曆」分頁的差別：那一頁畫的是 Google 已同步的**已核准**事件，
 * 本頁直接讀 leave_requests，含**審核中**的假單，且可見範圍由後端
 * 依觀看者的部門與代理關係決定（見 services/hr/leave_calendar.rs）。
 */
import { lazy, Suspense, useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
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

/** 範圍切換的選項：value 是篩選狀態值，labelKey 是顯示文字的 i18n 鍵（渲染時才 t()） */
const SCOPE_OPTIONS = [
    { value: 'all', labelKey: 'hrPages.calendar.leaveTab.scopeAll' },
    { value: 'mine', labelKey: 'hrPages.calendar.leaveTab.scopeMine' },
] as const

export function LeaveCalendarTab({ isActive }: LeaveCalendarTabProps) {
    const { t } = useTranslation()
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
        return Object.keys(LEAVE_TYPE_COLORS).filter(name => found.has(name))
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
                <div className="text-sm text-muted-foreground">{t('hrPages.calendar.leaveTab.loading')}</div>
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
                    <div className="font-medium">{t('hrPages.calendar.leaveTab.loadError')}</div>
                    <div className="text-sm text-muted-foreground max-w-md">
                        {t('hrPages.calendar.leaveTab.loadErrorHint')}
                    </div>
                </div>
                <Button variant="outline" onClick={() => void refetch()}>
                    {t('common.retry')}
                </Button>
            </div>
        )
    }

    return (
        <div className="space-y-3">
            {/* 範圍切換 + 圖例 */}
            <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                    {SCOPE_OPTIONS.map(opt => (
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
                            {t(opt.labelKey)}
                        </button>
                    ))}
                    {scope === 'department' && (
                        <span className="text-xs text-muted-foreground">
                            {t('hrPages.calendar.leaveTab.departmentScope')}
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
                        {t('hrPages.calendar.leaveTab.pendingLegend', { count: pendingCount })}
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
                        {t('hrPages.calendar.filterAll')}
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
                            ? t('hrPages.calendar.leaveTab.emptyForType', { type: activeType })
                            : scopeFilter === 'mine'
                              ? t('hrPages.calendar.leaveTab.emptyMine')
                              : t('hrPages.calendar.leaveTab.empty')}
                    </div>
                </div>
            ) : (
                <ErrorBoundary
                    fallback={
                        <div className="flex flex-col items-center justify-center py-12 gap-4">
                            <AlertTriangle className="h-12 w-12 text-destructive" />
                            <div className="text-center space-y-2">
                                <div className="font-medium">{t('hrPages.calendar.loadFailed')}</div>
                                <div className="text-sm text-muted-foreground max-w-md">
                                    {t('hrPages.calendar.loadFailedHint')}
                                </div>
                            </div>
                            <Button variant="outline" onClick={() => window.location.reload()}>
                                {t('hrPages.shared.action.refresh')}
                            </Button>
                        </div>
                    }
                >
                    <Suspense
                        fallback={
                            <div className="flex flex-col items-center justify-center py-12 gap-4">
                                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                                <div className="text-sm text-muted-foreground">{t('hrPages.calendar.loadingCalendar')}</div>
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
