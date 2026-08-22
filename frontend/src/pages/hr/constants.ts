import { parseDecimal, TAIWAN_TIMEZONE, uiLocale } from '@/lib/utils'
import { logger } from '@/lib/logger'
import { LEAVE_STATUS_NAMES } from '@/types/hr'
import type { StatusVariant } from '@/components/ui/status-badge'

export interface CreateOvertimeData {
    overtime_date: string
    start_time: string
    end_time: string
    overtime_type: string
    reason: string
}

export const OVERTIME_TYPE_NAMES: Record<string, string> = {
    A: '平日加班',
    B: '假日加班',
    C: '國定假日加班',
    D: '天災加班',
}

export const OVERTIME_STATUS_NAMES: Record<string, string> = {
    draft: '草稿',
    pending: '待審核',
    pending_admin_staff: '待行政審核',
    pending_admin: '待負責人審核',
    approved: '已核准',
    rejected: '已駁回',
    cancelled: '已取消',
    voided: '已作廢',
}

/** Format date string to localized format */
export const formatDate = (dateStr: string): string => {
    return new Date(dateStr).toLocaleDateString(uiLocale(), {
        timeZone: 'Asia/Taipei',
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        weekday: 'long',
    })
}

/**
 * Calculate estimated comp time hours
 * C (國定假日) and D (天災) fixed 8 hours comp time
 * A and B have no comp time
 */
export const calculateCompTime = (overtimeType: string): number => {
    if (overtimeType === 'C' || overtimeType === 'D') return 8.0
    return 0
}

/** 從開始/結束時間計算加班時數，以 0.5 小時為單位四捨五入 */
export const calculateOvertimeHours = (start: string, end: string): number => {
    const [sh, sm] = start.split(':').map(Number)
    const [eh, em] = end.split(':').map(Number)
    const minutes = (eh * 60 + em) - (sh * 60 + sm)
    const raw = minutes / 60
    return Math.round(raw * 2) / 2
}

// ============================================
// Leave helpers
// ============================================

/** 顯示請假時數（以 0.5 小時為單位，total_hours 優先） */
export const formatLeaveHours = (leave: { total_hours?: number | string | null; total_days: number | string }): string => {
    const hours = leave.total_hours != null ? parseDecimal(leave.total_hours) : parseDecimal(leave.total_days) * 8
    return `${hours} 小時`
}

/** 以台灣時區的「日」為單位取日期鍵（YYYY-MM-DD），避免跨時區差一天 */
const taipeiDayKey = (date: Date): string =>
    new Intl.DateTimeFormat('en-CA', {
        timeZone: TAIWAN_TIMEZONE,
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
    }).format(date)

/**
 * 假單自送審至今的等待天數；未送審回 null。
 *
 * `submittedAt` 來自 `leave_requests.submitted_at`（TIMESTAMPTZ，見 migrations/008），
 * 後端以 `sqlx::query_as` 直接對映到 `DateTime<Utc>`——沒有任何原始字串拼接路徑，
 * 型別系統保證這裡收到的值不是 `Invalid Date` 就是後端真的有 bug。仍保留
 * `Number.isNaN` 防呆並記錄，是為了「萬一真的發生」時看得見，而不是假設它會發生。
 */
export const getLeaveWaitingDays = (submittedAt: string | null | undefined): number | null => {
    if (!submittedAt) return null
    const submitted = new Date(submittedAt)
    if (Number.isNaN(submitted.getTime())) {
        logger.warn('[getLeaveWaitingDays] 收到無法解析的 submitted_at，已略過顯示：', submittedAt)
        return null
    }
    const from = Date.parse(`${taipeiDayKey(submitted)}T00:00:00Z`)
    const to = Date.parse(`${taipeiDayKey(new Date())}T00:00:00Z`)
    return Math.max(0, Math.round((to - from) / 86_400_000))
}

/** 等待天數的強調色：>14 天紅、>7 天橘，其餘維持次要文字色 */
export const getWaitingDaysClass = (days: number): string => {
    if (days > 14) return 'text-status-error-text font-medium'
    if (days > 7) return 'text-status-warning-text font-medium'
    return 'text-muted-foreground'
}

/** 取得請假狀態的 StatusBadge variant + label */
export const getLeaveStatusVariant = (status: string): { variant: StatusVariant; label: string } => {
    const label = LEAVE_STATUS_NAMES[status] || status
    switch (status) {
        case 'APPROVED':
            return { variant: 'success', label }
        case 'REJECTED':
            return { variant: 'error', label }
        case 'CANCELLED':
            return { variant: 'neutral', label }
        case 'DRAFT':
            return { variant: 'info', label }
        default:
            return { variant: 'warning', label }
    }
}
