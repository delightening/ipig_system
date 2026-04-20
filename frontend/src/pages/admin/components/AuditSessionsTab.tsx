import { LogOut, Monitor } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import type { UseMutationResult } from '@tanstack/react-query'
import type { AxiosResponse } from 'axios'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { formatDate, formatTime } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import type { SessionWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { AuditPagination } from './AuditPagination'

interface AuditSessionsTabProps {
    sessions: PaginatedResponse<SessionWithUser> | undefined
    isLoading: boolean
    currentPage: number
    onPageChange: (page: number) => void
    forceLogoutMutation: UseMutationResult<AxiosResponse, Error, string>
}

export function AuditSessionsTab({
    sessions,
    isLoading,
    currentPage,
    onPageChange,
    forceLogoutMutation,
}: AuditSessionsTabProps) {
    const activeSessions = sessions?.data?.filter(s => s.is_active) ?? []
    const { sortedData, sort, toggleSort } = useTableSort(activeSessions)

    return (
        <Card className="overflow-hidden">
            <CardHeader>
                <CardTitle>活躍 Sessions</CardTitle>
                <CardDescription>目前線上的使用者 Session</CardDescription>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/50 hover:bg-muted/50">
                            <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>使用者</SortableTableHead>
                            <SortableTableHead sortKey="started_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>開始時間</SortableTableHead>
                            <SortableTableHead sortKey="last_activity_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>最後活動</SortableTableHead>
                            <SortableTableHead sortKey="ip_address" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>IP</SortableTableHead>
                            <SortableTableHead sortKey="page_view_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>頁面瀏覽</SortableTableHead>
                            <SortableTableHead sortKey="action_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>操作次數</SortableTableHead>
                            <TableHead className="text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={8} cols={7} /></TableCell></TableRow>
                        ) : sortedData === undefined || sortedData.length === 0 ? (
                            <TableEmptyRow colSpan={7} icon={Monitor} title="沒有活躍的 Session" />
                        ) : (
                            sortedData.map((session) => (
                                <SessionRow
                                    key={session.id}
                                    session={session}
                                    onForceLogout={() => forceLogoutMutation.mutate(session.id)}
                                    isPending={forceLogoutMutation.isPending}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
                {sessions && (
                    <AuditPagination
                        total={sessions.total}
                        totalPages={sessions.total_pages}
                        currentPage={currentPage}
                        onPageChange={onPageChange}
                    />
                )}
            </CardContent>
        </Card>
    )
}

interface SessionRowProps {
    session: SessionWithUser
    onForceLogout: () => void
    isPending: boolean
}

function SessionRow({ session, onForceLogout, isPending }: SessionRowProps) {
    return (
        <TableRow>
            <TableCell>
                <div>
                    <div className="font-medium">{session.user_name}</div>
                    <div className="text-sm text-muted-foreground">{session.user_email}</div>
                </div>
            </TableCell>
            <TableCell className="whitespace-nowrap">
                <div>{formatDate(session.started_at)}</div>
                <div className="text-muted-foreground text-sm">{formatTime(session.started_at)}</div>
            </TableCell>
            <TableCell className="whitespace-nowrap">
                <div>{formatDate(session.last_activity_at)}</div>
                <div className="text-muted-foreground text-sm">{formatTime(session.last_activity_at)}</div>
            </TableCell>
            <TableCell className="text-muted-foreground text-sm">{session.ip_address || '-'}</TableCell>
            <TableCell>{session.page_view_count}</TableCell>
            <TableCell>{session.action_count}</TableCell>
            <TableCell>
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={onForceLogout}
                        disabled={isPending}
                    >
                        <LogOut className="h-4 w-4 mr-1" />
                        強制登出
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    )
}
