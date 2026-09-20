/**
 * 通知型別
 */

import { createLabelMap } from '@/lib/i18nLabels'

export type NotificationType =
    | 'low_stock'
    | 'expiry_warning'
    | 'document_approval'
    | 'protocol_status'
    | 'protocol_submitted'
    | 'vet_recommendation'
    | 'system_alert'
    | 'monthly_report'
    | 'leave_approval'
    | 'review_assignment'
    | 'review_comment'

/** 通知類型 → 顯示名稱（getter 版：每次讀取才依當下語言翻譯，見 `@/lib/i18nLabels`） */
export const notificationTypeNames: Record<NotificationType, string> = createLabelMap(
    'typesLabels.notificationType',
    [
        'low_stock',
        'expiry_warning',
        'document_approval',
        'protocol_status',
        'protocol_submitted',
        'vet_recommendation',
        'system_alert',
        'monthly_report',
        'leave_approval',
        'review_assignment',
        'review_comment',
    ],
)

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
    /** 0=一般；1=緊急置頂（待辦，完成前排在清單最上方） */
    priority?: number
    /**
     * 通知的**性質**：`'info'` 一般告知 / `'action'` 待辦。
     *
     * 與 `priority` 分工，兩者都要：`kind` 是歷史事實（一旦是待辦就永遠是），
     * `priority > 0` 才代表**還沒完成**。待處理清單 ＝ `kind='action' && priority>0`；
     * 完成後離開待處理，但仍以 `kind='action'` + `priority=0` 留在鈴鐺歷史裡。
     *
     * 選填是為了相容部署期間仍在跑的舊回應。
     */
    kind?: NotificationKind
}

/** 通知性質，見 {@link NotificationItem.kind} */
export type NotificationKind = 'info' | 'action'

/**
 * 通知的兩個入口。判準寫在後端（`?entry=`），前端只說自己是誰。
 *
 * `bell` **不等於** `kind='info'`：已完成的待辦也屬鈴鐺歷史。
 */
export type NotificationEntry = 'bell' | 'todo'

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

export type NotificationFrequency = 'immediate' | 'daily' | 'weekly' | 'monthly'

export interface NotificationRouting {
    id: string
    event_type: string
    role_code: string
    channel: string       // 'in_app' | 'email' | 'both'
    is_active: boolean
    description?: string
    /** 批次通知頻率 */
    frequency: NotificationFrequency
    /** 批次通知執行小時（0-23） */
    hour_of_day: number
    /** weekly 時有效：0=週日, 1=週一 ... 6=週六 */
    day_of_week: number | null
    created_at: string
    updated_at: string
}

export interface CreateNotificationRoutingRequest {
    event_type: string
    role_code: string
    channel?: string
    description?: string
    frequency?: NotificationFrequency
    hour_of_day?: number
    day_of_week?: number | null
}

export interface UpdateNotificationRoutingRequest {
    channel?: string
    is_active?: boolean
    description?: string
    frequency?: NotificationFrequency
    hour_of_day?: number
    day_of_week?: number | null
}

// ============================================
// 效期通知範圍設定（系統層級）
// ============================================

export interface ExpiryNotificationConfig {
    id: string
    /** 提前幾天開始預警（預設 60） */
    warn_days: number
    /** 過期超過幾天後停止通知（預設 90） */
    cutoff_days: number
    /** 過期超過此天數後轉月度彙整通知；null=停用 */
    monthly_threshold_days: number | null
    updated_at: string
    updated_by: string | null
}

export interface UpdateExpiryNotificationConfigRequest {
    warn_days?: number
    cutoff_days?: number
    /** null = 停用月度模式 */
    monthly_threshold_days?: number | null
}

/** 事件類型資訊 */
export interface EventTypeInfo {
    code: string
    name: string
}

/** 事件類型分類（含主要分組 AUP | Animal | ERP | HR） */
export interface EventTypeCategory {
    group: string
    category: string
    event_types: EventTypeInfo[]
}

/** 角色資訊 */
export interface RoleInfo {
    code: string
    name: string
}

/** 事件類型名稱對照（getter 版：每次讀取才依當下語言翻譯，見 `@/lib/i18nLabels`） */
export const eventTypeNames: Record<string, string> = createLabelMap(
    'typesLabels.eventType',
    [
        'protocol_submitted',
        'protocol_vet_review',
        'protocol_under_review',
        'protocol_resubmitted',
        'protocol_approved',
        'protocol_rejected',
        'review_comment_created',
        'all_reviews_completed',
        'all_comments_resolved',
        'leave_submitted',
        'overtime_submitted',
        'leave_approved',
        'overtime_approved',
        'document_submitted',
        'low_stock_alert',
        'expiry_alert',
        'emergency_medication',
        'animal_abnormal_record',
        'vet_recommendation_created',
        'animal_sudden_death',
        'euthanasia_order_created',
        'amendment_submitted',
        'amendment_decision_recorded',
        'amendment_approved',
        'amendment_rejected',
        'leave_cancelled',
        'po_pending_receipt',
    ],
)

/** 通道名稱對照（getter 版，見 `@/lib/i18nLabels`） */
export const channelNames: Record<string, string> = createLabelMap(
    'typesLabels.channel',
    ['in_app', 'email', 'both'],
)

/** 頻率名稱對照（getter 版，見 `@/lib/i18nLabels`） */
export const frequencyNames: Record<NotificationFrequency, string> = createLabelMap(
    'typesLabels.frequency',
    ['immediate', 'daily', 'weekly', 'monthly'],
)

/** 可設定批次頻率的事件類型（非 event-driven） */
export const BATCH_EVENT_TYPES = new Set([
    'expiry_alert',
    'low_stock_alert',
    'po_pending_receipt',
    'equipment_overdue',
])
