import { Clock, Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { parseDecimal } from '@/lib/utils'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { OVERTIME_TYPE_NAMES, formatDate } from '../constants'
import { OvertimeStatusBadge } from './OvertimeStatusBadge'

interface MyOvertimeTabContentProps {
    overtimeData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onSubmit: (id: string) => void
    onDelete: (id: string) => void
    isSubmitting: boolean
    isDeleting: boolean
}

export function MyOvertimeTabContent({
    overtimeData,
    isLoading,
    onSubmit,
    onDelete,
    isSubmitting,
    isDeleting,
}: MyOvertimeTabContentProps) {
    const columns: ColumnDef<OvertimeWithUser>[] = [
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
            key: 'type',
            header: '類型',
            cell: (ot) => OVERTIME_TYPE_NAMES[ot.overtime_type] || ot.overtime_type,
        },
        {
            key: 'hours',
            header: '加班時數',
            cell: (ot) => `${parseDecimal(ot.hours).toFixed(1)} 小時`,
        },
        {
            key: 'comp_time',
            header: '補休',
            className: 'text-status-success-text font-medium',
            cell: (ot) => `${parseDecimal(ot.comp_time_hours).toFixed(1)} 小時`,
        },
        {
            key: 'reason',
            header: '事由',
            className: 'max-w-[150px] whitespace-normal break-words',
            cell: (ot) => ot.reason,
        },
        {
            key: 'status',
            header: '狀態',
            cell: (ot) => <OvertimeStatusBadge status={ot.status} />,
        },
        {
            key: 'actions',
            header: '操作',
            className: 'text-right',
            cell: (ot) => (
                <div className="flex items-center justify-end gap-1">
                    {ot.status === 'draft' && (
                        <>
                            <Button
                                variant="default"
                                size="sm"
                                onClick={() => onSubmit(ot.id)}
                                disabled={isSubmitting}
                            >
                                <Send className="h-4 w-4 mr-1" />
                                送審
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onDelete(ot.id)}
                                disabled={isDeleting}
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                        </>
                    )}
                </div>
            ),
        },
    ]

    return (
        <Card>
            <DataTable
                columns={columns}
                data={overtimeData?.data}
                isLoading={isLoading}
                emptyIcon={Clock}
                emptyTitle="沒有加班記錄"
                rowKey={(row) => row.id}
            />
        </Card>
    )
}
