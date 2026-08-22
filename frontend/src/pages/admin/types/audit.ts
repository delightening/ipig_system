export interface AuditLog {
    id: string
    actor_user_id: string
    actor_email: string
    actor_name: string
    action: string
    entity_type: string
    entity_id: string
    entity_email?: string
    entity_name?: string
    before_data?: Record<string, unknown>
    after_data?: Record<string, unknown>
    created_at: string
}

export type AlertSortField = 'created_at' | 'alert_type' | 'severity' | 'status'
export type AlertSortOrder = 'asc' | 'desc'

export interface AlertSortConfig {
    field: AlertSortField
    order: AlertSortOrder
}
