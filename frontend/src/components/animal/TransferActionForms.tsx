import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'

import { transferApi } from '@/lib/api'
import type { AnimalTransfer, ProtocolListItem } from '@/lib/api'
import api from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Stethoscope, FileCheck, Loader2, AlertTriangle } from 'lucide-react'

// --- Vet Evaluate Form ---

export function VetEvaluateForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const [healthStatus, setHealthStatus] = useState('')
    const [fit, setFit] = useState(true)
    const [conditions, setConditions] = useState('')

    const mutation = useMutation({
        mutationFn: () => transferApi.vetEvaluate(transferId, {
            health_status: healthStatus,
            is_fit_for_transfer: fit,
            conditions: conditions || undefined,
        }),
        onSuccess: () => {
            toast({ title: '成功', description: '獸醫評估已提交' })
            setHealthStatus('')
            setConditions('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '評估失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-success-border">
            <CardHeader>
                <CardTitle className="text-base flex items-center gap-2">
                    <Stethoscope className="h-4 w-4 text-status-success-text" />
                    獸醫評估
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <div className="space-y-2">
                    <Label>健康狀態 *</Label>
                    <Textarea
                        value={healthStatus}
                        onChange={e => setHealthStatus(e.target.value)}
                        placeholder="描述動物當前健康狀態..."
                        className="min-h-[60px]"
                    />
                </div>
                <div className="flex items-center gap-3">
                    <Label>是否適合轉讓</Label>
                    <Select value={fit ? 'yes' : 'no'} onValueChange={v => setFit(v === 'yes')}>
                        <SelectTrigger className="w-[140px]">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="yes">適合</SelectItem>
                            <SelectItem value="no">不適合</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
                <div className="space-y-2">
                    <Label>附帶條件</Label>
                    <Input
                        value={conditions}
                        onChange={e => setConditions(e.target.value)}
                        placeholder="如有附帶條件請說明"
                    />
                </div>
                <Button
                    onClick={() => mutation.mutate()}
                    disabled={!healthStatus.trim() || mutation.isPending}
                    className="bg-emerald-600 hover:bg-emerald-700"
                >
                    {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                    提交評估
                </Button>
            </CardContent>
        </Card>
    )
}

// --- Assign Plan Form ---

export function AssignPlanForm({ transfer, invalidate }: { transfer: AnimalTransfer; invalidate: () => void }) {
    const [targetIacuc, setTargetIacuc] = useState('')

    const { data: approvedProtocols } = useQuery({
        queryKey: ['approved-protocols'],
        queryFn: async () => {
            const res = await api.get<ProtocolListItem[]>('/protocols')
            return res.data.filter(p =>
                (p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && p.iacuc_no
            )
        },
        staleTime: 600_000,
    })

    const mutation = useMutation({
        mutationFn: () => transferApi.assignPlan(transfer.id, { to_iacuc_no: targetIacuc }),
        onSuccess: () => {
            toast({ title: '成功', description: '已指定新計劃' })
            setTargetIacuc('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '指定失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-info-border">
            <CardHeader>
                <CardTitle className="text-base flex items-center gap-2">
                    <FileCheck className="h-4 w-4 text-status-info-text" />
                    指定新計劃
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <div className="space-y-2">
                    <Label>目標 IACUC No. *</Label>
                    <Select value={targetIacuc} onValueChange={setTargetIacuc}>
                        <SelectTrigger>
                            <SelectValue placeholder="選擇目標計劃..." />
                        </SelectTrigger>
                        <SelectContent>
                            {approvedProtocols?.filter(p => p.iacuc_no !== transfer.from_iacuc_no).map(p => (
                                <SelectItem key={p.id} value={p.iacuc_no!}>
                                    {p.iacuc_no} — {p.title}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                <Button
                    onClick={() => mutation.mutate()}
                    disabled={!targetIacuc || mutation.isPending}
                    className="bg-primary hover:bg-primary/90"
                >
                    {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                    確認指定
                </Button>
            </CardContent>
        </Card>
    )
}

// --- Reject Form ---

export function RejectForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const [reason, setReason] = useState('')

    const mutation = useMutation({
        mutationFn: () => transferApi.reject(transferId, { reason }),
        onSuccess: () => {
            toast({ title: '已拒絕', description: '轉讓申請已拒絕' })
            setReason('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '拒絕失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-error-border">
            <CardContent className="pt-4 space-y-3">
                <div className="flex items-center gap-2 text-status-error-text text-sm font-medium">
                    <AlertTriangle className="h-4 w-4" />
                    拒絕轉讓
                </div>
                <div className="flex gap-2">
                    <Input
                        value={reason}
                        onChange={e => setReason(e.target.value)}
                        placeholder="拒絕原因（必填）"
                        className="flex-1"
                    />
                    <Button
                        variant="destructive"
                        onClick={() => mutation.mutate()}
                        disabled={!reason.trim() || mutation.isPending}
                    >
                        {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        拒絕
                    </Button>
                </div>
            </CardContent>
        </Card>
    )
}
