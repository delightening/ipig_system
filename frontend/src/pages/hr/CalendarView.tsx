/**
 * FullCalendar 視圖元件（獨立模組，僅在日曆已連接時動態載入）
 * 避免在未設定日曆時載入 FullCalendar，防止 cssRules 等錯誤
 */
import { useState, useRef, useEffect } from 'react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import 'temporal-polyfill/global'
import FullCalendar from '@fullcalendar/react'
import monarchThemePlugin from '@fullcalendar/react/themes/monarch'
import dayGridPlugin from '@fullcalendar/react/daygrid'
import timeGridPlugin from '@fullcalendar/react/timegrid'
import interactionPlugin from '@fullcalendar/react/interaction'
import zhTwLocale from '@fullcalendar/react/locales/zh-tw'
import type { EventClickInfo } from '@fullcalendar/react'
import '@fullcalendar/react/skeleton.css'
import '@fullcalendar/react/themes/monarch/theme.css'
import '@fullcalendar/react/themes/monarch/palettes/blue.css'

import { uiLocale } from '@/lib/utils'
import { safeHref } from '@/lib/sanitize'
import { LEAVE_TYPE_COLORS } from '@/hooks/useCalendarEvents'
import { parseEventTitle, stripUnconfirmedSuffix } from './calendarEventTitle'
import type { FullCalendarEvent } from '@/hooks/useCalendarEvents'

interface CalendarViewProps {
    events: FullCalendarEvent[]
    onDatesSet: (dateInfo: { start: Date; end: Date }) => void
    /** 換月 fetch 進行中（有舊資料保留），顯示右上角小 spinner */
    isFetching?: boolean
}

/** 事件預覽彈出框的狀態 */
interface EventPopover {
    rawTitle: string
    leaveType?: string
    employeeName: string
    agentName?: string
    color?: string
    description?: string
    location?: string
    htmlLink?: string
    /** 以下為原生請假行事曆專用；Google 同步事件不會有 */
    /** 不足整日的假的時數（如 `4.5h`）；整天假為 undefined */
    hours?: string
    pendingStage?: string
    proxyConfirmed?: boolean
    departmentName?: string
    start: string
    end: string
    allDay: boolean
    x: number
    y: number
}

/** 格式化日期（台北時區，月/日） */
function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString(uiLocale(), {
        timeZone: 'Asia/Taipei',
        month: 'numeric',
        day: 'numeric',
    })
}

/** 格式化時間（台北時區，HH:MM） */
function formatTime(dateStr: string) {
    return new Date(dateStr).toLocaleTimeString(uiLocale(), {
        timeZone: 'Asia/Taipei',
        hour: '2-digit',
        minute: '2-digit',
    })
}

/** 格式化 popover 的時間顯示 */
function formatPopoverTime(t: TFunction, start: string, end: string, allDay: boolean): string {
    try {
        if (allDay) {
            const startDate = formatDate(start)
            // FullCalendar 全天事件的 end 是「下一天」，需減一天顯示
            const endMs = new Date(end).getTime() - 86400000
            const endDate = formatDate(new Date(endMs).toISOString())
            return t('hrPages.calendar.view.allDayRange', {
                range: startDate === endDate ? startDate : `${startDate} – ${endDate}`,
            })
        }
        const startFmt = `${formatDate(start)} ${formatTime(start)}`
        const endFmt = `${formatDate(end)} ${formatTime(end)}`
        // 同一天的時段只顯示一次日期
        if (formatDate(start) === formatDate(end)) {
            return `${formatDate(start)}  ${formatTime(start)} – ${formatTime(end)}`
        }
        return `${startFmt} – ${endFmt}`
    } catch {
        return `${start} – ${end}`
    }
}

export function CalendarView({ events, onDatesSet, isFetching = false }: CalendarViewProps) {
    const { t, i18n } = useTranslation()
    const [popover, setPopover] = useState<EventPopover | null>(null)
    const popoverRef = useRef<HTMLDivElement>(null)

    // 點擊頁面其他位置時關閉 popover
    useEffect(() => {
        if (!popover) return
        const handleClickOutside = (e: MouseEvent) => {
            if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
                setPopover(null)
            }
        }
        document.addEventListener('mousedown', handleClickOutside)
        return () => document.removeEventListener('mousedown', handleClickOutside)
    }, [popover])

    const handleEventClick = (info: EventClickInfo) => {
        info.jsEvent.preventDefault()
        const rect = info.el.getBoundingClientRect()
        const rawTitle = info.event.title
        const { leaveType, employeeName, agentName, hours } = parseEventTitle(rawTitle)
        const color = leaveType ? LEAVE_TYPE_COLORS[leaveType] : undefined

        setPopover({
            rawTitle,
            leaveType,
            employeeName,
            agentName: stripUnconfirmedSuffix(agentName),
            hours,
            color,
            description: info.event.extendedProps.description as string | undefined,
            location: info.event.extendedProps.location as string | undefined,
            htmlLink: info.event.extendedProps.htmlLink as string | undefined,
            pendingStage: info.event.extendedProps.pendingStage as string | undefined,
            proxyConfirmed: info.event.extendedProps.proxyConfirmed as boolean | undefined,
            departmentName: info.event.extendedProps.departmentName as string | undefined,
            start: info.event.startStr,
            end: info.event.endStr,
            allDay: info.event.allDay,
            x: rect.left + rect.width / 2,
            y: rect.bottom + 4,
        })
    }

    return (
        <div className="calendar-wrapper relative">
            {/* 換月期間的微小 loading 指示，不遮擋日曆本體 */}
            {isFetching && (
                <div className="absolute top-2 right-2 z-10">
                    <div className="h-3 w-3 rounded-full border-2 border-primary border-t-transparent animate-spin" />
                </div>
            )}

            <FullCalendar
                plugins={[monarchThemePlugin, dayGridPlugin, timeGridPlugin, interactionPlugin]}
                initialView="dayGridMonth"
                headerToolbar={{
                    left: 'prev,next today',
                    center: 'title',
                    right: 'dayGridMonth,timeGridWeek,timeGridDay',
                }}
                locales={[zhTwLocale]}
                // 英文是 FullCalendar 內建語系（不需註冊）；工具列／星期／月份文字隨 UI 語言切換
                locale={i18n.language?.startsWith('en') ? 'en' : 'zh-tw'}
                events={events}
                datesSet={onDatesSet}
                eventClick={handleEventClick}
                height="auto"
                dayMaxEvents={3}
            />

            {/* 事件預覽 Popover */}
            {popover && (
                <div
                    ref={popoverRef}
                    className="fixed z-50 bg-popover border rounded-lg shadow-lg p-3 min-w-[230px] max-w-[320px] animate-in fade-in-0 zoom-in-95"
                    style={{
                        left: Math.min(popover.x, window.innerWidth - 340),
                        top: Math.min(popover.y, window.innerHeight - 220),
                    }}
                >
                    <div className="space-y-2">
                        {/* 假別 badge + 員工名 */}
                        <div className="flex items-start gap-2">
                            {popover.leaveType ? (
                                <>
                                    <span
                                        className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium text-white shrink-0 mt-0.5"
                                        style={{ backgroundColor: popover.color }}
                                    >
                                        {popover.leaveType}
                                    </span>
                                    <span className="font-medium text-sm leading-snug">
                                        {popover.employeeName}
                                    </span>
                                </>
                            ) : (
                                <span className="font-medium text-sm leading-snug">{popover.rawTitle}</span>
                            )}
                        </div>

                        {/* 部門（原生請假行事曆才有） */}
                        {popover.departmentName && (
                            <div className="text-xs text-muted-foreground">
                                {popover.departmentName}
                            </div>
                        )}

                        {/* 審核關卡：未核准的假單必須看得出還在等誰，
                            否則使用者會把半透明色塊誤讀成已定案的假 */}
                        {popover.pendingStage && (
                            <div className="text-xs">
                                <span className="inline-flex items-center px-2 py-0.5 rounded-full border border-dashed border-amber-500 text-amber-700 dark:text-amber-400">
                                    {popover.pendingStage}
                                </span>
                            </div>
                        )}

                        {/* 代理人 */}
                        {popover.agentName && (
                            <div className="text-xs text-muted-foreground">
                                {t('hrPages.calendar.view.delegate', { name: popover.agentName })}
                                {popover.proxyConfirmed === false && (
                                    <span className="ml-1 text-amber-600 dark:text-amber-400">
                                        {t('hrPages.calendar.view.notYetConfirmed')}
                                    </span>
                                )}
                            </div>
                        )}

                        {/* 時間。不足整日的假沒有起訖時段可畫，只能把時數標出來，
                            否則半天假與整天假在 popover 上長得一模一樣 */}
                        <div className="text-xs text-muted-foreground">
                            {formatPopoverTime(t, popover.start, popover.end, popover.allDay)}
                            {popover.hours && <span className="ml-1">{t('hrPages.calendar.view.totalHours', { hours: popover.hours })}</span>}
                        </div>

                        {/* 地點 */}
                        {popover.location && (
                            <div className="text-xs text-muted-foreground flex items-start gap-1">
                                <span className="shrink-0">📍</span>
                                <span>{popover.location}</span>
                            </div>
                        )}

                        {/* 描述 */}
                        {popover.description && (
                            <div className="text-xs text-muted-foreground border-t pt-2 mt-2 line-clamp-3">
                                {popover.description}
                            </div>
                        )}

                        {/* 操作按鈕列 */}
                        <div className="flex items-center justify-end gap-2 pt-1">
                            {safeHref(popover.htmlLink) && (
                                <a
                                    href={safeHref(popover.htmlLink)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-xs text-primary hover:underline"
                                >
                                    {t('hrPages.calendar.view.openInGoogle')}
                                </a>
                            )}
                            <button
                                type="button"
                                onClick={() => setPopover(null)}
                                className="text-xs text-muted-foreground hover:text-foreground"
                            >
                                {t('common.closeDialog')}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}
