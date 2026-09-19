import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, RotateCw, XCircle, Loader2, Mail } from 'lucide-react'

import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/apiError'
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
import { invitationStatusColors } from '@/types/invitation'

// 分頁標籤與狀態徽章共用同一組 i18n 鍵（module 級常數不可存翻譯後字串，故存鍵、渲染時才 t()）
const STATUS_LABEL_KEYS: Record<InvitationStatus | 'all', string> = {
    all: 'adminUsers.invitations.status.all',
    pending: 'adminUsers.invitations.status.pending',
    accepted: 'adminUsers.invitations.status.accepted',
    expired: 'adminUsers.invitations.status.expired',
    revoked: 'adminUsers.invitations.status.revoked',
}

const STATUS_TABS: Array<InvitationStatus | 'all'> = ['all', 'pending', 'accepted', 'expired', 'revoked']

export function InvitationsPage() {
    const { t } = useTranslation()
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
            toast({ title: t('adminUsers.invitations.toast.revoked') })
            queryClient.invalidateQueries({ queryKey: ['invitations'] })
        },
        onError: (err) => toast({ variant: 'destructive', title: getApiErrorMessage(err) }),
    })

    const resendMutation = useMutation({
        mutationFn: (id: string) => invitationApi.resend(id),
        onSuccess: (res) => {
            toast({ title: t('adminUsers.invitations.toast.resent'), description: t('adminUsers.invitations.toast.linkUpdated', { link: res.data.invite_link.slice(0, 40) }) })
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
                title={t('nav.hrInvitations')}
                description={t('adminUsers.invitations.description')}
                actions={
                    <GuestHide>
                        <Can permission={PERMISSIONS.INVITATION_CREATE}>
                            <Button size="sm" onClick={() => setShowCreateDialog(true)}>
                                <Plus className="h-4 w-4 mr-2" />
                                {t('adminUsers.invitations.addButton')}
                            </Button>
                        </Can>
                    </GuestHide>
                }
            />

            {/* Status filter tabs */}
            <div className="flex gap-1 border-b">
                {STATUS_TABS.map(tab => (
                    <button
                        key={tab}
                        onClick={() => { setStatusFilter(tab); setPage(1) }}
                        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                            statusFilter === tab
                                ? 'border-primary text-primary'
                                : 'border-transparent text-muted-foreground hover:text-foreground'
                        }`}
                    >
                        {t(STATUS_LABEL_KEYS[tab])}
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
                                <TableHead style={{ minWidth: 150 }} className="hidden @[750px]:table-cell">{t('adminUsers.invitations.col.organization')}</TableHead>
                                <TableHead style={{ width: 90 }} className="whitespace-nowrap">{t('adminUsers.invitations.col.status')}</TableHead>
                                <TableHead style={{ minWidth: 96 }} className="hidden @[750px]:table-cell whitespace-nowrap">{t('adminUsers.invitations.col.invitedBy')}</TableHead>
                                <TableHead style={{ minWidth: 104 }} className="hidden @[750px]:table-cell whitespace-nowrap">{t('adminUsers.invitations.col.createdAt')}</TableHead>
                                <TableHead style={{ minWidth: 104 }} className="whitespace-nowrap">{t('adminUsers.invitations.col.expiresAt')}</TableHead>
                                <TableHead style={{ width: 80 }}>{t('common.actions')}</TableHead>
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
                                <TableEmptyRow colSpan={7} icon={Mail} title={t('adminUsers.invitations.empty')} />
                            ) : (
                                invitations.map(inv => (
                                    <TableRow key={inv.id}>
                                        <TableCell style={{ minWidth: 180 }} className="font-medium break-all">{inv.email}</TableCell>
                                        <TableCell style={{ minWidth: 150 }} className="hidden @[750px]:table-cell whitespace-normal break-words">{inv.organization || '-'}</TableCell>
                                        <TableCell style={{ width: 90 }} className="whitespace-nowrap">
                                            <Badge variant={invitationStatusColors[inv.status]}>
                                                {t(STATUS_LABEL_KEYS[inv.status])}
                                            </Badge>
                                        </TableCell>
                                        <TableCell style={{ minWidth: 96 }} className="hidden @[750px]:table-cell whitespace-nowrap">{inv.invited_by_name}</TableCell>
                                        <TableCell style={{ minWidth: 104 }} className="hidden @[750px]:table-cell whitespace-nowrap text-xs text-muted-foreground">{formatDate(inv.created_at)}</TableCell>
                                        <TableCell style={{ minWidth: 104 }} className="whitespace-nowrap text-xs text-muted-foreground">{formatDate(inv.expires_at)}</TableCell>
                                        <TableCell style={{ width: 80 }}>
                                            <GuestHide>
                                                {(inv.status === 'pending' || inv.status === 'expired') && (
                                                    <div className="flex gap-1">
                                                        <Can permission={PERMISSIONS.INVITATION_RESEND}>
                                                            <Button variant="ghost" size="icon" onClick={() => resendMutation.mutate(inv.id)} disabled={resendMutation.isPending} title={t('adminUsers.invitations.resend')}>
                                                                {resendMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCw className="h-4 w-4" />}
                                                            </Button>
                                                        </Can>
                                                        {inv.status === 'pending' && (
                                                            <Can permission={PERMISSIONS.INVITATION_REVOKE}>
                                                                <Button variant="ghost" size="icon" onClick={() => revokeMutation.mutate(inv.id)} disabled={revokeMutation.isPending} title={t('adminUsers.invitations.revoke')}>
                                                                    <XCircle className="h-4 w-4 text-destructive" />
                                                                </Button>
                                                            </Can>
                                                        )}
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
                            <p className="text-sm">{t('adminUsers.invitations.empty')}</p>
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
                                        {t(STATUS_LABEL_KEYS[inv.status])}
                                    </Badge>
                                </div>
                                <div className="text-xs text-muted-foreground">
                                    {t('adminUsers.invitations.cardMeta', { name: inv.invited_by_name, created: formatDate(inv.created_at), expires: formatDate(inv.expires_at) })}
                                </div>
                                <GuestHide>
                                    {(inv.status === 'pending' || inv.status === 'expired') && (
                                        <div className="flex justify-end gap-1 pt-1 border-t">
                                            <Can permission={PERMISSIONS.INVITATION_RESEND}>
                                                <Button variant="ghost" size="icon" onClick={() => resendMutation.mutate(inv.id)} disabled={resendMutation.isPending} title={t('adminUsers.invitations.resend')}>
                                                    {resendMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCw className="h-4 w-4" />}
                                                </Button>
                                            </Can>
                                            {inv.status === 'pending' && (
                                                <Can permission={PERMISSIONS.INVITATION_REVOKE}>
                                                    <Button variant="ghost" size="icon" onClick={() => revokeMutation.mutate(inv.id)} disabled={revokeMutation.isPending} title={t('adminUsers.invitations.revoke')}>
                                                        <XCircle className="h-4 w-4 text-destructive" />
                                                    </Button>
                                                </Can>
                                            )}
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
                        {t('admin.userTable.pageInfo', { total, current: page, totalPages })}
                    </p>
                    <div className="flex gap-2">
                        <Button variant="outline" size="sm" onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page <= 1}>
                            {t('common.previous')}
                        </Button>
                        <Button variant="outline" size="sm" onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page >= totalPages}>
                            {t('common.next')}
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
