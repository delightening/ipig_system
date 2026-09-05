import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

/**
 * CodeRabbit PR #35 第二輪：月報 demo 路由比對「是否為當月」時要用台灣時區，
 * 不能用瀏覽器本地時間——`MonthlyReportTab` 的預設月份是用 `taipeiCurrentMonth()`
 * 算的，兩邊基準不一致時，時區落後台灣的訪客（例：美西）預設查詢會被判成
 * 「非當月」而回空。
 *
 * ⚠️ 這支測試必須把 `process.env.TZ` 切到非台灣時區才測得出這個 bug——本機
 * 開發環境剛好就是 Asia/Taipei，兩種寫法在這裡會巧合地給出一樣的答案。
 */

const ORIGINAL_TZ = process.env.TZ

describe('guest-demo monthly-report 的當月判斷（時區）', () => {
    beforeEach(() => {
        // 模擬時區落後台灣的瀏覽器（美西）。
        process.env.TZ = 'America/Los_Angeles'
        vi.useFakeTimers()
        // UTC 2026-08-31T16:30 = 台灣 2026-09-01 00:30（已跨日）
        //                      = 美西 2026-08-31 09:30（仍是 8 月）
        vi.setSystemTime(new Date('2026-08-31T16:30:00Z'))
    })

    afterEach(() => {
        vi.useRealTimers()
        process.env.TZ = ORIGINAL_TZ
    })

    it('請求台灣當月（9 月）回傳 demo 資料，即使瀏覽器本地時間仍是 8 月', async () => {
        const { getGuestDemoData } = await import('../routes')
        const { DEMO_MONTHLY_REPORT } = await import('../hr')
        expect(getGuestDemoData('/hr/attendance/monthly-report?year=2026&month=9', 'GET')).toBe(
            DEMO_MONTHLY_REPORT,
        )
    })

    it('請求瀏覽器本地當月（8 月，非台灣當月）回空', async () => {
        const { getGuestDemoData } = await import('../routes')
        expect(getGuestDemoData('/hr/attendance/monthly-report?year=2026&month=8', 'GET')).toEqual([])
    })

    it('未帶 year/month 一律回 demo 資料（無關時區判斷）', async () => {
        const { getGuestDemoData } = await import('../routes')
        const { DEMO_MONTHLY_REPORT } = await import('../hr')
        expect(getGuestDemoData('/hr/attendance/monthly-report', 'GET')).toBe(DEMO_MONTHLY_REPORT)
    })
})
