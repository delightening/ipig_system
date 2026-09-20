import type { TFunction } from 'i18next'
import { Mail, MessageSquare, Radio } from 'lucide-react'

import type { NotificationRouting } from './types'

/** 通知管道選項；`labelKey` 為 i18n 鍵，渲染時才 `t(labelKey)`（避免模組級常數凍結語言）。 */
export const channelOptions = [
    { value: 'in_app', labelKey: 'adminOps.notificationRouting.channels.inApp', icon: MessageSquare },
    { value: 'email', labelKey: 'adminOps.notificationRouting.channels.email', icon: Mail },
    { value: 'both', labelKey: 'adminOps.notificationRouting.channels.both', icon: Radio },
] as const

export const GROUP_KEYS = ['AUP', 'Animal', 'ERP', 'HR', 'Equipment'] as const

export type GroupKey = (typeof GROUP_KEYS)[number]

/** 關係型 resolver key → 人類可讀標籤的 i18n 鍵（對齊後端 resolvers.rs::resolver_meta）。 */
export const resolverLabelKeys: Record<string, string> = {
    protocol_pi_sd: 'adminOps.notificationRouting.resolvers.protocolPiSd',
    protocol_pi: 'adminOps.notificationRouting.resolvers.protocolPi',
    assigned_reviewers: 'adminOps.notificationRouting.resolvers.assignedReviewers',
    event_subject: 'adminOps.notificationRouting.resolvers.eventSubject',
    leave_request_approvers: 'adminOps.notificationRouting.resolvers.leaveRequestApprovers',
}

/**
 * 規則的收件人來源顯示標籤：
 * - resolver 型 → resolver 標籤（關係型，由事件動態決定）。
 * - role 型 → 角色中文名（fallback 為代碼）。
 */
export function recipientLabel(
    rule: NotificationRouting,
    roleNameMap: Record<string, string>,
    t: TFunction,
): string {
    if (rule.target_kind === 'resolver') {
        const labelKey = resolverLabelKeys[rule.target_value]
        return (labelKey ? t(labelKey) : '') || rule.target_value
    }
    return roleNameMap[rule.target_value] || rule.target_value
}
