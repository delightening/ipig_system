import { FileText, Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { StatusBadge } from '@/components/ui/status-badge'
import { Card } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/utils'
import { LEAVE_TYPE_NAMES } from '@/types/hr'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours, getLeaveStatusVariant } from '../constants'

interface MyLeavesTabContentProps {
    leaves: LeaveRequestWithUser[] | undefined
    isLoading: boolean
    onSubmit: (id: string) => void
    onCancel: (id: string) => void
    submitPending: boolean
    cancelPending: boolean
}

export function MyLeavesTabContent({
    leaves,
    isLoading,
    onSubmit,
    onCancel,
    submitPending,
    cancelPending,
}: MyLeavesTabContentProps) {
    const columns: ColumnDef<LeaveRequestWithUser>[] = [
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
            className: 'max-w-[200px] whitespace-normal break-words',
            cell: (leave) => leave.reason,
        },
        {
            key: 'status',
            header: '狀態',
            cell: (leave) => {
                const status = getLeaveStatusVariant(leave.status)
                return (
                    <StatusBadge variant={status.variant}>
                        {status.label}
                    </StatusBadge>
                )
            },
        },
        {
            key: 'actions',
            header: '操作',
            className: 'text-right',
            cell: (leave) => (
                <div className="flex items-center justify-end gap-1">
                    {leave.status === 'DRAFT' && (
                        <>
                            <Button
                                variant="default"
                                size="sm"
                                onClick={() => onSubmit(leave.id)}
                                disabled={submitPending}
                            >
                                <Send className="h-4 w-4 mr-1" />
                                送審
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onCancel(leave.id)}
                                disabled={cancelPending}
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                        </>
                    )}
                    {(leave.status.startsWith('PENDING') || leave.status === 'APPROVED') && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onCancel(leave.id)}
                            disabled={cancelPending}
                        >
                            取消
                        </Button>
                    )}
                </div>
            ),
        },
    ]

    return (
        <Card>
            <DataTable
                columns={columns}
                data={leaves}
                isLoading={isLoading}
                emptyIcon={FileText}
                emptyTitle="沒有請假記錄"
                rowKey={(row) => row.id}
            />
        </Card>
    )
}
