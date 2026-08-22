import { Mail, MessageSquare, Radio } from 'lucide-react'

import type { NotificationRouting } from './types'

export const channelOptions = [
    { value: 'in_app', label: '站內通知', icon: MessageSquare },
    { value: 'email', label: 'Email', icon: Mail },
    { value: 'both', label: '站內 + Email', icon: Radio },
] as const

export const GROUP_KEYS = ['AUP', 'Animal', 'ERP', 'HR', 'Equipment'] as const

export type GroupKey = (typeof GROUP_KEYS)[number]

/** 關係型 resolver key → 人類可讀標籤（對齊後端 resolvers.rs::resolver_meta）。 */
export const resolverNameMap: Record<string, string> = {
    protocol_pi_sd: '計畫 PI / 計劃負責人(SD)',
    protocol_pi: '計畫 PI',
    assigned_reviewers: '被指派審查委員',
    event_subject: '當事人本人',
    leave_request_approvers: '核准經手人',
}

/**
 * 規則的收件人來源顯示標籤：
 * - resolver 型 → resolver 標籤（關係型，由事件動態決定）。
 * - role 型 → 角色中文名（fallback 為代碼）。
 */
export function recipientLabel(
    rule: NotificationRouting,
    roleNameMap: Record<string, string>,
): string {
    if (rule.target_kind === 'resolver') {
        return resolverNameMap[rule.target_value] || rule.target_value
    }
    return roleNameMap[rule.target_value] || rule.target_value
}
