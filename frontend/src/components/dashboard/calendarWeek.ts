import { addDays, startOfISOWeek } from 'date-fns'

import type { CalendarEvent } from '@/types/hr'

/** 本週 7 天中的單日分桶（週一～週日，台北時區） */
export interface WeekDayBucket {
    /** 該日的 Date（本地 00:00），用於顯示星期與日期數字 */
    date: Date
    /** 該日台北時區日期鍵（yyyy-MM-dd），用於事件比對 */
    dateKey: string
    /** 是否為今天 */
    isToday: boolean
    /** 落在該日（依事件開始時間之台北日期）的事件，已排序 */
    events: CalendarEvent[]
}

/**
 * 取得某時間在台北時區的日期鍵（yyyy-MM-dd）。
 * 以 en-CA locale 取得 ISO 風格年月日，確保跨時區比對一致。
 */
export function taipeiDateKey(date: Date): string {
    return date.toLocaleDateString('en-CA', { timeZone: 'Asia/Taipei' })
}

/**
 * 將事件依「開始時間的台北日期」分桶到 reference 所在 ISO 週的 7 天（週一～週日）。
 * 跨日事件僅落在其開始日（dashboard 輕量週曆的簡化處理）。
 * 每日事件排序：全天事件在前，其餘依開始時間遞增。
 */
export function buildWeekDays(events: CalendarEvent[], referenceDate: Date): WeekDayBucket[] {
    const weekStart = startOfISOWeek(referenceDate)
    const todayKey = taipeiDateKey(referenceDate)
    const days: WeekDayBucket[] = Array.from({ length: 7 }, (_, i) => {
        const date = addDays(weekStart, i)
        const dateKey = taipeiDateKey(date)
        return { date, dateKey, isToday: dateKey === todayKey, events: [] }
    })

    const byKey = new Map(days.map((d) => [d.dateKey, d]))
    for (const event of events) {
        byKey.get(taipeiDateKey(new Date(event.start)))?.events.push(event)
    }

    for (const day of days) {
        day.events.sort((a, b) => {
            if (a.all_day !== b.all_day) return a.all_day ? -1 : 1
            return new Date(a.start).getTime() - new Date(b.start).getTime()
        })
    }
    return days
}
