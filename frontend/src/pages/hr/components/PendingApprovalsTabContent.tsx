import { useTranslation } from 'react-i18next'
import { Clock } from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { parseDecimal } from '@/lib/utils'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDate } from '../constants'
import { ApprovalActionsCell } from './ApprovalActionsCell'

interface PendingApprovalsTabContentProps {
    pendingData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    isApproving: boolean
    isRejecting: boolean
}

export function PendingApprovalsTabContent({
    pendingData,
    isLoading,
    onApprove,
    onReject,
    isApproving,
    isRejecting,
}: PendingApprovalsTabContentProps) {
    const { t } = useTranslation()
    const columns: ColumnDef<OvertimeWithUser>[] = [
        {
            key: 'applicant',
            header: t('hrPages.shared.col.applicant'),
            cell: (ot) => (
                <div>
                    <div className="font-medium">{ot.user_name}</div>
                    <div className="text-sm text-muted-foreground">{ot.user_email}</div>
                </div>
            ),
        },
        {
            key: 'date',
            header: t('hrPages.shared.col.date'),
            cell: (ot) => (
                <span className="whitespace-nowrap">{formatDate(ot.overtime_date)}</span>
            ),
        },
        {
            key: 'time',
            header: t('hrPages.shared.col.time'),
            cell: (ot) => `${ot.start_time} ~ ${ot.end_time}`,
        },
        {
            key: 'hours',
            header: t('hrPages.overtime.hoursColumn'),
            cell: (ot) => t('hrPages.shared.hoursValue', { hours: parseDecimal(ot.hours).toFixed(1) }),
        },
        {
            key: 'reason',
            header: t('hrPages.shared.col.reason'),
            className: 'max-w-[200px] whitespace-normal break-words',
            cell: (ot) => ot.reason,
        },
        {
            key: 'actions',
            header: t('common.actions'),
            className: 'text-right',
            // R72-2：僅當前使用者於此階段可核准（後端 can_approve）時才顯示核准/駁回鈕
            cell: (ot) => (
                <ApprovalActionsCell
                    id={ot.id}
                    canApprove={ot.can_approve}
                    onApprove={onApprove}
                    onReject={onReject}
                    approvePending={isApproving}
                    rejectPending={isRejecting}
                />
            ),
        },
    ]

    return (
        <Card>
                <CardHeader>
                    <CardTitle>{t('hrPages.overtime.pending.title')}</CardTitle>
                    <CardDescription>{t('hrPages.overtime.pending.description')}</CardDescription>
                </CardHeader>
                <CardContent>
                    <DataTable
                        columns={columns}
                        data={pendingData?.data}
                        isLoading={isLoading}
                        emptyIcon={Clock}
                        emptyTitle={t('hrPages.overtime.pending.empty')}
                        rowKey={(row) => row.id}
                    />
                </CardContent>
            </Card>
    )
}
