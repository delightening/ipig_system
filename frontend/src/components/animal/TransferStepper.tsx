import { CheckCircle2, Clock, FileCheck, Stethoscope, UserCheck, XCircle } from 'lucide-react'

import type { AnimalTransfer, AnimalTransferStatus } from '@/lib/api'

const TRANSFER_STEPS: { status: AnimalTransferStatus; label: string; icon: typeof Clock }[] = [
    { status: 'pending', label: '發起', icon: Clock },
    { status: 'vet_evaluated', label: '獸醫評估', icon: Stethoscope },
    { status: 'plan_assigned', label: '指定新計劃', icon: FileCheck },
    { status: 'pi_approved', label: 'PI 同意', icon: UserCheck },
    { status: 'completed', label: '完成', icon: CheckCircle2 },
]

function getStepIndex(status: AnimalTransferStatus): number {
    if (status === 'rejected') return -1
    return TRANSFER_STEPS.findIndex(s => s.status === status)
}

export function TransferStepper({ transfer }: { transfer: AnimalTransfer }) {
    const currentIdx = getStepIndex(transfer.status)
    const isRejected = transfer.status === 'rejected'

    return (
        <div className="flex items-center gap-1 w-full overflow-x-auto py-2">
            {TRANSFER_STEPS.map((step, idx) => {
                const Icon = step.icon
                const isDone = !isRejected && currentIdx >= idx
                const isCurrent = !isRejected && currentIdx === idx

                return (
                    <div key={step.status} className="flex items-center flex-1 min-w-0">
                        <div className={`flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium whitespace-nowrap
              ${isDone ? 'bg-indigo-100 text-status-info-text' : 'bg-muted text-muted-foreground'}
              ${isCurrent ? 'ring-2 ring-indigo-400' : ''}
            `}>
                            <Icon className="h-3.5 w-3.5 flex-shrink-0" />
                            <span className="hidden sm:inline">{step.label}</span>
                        </div>
                        {idx < TRANSFER_STEPS.length - 1 && (
                            <div className={`h-0.5 flex-1 mx-1 min-w-[12px] ${isDone && idx < currentIdx ? 'bg-indigo-400' : 'bg-muted'}`} />
                        )}
                    </div>
                )
            })}
            {isRejected && (
                <div className="flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium bg-status-error-bg text-status-error-text ring-2 ring-red-400 ml-2">
                    <XCircle className="h-3.5 w-3.5" />
                    <span>已拒絕</span>
                </div>
            )}
        </div>
    )
}
