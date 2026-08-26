import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

/**
 * CodeRabbit PR #69 review：`submitted_at` 來自後端 `DateTime<Utc>`（型別保證，
 * 見 `getLeaveWaitingDays` 的函式註解），理論上不會是無法解析的字串，但仍要對
 * 「萬一真的發生」的情況鎖住行為——不得丟出例外把整個表格弄壞，且要留下記錄
 * 而不是完全靜默。
 */

vi.mock('@/lib/logger', () => ({ logger: { warn: vi.fn(), log: vi.fn(), error: vi.fn() } }))

const { getLeaveWaitingDays } = await import('../constants')
const { logger } = await import('@/lib/logger')

describe('getLeaveWaitingDays', () => {
    beforeEach(() => {
        vi.useFakeTimers()
        vi.setSystemTime(new Date('2026-08-10T12:00:00Z'))
        vi.mocked(logger.warn).mockClear()
    })

    afterEach(() => {
        vi.useRealTimers()
    })

    it('未送審（null/undefined）回傳 null，不記錄', () => {
        expect(getLeaveWaitingDays(null)).toBeNull()
        expect(getLeaveWaitingDays(undefined)).toBeNull()
        expect(logger.warn).not.toHaveBeenCalled()
    })

    it('合法 ISO 時間戳正確算出天數', () => {
        expect(getLeaveWaitingDays('2026-08-05T03:00:00Z')).toBe(5)
    })

    it('無法解析的字串回傳 null，且記錄警告（不是完全靜默）', () => {
        expect(getLeaveWaitingDays('not-a-date')).toBeNull()
        expect(logger.warn).toHaveBeenCalledTimes(1)
        // 實作已上移 lib/waitingDays.ts，log 前綴隨之改為泛用名稱（本別名仍是 HR 的入口）
        expect(vi.mocked(logger.warn).mock.calls[0][0]).toContain('getWaitingDays')
        expect(vi.mocked(logger.warn).mock.calls[0][1]).toBe('not-a-date')
    })
})
