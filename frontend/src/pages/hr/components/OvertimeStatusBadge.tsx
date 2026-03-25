import { StatusBadge } from '@/components/ui/status-badge'

import { OVERTIME_STATUS_NAMES } from '../constants'

/** Get badge component for overtime status */
export function OvertimeStatusBadge({ status }: { status: string }) {
    const statusName = OVERTIME_STATUS_NAMES[status] || status
    switch (status) {
        case 'approved':
            return <StatusBadge variant="success">{statusName}</StatusBadge>
        case 'rejected':
            return <StatusBadge variant="error">{statusName}</StatusBadge>
        case 'cancelled':
            return <StatusBadge variant="neutral">{statusName}</StatusBadge>
        case 'draft':
            return <StatusBadge variant="info">{statusName}</StatusBadge>
        case 'pending_admin_staff':
            return <StatusBadge variant="warning">{statusName}</StatusBadge>
        case 'pending_admin':
            return <StatusBadge variant="warning">{statusName}</StatusBadge>
        default:
            return <StatusBadge variant="neutral">{statusName}</StatusBadge>
    }
}
