export interface NotificationRouting {
    id: string
    event_type: string
    /** 角色型規則為角色代碼；resolver 型規則為 null（過渡保留欄位）。 */
    role_code: string | null
    channel: string
    is_active: boolean
    description: string | null
    frequency: string        // 'immediate' | 'daily' | 'weekly' | 'monthly'
    hour_of_day: number      // 0-23
    day_of_week: number | null  // 0-6, weekly 時有效
    created_at: string
    updated_at: string
    /** 收件人來源種類：'role' | 'resolver'。 */
    target_kind: string
    /** role 時為角色代碼；resolver 時為 resolver key。 */
    target_value: string
}

export interface EventTypeInfo {
    code: string
    name: string
}

export interface EventTypeCategory {
    group: string
    category: string
    event_types: EventTypeInfo[]
}

export interface RoleInfo {
    code: string
    name: string
}

export interface RecipientMember {
    id: string
    display_name: string
    email: string
}

export interface RoutingRuleRecipients {
    rule_id: string
    target_kind: string
    target_value: string
    channel: string
    is_active: boolean
    label: string
    /** resolver 型：對象由事件動態決定的說明；角色型為 null。 */
    description: string | null
    /** 是否固定收件人、不可調整（resolver 型為 true）。 */
    fixed: boolean
    /** 角色型：實際持有該角色的使用者；resolver 型為空。 */
    members: RecipientMember[]
}

export interface RoutingRecipientsPreview {
    event_type: string
    rules: RoutingRuleRecipients[]
}

/** 固定通知（流程決定收件人、不經路由、不可調整）—— 唯讀 notes。 */
export interface FixedNotification {
    event_label: string
    trigger: string
    recipient_label: string
    channel: string
    group: string
    note: string
}

export interface CreateRoutingData {
    event_type: string
    role_code: string
    channel: string
    description: string
}

export interface UpdateRoutingData {
    channel?: string
    is_active?: boolean
    description?: string
    frequency?: string
    hour_of_day?: number
    day_of_week?: number | null
}
