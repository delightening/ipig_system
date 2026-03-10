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

    return (
        <div className="space-y-4">
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>偵測時間</TableHead>
                        <TableHead>員工</TableHead>
                        <TableHead>假別</TableHead>
                        <TableHead>衝突類型</TableHead>
                        <TableHead>差異</TableHead>
                        <TableHead>操作</TableHead>
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
                        <TableRow>
                            <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                沒有待處理的衝突
                            </TableCell>
                        </TableRow>
                    ) : (
                        conflicts?.data?.map((c) => (
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
                                    <div className="flex gap-2">
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
