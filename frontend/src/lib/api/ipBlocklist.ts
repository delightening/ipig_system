import api from './client'

export interface IpBlocklistEntry {
    id: string
    ip_address: string
    reason: string
    source: string
    alert_id: string | null
    blocked_at: string
    blocked_until: string | null
    blocked_by: string | null
    hit_count: number
    last_hit_at: string | null
    unblocked_at: string | null
    unblocked_by: string | null
    unblocked_reason: string | null
}

export interface AddIpBlocklistRequest {
    ip_address: string
    reason: string
    ttl_hours?: number | null
}

export const ipBlocklistApi = {
    list: (params?: { only_active?: boolean; limit?: number; offset?: number }) =>
        api.get<IpBlocklistEntry[]>('/admin/audit/ip-blocklist', { params }),
    add: (data: AddIpBlocklistRequest) =>
        api.post<{ id: string }>('/admin/audit/ip-blocklist', data),
    unblock: (id: string, reason: string) =>
        api.post<{ ok: boolean }>(`/admin/audit/ip-blocklist/${id}/unblock`, { reason }),
}
