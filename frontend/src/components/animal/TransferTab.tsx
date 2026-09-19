import { useState } from 'react'
import { GuestHide } from '@/components/ui/guest-hide'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { transferApi } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ArrowRightLeft, Loader2, Plus } from 'lucide-react'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useAuthHasPermission, useAuthUser } from '@/stores/auth'

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
    const { t } = useTranslation()
    const user = useAuthUser()
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
    const activeTransfer = transfers.find(tr =>
        !['completed', 'rejected'].includes(tr.status)
    )

    // 歷史轉讓
    const historyTransfers = transfers.filter(tr =>
        ['completed', 'rejected'].includes(tr.status)
    )

    // 動作閘：與後端 `handlers/animal/transfer.rs` 逐段對齊（P0-3，2026-09-05）。
    //
    // ⚠️ 這裡原本全部用角色硬判，且 admin 那格寫的是 `roles.includes('ADMIN')`——
    // 全 codebase 的角色代碼是 `'admin'` / `'SYSTEM_ADMIN'`（`constants.rs:188-189`），
    // **沒有任何一處用大寫 `'ADMIN'`**，所以那個變數恆為 `false`：
    // `canComplete = isAdmin && …` 代表「完成轉讓」按鈕從不出現，執行秘書也看不到
    // 任何轉讓操作。改用權限碼後不必再自己判 admin——`hasPermission()` 已比照後端
    // `is_admin()` 對 `admin` / `SYSTEM_ADMIN` 短路（`stores/auth.ts:261-263`）。
    const hasPermission = useAuthHasPermission()

    // 協調段（發起 / 指定新計畫 / 完成 / 拒絕）＝後端的 `animal.transfer.manage`。
    const canManageTransfer = hasPermission(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
    // 簽署權責（核准）在後端是 `check_transfer_signing_authority`：VET 角色**或**
    // 轉出 / 轉入計畫的 `pi_user_id`，不是權限碼，所以這裡只能用角色近似。
    // 前端拿不到「是不是這兩張計畫的 PI」，故 PI 角色一律顯示、由後端做最終判定；
    // admin 若非 VET 也非兩造 PI 同樣會被後端擋，因此**刻意不放行 admin**，
    // 免得出現「按鈕看得到、按下去 403」。
    const isVet = user?.roles?.includes('VET') ?? false
    const isPI = user?.roles?.includes('PI') ?? false

    // 發起原本完全沒有權限判斷（只看動物狀態），任何人都看得到按鈕、按下去吃 403。
    const canInitiate = canManageTransfer && (animalStatus === 'completed') && !activeTransfer
    const canVetEvaluate = hasPermission(PERMISSIONS.ANIMAL_VET_RECOMMEND) && activeTransfer?.status === 'pending'
    const canAssignPlan = canManageTransfer && activeTransfer?.status === 'vet_evaluated'
    const canApprove = (isVet || isPI) && activeTransfer?.status === 'plan_assigned'
    const canComplete = canManageTransfer && activeTransfer?.status === 'pi_approved'
    const canReject = canManageTransfer && !!activeTransfer && !['completed', 'rejected'].includes(activeTransfer.status)

    if (isLoading) {
        return <div className="flex items-center justify-center py-12"><Loader2 className="h-6 w-6 animate-spin text-muted-foreground" /></div>
    }

    return (
        <div className="space-y-6">
            {/* 發起轉讓按鈕 */}
            {canInitiate && !showInitiateForm && (
                <GuestHide>
                    <Button
                        className="bg-primary hover:bg-primary/90"
                        onClick={() => setShowInitiateForm(true)}
                    >
                        <Plus className="h-4 w-4 mr-2" />
                        {t('animalActions.transfer.tab.initiate')}
                    </Button>
                </GuestHide>
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
                        <p>{t('animalActions.transfer.tab.empty')}</p>
                        {canInitiate && <p className="text-xs mt-1">{t('animalActions.transfer.tab.emptyHint')}</p>}
                    </CardContent>
                </Card>
            )}

            {/* 歷史紀錄 */}
            <TransferHistoryList transfers={historyTransfers} />
        </div>
    )
}
