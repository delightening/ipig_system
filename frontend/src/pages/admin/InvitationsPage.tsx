import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, RotateCw, XCircle, Loader2 } from 'lucide-react'

import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/validation'
import { formatDate } from '@/lib/utils'
import { STALE_TIME } from '@/lib/query'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/ui/page-header'
import { toast } from '@/components/ui/use-toast'
import {
    Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import { InvitationCreateDialog } from './components/InvitationCreateDialog'
import type { InvitationStatus } from '@/types/invitation'
import { invitationStatusNames, invitationStatusColors } from '@/types/invitation'

const STATUS_TABS: Array<{ label: string; value: InvitationStatus | 'all' }> = [
    { label: '全部', value: 'all' },
    { label: '待接受', value: 'pending' },
    { label: '已接受', value: 'accepted' },
    { label: '已過期', value: 'expired' },
    { label: '已撤銷', value: 'revoked' },
]

export function InvitationsPage() {
    const queryClient = useQueryClient()
    const [statusFilter, setStatusFilter] = useState<InvitationStatus | 'all'>('all')
    const [page, setPage] = useState(1)
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const perPage = 20

    const queryParams = {
        status: statusFilter === 'all' ? undefined : statusFilter,
        page,
        per_page: perPage,
    }

    const { data, isLoading } = useQuery({
        queryKey: ['invitations', queryParams],
        queryFn: () => invitationApi.list(queryParams).then(r => r.data),
        staleTime: STALE_TIME.LIST,
    })

    const revokeMutation = useMutation({
        mutationFn: (id: string) => invitationApi.revoke(id),
        onSuccess: () => {
            toast({ title: '已撤銷邀請' })
            queryClient.invalidateQueries({ queryKey: ['invitations'] })
        },
        onError: (err) => toast({ variant: 'destructive', title: getApiErrorMessage(err) }),
    })

    const resendMutation = useMutation({
        mutationFn: (id: string) => invitationApi.resend(id),
        onSuccess: (res) => {
            toast({ title: '已重新發送邀請', description: `連結已更新：${res.data.invite_link.slice(0, 40)}...` })
            queryClient.invalidateQueries({ queryKey: ['invitations'] })
        },
        onError: (err) => toast({ variant: 'destructive', title: getApiErrorMessage(err) }),
    })

    const invitations = data?.data ?? []
    const total = data?.total ?? 0
    const totalPages = Math.ceil(total / perPage)

    return (
        <div className="space-y-6">
            <PageHeader
                title="邀請管理"
                description="管理客戶邀請連結"
                actions={
                    <Button size="sm" onClick={() => setShowCreateDialog(true)}>
                        <Plus className="h-4 w-4 mr-2" />
                        新增邀請
                    </Button>
                }
            />

            {/* Status filter tabs */}
            <div className="flex gap-1 border-b">
                {STATUS_TABS.map(tab => (
                    <button
                        key={tab.value}
                        onClick={() => { setStatusFilter(tab.value); setPage(1) }}
                        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                            statusFilter === tab.value
                                ? 'border-primary text-primary'
                                : 'border-transparent text-muted-foreground hover:text-foreground'
                        }`}
                    >
                        {tab.label}
                    </button>
                ))}
            </div>

            {/* Table */}
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Email</TableHead>
                            <TableHead>組織</TableHead>
                            <TableHead>狀態</TableHead>
                            <TableHead>邀請人</TableHead>
                            <TableHead>建立時間</TableHead>
                            <TableHead>到期時間</TableHead>
                            <TableHead className="w-[120px]">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            Array.from({ length: 5 }).map((_, i) => (
                                <TableRow key={i}>
                                    {Array.from({ length: 7 }).map((_, j) => (
                                        <TableCell key={j}><div className="h-4 w-full animate-pulse rounded bg-muted" /></TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : invitations.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                    尚無邀請紀錄
                                </TableCell>
                            </TableRow>
                        ) : (
                            invitations.map(inv => (
                                <TableRow key={inv.id}>
                                    <TableCell className="font-medium">{inv.email}</TableCell>
                                    <TableCell>{inv.organization || '-'}</TableCell>
                                    <TableCell>
                                        <Badge variant={invitationStatusColors[inv.status]}>
                                            {invitationStatusNames[inv.status]}
                                        </Badge>
                                    </TableCell>
                                    <TableCell>{inv.invited_by_name}</TableCell>
                                    <TableCell>{formatDate(inv.created_at)}</TableCell>
                                    <TableCell>{formatDate(inv.expires_at)}</TableCell>
                                    <TableCell>
                                        {inv.status === 'pending' && (
                                            <div className="flex gap-1">
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    onClick={() => resendMutation.mutate(inv.id)}
                                                    disabled={resendMutation.isPending}
                                                    title="重新發送"
                                                >
                                                    {resendMutation.isPending ? (
                                                        <Loader2 className="h-4 w-4 animate-spin" />
                                                    ) : (
                                                        <RotateCw className="h-4 w-4" />
                                                    )}
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    onClick={() => revokeMutation.mutate(inv.id)}
                                                    disabled={revokeMutation.isPending}
                                                    title="撤銷"
                                                >
                                                    <XCircle className="h-4 w-4 text-destructive" />
                                                </Button>
                                            </div>
                                        )}
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
                <div className="flex items-center justify-between">
                    <p className="text-sm text-muted-foreground">
                        共 {total} 筆，第 {page} / {totalPages} 頁
                    </p>
                    <div className="flex gap-2">
                        <Button variant="outline" size="sm" onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page <= 1}>
                            上一頁
                        </Button>
                        <Button variant="outline" size="sm" onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page >= totalPages}>
                            下一頁
                        </Button>
                    </div>
                </div>
            )}

            <InvitationCreateDialog
                open={showCreateDialog}
                onOpenChange={setShowCreateDialog}
                onSuccess={() => queryClient.invalidateQueries({ queryKey: ['invitations'] })}
            />
        </div>
    )
}
