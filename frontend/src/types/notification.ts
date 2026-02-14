/**
 * 通知型別
 */

export type NotificationType =
    | 'low_stock'
    | 'expiry_warning'
    | 'document_approval'
    | 'protocol_status'
    | 'vet_recommendation'
    | 'system_alert'
    | 'monthly_report'

export const notificationTypeNames: Record<NotificationType, string> = {
    low_stock: '低庫存預警',
    expiry_warning: '效期預警',
    document_approval: '單據審核',
    protocol_status: '計畫狀態',
    vet_recommendation: '獸醫師建議',
    system_alert: '系統通知',
    monthly_report: '月報',
}

export interface NotificationItem {
    id: string
    type: NotificationType
    title: string
    content?: string
    is_read: boolean
    read_at?: string
    related_entity_type?: string
    related_entity_id?: string
    created_at: string
}

export interface NotificationListResponse {
    data: NotificationItem[]
    total: number
    page: number
    per_page: number
}

export interface UnreadNotificationCount {
    count: number
}

export interface MarkNotificationsReadRequest {
    notification_ids: string[]
}

// 通知設定
export interface NotificationSettings {
    user_id: string
    email_low_stock: boolean
    email_expiry_warning: boolean
    email_document_approval: boolean
    email_protocol_status: boolean
    email_monthly_report: boolean
    expiry_warning_days: number
    low_stock_notify_immediately: boolean
    updated_at: string
}

export interface UpdateNotificationSettingsRequest {
    email_low_stock?: boolean
    email_expiry_warning?: boolean
    email_document_approval?: boolean
    email_protocol_status?: boolean
    email_monthly_report?: boolean
    expiry_warning_days?: number
    low_stock_notify_immediately?: boolean
}
