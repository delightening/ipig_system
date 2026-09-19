/**
 * 日曆「同步歷史」分頁元件
 * 顯示同步記錄表格與分頁控制
 */
import { useMemo } from 'react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { StatusBadge } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import type { CalendarSyncHistory } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDateTime } from '@/lib/utils'
import { RefreshCw } from 'lucide-react'

interface SyncHistoryTabProps {
    syncHistory: PaginatedResponse<CalendarSyncHistory> | undefined
    loadingHistory: boolean
    currentPage: number
    onPageChange: (page: number) => void
}

/** 狀態標籤 */
function getStatusBadge(t: TFunction, status: string) {
    switch (status) {
        case 'completed':
            return <StatusBadge variant="success">{t('hrPages.calendar.history.completed')}</StatusBadge>
        case 'completed_with_errors':
            return <StatusBadge variant="warning">{t('hrPages.calendar.history.completedWithErrors')}</StatusBadge>
        case 'running':
            return <StatusBadge variant="info">{t('hrPages.calendar.history.running')}</StatusBadge>
        case 'failed':
            return <StatusBadge variant="error">{t('hrPages.calendar.status.failed')}</StatusBadge>
        default:
            return <StatusBadge variant="neutral">{status}</StatusBadge>
    }
}

export function SyncHistoryTab({ syncHistory, loadingHistory, currentPage, onPageChange }: SyncHistoryTabProps) {
    const { t } = useTranslation()
    const totalPages = syncHistory?.total_pages ?? 1

    const columns = useMemo<ColumnDef<CalendarSyncHistory>[]>(() => [
        { key: 'time', header: t('hrPages.shared.col.time'), className: 'whitespace-nowrap', cell: (h) => formatDateTime(h.started_at) },
        { key: 'type', header: t('hrPages.shared.col.type'), cell: (h) => h.job_type === 'manual' ? t('hrPages.calendar.history.manual') : t('hrPages.calendar.history.automatic') },
        { key: 'status', header: t('hrPages.shared.col.status'), cell: (h) => getStatusBadge(t, h.status) },
        { key: 'created', header: t('common.create'), cell: (h) => h.events_created },
        { key: 'updated', header: t('common.update'), cell: (h) => h.events_updated },
        { key: 'deleted', header: t('common.delete'), cell: (h) => h.events_deleted },
        {
            key: 'conflicts', header: t('hrPages.calendar.history.conflicts'),
            cell: (h) => h.conflicts_detected > 0 ? <Badge variant="secondary">{h.conflicts_detected}</Badge> : null,
        },
        { key: 'duration', header: t('hrPages.calendar.history.duration'), cell: (h) => h.duration_ms ? `${(h.duration_ms / 1000).toFixed(1)}s` : '-' },
    ], [t])

    return (
        <DataTable
            columns={columns}
            data={syncHistory?.data}
            isLoading={loadingHistory}
            emptyIcon={RefreshCw}
            emptyTitle={t('hrPages.calendar.history.empty')}
            rowKey={(h) => h.id}
            page={currentPage}
            totalPages={totalPages}
            totalItems={syncHistory?.total}
            onPageChange={onPageChange}
        />
    )
}
