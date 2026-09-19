import { describe, expect, it } from 'vitest'

import { attendanceTimesToIso, isoToTaipeiTimeInput, taipeiTimeInputToIso } from '../attendanceTime'

/**
 * 補卡表單的時間換算。
 *
 * `attendance_records` 存 UTC、表單填台灣時間，兩邊差 8 小時。
 * 這組測試釘住那個 offset：換算方向寫反的話，補的卡會整整早／晚 8 小時，
 * 而畫面上看起來完全正常（因為顯示時又轉了一次回台灣時間）。
 */
describe('taipeiTimeInputToIso', () => {
    it('把台灣時間換算成 UTC（減 8 小時）', () => {
        expect(taipeiTimeInputToIso('2026-08-26', '08:30')).toBe('2026-08-26T00:30:00.000Z')
        expect(taipeiTimeInputToIso('2026-08-26', '17:30')).toBe('2026-08-26T09:30:00.000Z')
    })

    it('台灣時間的凌晨會落到前一天的 UTC 日期', () => {
        expect(taipeiTimeInputToIso('2026-08-26', '07:00')).toBe('2026-08-25T23:00:00.000Z')
    })

    it('任一欄為空回 null（代表那一邊不填）', () => {
        expect(taipeiTimeInputToIso('', '08:30')).toBeNull()
        expect(taipeiTimeInputToIso('2026-08-26', '')).toBeNull()
    })

    it('無法解析的輸入回 null，不回 Invalid Date 字串', () => {
        expect(taipeiTimeInputToIso('not-a-date', '08:30')).toBeNull()
    })
})

describe('isoToTaipeiTimeInput', () => {
    it('把 UTC 換算回台灣時間的 HH:mm（加 8 小時）', () => {
        expect(isoToTaipeiTimeInput('2026-08-26T00:30:00Z')).toBe('08:30')
        expect(isoToTaipeiTimeInput('2026-08-26T09:30:00Z')).toBe('17:30')
    })

    it('跨日：UTC 前一天深夜＝台灣當天凌晨', () => {
        expect(isoToTaipeiTimeInput('2026-08-25T23:00:00Z')).toBe('07:00')
    })

    it('空值或壞值回空字串（表單顯示為未填）', () => {
        expect(isoToTaipeiTimeInput(null)).toBe('')
        expect(isoToTaipeiTimeInput('garbage')).toBe('')
    })

    it('與 taipeiTimeInputToIso 互為反向', () => {
        const iso = taipeiTimeInputToIso('2026-08-26', '13:45')
        expect(iso).not.toBeNull()
        expect(isoToTaipeiTimeInput(iso)).toBe('13:45')
    })
})

/**
 * 夜班的下班時間屬於次日（CodeRabbit 於 PR #35 指出）。
 *
 * 修好之前：22:00→06:00 兩邊都掛 work_date，換算出的下班比上班早 16 小時，
 * 後端「下班必須晚於上班」直接擋掉——**夜班根本補登不了**。
 * 這組測試釘住跨日判定，並且刻意同時斷言「白班不受影響」，
 * 免得日後有人把跨日條件放寬成無條件 +1 天。
 */
describe('attendanceTimesToIso', () => {
    it('夜班：下班早於上班 → 下班落到次日', () => {
        const { clockInIso, clockOutIso } = attendanceTimesToIso('2026-08-25', '22:00', '06:00')
        expect(clockInIso).toBe('2026-08-25T14:00:00.000Z')
        // 台灣 8/26 06:00 = UTC 8/25 22:00，比上班晚 8 小時
        expect(clockOutIso).toBe('2026-08-25T22:00:00.000Z')
        expect(new Date(clockOutIso!).getTime()).toBeGreaterThan(new Date(clockInIso!).getTime())
    })

    it('白班：下班晚於上班 → 兩邊同一天，行為不變', () => {
        const { clockInIso, clockOutIso } = attendanceTimesToIso('2026-08-25', '08:30', '17:30')
        expect(clockInIso).toBe('2026-08-25T00:30:00.000Z')
        expect(clockOutIso).toBe('2026-08-25T09:30:00.000Z')
    })

    it('跨月夜班：次日換算跨過月底', () => {
        const { clockOutIso } = attendanceTimesToIso('2026-08-31', '23:00', '07:00')
        // 台灣 9/1 07:00 = UTC 8/31 23:00
        expect(clockOutIso).toBe('2026-08-31T23:00:00.000Z')
    })

    it('上班未填時不判跨日——沒有比較基準', () => {
        const { clockInIso, clockOutIso } = attendanceTimesToIso('2026-08-25', '', '06:00')
        expect(clockInIso).toBeNull()
        expect(clockOutIso).toBe('2026-08-24T22:00:00.000Z')
    })

    it('下班未填回 null，不會誤判成跨日', () => {
        const { clockInIso, clockOutIso } = attendanceTimesToIso('2026-08-25', '09:00', '')
        expect(clockInIso).toBe('2026-08-25T01:00:00.000Z')
        expect(clockOutIso).toBeNull()
    })

    it('上下班同一分鐘不算跨日（交給後端擋，不在這裡多加一份規則）', () => {
        const { clockOutIso } = attendanceTimesToIso('2026-08-25', '09:00', '09:00')
        expect(clockOutIso).toBe('2026-08-25T01:00:00.000Z')
    })
})
