/**
 * 日曆「同步歷史」分頁元件
 * 顯示同步記錄表格與分頁控制
 */
import { useMemo } from 'react'
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
function getStatusBadge(status: string) {
    switch (status) {
        case 'completed':
            return <StatusBadge variant="success">完成</StatusBadge>
        case 'completed_with_errors':
            return <StatusBadge variant="warning">部分錯誤</StatusBadge>
        case 'running':
            return <StatusBadge variant="info">執行中</StatusBadge>
        case 'failed':
            return <StatusBadge variant="error">失敗</StatusBadge>
        default:
            return <StatusBadge variant="neutral">{status}</StatusBadge>
    }
}

export function SyncHistoryTab({ syncHistory, loadingHistory, currentPage, onPageChange }: SyncHistoryTabProps) {
    const { t } = useTranslation()
    const totalPages = syncHistory?.total_pages ?? 1

    const columns = useMemo<ColumnDef<CalendarSyncHistory>[]>(() => [
        { key: 'time', header: '時間', className: 'whitespace-nowrap', cell: (h) => formatDateTime(h.started_at) },
        { key: 'type', header: '類型', cell: (h) => h.job_type === 'manual' ? '手動' : '自動' },
        { key: 'status', header: '狀態', cell: (h) => getStatusBadge(h.status) },
        { key: 'created', header: t('common.create'), cell: (h) => h.events_created },
        { key: 'updated', header: t('common.update'), cell: (h) => h.events_updated },
        { key: 'deleted', header: t('common.delete'), cell: (h) => h.events_deleted },
        {
            key: 'conflicts', header: '衝突',
            cell: (h) => h.conflicts_detected > 0 ? <Badge variant="secondary">{h.conflicts_detected}</Badge> : null,
        },
        { key: 'duration', header: '耗時', cell: (h) => h.duration_ms ? `${(h.duration_ms / 1000).toFixed(1)}s` : '-' },
    ], [t])

    return (
        <DataTable
            columns={columns}
            data={syncHistory?.data}
            isLoading={loadingHistory}
            emptyIcon={RefreshCw}
            emptyTitle="沒有同步記錄"
            rowKey={(h) => h.id}
            page={currentPage}
            totalPages={totalPages}
            totalItems={syncHistory?.total}
            onPageChange={onPageChange}
        />
    )
}
