import { LogOut } from 'lucide-react'
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
import { formatDateTime } from '@/lib/utils'
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

    return (
        <Card>
            <CardHeader>
                <CardTitle>活躍 Sessions</CardTitle>
                <CardDescription>目前線上的使用者 Session</CardDescription>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>使用者</TableHead>
                            <TableHead>開始時間</TableHead>
                            <TableHead>最後活動</TableHead>
                            <TableHead>IP</TableHead>
                            <TableHead>頁面瀏覽</TableHead>
                            <TableHead>操作次數</TableHead>
                            <TableHead>操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8">載入中...</TableCell>
                            </TableRow>
                        ) : activeSessions.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                    沒有活躍的 Session
                                </TableCell>
                            </TableRow>
                        ) : (
                            activeSessions.map((session) => (
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
            <TableCell className="whitespace-nowrap">{formatDateTime(session.started_at)}</TableCell>
            <TableCell className="whitespace-nowrap">{formatDateTime(session.last_activity_at)}</TableCell>
            <TableCell className="text-muted-foreground text-sm">{session.ip_address || '-'}</TableCell>
            <TableCell>{session.page_view_count}</TableCell>
            <TableCell>{session.action_count}</TableCell>
            <TableCell>
                <Button
                    variant="destructive"
                    size="sm"
                    onClick={onForceLogout}
                    disabled={isPending}
                >
                    <LogOut className="h-4 w-4 mr-1" />
                    強制登出
                </Button>
            </TableCell>
        </TableRow>
    )
}
