import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Ban, ShieldOff, Plus } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'

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
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { TableEmptyRow, TableErrorRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { ipBlocklistApi, type IpBlocklistEntry } from '@/lib/api/ipBlocklist'
import { formatDate } from '@/lib/utils'

function sourceBadge(source: string, t: (key: string) => string) {
    switch (source) {
        case 'honeypot':
            return <Badge variant="destructive">{t('admin.ipBlocklistTab.sourceHoneypot')}</Badge>
        case 'R22-6_idor':
            return <Badge variant="destructive">{t('admin.ipBlocklistTab.sourceIdor')}</Badge>
        case 'R22-1_ratelimit':
            return <Badge variant="warning">{t('admin.ipBlocklistTab.sourceRatelimit')}</Badge>
        case 'manual':
            return <Badge variant="secondary">{t('admin.ipBlocklistTab.sourceManual')}</Badge>
        default:
            return <Badge variant="secondary">{source}</Badge>
    }
}

export function IpBlocklistTab() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [showHistory, setShowHistory] = useState(false)
    const [addOpen, setAddOpen] = useState(false)
    const [addIp, setAddIp] = useState('')
    const [addReason, setAddReason] = useState('')
    const [addTtl, setAddTtl] = useState<string>('')
    const [unblockTarget, setUnblockTarget] = useState<IpBlocklistEntry | null>(null)
    const [unblockReason, setUnblockReason] = useState('')

    const { data, isLoading, isError } = useQuery({
        queryKey: ['ip-blocklist', { only_active: !showHistory }],
        queryFn: () =>
            ipBlocklistApi
                .list({ only_active: !showHistory, limit: 200 })
                .then((r) => r.data),
    })

    const addMutation = useMutation({
        mutationFn: ipBlocklistApi.add,
        onSuccess: () => {
            toast({ title: t('admin.ipBlocklistTab.toastAdded') })
            queryClient.invalidateQueries({ queryKey: ['ip-blocklist'] })
            setAddOpen(false)
            setAddIp('')
            setAddReason('')
            setAddTtl('')
        },
    })

    const unblockMutation = useMutation({
        mutationFn: ({ id, reason }: { id: string; reason: string }) =>
            ipBlocklistApi.unblock(id, reason),
        onSuccess: () => {
            toast({ title: t('admin.ipBlocklistTab.toastUnblocked') })
            queryClient.invalidateQueries({ queryKey: ['ip-blocklist'] })
            setUnblockTarget(null)
            setUnblockReason('')
        },
    })

    const rows = data ?? []

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle className="flex items-center gap-2">
                        <Ban className="h-5 w-5" />
                        {t('admin.ipBlocklistTab.title')}
                    </CardTitle>
                    <CardDescription>
                        {t('admin.ipBlocklistTab.description')}
                    </CardDescription>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setShowHistory((v) => !v)}
                    >
                        {showHistory
                            ? t('admin.ipBlocklistTab.showActiveOnly')
                            : t('admin.ipBlocklistTab.showWithHistory')}
                    </Button>
                    <Button size="sm" onClick={() => setAddOpen(true)}>
                        <Plus className="h-4 w-4 mr-1" />
                        {t('admin.ipBlocklistTab.manualAdd')}
                    </Button>
                </div>
            </CardHeader>
            <CardContent>
                <div className="rounded-lg bg-card overflow-hidden [&>div]:overflow-x-hidden">
                    <Table>
                        <TableHeader className="bg-muted/50 hover:bg-muted/50">
                            <TableRow>
                                <TableHead>{t('admin.ipBlocklistTab.colIp')}</TableHead>
                                <TableHead>{t('admin.ipBlocklistTab.colSource')}</TableHead>
                                <TableHead>{t('admin.ipBlocklistTab.colReason')}</TableHead>
                                <TableHead>{t('admin.ipBlocklistTab.colBlockedAt')}</TableHead>
                                <TableHead>{t('admin.ipBlocklistTab.colExpiry')}</TableHead>
                                <TableHead className="text-right">{t('admin.ipBlocklistTab.colHits')}</TableHead>
                                <TableHead className="w-32 text-right">{t('common.actions')}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={7} className="p-0">
                                        <TableSkeleton rows={5} cols={7} />
                                    </TableCell>
                                </TableRow>
                            ) : isError && rows.length === 0 ? (
                                <TableErrorRow colSpan={7} />
                            ) : rows.length === 0 ? (
                                <TableEmptyRow colSpan={7} icon={Ban} title={t('admin.ipBlocklistTab.emptyTitle')} />
                            ) : (
                                rows.map((row) => {
                                    const isActive = !row.unblocked_at
                                    return (
                                        <TableRow key={row.id}>
                                            <TableCell className="font-mono">
                                                {row.ip_address}
                                            </TableCell>
                                            <TableCell>{sourceBadge(row.source, t)}</TableCell>
                                            <TableCell className="max-w-xs">
                                                {row.reason}
                                            </TableCell>
                                            <TableCell>{formatDate(row.blocked_at)}</TableCell>
                                            <TableCell>
                                                {row.unblocked_at
                                                    ? t('admin.ipBlocklistTab.unblockedAt', {
                                                          date: formatDate(row.unblocked_at),
                                                      })
                                                    : row.blocked_until
                                                        ? formatDate(row.blocked_until)
                                                        : t('admin.ipBlocklistTab.permanent')}
                                            </TableCell>
                                            <TableCell className="text-right tabular-nums">
                                                {row.hit_count}
                                            </TableCell>
                                            <TableCell className="text-right">
                                                {isActive && (
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => setUnblockTarget(row)}
                                                    >
                                                        <ShieldOff className="h-4 w-4 mr-1" />
                                                        {t('admin.ipBlocklistTab.unblock')}
                                                    </Button>
                                                )}
                                            </TableCell>
                                        </TableRow>
                                    )
                                })
                            )}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>

            {/* 手動新增 Dialog */}
            <Dialog open={addOpen} onOpenChange={setAddOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{t('admin.ipBlocklistTab.addDialogTitle')}</DialogTitle>
                        <DialogDescription>
                            {t('admin.ipBlocklistTab.addDialogDescription')}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-3">
                        <div>
                            <Label htmlFor="ip">{t('admin.ipBlocklistTab.ipLabel')}</Label>
                            <Input
                                id="ip"
                                placeholder={t('admin.ipBlocklistTab.ipPlaceholder')}
                                value={addIp}
                                onChange={(e) => setAddIp(e.target.value)}
                            />
                        </div>
                        <div>
                            <Label htmlFor="reason">{t('admin.ipBlocklistTab.colReason')}</Label>
                            <Input
                                id="reason"
                                placeholder={t('admin.ipBlocklistTab.reasonPlaceholder')}
                                value={addReason}
                                onChange={(e) => setAddReason(e.target.value)}
                            />
                        </div>
                        <div>
                            <Label htmlFor="ttl">{t('admin.ipBlocklistTab.ttlLabel')}</Label>
                            <Input
                                id="ttl"
                                type="number"
                                min={1}
                                placeholder={t('admin.ipBlocklistTab.ttlPlaceholder')}
                                value={addTtl}
                                onChange={(e) => setAddTtl(e.target.value)}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setAddOpen(false)}>
                            {t('common.cancel')}
                        </Button>
                        <Button
                            disabled={!addIp || !addReason || addMutation.isPending}
                            onClick={() =>
                                addMutation.mutate({
                                    ip_address: addIp.trim(),
                                    reason: addReason.trim(),
                                    ttl_hours: addTtl ? Number(addTtl) : null,
                                })
                            }
                        >
                            {t('admin.ipBlocklistTab.blockAction')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 解除封鎖 Dialog */}
            <Dialog open={!!unblockTarget} onOpenChange={(o) => !o && setUnblockTarget(null)}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{t('admin.ipBlocklistTab.unblockDialogTitle')}</DialogTitle>
                        <DialogDescription>
                            {t('admin.ipBlocklistTab.unblockDialogDescription', {
                                ip: unblockTarget?.ip_address,
                            })}
                        </DialogDescription>
                    </DialogHeader>
                    <div>
                        <Label htmlFor="unblockReason">{t('admin.ipBlocklistTab.unblockReasonLabel')}</Label>
                        <Input
                            id="unblockReason"
                            placeholder={t('admin.ipBlocklistTab.unblockReasonPlaceholder')}
                            value={unblockReason}
                            onChange={(e) => setUnblockReason(e.target.value)}
                        />
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setUnblockTarget(null)}>
                            {t('common.cancel')}
                        </Button>
                        <Button
                            disabled={!unblockReason || unblockMutation.isPending}
                            onClick={() =>
                                unblockTarget &&
                                unblockMutation.mutate({
                                    id: unblockTarget.id,
                                    reason: unblockReason.trim(),
                                })
                            }
                        >
                            {t('admin.ipBlocklistTab.unblock')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </Card>
    )
}
