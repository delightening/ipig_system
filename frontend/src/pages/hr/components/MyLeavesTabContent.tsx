import { useTranslation } from 'react-i18next'
import { FileText, Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { StatusBadge } from '@/components/ui/status-badge'
import { Card } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/utils'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours, getLeaveStatusVariant, leaveTypeLabel } from '../constants'

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
    const { t } = useTranslation()
    const columns: ColumnDef<LeaveRequestWithUser>[] = [
        {
            key: 'leave_type',
            header: t('hrPages.shared.col.leaveType'),
            cell: (leave) => leaveTypeLabel(t, leave.leave_type),
        },
        {
            key: 'date',
            header: t('hrPages.shared.col.date'),
            cell: (leave) => (
                <span className="whitespace-nowrap">
                    {formatDate(leave.start_date)}
                    {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                </span>
            ),
        },
        {
            key: 'hours',
            header: t('hrPages.shared.col.hours'),
            cell: (leave) => formatLeaveHours(t, leave),
        },
        {
            key: 'reason',
            header: t('hrPages.shared.col.reason'),
            className: 'max-w-[200px] whitespace-normal break-words',
            cell: (leave) => leave.reason,
        },
        {
            key: 'status',
            header: t('hrPages.shared.col.status'),
            cell: (leave) => {
                const status = getLeaveStatusVariant(t, leave.status)
                return (
                    <StatusBadge variant={status.variant}>
                        {status.label}
                    </StatusBadge>
                )
            },
        },
        {
            key: 'actions',
            header: t('common.actions'),
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
                                {t('hrPages.shared.action.submit')}
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
                            {t('common.cancel')}
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
                emptyTitle={t('hrPages.leaves.mine.empty')}
                rowKey={(row) => row.id}
            />
        </Card>
    )
}
