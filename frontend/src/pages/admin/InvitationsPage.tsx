import { useState } from 'react'
import { GuestHide } from '@/components/ui/guest-hide'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, RotateCw, XCircle, Loader2, Mail } from 'lucide-react'

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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
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
                    <GuestHide>
                        <Button size="sm" onClick={() => setShowCreateDialog(true)}>
                            <Plus className="h-4 w-4 mr-2" />
                            新增邀請
                        </Button>
                    </GuestHide>
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
            <div className="rounded-lg border bg-card overflow-hidden @container">
                <div className="hidden @[600px]:block overflow-x-auto">
                    <Table className="w-full" style={{ minWidth: 450 }}>
                        <TableHeader>
                            <TableRow className="bg-muted/50 hover:bg-muted/50">
                                <TableHead style={{ minWidth: 180 }}>Email</TableHead>
                                <TableHead style={{ width: 100 }} className="hidden @[750px]:table-cell">組織</TableHead>
                                <TableHead style={{ width: 90 }}>狀態</TableHead>
                                <TableHead style={{ width: 100 }} className="hidden @[750px]:table-cell">邀請人</TableHead>
                                <TableHead style={{ width: 100 }} className="hidden @[750px]:table-cell">建立時間</TableHead>
                                <TableHead style={{ width: 100 }}>到期時間</TableHead>
                                <TableHead style={{ width: 80 }}>操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={7} className="p-0">
                                        <TableSkeleton rows={5} cols={7} />
                                    </TableCell>
                                </TableRow>
                            ) : invitations.length === 0 ? (
                                <TableEmptyRow colSpan={7} icon={Mail} title="尚無邀請紀錄" />
                            ) : (
                                invitations.map(inv => (
                                    <TableRow key={inv.id}>
                                        <TableCell style={{ minWidth: 180 }} className="font-medium break-all">{inv.email}</TableCell>
                                        <TableCell style={{ width: 100 }} className="hidden @[750px]:table-cell whitespace-normal break-words">{inv.organization || '-'}</TableCell>
                                        <TableCell style={{ width: 90 }}>
                                            <Badge variant={invitationStatusColors[inv.status]}>
                                                {invitationStatusNames[inv.status]}
                                            </Badge>
                                        </TableCell>
                                        <TableCell style={{ width: 100 }} className="hidden @[750px]:table-cell whitespace-normal break-words">{inv.invited_by_name}</TableCell>
                                        <TableCell style={{ width: 100 }} className="hidden @[750px]:table-cell text-xs text-muted-foreground">{formatDate(inv.created_at)}</TableCell>
                                        <TableCell style={{ width: 100 }} className="text-xs text-muted-foreground">{formatDate(inv.expires_at)}</TableCell>
                                        <TableCell style={{ width: 80 }}>
                                            <GuestHide>
                                                {inv.status === 'pending' && (
                                                    <div className="flex gap-1">
                                                        <Button variant="ghost" size="icon" onClick={() => resendMutation.mutate(inv.id)} disabled={resendMutation.isPending} title="重新發送">
                                                            {resendMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCw className="h-4 w-4" />}
                                                        </Button>
                                                        <Button variant="ghost" size="icon" onClick={() => revokeMutation.mutate(inv.id)} disabled={revokeMutation.isPending} title="撤銷">
                                                            <XCircle className="h-4 w-4 text-destructive" />
                                                        </Button>
                                                    </div>
                                                )}
                                            </GuestHide>
                                        </TableCell>
                                    </TableRow>
                                ))
                            )}
                        </TableBody>
                    </Table>
                </div>

                {/* Card view: container < 600px */}
                <div className="@[600px]:hidden divide-y">
                    {isLoading ? (
                        <div className="p-3"><TableSkeleton rows={3} cols={1} /></div>
                    ) : invitations.length === 0 ? (
                        <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                            <Mail className="h-8 w-8" />
                            <p className="text-sm">尚無邀請紀錄</p>
                        </div>
                    ) : (
                        invitations.map(inv => (
                            <div key={inv.id} className="p-3 space-y-2">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="min-w-0 flex-1">
                                        <div className="font-medium text-sm break-all">{inv.email}</div>
                                        {inv.organization && <div className="text-xs text-muted-foreground">{inv.organization}</div>}
                                    </div>
                                    <Badge variant={invitationStatusColors[inv.status]}>
                                        {invitationStatusNames[inv.status]}
                                    </Badge>
                                </div>
                                <div className="text-xs text-muted-foreground">
                                    {inv.invited_by_name} · 建立 {formatDate(inv.created_at)} · 到期 {formatDate(inv.expires_at)}
                                </div>
                                <GuestHide>
                                    {inv.status === 'pending' && (
                                        <div className="flex justify-end gap-1 pt-1 border-t">
                                            <Button variant="ghost" size="icon" onClick={() => resendMutation.mutate(inv.id)} disabled={resendMutation.isPending} title="重新發送">
                                                {resendMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCw className="h-4 w-4" />}
                                            </Button>
                                            <Button variant="ghost" size="icon" onClick={() => revokeMutation.mutate(inv.id)} disabled={revokeMutation.isPending} title="撤銷">
                                                <XCircle className="h-4 w-4 text-destructive" />
                                            </Button>
                                        </div>
                                    )}
                                </GuestHide>
                            </div>
                        ))
                    )}
                </div>
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
