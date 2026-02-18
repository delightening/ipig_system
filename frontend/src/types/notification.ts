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

// ============================================
// 通知路由規則
// ============================================

export interface NotificationRouting {
    id: string
    event_type: string
    role_code: string
    channel: string       // 'in_app' | 'email' | 'both'
    is_active: boolean
    description?: string
    created_at: string
    updated_at: string
}

export interface CreateNotificationRoutingRequest {
    event_type: string
    role_code: string
    channel?: string
    description?: string
}

export interface UpdateNotificationRoutingRequest {
    channel?: string
    is_active?: boolean
    description?: string
}

/** 事件類型資訊 */
export interface EventTypeInfo {
    code: string
    name: string
}

/** 事件類型分類 */
export interface EventTypeCategory {
    category: string
    event_types: EventTypeInfo[]
}

/** 角色資訊 */
export interface RoleInfo {
    code: string
    name: string
}

/** 事件類型中文名稱對照 */
export const eventTypeNames: Record<string, string> = {
    protocol_submitted: '計畫提交',
    protocol_vet_review: '獸醫審查',
    protocol_under_review: '委員審查',
    protocol_resubmitted: '重新提交',
    protocol_approved: '計畫核准',
    protocol_rejected: '計畫駁回',
    review_comment_created: '新審查意見',
    all_reviews_completed: '所有審查意見送出',
    all_comments_resolved: '所有意見已解決',
    leave_submitted: '請假申請',
    overtime_submitted: '加班申請',
    leave_approved: '請假核准',
    overtime_approved: '加班核准',
    document_submitted: '採購單提交',
    low_stock_alert: '低庫存預警',
    expiry_alert: '效期預警',
    emergency_medication: '緊急給藥',
    animal_abnormal_record: '動物異常紀錄',
    vet_recommendation_created: '獸醫師建議',
    animal_sudden_death: '動物猝死',
    euthanasia_order_created: '安樂死申請',
    amendment_submitted: '修正案提交',
    amendment_decision_recorded: '修正案審查決定',
    amendment_approved: '修正案核准',
    amendment_rejected: '修正案駁回',
}

/** 通道中文名稱對照 */
export const channelNames: Record<string, string> = {
    in_app: '站內通知',
    email: 'Email',
    both: '兩者',
}
