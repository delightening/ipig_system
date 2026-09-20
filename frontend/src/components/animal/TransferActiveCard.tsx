import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import type { AnimalTransfer } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ArrowRightLeft } from 'lucide-react'

import { TransferStepper } from './TransferStepper'
import { useTransferInvalidate } from './useTransferMutations'
import { VetEvaluateForm, AssignPlanForm, RejectForm } from './TransferActionForms'
import { ApproveForm, CompleteForm } from './TransferSignatureForms'

interface TransferActiveCardProps {
    animalId: string
    transfer: AnimalTransfer
    canVetEvaluate: boolean
    canAssignPlan: boolean
    canApprove: boolean
    canComplete: boolean
    canReject: boolean
}

export function TransferActiveCard({
    animalId, transfer, canVetEvaluate, canAssignPlan, canApprove, canComplete, canReject,
}: TransferActiveCardProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const invalidate = useTransferInvalidate(animalId, queryClient)

    return (
        <Card className="border-status-info-border bg-status-info-bg/30">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <CardTitle className="text-lg flex items-center gap-2">
                        <ArrowRightLeft className="h-5 w-5 text-status-info-text" />
                        {t('animalActions.transfer.active.title')}
                    </CardTitle>
                    <Badge className="bg-indigo-100 text-status-info-text">{t(`animalActions.transfer.status.${transfer.status}`)}</Badge>
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                <TransferStepper transfer={transfer} />
                <TransferInfoGrid transfer={transfer} />
                {canVetEvaluate && <VetEvaluateForm transferId={transfer.id} invalidate={invalidate} />}
                {canAssignPlan && <AssignPlanForm transfer={transfer} invalidate={invalidate} />}
                {canApprove && <ApproveForm transferId={transfer.id} invalidate={invalidate} />}
                {canComplete && <CompleteForm transferId={transfer.id} invalidate={invalidate} />}
                {canReject && <RejectForm transferId={transfer.id} invalidate={invalidate} />}
            </CardContent>
        </Card>
    )
}

// --- Transfer Info Grid ---

function TransferInfoGrid({ transfer }: { transfer: AnimalTransfer }) {
    const { t } = useTranslation()
    return (
        <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
                <span className="text-muted-foreground">{t('animalActions.transfer.typeLabel')}</span>
                <p className="font-medium">{t(`animalActions.transfer.type.${transfer.transfer_type === 'external' ? 'external' : 'internal'}`)}</p>
            </div>
            <div>
                <span className="text-muted-foreground">{t('animalActions.transfer.active.fromPlan')}</span>
                <p className="font-medium">{transfer.from_iacuc_no}</p>
            </div>
            {transfer.to_iacuc_no && (
                <div>
                    <span className="text-muted-foreground">{t('animalActions.transfer.active.toPlan')}</span>
                    <p className="font-medium">{transfer.to_iacuc_no}</p>
                </div>
            )}
            <div className="col-span-2">
                <span className="text-muted-foreground">{t('animalActions.transfer.active.reason')}</span>
                <p className="font-medium">{transfer.reason}</p>
            </div>
            {transfer.remark && (
                <div className="col-span-2">
                    <span className="text-muted-foreground">{t('animalActions.common.notes')}</span>
                    <p>{transfer.remark}</p>
                </div>
            )}
            <div>
                <span className="text-muted-foreground">{t('animalActions.transfer.active.initiatedAt')}</span>
                <p>{new Date(transfer.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</p>
            </div>
        </div>
    )
}
