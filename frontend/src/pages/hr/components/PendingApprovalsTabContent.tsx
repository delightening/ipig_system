import { CheckCircle, Clock, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { parseDecimal } from '@/lib/utils'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDate } from '../constants'

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
    const columns: ColumnDef<OvertimeWithUser>[] = [
        {
            key: 'applicant',
            header: '申請人',
            cell: (ot) => (
                <div>
                    <div className="font-medium">{ot.user_name}</div>
                    <div className="text-sm text-muted-foreground">{ot.user_email}</div>
                </div>
            ),
        },
        {
            key: 'date',
            header: '日期',
            cell: (ot) => (
                <span className="whitespace-nowrap">{formatDate(ot.overtime_date)}</span>
            ),
        },
        {
            key: 'time',
            header: '時間',
            cell: (ot) => `${ot.start_time} ~ ${ot.end_time}`,
        },
        {
            key: 'hours',
            header: '加班時數',
            cell: (ot) => `${parseDecimal(ot.hours).toFixed(1)} 小時`,
        },
        {
            key: 'reason',
            header: '事由',
            className: 'max-w-[200px] whitespace-normal break-words',
            cell: (ot) => ot.reason,
        },
        {
            key: 'actions',
            header: '操作',
            className: 'text-right',
            cell: (ot) => (
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="default"
                        size="sm"
                        onClick={() => onApprove(ot.id)}
                        disabled={isApproving}
                    >
                        <CheckCircle className="h-4 w-4 mr-1" />
                        核准
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => onReject(ot.id, '不符合規定')}
                        disabled={isRejecting}
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
                    <CardTitle>待審核加班</CardTitle>
                    <CardDescription>您需要審核的加班申請</CardDescription>
                </CardHeader>
                <CardContent>
                    <DataTable
                        columns={columns}
                        data={pendingData?.data}
                        isLoading={isLoading}
                        emptyIcon={Clock}
                        emptyTitle="沒有待審核的加班"
                        rowKey={(row) => row.id}
                    />
                </CardContent>
            </Card>
    )
}
