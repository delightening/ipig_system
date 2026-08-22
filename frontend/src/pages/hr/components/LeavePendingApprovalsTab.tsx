import { FileText, UserCheck, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatDate } from '@/lib/utils'
import { LEAVE_STATUS_NAMES, LEAVE_TYPE_NAMES } from '@/types/hr'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours } from '../constants'
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
            className: 'max-w-[200px] whitespace-normal break-words',
            cell: (leave) => leave.reason,
        },
        {
            key: 'status',
            header: '狀態',
            cell: (leave) => LEAVE_STATUS_NAMES[leave.status] || leave.status,
        },
        {
            key: 'actions',
            header: '操作',
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
                                確認代理
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onProxyReject(leave.id)}
                                disabled={proxyRejectPending}
                            >
                                <XCircle className="h-4 w-4 mr-1" />
                                退回
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
                <CardTitle>待我審核</CardTitle>
                <CardDescription>您需要審核，或身為職務代理人需確認的請假申請</CardDescription>
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
