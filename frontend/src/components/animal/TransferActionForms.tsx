import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { transferApi } from '@/lib/api'
import type { AnimalTransfer } from '@/lib/api'
import { useAssignableProtocols } from '@/hooks/useAssignableProtocols'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Stethoscope, FileCheck, Loader2, AlertTriangle } from 'lucide-react'

// --- Vet Evaluate Form ---

export function VetEvaluateForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const { t } = useTranslation()
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
            toast({ title: t('common.success'), description: t('animalActions.transfer.vetEvaluate.submitted') })
            setHealthStatus('')
            setConditions('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.vetEvaluate.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-success-border">
            <CardHeader>
                <CardTitle className="text-base flex items-center gap-2">
                    <Stethoscope className="h-4 w-4 text-status-success-text" />
                    {t('animalActions.transfer.vetEvaluate.title')}
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <div className="space-y-2">
                    <Label>{t('animalActions.transfer.vetEvaluate.healthStatusLabel')}</Label>
                    <Textarea
                        value={healthStatus}
                        onChange={e => setHealthStatus(e.target.value)}
                        placeholder={t('animalActions.transfer.vetEvaluate.healthStatusHint')}
                        className="min-h-[60px]"
                    />
                </div>
                <div className="flex items-center gap-3">
                    <Label>{t('animalActions.transfer.vetEvaluate.fitLabel')}</Label>
                    <Select value={fit ? 'yes' : 'no'} onValueChange={v => setFit(v === 'yes')}>
                        <SelectTrigger className="w-[140px]">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="yes">{t('animalActions.transfer.vetEvaluate.fit')}</SelectItem>
                            <SelectItem value="no">{t('animalActions.transfer.vetEvaluate.notFit')}</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
                <div className="space-y-2">
                    <Label>{t('animalActions.transfer.vetEvaluate.conditionsLabel')}</Label>
                    <Input
                        value={conditions}
                        onChange={e => setConditions(e.target.value)}
                        placeholder={t('animalActions.transfer.vetEvaluate.conditionsHint')}
                    />
                </div>
                <Button
                    onClick={() => mutation.mutate()}
                    disabled={!healthStatus.trim() || mutation.isPending}
                    className="bg-emerald-600 hover:bg-emerald-700"
                >
                    {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                    {t('animalActions.transfer.vetEvaluate.submit')}
                </Button>
            </CardContent>
        </Card>
    )
}

// --- Assign Plan Form ---

export function AssignPlanForm({ transfer, invalidate }: { transfer: AnimalTransfer; invalidate: () => void }) {
    const { t } = useTranslation()
    const [targetIacuc, setTargetIacuc] = useState('')

    const { data: approvedProtocols } = useAssignableProtocols()

    const mutation = useMutation({
        mutationFn: () => transferApi.assignPlan(transfer.id, { to_iacuc_no: targetIacuc }),
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('animalActions.transfer.status.plan_assigned') })
            setTargetIacuc('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.assignPlan.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-info-border">
            <CardHeader>
                <CardTitle className="text-base flex items-center gap-2">
                    <FileCheck className="h-4 w-4 text-status-info-text" />
                    {t('animalActions.transfer.assignPlan.title')}
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <div className="space-y-2">
                    <Label>{t('animalActions.transfer.assignPlan.targetLabel')}</Label>
                    <Select value={targetIacuc} onValueChange={setTargetIacuc}>
                        <SelectTrigger>
                            <SelectValue placeholder={t('animalActions.transfer.assignPlan.targetHint')} />
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
                    {t('animalActions.transfer.assignPlan.confirm')}
                </Button>
            </CardContent>
        </Card>
    )
}

// --- Reject Form ---

export function RejectForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const { t } = useTranslation()
    const [reason, setReason] = useState('')

    const mutation = useMutation({
        mutationFn: () => transferApi.reject(transferId, { reason }),
        onSuccess: () => {
            toast({ title: t('animalActions.transfer.status.rejected'), description: t('animalActions.transfer.reject.rejectedDescription') })
            setReason('')
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.reject.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-error-border">
            <CardContent className="pt-4 space-y-3">
                <div className="flex items-center gap-2 text-status-error-text text-sm font-medium">
                    <AlertTriangle className="h-4 w-4" />
                    {t('animalActions.transfer.reject.title')}
                </div>
                <div className="flex gap-2">
                    <Input
                        value={reason}
                        onChange={e => setReason(e.target.value)}
                        placeholder={t('animalActions.transfer.reject.reasonHint')}
                        className="flex-1"
                    />
                    <Button
                        variant="destructive"
                        onClick={() => mutation.mutate()}
                        disabled={!reason.trim() || mutation.isPending}
                    >
                        {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {t('animalActions.transfer.reject.action')}
                    </Button>
                </div>
            </CardContent>
        </Card>
    )
}
