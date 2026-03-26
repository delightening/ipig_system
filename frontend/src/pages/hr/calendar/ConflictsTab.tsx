/**
 * 日曆「衝突處理」分頁元件
 * 顯示待處理衝突列表、解決操作與分頁控制
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
import type { ConflictWithDetails } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDateTime } from '@/lib/utils'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { FileText } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'

interface ConflictsTabProps {
    conflicts: PaginatedResponse<ConflictWithDetails> | undefined
    loadingConflicts: boolean
    onResolve: (params: { id: string; resolution: string }) => void
    resolvePending: boolean
    currentPage: number
    onPageChange: (page: number) => void
}

export function ConflictsTab({ conflicts, loadingConflicts, onResolve, resolvePending, currentPage, onPageChange }: ConflictsTabProps) {
    const totalPages = conflicts?.total_pages ?? 1
    const { sortedData, sort, toggleSort } = useTableSort(conflicts?.data)

    return (
        <div className="space-y-4">
            <Table>
                <TableHeader>
                    <TableRow>
                        <SortableTableHead sortKey="detected_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>偵測時間</SortableTableHead>
                        <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>員工</SortableTableHead>
                        <SortableTableHead sortKey="leave_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>假別</SortableTableHead>
                        <SortableTableHead sortKey="conflict_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>衝突類型</SortableTableHead>
                        <TableHead>差異</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {loadingConflicts ? (
                        <TableRow>
                            <TableCell colSpan={6} className="text-center py-8">
                                載入中...
                            </TableCell>
                        </TableRow>
                    ) : conflicts?.data?.length === 0 ? (
                        <TableEmptyRow colSpan={6} icon={FileText} title="沒有待處理的衝突" />
                    ) : (
                        (sortedData ?? conflicts?.data)?.map((c) => (
                            <TableRow key={c.id}>
                                <TableCell className="whitespace-nowrap">
                                    {formatDateTime(c.detected_at)}
                                </TableCell>
                                <TableCell>{c.user_name || '-'}</TableCell>
                                <TableCell>{c.leave_type || '-'}</TableCell>
                                <TableCell>
                                    <Badge variant="outline">{c.conflict_type}</Badge>
                                </TableCell>
                                <TableCell className="max-w-[200px] truncate">
                                    {c.difference_summary || '-'}
                                </TableCell>
                                <TableCell>
                                    <div className="flex items-center justify-end gap-1">
                                        <Button
                                            variant="default"
                                            size="sm"
                                            onClick={() =>
                                                onResolve({
                                                    id: c.id,
                                                    resolution: 'keep_ipig',
                                                })
                                            }
                                            disabled={resolvePending}
                                        >
                                            保留 iPig
                                        </Button>
                                        <Button
                                            variant="secondary"
                                            size="sm"
                                            onClick={() =>
                                                onResolve({
                                                    id: c.id,
                                                    resolution: 'accept_google',
                                                })
                                            }
                                            disabled={resolvePending}
                                        >
                                            接受 Google
                                        </Button>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() =>
                                                onResolve({
                                                    id: c.id,
                                                    resolution: 'dismiss',
                                                })
                                            }
                                            disabled={resolvePending}
                                        >
                                            忽略
                                        </Button>
                                    </div>
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
                        第 {currentPage} 頁，共 {totalPages} 頁（{conflicts?.total ?? 0} 筆）
                    </div>
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage - 1)}
                            disabled={currentPage <= 1 || loadingConflicts}
                        >
                            上一頁
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage + 1)}
                            disabled={currentPage >= totalPages || loadingConflicts}
                        >
                            下一頁
                        </Button>
                    </div>
                </div>
            )}
        </div>
    )
}
