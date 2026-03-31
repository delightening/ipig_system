export interface NotificationRouting {
    id: string
    event_type: string
    role_code: string
    channel: string
    is_active: boolean
    description: string | null
    frequency: string        // 'immediate' | 'daily' | 'weekly' | 'monthly'
    hour_of_day: number      // 0-23
    day_of_week: number | null  // 0-6, weekly 時有效
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
    frequency?: string
    hour_of_day?: number
    day_of_week?: number | null
}
