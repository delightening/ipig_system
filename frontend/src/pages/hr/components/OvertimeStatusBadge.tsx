import { Badge } from '@/components/ui/badge'

import { OVERTIME_STATUS_NAMES } from '../constants'

/** Get badge component for overtime status */
export function OvertimeStatusBadge({ status }: { status: string }) {
    const statusName = OVERTIME_STATUS_NAMES[status] || status
    switch (status) {
        case 'approved':
            return <Badge className="bg-green-500">{statusName}</Badge>
        case 'rejected':
            return <Badge variant="destructive">{statusName}</Badge>
        case 'cancelled':
            return <Badge variant="secondary">{statusName}</Badge>
        case 'draft':
            return <Badge variant="outline">{statusName}</Badge>
        case 'pending_admin_staff':
            return <Badge className="bg-yellow-500">{statusName}</Badge>
        case 'pending_admin':
            return <Badge className="bg-orange-500">{statusName}</Badge>
        default:
            return <Badge>{statusName}</Badge>
    }
}
