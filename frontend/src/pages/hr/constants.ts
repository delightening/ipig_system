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
}

/** Format date string to localized format */
export const formatDate = (dateStr: string): string => {
    return new Date(dateStr).toLocaleDateString('zh-TW', {
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
