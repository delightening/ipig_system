/**
 * 日曆事件相關的自訂 Hook
 * 封裝事件查詢、FullCalendar 格式轉換、日期範圍管理
 */
import { useState, useCallback, useMemo } from 'react'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { format, startOfMonth, endOfMonth } from 'date-fns'
import api from '@/lib/api'
import type { CalendarEvent } from '@/types/hr'

/** FullCalendar 事件格式 */
export interface FullCalendarEvent {
    id: string
    title: string
    start: string
    end: string
    allDay: boolean
    backgroundColor?: string
    borderColor?: string
    extendedProps?: { description?: string; location?: string; htmlLink?: string }
}

/** 假別 → 顏色對照表（匯出供篩選 chips 使用） */
export const LEAVE_TYPE_COLORS: Record<string, string> = {
    '特休假': '#22c55e',
    '病假':   '#f97316',
    '事假':   '#3b82f6',
    '婚假':   '#ec4899',
    '喪假':   '#6b7280',
    '補休假': '#a855f7',
    '產假':   '#14b8a6',
    '陪產假': '#06b6d4',
    '生理假': '#f43f5e',
    '公假':   '#64748b',
}

/**
 * 從 summary 字串解析假別標籤（匯出供篩選 chips 使用）
 * summary 格式：[假別名稱] 員工名稱（代理人）
 */
export function getLeaveType(summary: string): string | undefined {
    return summary.match(/^\[(.+?)\]/)?.[1]
}

function getLeaveTypeColor(summary: string): string | undefined {
    const type = getLeaveType(summary)
    return type ? LEAVE_TYPE_COLORS[type] : undefined
}

/**
 * 管理日曆事件的查詢、格式轉換、日期範圍控制
 *
 * 設計重點：
 * - placeholderData: keepPreviousData → 換月時保留舊事件顯示，等新資料到才換，無閃爍
 * - queryKey 使用格式化字串，避免 Date 物件每次 render 都是新實例造成多餘 refetch
 */
export function useCalendarEvents(isConfigured: boolean, isActive: boolean) {
    const [calendarDateRange, setCalendarDateRange] = useState(() => ({
        start: startOfMonth(new Date()),
        end: endOfMonth(new Date()),
    }))

    const startStr = format(calendarDateRange.start, 'yyyy-MM-dd')
    const endStr = format(calendarDateRange.end, 'yyyy-MM-dd')

    const {
        data: calendarEvents,
        isLoading,
        isFetching,
    } = useQuery({
        queryKey: ['calendar-events', startStr, endStr],
        queryFn: async () => {
            const res = await api.get<CalendarEvent[]>(
                `/hr/calendar/events?start_date=${startStr}&end_date=${endStr}`
            )
            return res.data
        },
        enabled: isActive && isConfigured,
        placeholderData: keepPreviousData,
        staleTime: 5 * 60 * 1000,
    })

    /**
     * FullCalendar datesSet 回呼：字串比較去重後直接更新日期範圍
     * FullCalendar 掛載時也會觸發一次，字串相同時 state 不更新，不觸發多餘請求
     */
    const handleDatesSet = useCallback((dateInfo: { start: Date; end: Date }) => {
        const newStartStr = format(dateInfo.start, 'yyyy-MM-dd')
        setCalendarDateRange(prev => {
            if (format(prev.start, 'yyyy-MM-dd') === newStartStr) return prev
            return { start: dateInfo.start, end: dateInfo.end }
        })
    }, [])

    /** 轉換事件為 FullCalendar 格式，並依假別加上顏色 */
    const fullCalendarEvents: FullCalendarEvent[] = useMemo(() => {
        return calendarEvents?.map(event => {
            const color = getLeaveTypeColor(event.summary)
            return {
                id: event.id,
                title: event.summary,
                start: event.start,
                end: event.end,
                allDay: event.all_day,
                ...(color ? { backgroundColor: color, borderColor: color } : {}),
                extendedProps: {
                    description: event.description,
                    location: event.location,
                    htmlLink: event.html_link,
                },
            }
        }) ?? []
    }, [calendarEvents])

    return {
        calendarDateRange,
        fullCalendarEvents,
        isLoading,
        isFetching,
        handleDatesSet,
    }
}
