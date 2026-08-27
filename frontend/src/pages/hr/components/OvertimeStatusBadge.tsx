import { PendingOwnerBadge } from '@/components/PendingOwnerBadge'
import { StatusBadge } from '@/components/ui/status-badge'
import type { PendingOwner } from '@/types/pendingOwner'

import { OVERTIME_STATUS_NAMES } from '../constants'

interface OvertimeStatusBadgeProps {
    status: string
    /**
     * 這筆卡在誰手上（後端 `pending_owner`，僅待審中有值）。
     * 傳了就自動附 hover 說明；沒傳（例：只拿得到 status 的呼叫端）行為與從前相同。
     */
    pendingOwner?: PendingOwner | null
}

/** Get badge component for overtime status */
export function OvertimeStatusBadge({ status, pendingOwner }: OvertimeStatusBadgeProps) {
    const statusName = OVERTIME_STATUS_NAMES[status] || status
    const badge = (() => {
        switch (status) {
            case 'approved':
                return <StatusBadge variant="success">{statusName}</StatusBadge>
            case 'rejected':
                return <StatusBadge variant="error">{statusName}</StatusBadge>
            case 'cancelled':
            case 'voided':
                return <StatusBadge variant="neutral">{statusName}</StatusBadge>
            case 'draft':
                return <StatusBadge variant="info">{statusName}</StatusBadge>
            case 'pending_admin_staff':
            case 'pending_admin':
                return <StatusBadge variant="warning">{statusName}</StatusBadge>
            default:
                return <StatusBadge variant="neutral">{statusName}</StatusBadge>
        }
    })()

    return <PendingOwnerBadge owner={pendingOwner}>{badge}</PendingOwnerBadge>
}
