import { useState } from 'react'
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
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { ipBlocklistApi, type IpBlocklistEntry } from '@/lib/api/ipBlocklist'
import { formatDate } from '@/lib/utils'

function sourceBadge(source: string) {
    switch (source) {
        case 'honeypot':
            return <Badge variant="destructive">蜜罐命中（永久）</Badge>
        case 'R22-6_idor':
            return <Badge variant="destructive">IDOR 探測</Badge>
        case 'R22-1_ratelimit':
            return <Badge variant="warning">速率限制升級</Badge>
        case 'manual':
            return <Badge variant="secondary">手動</Badge>
        default:
            return <Badge variant="secondary">{source}</Badge>
    }
}

export function IpBlocklistTab() {
    const queryClient = useQueryClient()
    const [showHistory, setShowHistory] = useState(false)
    const [addOpen, setAddOpen] = useState(false)
    const [addIp, setAddIp] = useState('')
    const [addReason, setAddReason] = useState('')
    const [addTtl, setAddTtl] = useState<string>('')
    const [unblockTarget, setUnblockTarget] = useState<IpBlocklistEntry | null>(null)
    const [unblockReason, setUnblockReason] = useState('')

    const { data, isLoading } = useQuery({
        queryKey: ['ip-blocklist', { only_active: !showHistory }],
        queryFn: () =>
            ipBlocklistApi
                .list({ only_active: !showHistory, limit: 200 })
                .then((r) => r.data),
    })

    const addMutation = useMutation({
        mutationFn: ipBlocklistApi.add,
        onSuccess: () => {
            toast({ title: '已新增 IP 黑名單' })
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
            toast({ title: '已解除封鎖' })
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
                        IP 黑名單
                    </CardTitle>
                    <CardDescription>
                        自動封鎖（IDOR 探測 / 速率限制升級 / 蜜罐命中）與手動維護
                    </CardDescription>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setShowHistory((v) => !v)}
                    >
                        {showHistory ? '只看生效中' : '含歷史紀錄'}
                    </Button>
                    <Button size="sm" onClick={() => setAddOpen(true)}>
                        <Plus className="h-4 w-4 mr-1" />
                        手動新增
                    </Button>
                </div>
            </CardHeader>
            <CardContent>
                <div className="rounded-lg bg-card overflow-hidden [&>div]:overflow-x-hidden">
                    <Table>
                        <TableHeader className="bg-muted/50 hover:bg-muted/50">
                            <TableRow>
                                <TableHead>IP</TableHead>
                                <TableHead>來源</TableHead>
                                <TableHead>原因</TableHead>
                                <TableHead>封鎖時間</TableHead>
                                <TableHead>到期</TableHead>
                                <TableHead className="text-right">命中</TableHead>
                                <TableHead className="w-32 text-right">操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={7} className="p-0">
                                        <TableSkeleton rows={5} cols={7} />
                                    </TableCell>
                                </TableRow>
                            ) : rows.length === 0 ? (
                                <TableEmptyRow colSpan={7} icon={Ban} title="目前沒有封鎖中的 IP" />
                            ) : (
                                rows.map((row) => {
                                    const isActive = !row.unblocked_at
                                    return (
                                        <TableRow key={row.id}>
                                            <TableCell className="font-mono">
                                                {row.ip_address}
                                            </TableCell>
                                            <TableCell>{sourceBadge(row.source)}</TableCell>
                                            <TableCell className="max-w-xs">
                                                {row.reason}
                                            </TableCell>
                                            <TableCell>{formatDate(row.blocked_at)}</TableCell>
                                            <TableCell>
                                                {row.unblocked_at
                                                    ? `已解除 ${formatDate(row.unblocked_at)}`
                                                    : row.blocked_until
                                                        ? formatDate(row.blocked_until)
                                                        : '永久'}
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
                                                        解除
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
                        <DialogTitle>手動封鎖 IP</DialogTitle>
                        <DialogDescription>
                            新增一筆 IP 黑名單；TTL 留空表示永久。
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-3">
                        <div>
                            <Label htmlFor="ip">IP 位址</Label>
                            <Input
                                id="ip"
                                placeholder="例：203.0.113.50 或 2001:db8::1"
                                value={addIp}
                                onChange={(e) => setAddIp(e.target.value)}
                            />
                        </div>
                        <div>
                            <Label htmlFor="reason">原因</Label>
                            <Input
                                id="reason"
                                placeholder="描述封鎖理由（供後續稽核）"
                                value={addReason}
                                onChange={(e) => setAddReason(e.target.value)}
                            />
                        </div>
                        <div>
                            <Label htmlFor="ttl">TTL 小時（選填）</Label>
                            <Input
                                id="ttl"
                                type="number"
                                min={1}
                                placeholder="留空 = 永久"
                                value={addTtl}
                                onChange={(e) => setAddTtl(e.target.value)}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setAddOpen(false)}>
                            取消
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
                            封鎖
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 解除封鎖 Dialog */}
            <Dialog open={!!unblockTarget} onOpenChange={(o) => !o && setUnblockTarget(null)}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>解除封鎖</DialogTitle>
                        <DialogDescription>
                            即將解除 {unblockTarget?.ip_address} 的封鎖。
                        </DialogDescription>
                    </DialogHeader>
                    <div>
                        <Label htmlFor="unblockReason">解除原因</Label>
                        <Input
                            id="unblockReason"
                            placeholder="描述解除理由（供後續稽核）"
                            value={unblockReason}
                            onChange={(e) => setUnblockReason(e.target.value)}
                        />
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setUnblockTarget(null)}>
                            取消
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
                            解除
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </Card>
    )
}
