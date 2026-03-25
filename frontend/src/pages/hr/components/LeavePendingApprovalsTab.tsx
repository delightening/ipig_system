import { CheckCircle, FileText, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/utils'
import { LEAVE_TYPE_NAMES } from '@/types/hr'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours } from '../constants'

interface LeavePendingApprovalsTabProps {
    leaves: LeaveRequestWithUser[] | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    approvePending: boolean
    rejectPending: boolean
}

export function LeavePendingApprovalsTab({
    leaves,
    isLoading,
    onApprove,
    onReject,
    approvePending,
    rejectPending,
}: LeavePendingApprovalsTabProps) {
    const columns: ColumnDef<LeaveRequestWithUser>[] = [
        {
            key: 'applicant',
            header: '申請人',
            cell: (leave) => (
                <div>
                    <div className="font-medium">{leave.user_name}</div>
                    <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                </div>
            ),
        },
        {
            key: 'leave_type',
            header: '假別',
            cell: (leave) => LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type,
        },
        {
            key: 'date',
            header: '日期',
            cell: (leave) => (
                <span className="whitespace-nowrap">
                    {formatDate(leave.start_date)}
                    {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                </span>
            ),
        },
        {
            key: 'hours',
            header: '時數',
            cell: (leave) => formatLeaveHours(leave),
        },
        {
            key: 'reason',
            header: '事由',
            className: 'max-w-[200px] truncate',
            cell: (leave) => leave.reason,
        },
        {
            key: 'actions',
            header: '操作',
            className: 'text-right',
            cell: (leave) => (
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="default"
                        size="sm"
                        onClick={() => onApprove(leave.id)}
                        disabled={approvePending}
                    >
                        <CheckCircle className="h-4 w-4 mr-1" />
                        核准
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => onReject(leave.id, '不符合規定')}
                        disabled={rejectPending}
                    >
                        <XCircle className="h-4 w-4 mr-1" />
                        駁回
                    </Button>
                </div>
            ),
        },
    ]

    return (
        <Card>
            <CardHeader>
                <CardTitle>待審核請假</CardTitle>
                <CardDescription>您需要審核的請假申請</CardDescription>
            </CardHeader>
            <CardContent>
                <DataTable
                    columns={columns}
                    data={leaves}
                    isLoading={isLoading}
                    emptyIcon={FileText}
                    emptyTitle="沒有待審核的請假"
                    rowKey={(row) => row.id}
                />
            </CardContent>
        </Card>
    )
}
