import { useTranslation } from 'react-i18next'
import { FileText, UserCheck, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/utils'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours, leaveStatusLabel, leaveTypeLabel } from '../constants'
import { ApprovalActionsCell } from './ApprovalActionsCell'

interface LeavePendingApprovalsTabProps {
    leaves: LeaveRequestWithUser[] | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    onProxyConfirm: (id: string) => void
    onProxyReject: (id: string) => void
    approvePending: boolean
    rejectPending: boolean
    proxyConfirmPending: boolean
    proxyRejectPending: boolean
}

export function LeavePendingApprovalsTab({
    leaves,
    isLoading,
    onApprove,
    onReject,
    onProxyConfirm,
    onProxyReject,
    approvePending,
    rejectPending,
    proxyConfirmPending,
    proxyRejectPending,
}: LeavePendingApprovalsTabProps) {
    const { t } = useTranslation()
    const columns: ColumnDef<LeaveRequestWithUser>[] = [
        {
            key: 'applicant',
            header: t('hrPages.shared.col.applicant'),
            cell: (leave) => (
                <div>
                    <div className="font-medium">{leave.user_name}</div>
                    <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                </div>
            ),
        },
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
            cell: (leave) => leaveStatusLabel(t, leave.status),
        },
        {
            key: 'actions',
            header: t('common.actions'),
            className: 'text-right',
            // 依當前使用者於此列的角色顯示對應動作：
            //   can_confirm_proxy → 代理人「確認 / 退回」；can_approve → 主管/負責人「核准 / 駁回」。
            cell: (leave) => {
                if (leave.can_confirm_proxy) {
                    return (
                        <div className="flex items-center justify-end gap-1">
                            <Button
                                variant="default"
                                size="sm"
                                onClick={() => onProxyConfirm(leave.id)}
                                disabled={proxyConfirmPending}
                            >
                                <UserCheck className="h-4 w-4 mr-1" />
                                {t('hrPages.shared.action.confirmDelegate')}
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onProxyReject(leave.id)}
                                disabled={proxyRejectPending}
                            >
                                <XCircle className="h-4 w-4 mr-1" />
                                {t('hrPages.shared.action.return')}
                            </Button>
                        </div>
                    )
                }
                return (
                    <ApprovalActionsCell
                        id={leave.id}
                        canApprove={leave.can_approve}
                        onApprove={onApprove}
                        onReject={onReject}
                        approvePending={approvePending}
                        rejectPending={rejectPending}
                    />
                )
            },
        },
    ]

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t('hrPages.shared.pendingMyReview')}</CardTitle>
                <CardDescription>{t('hrPages.leaves.pending.description')}</CardDescription>
            </CardHeader>
            <CardContent>
                <DataTable
                    columns={columns}
                    data={leaves}
                    isLoading={isLoading}
                    emptyIcon={FileText}
                    emptyTitle={t('hrPages.leaves.pending.empty')}
                    rowKey={(row) => row.id}
                />
            </CardContent>
        </Card>
    )
}
