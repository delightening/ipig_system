import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { transferApi } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { ArrowRightLeft, Loader2 } from 'lucide-react'

import { useTransferInvalidate } from './useTransferMutations'

interface TransferInitiateFormProps {
    animalId: string
    earTag: string
    onClose: () => void
}

export function TransferInitiateForm({ animalId, earTag, onClose }: TransferInitiateFormProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const invalidate = useTransferInvalidate(animalId, queryClient)

    const [reason, setReason] = useState('')
    const [remark, setRemark] = useState('')
    const [transferType, setTransferType] = useState<'external' | 'internal'>('internal')

    const initiateMutation = useMutation({
        mutationFn: () => transferApi.initiate(animalId, {
            reason,
            remark: remark || undefined,
            transfer_type: transferType,
        }),
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('animalActions.transfer.initiate.submitted') })
            onClose()
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.initiate.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-info-border">
            <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                    <ArrowRightLeft className="h-5 w-5 text-status-info-text" />
                    {t('animalActions.transfer.initiate.title', { earTag })}
                </CardTitle>
                <CardDescription>
                    {t('animalActions.transfer.initiate.description')}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="space-y-2">
                    <Label htmlFor="transfer-reason">{t('animalActions.transfer.initiate.reasonLabel')}</Label>
                    <Textarea
                        id="transfer-reason"
                        value={reason}
                        onChange={e => setReason(e.target.value)}
                        placeholder={t('animalActions.transfer.initiate.reasonHint')}
                        className="min-h-[80px]"
                    />
                </div>
                <div className="space-y-2">
                    <Label>{t('animalActions.transfer.typeLabel')}</Label>
                    <Select value={transferType} onValueChange={(v: 'external' | 'internal') => setTransferType(v)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="internal">{t('animalActions.transfer.type.internal')}</SelectItem>
                            <SelectItem value="external">{t('animalActions.transfer.type.external')}</SelectItem>
                        </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">
                        {transferType === 'external' ? t('animalActions.transfer.initiate.externalHint') : t('animalActions.transfer.initiate.internalHint')}
                    </p>
                </div>
                <div className="space-y-2">
                    <Label htmlFor="transfer-remark">{t('animalActions.common.notes')}</Label>
                    <Input
                        id="transfer-remark"
                        value={remark}
                        onChange={e => setRemark(e.target.value)}
                        placeholder={t('animalActions.transfer.initiate.remarkHint')}
                    />
                </div>
                <div className="flex gap-2">
                    <Button
                        onClick={() => initiateMutation.mutate()}
                        disabled={!reason.trim() || initiateMutation.isPending}
                        className="bg-primary hover:bg-primary/90"
                    >
                        {initiateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {t('animalActions.transfer.initiate.confirm')}
                    </Button>
                    <Button variant="outline" onClick={onClose}>{t('common.cancel')}</Button>
                </div>
            </CardContent>
        </Card>
    )
}
