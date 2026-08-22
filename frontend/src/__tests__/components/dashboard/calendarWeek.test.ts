import { describe, it, expect } from 'vitest'
import { buildWeekDays, taipeiDateKey } from '@/components/dashboard'
import type { CalendarEvent } from '@/types/hr'

function makeEvent(id: string, start: string, opts: Partial<CalendarEvent> = {}): CalendarEvent {
    return { id, summary: `event-${id}`, start, end: start, all_day: false, ...opts }
}

describe('buildWeekDays', () => {
    // 2026-06-17 為週三，所屬 ISO 週為 06-15(一)～06-21(日)
    const reference = new Date('2026-06-17T10:00:00+08:00')

    it('回傳本 ISO 週的 7 個相異且遞增的日期', () => {
        const days = buildWeekDays([], reference)
        expect(days).toHaveLength(7)
        const keys = days.map((d) => d.dateKey)
        expect(new Set(keys).size).toBe(7)
        expect([...keys].sort()).toEqual(keys)
    })

    it('恰有一天標記為今天，且對應 reference 的台北日期', () => {
        const days = buildWeekDays([], reference)
        const todays = days.filter((d) => d.isToday)
        expect(todays).toHaveLength(1)
        expect(todays[0].dateKey).toBe(taipeiDateKey(reference))
    })

    it('事件依開始時間的台北日期落入對應分桶', () => {
        const ev = makeEvent('a', '2026-06-18T09:00:00+08:00')
        const days = buildWeekDays([ev], reference)
        const bucket = days.find((d) => d.dateKey === '2026-06-18')
        expect(bucket).toBeDefined()
        expect(bucket?.events.map((e) => e.id)).toEqual(['a'])
    })

    it('落在本週以外的事件不被納入任何分桶', () => {
        const ev = makeEvent('far', '2026-07-01T09:00:00+08:00')
        const days = buildWeekDays([ev], reference)
        expect(days.flatMap((d) => d.events)).toHaveLength(0)
    })

    it('同日事件排序：全天事件在前，其餘依開始時間遞增', () => {
        const evening = makeEvent('evening', '2026-06-18T18:00:00+08:00')
        const morning = makeEvent('morning', '2026-06-18T08:00:00+08:00')
        const allDay = makeEvent('allday', '2026-06-18T00:00:00+08:00', { all_day: true })
        const days = buildWeekDays([evening, morning, allDay], reference)
        const bucket = days.find((d) => d.dateKey === '2026-06-18')
        expect(bucket?.events.map((e) => e.id)).toEqual(['allday', 'morning', 'evening'])
    })
})
