import { describe, expect, it } from 'vitest'

import { isoToTaipeiTimeInput, taipeiTimeInputToIso } from '../attendanceTime'

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
