/**
 * 日曆「同步歷史」分頁元件
 * 顯示同步記錄表格與分頁控制
 */
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import type { CalendarSyncHistory } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDateTime } from '@/lib/utils'

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
            return <Badge className="bg-green-500">完成</Badge>
        case 'completed_with_errors':
            return <Badge className="bg-yellow-500">部分錯誤</Badge>
        case 'running':
            return <Badge className="bg-blue-500">執行中</Badge>
        case 'failed':
            return <Badge variant="destructive">失敗</Badge>
        default:
            return <Badge variant="secondary">{status}</Badge>
    }
}

export function SyncHistoryTab({ syncHistory, loadingHistory, currentPage, onPageChange }: SyncHistoryTabProps) {
    const totalPages = syncHistory?.total_pages ?? 1

    return (
        <div className="space-y-4">
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>時間</TableHead>
                        <TableHead>類型</TableHead>
                        <TableHead>狀態</TableHead>
                        <TableHead>新增</TableHead>
                        <TableHead>更新</TableHead>
                        <TableHead>刪除</TableHead>
                        <TableHead>衝突</TableHead>
                        <TableHead>耗時</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {loadingHistory ? (
                        <TableRow>
                            <TableCell colSpan={8} className="text-center py-8">
                                載入中...
                            </TableCell>
                        </TableRow>
                    ) : syncHistory?.data?.length === 0 ? (
                        <TableRow>
                            <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                沒有同步記錄
                            </TableCell>
                        </TableRow>
                    ) : (
                        syncHistory?.data?.map((h) => (
                            <TableRow key={h.id}>
                                <TableCell className="whitespace-nowrap">
                                    {formatDateTime(h.started_at)}
                                </TableCell>
                                <TableCell>
                                    {h.job_type === 'manual' ? '手動' : '自動'}
                                </TableCell>
                                <TableCell>{getStatusBadge(h.status)}</TableCell>
                                <TableCell>{h.events_created}</TableCell>
                                <TableCell>{h.events_updated}</TableCell>
                                <TableCell>{h.events_deleted}</TableCell>
                                <TableCell>
                                    {h.conflicts_detected > 0 && (
                                        <Badge variant="secondary">{h.conflicts_detected}</Badge>
                                    )}
                                </TableCell>
                                <TableCell>
                                    {h.duration_ms ? `${(h.duration_ms / 1000).toFixed(1)}s` : '-'}
                                </TableCell>
                            </TableRow>
                        ))
                    )}
                </TableBody>
            </Table>

            {/* 分頁控制 */}
            {totalPages > 1 && (
                <div className="flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">
                        第 {currentPage} 頁，共 {totalPages} 頁（{syncHistory?.total ?? 0} 筆）
                    </div>
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage - 1)}
                            disabled={currentPage <= 1 || loadingHistory}
                        >
                            上一頁
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage + 1)}
                            disabled={currentPage >= totalPages || loadingHistory}
                        >
                            下一頁
                        </Button>
                    </div>
                </div>
            )}
        </div>
    )
}
