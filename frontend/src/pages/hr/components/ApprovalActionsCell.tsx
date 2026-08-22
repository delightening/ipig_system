import { CheckCircle, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface ApprovalActionsCellProps {
    id: string
    canApprove: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    approvePending: boolean
    rejectPending: boolean
    rejectReason?: string
}

export function ApprovalActionsCell({
    id,
    canApprove,
    onApprove,
    onReject,
    approvePending,
    rejectPending,
    rejectReason = '不符合規定',
}: ApprovalActionsCellProps) {
    if (!canApprove) {
        return (
            <span className="text-muted-foreground">
                <span aria-hidden="true">—</span>
                <span className="sr-only">無核准權限</span>
            </span>
        )
    }

    return (
        <div className="flex items-center justify-end gap-1">
            <Button
                variant="default"
                size="sm"
                onClick={() => onApprove(id)}
                disabled={approvePending}
            >
                <CheckCircle className="h-4 w-4 mr-1" />
                核准
            </Button>
            <Button
                variant="destructive"
                size="sm"
                onClick={() => onReject(id, rejectReason)}
                disabled={rejectPending}
            >
                <XCircle className="h-4 w-4 mr-1" />
                駁回
            </Button>
        </div>
    )
}
