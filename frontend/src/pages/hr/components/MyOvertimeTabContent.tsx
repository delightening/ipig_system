import { useTranslation } from 'react-i18next'
import { Clock, Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { parseDecimal } from '@/lib/utils'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { overtimeTypeLabel, formatDate } from '../constants'
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
    const { t } = useTranslation()
    const columns: ColumnDef<OvertimeWithUser>[] = [
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
            key: 'type',
            header: t('hrPages.shared.col.type'),
            cell: (ot) => overtimeTypeLabel(t, ot.overtime_type),
        },
        {
            key: 'hours',
            header: t('hrPages.overtime.hoursColumn'),
            cell: (ot) => t('hrPages.shared.hoursValue', { hours: parseDecimal(ot.hours).toFixed(1) }),
        },
        {
            key: 'comp_time',
            header: t('hrPages.shared.col.compLeave'),
            className: 'text-status-success-text font-medium',
            cell: (ot) => t('hrPages.shared.hoursValue', { hours: parseDecimal(ot.comp_time_hours).toFixed(1) }),
        },
        {
            key: 'reason',
            header: t('hrPages.shared.col.reason'),
            className: 'max-w-[150px] whitespace-normal break-words',
            cell: (ot) => ot.reason,
        },
        {
            key: 'status',
            header: t('hrPages.shared.col.status'),
            cell: (ot) => <OvertimeStatusBadge status={ot.status} pendingOwner={ot.pending_owner} />,
        },
        {
            key: 'actions',
            header: t('common.actions'),
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
                                {t('hrPages.shared.action.submit')}
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
                emptyTitle={t('hrPages.overtime.mine.empty')}
                rowKey={(row) => row.id}
            />
        </Card>
    )
}
