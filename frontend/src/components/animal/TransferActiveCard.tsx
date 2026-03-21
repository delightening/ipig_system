import { useQueryClient } from '@tanstack/react-query'

import type { AnimalTransfer } from '@/lib/api'
import { transferStatusNames, transferTypeNames } from '@/lib/api'
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
    const queryClient = useQueryClient()
    const invalidate = useTransferInvalidate(animalId, queryClient)

    return (
        <Card className="border-indigo-200 bg-indigo-50/30">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <CardTitle className="text-lg flex items-center gap-2">
                        <ArrowRightLeft className="h-5 w-5 text-indigo-600" />
                        進行中的轉讓
                    </CardTitle>
                    <Badge className="bg-indigo-100 text-indigo-700">{transferStatusNames[transfer.status]}</Badge>
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
    return (
        <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
                <span className="text-slate-500">轉讓類型</span>
                <p className="font-medium">{transferTypeNames[transfer.transfer_type === 'external' ? 'external' : 'internal']}</p>
            </div>
            <div>
                <span className="text-slate-500">原計劃</span>
                <p className="font-medium">{transfer.from_iacuc_no}</p>
            </div>
            {transfer.to_iacuc_no && (
                <div>
                    <span className="text-slate-500">新計劃</span>
                    <p className="font-medium">{transfer.to_iacuc_no}</p>
                </div>
            )}
            <div className="col-span-2">
                <span className="text-slate-500">原因</span>
                <p className="font-medium">{transfer.reason}</p>
            </div>
            {transfer.remark && (
                <div className="col-span-2">
                    <span className="text-slate-500">備註</span>
                    <p>{transfer.remark}</p>
                </div>
            )}
            <div>
                <span className="text-slate-500">發起時間</span>
                <p>{new Date(transfer.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}</p>
            </div>
        </div>
    )
}
