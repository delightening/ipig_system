import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'
import { Loader2, AlertOctagon, Clock } from 'lucide-react'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    animalId: string
    earTag: string
    iacucNo?: string | null
}

export function EuthanasiaOrderDialog({ open, onOpenChange, animalId, earTag, iacucNo }: Props) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const [reason, setReason] = useState('')
    const [confirmed, setConfirmed] = useState(false)

    // Reset when dialog opens
    useEffect(() => {
        if (open) {
            setReason('')
            setConfirmed(false)
        }
    }, [open])

    const mutation = useMutation({
        mutationFn: async () => {
            const payload = {
                animal_id: animalId,
                reason: reason,
            }
            return api.post('/euthanasia/orders', payload)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
            // 待處理安樂死單清單用 ['euthanasia-pending']（EuthanasiaPendingPanel）；
            // 原 ['euthanasia-orders'] 無 query 消費（死 key）→ 開單後待處理面板不刷新。
            queryClient.invalidateQueries({ queryKey: ['euthanasia-pending'] })
            toast({
                title: t('animalActions.euthanasia.order.createdTitle'),
                description: t('animalActions.euthanasia.order.createdDescription'),
            })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('animalActions.euthanasia.order.createFailed')),
                variant: 'destructive',
            })
        },
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!reason.trim()) {
            toast({ title: t('common.error'), description: t('animalActions.euthanasia.order.reasonRequired'), variant: 'destructive' })
            return
        }
        if (!confirmed) {
            toast({ title: t('common.error'), description: t('animalActions.euthanasia.order.confirmRequired'), variant: 'destructive' })
            return
        }
        mutation.mutate()
    }

    // Calculate deadline (24 hours from now)
    const deadline = new Date()
    deadline.setHours(deadline.getHours() + 24)
    const deadlineStr = deadline.toLocaleString(uiLocale(), {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    })

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-error-text">
                        <AlertOctagon className="h-5 w-5" />
                        {t('animalActions.euthanasia.order.title')}
                    </DialogTitle>
                    <DialogDescription>
                        <span className="font-medium">{t('animalActions.common.earTagLabel', { earTag })}</span>
                        {iacucNo && <span className="ml-4">IACUC No.: {iacucNo}</span>}
                    </DialogDescription>
                </DialogHeader>

                <form onSubmit={handleSubmit} className="space-y-4">
                    {/* Warning Box */}
                    <div className="bg-status-error-bg border border-status-error-border rounded-lg p-4">
                        <h4 className="font-medium text-status-error-text mb-2 flex items-center gap-2">
                            <Clock className="h-4 w-4" />
                            {t('animalActions.euthanasia.order.notesTitle')}
                        </h4>
                        <ul className="text-sm text-status-error-text space-y-1 list-disc pl-5">
                            <li>{t('animalActions.euthanasia.order.note1')}</li>
                            <li>
                                <Trans
                                    i18nKey="animalActions.euthanasia.order.note2"
                                    components={{ strong: <strong /> }}
                                />
                            </li>
                            <li>{t('animalActions.euthanasia.order.note3')}</li>
                            <li>
                                <strong>{t('animalActions.euthanasia.order.executionDeadline', { deadline: deadlineStr })}</strong>
                            </li>
                        </ul>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="reason">{t('animalActions.euthanasia.order.reasonLabel')}</Label>
                        <Textarea
                            id="reason"
                            value={reason}
                            onChange={(e) => setReason(e.target.value)}
                            placeholder={t('animalActions.euthanasia.order.reasonHint')}
                            className="min-h-[120px]"
                            required
                        />
                    </div>

                    <label className="flex items-start gap-2 cursor-pointer">
                        <input
                            type="checkbox"
                            id="confirm"
                            checked={confirmed}
                            onChange={(e) => setConfirmed(e.target.checked)}
                            className="h-4 w-4 mt-1 text-status-error-text rounded"
                        />
                        <span className="text-sm text-muted-foreground">
                            {t('animalActions.euthanasia.order.confirmCheckbox')}
                        </span>
                    </label>

                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                            {t('common.cancel')}
                        </Button>
                        <Button
                            type="submit"
                            className="bg-destructive hover:bg-destructive/90"
                            disabled={mutation.isPending || !reason.trim() || !confirmed}
                        >
                            {mutation.isPending ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                                <AlertOctagon className="h-4 w-4 mr-2" />
                            )}
                            {t('animalActions.euthanasia.order.confirmCreate')}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
