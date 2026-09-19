import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { AMENDMENT_CHANGE_ITEM_OPTIONS } from '@/lib/api'
import type {
    Amendment,
    CreateHistoricalAmendmentRequest,
    RecordHistoricalReviewsRequest,
} from '@/types/amendment'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2 } from 'lucide-react'
import { HistoricalReviewersEditor, type ReviewerRow } from './HistoricalReviewersEditor'

interface Props {
    protocolId: string
    open: boolean
    onOpenChange: (open: boolean) => void
    onSuccess: () => void
}

const toIso = (d: string) => (d ? new Date(d).toISOString() : undefined)

export function HistoricalAmendmentDialog({ protocolId, open, onOpenChange, onSuccess }: Props) {
    const { t } = useTranslation()
    const [title, setTitle] = useState('')
    const [description, setDescription] = useState('')
    const [changeItems, setChangeItems] = useState<string[]>([])
    const [amendmentType, setAmendmentType] = useState<'MAJOR' | 'MINOR'>('MINOR')
    const [submittedAt, setSubmittedAt] = useState('')
    const [classifiedAt, setClassifiedAt] = useState('')
    const [effectiveFrom, setEffectiveFrom] = useState('')
    const [classificationRemark, setClassificationRemark] = useState('')
    const [reviewers, setReviewers] = useState<ReviewerRow[]>([])

    const reset = () => {
        setTitle(''); setDescription(''); setChangeItems([]); setAmendmentType('MINOR')
        setSubmittedAt(''); setClassifiedAt(''); setEffectiveFrom(''); setClassificationRemark('')
        setReviewers([])
    }

    const mutation = useMutation({
        mutationFn: async () => {
            const createReq: CreateHistoricalAmendmentRequest = {
                protocol_id: protocolId,
                title: title.trim(),
                description: description.trim() || undefined,
                change_items: changeItems.length ? changeItems : undefined,
                amendment_type: amendmentType,
                submitted_at: toIso(submittedAt),
                classified_at: toIso(classifiedAt),
                classification_remark: classificationRemark.trim() || undefined,
            }
            const created = await api.post<Amendment>('/amendments/historical', createReq)
            const id = created.data.id
            if (amendmentType === 'MAJOR') {
                const valid = reviewers.filter((r) => r.reviewer_id || r.reviewer_name.trim())
                if (valid.length) {
                    const reviewReq: RecordHistoricalReviewsRequest = {
                        reviewers: valid.map((r) => ({
                            reviewer_id: r.reviewer_id ?? undefined,
                            reviewer_name: r.reviewer_id ? undefined : r.reviewer_name.trim(),
                            decision: r.decision,
                            comment: r.comment.trim() || undefined,
                        })),
                    }
                    await api.post(`/amendments/${id}/historical-reviews`, reviewReq)
                }
            }
            await api.post(`/amendments/${id}/finalize-historical`, { effective_from: toIso(effectiveFrom) })
        },
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('protocolComponents.historicalAmendment.success') })
            onSuccess()
            reset()
            onOpenChange(false)
        },
        onError: (error: unknown) =>
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('protocolComponents.historicalAmendment.failed')), variant: 'destructive' }),
    })

    const submit = () => {
        if (!title.trim()) {
            toast({ title: t('common.error'), description: t('protocolComponents.historicalAmendment.titleRequired'), variant: 'destructive' })
            return
        }
        if (!effectiveFrom) {
            toast({ title: t('common.error'), description: t('protocolComponents.historicalAmendment.effectiveFromRequired'), variant: 'destructive' })
            return
        }
        mutation.mutate()
    }

    const toggleItem = (v: string) =>
        setChangeItems((prev) => (prev.includes(v) ? prev.filter((x) => x !== v) : [...prev, v]))

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="lg" className="max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>{t('protocolComponents.historicalAmendment.title')}</DialogTitle>
                    <DialogDescription>
                        {t('protocolComponents.historicalAmendment.description')}
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-2">
                    <div className="space-y-2">
                        <Label htmlFor="hist-title">{t('protocolComponents.historicalAmendment.fields.title')}</Label>
                        <Input id="hist-title" value={title} onChange={(e) => setTitle(e.target.value)} placeholder={t('protocolComponents.historicalAmendment.fields.titlePlaceholder')} />
                    </div>
                    <div className="space-y-2">
                        <Label>{t('protocolComponents.historicalAmendment.fields.type')}</Label>
                        <div className="flex gap-2">
                            <Button type="button" variant={amendmentType === 'MINOR' ? 'default' : 'outline'} onClick={() => setAmendmentType('MINOR')}>
                                {t('protocolComponents.historicalAmendment.fields.typeMinor')}
                            </Button>
                            <Button type="button" variant={amendmentType === 'MAJOR' ? 'default' : 'outline'} onClick={() => setAmendmentType('MAJOR')}>
                                {t('protocolComponents.historicalAmendment.fields.typeMajor')}
                            </Button>
                        </div>
                    </div>
                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                        <div className="space-y-1">
                            <Label htmlFor="hist-submitted">{t('protocolComponents.historicalAmendment.fields.submittedAt')}</Label>
                            <Input id="hist-submitted" type="date" value={submittedAt} onChange={(e) => setSubmittedAt(e.target.value)} />
                        </div>
                        <div className="space-y-1">
                            <Label htmlFor="hist-classified">{t('protocolComponents.historicalAmendment.fields.classifiedAt')}</Label>
                            <Input id="hist-classified" type="date" value={classifiedAt} onChange={(e) => setClassifiedAt(e.target.value)} />
                        </div>
                        <div className="space-y-1">
                            <Label htmlFor="hist-effective">{t('protocolComponents.historicalAmendment.fields.effectiveFrom')}</Label>
                            <Input id="hist-effective" type="date" value={effectiveFrom} onChange={(e) => setEffectiveFrom(e.target.value)} />
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label>{t('protocolComponents.historicalAmendment.fields.changeItems')}</Label>
                        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
                            {AMENDMENT_CHANGE_ITEM_OPTIONS.map((o) => (
                                <label key={o.value} className="flex items-center gap-2 text-sm cursor-pointer">
                                    <Checkbox checked={changeItems.includes(o.value)} onCheckedChange={() => toggleItem(o.value)} />
                                    {t(`amendments.changeItemLabels.${o.value}`, { defaultValue: o.label })}
                                </label>
                            ))}
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="hist-desc">{t('protocolComponents.historicalAmendment.fields.description')}</Label>
                        <Textarea id="hist-desc" value={description} onChange={(e) => setDescription(e.target.value)} rows={3} placeholder={t('protocolComponents.historicalAmendment.fields.descriptionPlaceholder')} />
                    </div>
                    {amendmentType === 'MAJOR' && (
                        <HistoricalReviewersEditor reviewers={reviewers} onChange={setReviewers} />
                    )}
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
                    <Button onClick={submit} disabled={mutation.isPending}>
                        {mutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                        {t('protocolComponents.historicalAmendment.submit')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
