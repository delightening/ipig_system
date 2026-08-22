import api from './client'

/**
 * 帳號鎖定狀態。
 *
 * 鎖定沒有持久旗標——後端每次登入現算「視窗內失敗次數 >= 門檻」，
 * 故 `unlock_at` 是推算出來的自動解鎖時刻，不是資料庫欄位。
 */
export interface AccountLockoutStatus {
    email: string
    /** email 是否對應到實際帳號；帳號枚舉產生的警報會是 false */
    user_exists: boolean
    locked: boolean
    fail_count: number
    max_attempts: number
    unlock_at: string | null
    cleared_at: string | null
}

export interface IpBlockStatus {
    id: string
    ip_address: string
    reason: string
    source: string
    blocked_until: string | null
}

export interface AlertLockStatus {
    account: AccountLockoutStatus | null
    ip: IpBlockStatus | null
}

export const alertLockApi = {
    status: (alertId: string) =>
        api.get<AlertLockStatus>(`/admin/audit/alerts/${alertId}/lock-status`),
    clearAccountLockout: (email: string, reason: string) =>
        api.post<{ ok: boolean }>('/admin/audit/account-lockout/clear', { email, reason }),
}
