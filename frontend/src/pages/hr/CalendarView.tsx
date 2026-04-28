/**
 * FullCalendar 視圖元件（獨立模組，僅在日曆已連接時動態載入）
 * 避免在未設定日曆時載入 FullCalendar，防止 cssRules 等錯誤
 */
import { useState, useRef, useEffect } from 'react'
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import interactionPlugin from '@fullcalendar/interaction'
import type { EventClickArg } from '@fullcalendar/core'
import { LEAVE_TYPE_COLORS } from '@/hooks/useCalendarEvents'
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
    start: string
    end: string
    allDay: boolean
    x: number
    y: number
}

/**
 * 解析事件標題為結構化資訊
 * 格式：[假別] 員工名（代理人）  或  [假別] 員工名
 * 括號為全形中文括號 （）
 */
function parseEventTitle(title: string): { leaveType?: string; employeeName: string; agentName?: string } {
    const match = title.match(/^\[(.+?)\]\s*(.+?)(?:（(.+?)）\s*)?$/)
    if (match) {
        return {
            leaveType: match[1],
            employeeName: match[2].trim(),
            agentName: match[3]?.trim(),
        }
    }
    return { employeeName: title }
}

/** 格式化日期（台北時區，月/日） */
function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString('zh-TW', {
        timeZone: 'Asia/Taipei',
        month: 'numeric',
        day: 'numeric',
    })
}

/** 格式化時間（台北時區，HH:MM） */
function formatTime(dateStr: string) {
    return new Date(dateStr).toLocaleTimeString('zh-TW', {
        timeZone: 'Asia/Taipei',
        hour: '2-digit',
        minute: '2-digit',
    })
}

/** 格式化 popover 的時間顯示 */
function formatPopoverTime(start: string, end: string, allDay: boolean): string {
    try {
        if (allDay) {
            const startDate = formatDate(start)
            // FullCalendar 全天事件的 end 是「下一天」，需減一天顯示
            const endMs = new Date(end).getTime() - 86400000
            const endDate = formatDate(new Date(endMs).toISOString())
            return startDate === endDate ? `${startDate}（全天）` : `${startDate} – ${endDate}（全天）`
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

    const handleEventClick = (info: EventClickArg) => {
        info.jsEvent.preventDefault()
        const rect = info.el.getBoundingClientRect()
        const rawTitle = info.event.title
        const { leaveType, employeeName, agentName } = parseEventTitle(rawTitle)
        const color = leaveType ? LEAVE_TYPE_COLORS[leaveType] : undefined

        setPopover({
            rawTitle,
            leaveType,
            employeeName,
            agentName,
            color,
            description: info.event.extendedProps.description as string | undefined,
            location: info.event.extendedProps.location as string | undefined,
            htmlLink: info.event.extendedProps.htmlLink as string | undefined,
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
                plugins={[dayGridPlugin, timeGridPlugin, interactionPlugin]}
                initialView="dayGridMonth"
                headerToolbar={{
                    left: 'prev,next today',
                    center: 'title',
                    right: 'dayGridMonth,timeGridWeek,timeGridDay',
                }}
                locale="zh-tw"
                buttonText={{
                    today: '今天',
                    month: '月',
                    week: '週',
                    day: '日',
                }}
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

                        {/* 代理人 */}
                        {popover.agentName && (
                            <div className="text-xs text-muted-foreground">
                                代理：{popover.agentName}
                            </div>
                        )}

                        {/* 時間 */}
                        <div className="text-xs text-muted-foreground">
                            {formatPopoverTime(popover.start, popover.end, popover.allDay)}
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
                            {popover.htmlLink && (
                                <a
                                    href={popover.htmlLink}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-xs text-primary hover:underline"
                                >
                                    在 Google Calendar 中開啟 →
                                </a>
                            )}
                            <button
                                type="button"
                                onClick={() => setPopover(null)}
                                className="text-xs text-muted-foreground hover:text-foreground"
                            >
                                關閉
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}
