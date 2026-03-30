import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import { transferApi } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ArrowRightLeft, Loader2, Plus } from 'lucide-react'
import { useAuthStore } from '@/stores/auth'

import { TransferInitiateForm } from './TransferInitiateForm'
import { TransferActiveCard } from './TransferActiveCard'
import { TransferHistoryList } from './TransferHistoryList'

// ============================================
// Props
// ============================================

interface Props {
    animalId: string
    animalStatus: string
    earTag: string
}

// ============================================
// 主元件
// ============================================

export function TransferTab({ animalId, animalStatus, earTag }: Props) {
    const user = useAuthStore(s => s.user)
    const [showInitiateForm, setShowInitiateForm] = useState(false)

    // 查詢轉讓記錄
    const { data: transfers = [], isLoading } = useQuery({
        queryKey: ['animal-transfers', animalId],
        queryFn: async () => {
            const res = await transferApi.list(animalId)
            return res.data
        },
        staleTime: 30_000,
    })

    // 進行中的轉讓
    const activeTransfer = transfers.find(t =>
        !['completed', 'rejected'].includes(t.status)
    )

    // 歷史轉讓
    const historyTransfers = transfers.filter(t =>
        ['completed', 'rejected'].includes(t.status)
    )

    // 角色判斷
    const isVet = user?.roles?.includes('VET') ?? false
    const isPI = user?.roles?.includes('PI') ?? false
    const isAdmin = user?.roles?.includes('ADMIN') ?? false
    const canInitiate = (animalStatus === 'completed') && !activeTransfer
    const canVetEvaluate = (isVet || isAdmin) && activeTransfer?.status === 'pending'
    const canAssignPlan = (isAdmin || isPI) && activeTransfer?.status === 'vet_evaluated'
    const canApprove = (isAdmin || isPI) && activeTransfer?.status === 'plan_assigned'
    const canComplete = isAdmin && activeTransfer?.status === 'pi_approved'
    const canReject = (isAdmin || isPI || isVet) && !!activeTransfer && !['completed', 'rejected'].includes(activeTransfer.status)

    if (isLoading) {
        return <div className="flex items-center justify-center py-12"><Loader2 className="h-6 w-6 animate-spin text-muted-foreground" /></div>
    }

    return (
        <div className="space-y-6">
            {/* 發起轉讓按鈕 */}
            {canInitiate && !showInitiateForm && (
                <Button
                    className="bg-indigo-600 hover:bg-indigo-700"
                    onClick={() => setShowInitiateForm(true)}
                >
                    <Plus className="h-4 w-4 mr-2" />
                    發起轉讓
                </Button>
            )}

            {/* 發起表單 */}
            {showInitiateForm && (
                <TransferInitiateForm
                    animalId={animalId}
                    earTag={earTag}
                    onClose={() => setShowInitiateForm(false)}
                />
            )}

            {/* 進行中的轉讓 */}
            {activeTransfer && (
                <TransferActiveCard
                    animalId={animalId}
                    transfer={activeTransfer}
                    canVetEvaluate={!!canVetEvaluate}
                    canAssignPlan={!!canAssignPlan}
                    canApprove={!!canApprove}
                    canComplete={!!canComplete}
                    canReject={canReject}
                />
            )}

            {/* 無資料提示 */}
            {!activeTransfer && !showInitiateForm && historyTransfers.length === 0 && (
                <Card className="bg-muted">
                    <CardContent className="py-8 text-center text-muted-foreground">
                        <ArrowRightLeft className="h-8 w-8 mx-auto mb-3 text-muted-foreground" />
                        <p>此動物尚無轉讓記錄</p>
                        {canInitiate && <p className="text-xs mt-1">點擊上方按鈕發起轉讓</p>}
                    </CardContent>
                </Card>
            )}

            {/* 歷史紀錄 */}
            <TransferHistoryList transfers={historyTransfers} />
        </div>
    )
}
