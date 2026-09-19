import type { TFunction } from 'i18next'

import { parseDecimal, uiLocale } from '@/lib/utils'
import { LEAVE_STATUS_NAMES, LEAVE_TYPE_NAMES } from '@/types/hr'
import type { StatusVariant } from '@/components/ui/status-badge'

export interface CreateOvertimeData {
    overtime_date: string
    start_time: string
    end_time: string
    overtime_type: string
    reason: string
}

// 加班類型／狀態的顯示文字：模組層只存 i18n 鍵（code → 鍵），渲染時才 t()，
// 語言切換後才會跟著更新。code 本身（A/B/C/D、draft…）是送後端／比對用的值，不動。
export const OVERTIME_TYPE_LABEL_KEYS: Record<string, string> = {
    A: 'hrPages.shared.overtimeType.A',
    B: 'hrPages.shared.overtimeType.B',
    C: 'hrPages.shared.overtimeType.C',
    D: 'hrPages.shared.overtimeType.D',
}

export const OVERTIME_STATUS_LABEL_KEYS: Record<string, string> = {
    draft: 'hrPages.shared.overtimeStatus.draft',
    pending: 'hrPages.shared.overtimeStatus.pending',
    pending_admin_staff: 'hrPages.shared.overtimeStatus.pending_admin_staff',
    pending_admin: 'hrPages.shared.overtimeStatus.pending_admin',
    approved: 'hrPages.shared.overtimeStatus.approved',
    rejected: 'hrPages.shared.overtimeStatus.rejected',
    cancelled: 'hrPages.shared.overtimeStatus.cancelled',
    voided: 'hrPages.shared.overtimeStatus.voided',
}

/** 下拉選單用的 code 清單（順序沿用對照表定義） */
export const OVERTIME_TYPE_CODES = Object.keys(OVERTIME_TYPE_LABEL_KEYS)
export const OVERTIME_STATUS_CODES = Object.keys(OVERTIME_STATUS_LABEL_KEYS)

export const overtimeTypeLabel = (t: TFunction, code: string): string => {
    const key = OVERTIME_TYPE_LABEL_KEYS[code]
    return key ? t(key) : code
}

export const overtimeStatusLabel = (t: TFunction, code: string): string => {
    const key = OVERTIME_STATUS_LABEL_KEYS[code]
    return key ? t(key) : code
}

// 假別／請假狀態：code 清單與「對照表沒收錄時的後備文字」仍取自 `@/types/hr`
// （該檔不歸本頁管），顯示文字改走 i18n 鍵；日後 types 新增 code 而這裡還沒加鍵時，
// 行為與改前相同（顯示 types 的名稱，再沒有就顯示 code）。
const LEAVE_TYPE_LABEL_KEYS: Record<string, string> = {
    ANNUAL: 'hrPages.shared.leaveType.ANNUAL',
    PERSONAL: 'hrPages.shared.leaveType.PERSONAL',
    SICK: 'hrPages.shared.leaveType.SICK',
    COMPENSATORY: 'hrPages.shared.leaveType.COMPENSATORY',
    MARRIAGE: 'hrPages.shared.leaveType.MARRIAGE',
    BEREAVEMENT: 'hrPages.shared.leaveType.BEREAVEMENT',
    MATERNITY: 'hrPages.shared.leaveType.MATERNITY',
    PATERNITY: 'hrPages.shared.leaveType.PATERNITY',
    MENSTRUAL: 'hrPages.shared.leaveType.MENSTRUAL',
    OFFICIAL: 'hrPages.shared.leaveType.OFFICIAL',
}

const LEAVE_STATUS_LABEL_KEYS: Record<string, string> = {
    DRAFT: 'hrPages.shared.leaveStatus.DRAFT',
    PENDING_PROXY: 'hrPages.shared.leaveStatus.PENDING_PROXY',
    PENDING_L1: 'hrPages.shared.leaveStatus.PENDING_L1',
    PENDING_L2: 'hrPages.shared.leaveStatus.PENDING_L2',
    PENDING_HR: 'hrPages.shared.leaveStatus.PENDING_HR',
    PENDING_GM: 'hrPages.shared.leaveStatus.PENDING_GM',
    PENDING_DIRECTOR: 'hrPages.shared.leaveStatus.PENDING_DIRECTOR',
    APPROVED: 'hrPages.shared.leaveStatus.APPROVED',
    REJECTED: 'hrPages.shared.leaveStatus.REJECTED',
    CANCELLED: 'hrPages.shared.leaveStatus.CANCELLED',
    REVOKED: 'hrPages.shared.leaveStatus.REVOKED',
}

export const LEAVE_TYPE_CODES = Object.keys(LEAVE_TYPE_NAMES)
export const LEAVE_STATUS_CODES = Object.keys(LEAVE_STATUS_NAMES)

export const leaveTypeLabel = (t: TFunction, code: string): string => {
    const key = LEAVE_TYPE_LABEL_KEYS[code]
    return key ? t(key) : (LEAVE_TYPE_NAMES[code] || code)
}

export const leaveStatusLabel = (t: TFunction, code: string): string => {
    const key = LEAVE_STATUS_LABEL_KEYS[code]
    return key ? t(key) : (LEAVE_STATUS_NAMES[code] || code)
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
export const formatLeaveHours = (
    t: TFunction,
    leave: { total_hours?: number | string | null; total_days: number | string },
): string => {
    const hours = leave.total_hours != null ? parseDecimal(leave.total_hours) : parseDecimal(leave.total_days) * 8
    return t('hrPages.shared.hoursValue', { hours })
}

/**
 * 假單自送審至今的等待天數；未送審回 null。
 *
 * 實作已上移到 `lib/waitingDays.ts`——待處理人徽章要對全站在途狀態顯示同一個
 * 「已等待 N 天」，不能只服務假單。此處保留假單語意的別名，呼叫端不必改。
 */
export { getWaitingDays as getLeaveWaitingDays, getWaitingDaysClass } from '@/lib/waitingDays'

/** 取得請假狀態的 StatusBadge variant + label */
export const getLeaveStatusVariant = (t: TFunction, status: string): { variant: StatusVariant; label: string } => {
    const label = leaveStatusLabel(t, status)
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
