export interface NotificationRouting {
    id: string
    event_type: string
    role_code: string
    channel: string
    is_active: boolean
    description: string | null
    created_at: string
    updated_at: string
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
}
