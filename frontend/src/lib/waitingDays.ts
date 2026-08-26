import { logger } from '@/lib/logger'
import { TAIWAN_TIMEZONE } from '@/lib/utils'

/** 以台灣時區的「日」為單位取日期鍵（YYYY-MM-DD），避免跨時區差一天 */
const taipeiDayKey = (date: Date): string =>
    new Intl.DateTimeFormat('en-CA', {
        timeZone: TAIWAN_TIMEZONE,
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
    }).format(date)

/**
 * 自某個時間點至今的等待天數；無時間點回 null。
 *
 * 原本是 `pages/hr/constants.ts` 的 `getLeaveWaitingDays`（只服務假單），
 * 待處理人徽章要對全站在途狀態顯示「已等待 N 天」，故上移到 `lib/`。
 *
 * `since` 來自後端的 `DateTime<Utc>`（`sqlx::query_as` 直接對映，沒有字串拼接路徑），
 * 型別系統保證這裡收到的值不是 `Invalid Date` 就是後端真的有 bug。仍保留
 * `Number.isNaN` 防呆並記錄，是為了「萬一真的發生」時看得見，而不是假設它會發生。
 */
export const getWaitingDays = (since: string | null | undefined): number | null => {
    if (!since) return null
    const from = new Date(since)
    if (Number.isNaN(from.getTime())) {
        logger.warn('[getWaitingDays] 收到無法解析的時間值，已略過顯示：', since)
        return null
    }
    const fromMs = Date.parse(`${taipeiDayKey(from)}T00:00:00Z`)
    const toMs = Date.parse(`${taipeiDayKey(new Date())}T00:00:00Z`)
    return Math.max(0, Math.round((toMs - fromMs) / 86_400_000))
}

/** 等待天數的強調色：>14 天紅、>7 天橘，其餘維持次要文字色 */
export const getWaitingDaysClass = (days: number): string => {
    if (days > 14) return 'text-status-error-text font-medium'
    if (days > 7) return 'text-status-warning-text font-medium'
    return 'text-muted-foreground'
}
