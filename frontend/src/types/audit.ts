/**
 * 稽核日誌型別
 *
 * 注意：UserActivityLog 已定義在 ./hr.ts
 */

export interface AuditLogWithActor {
    id: string
    actor_user_id: string
    actor_name: string
    action: string
    entity_type: string
    entity_id: string
    before_data?: Record<string, unknown>
    after_data?: Record<string, unknown>
    created_at: string
}
