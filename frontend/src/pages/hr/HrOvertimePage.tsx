import { useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, Clock, Users } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useAuthHasPermission, useAuthIsAdmin } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { CreateOvertimeDialog } from './components/CreateOvertimeDialog'
import { MyOvertimeTabContent } from './components/MyOvertimeTabContent'
import { PendingApprovalsTabContent } from './components/PendingApprovalsTabContent'
import { AllRecordsTabContent } from './components/AllRecordsTabContent'
import { useMyOvertime, usePendingOvertime, useOvertimeMutations } from './hooks/useOvertimeMutations'
import type { CreateOvertimeData } from './constants'

export function HrOvertimePage() {
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const hasPermission = useAuthHasPermission()
    const isAdmin = useAuthIsAdmin()
    const { dialogState, confirm } = useConfirmDialog()

    // 與後端 list_overtime 的判準對齊（`is_admin || has_permission("hr.overtime.view_all")`）。
    // 原本硬編 admin/ADMIN_STAFF 兩個角色名，漏掉同樣持有該權限的 DIRECTOR。
    const canViewAll = isAdmin || hasPermission('hr.overtime.view_all')

    // 取得員工清單（供申請人篩選用）
    const { data: staffList } = useQuery({
        queryKey: queryKeys.hr.internalUsers,
        queryFn: async () => {
            const res = await api.get<{ id: string; display_name: string; email: string }[]>('/hr/internal-users')
            return res.data
        },
        enabled: canViewAll,
    })

    const { data: myOvertime, isLoading: loadingOvertime } = useMyOvertime()
    const { data: pendingOvertime, isLoading: loadingPending } = usePendingOvertime()

    const {
        createOvertime,
        submitOvertime,
        approveOvertime,
        rejectOvertime,
        deleteOvertime,
    } = useOvertimeMutations()

    const handleCreateOvertime = (data: CreateOvertimeData) => {
        createOvertime.mutate(data, {
            onSuccess: () => setShowCreateDialog(false),
        })
    }

    const [searchParams] = useSearchParams()
    const activeTab = searchParams.get('tab') ?? 'my-overtime'

    return (
        <div className="space-y-6">
            <PageHeader
                title="加班管理"
                description="申請加班與累積補休時數"
                actions={
                    <GuestHide>
                        <CreateOvertimeDialog
                            open={showCreateDialog}
                            onOpenChange={setShowCreateDialog}
                            onSubmit={handleCreateOvertime}
                            isPending={createOvertime.isPending}
                        />
                    </GuestHide>
                }
            />

            <PageTabs
                tabs={[
                    { value: 'my-overtime', label: '我的加班', icon: Clock },
                    { value: 'approvals', label: '待我審核', icon: CheckCircle, badge: pendingOvertime?.total },
                    { value: 'all-records', label: '加班紀錄', icon: Users, hidden: !canViewAll },
                ]}
                defaultTab="my-overtime"
            >
                <PageTabContent value="my-overtime" className="space-y-4">
                    <MyOvertimeTabContent
                        overtimeData={myOvertime}
                        isLoading={loadingOvertime}
                        onSubmit={(id) => submitOvertime.mutate(id)}
                        onDelete={(id) => deleteOvertime.mutate(id)}
                        isSubmitting={submitOvertime.isPending}
                        isDeleting={deleteOvertime.isPending}
                    />
                </PageTabContent>

                <PageTabContent value="approvals" className="space-y-4">
                    <PendingApprovalsTabContent
                        pendingData={pendingOvertime}
                        isLoading={loadingPending}
                        onApprove={async (id) => {
                            // R72-2：核准前二次確認（已開啟確認框時忽略，避免並發覆寫狀態）
                            if (dialogState.open) return
                            const ok = await confirm({ title: '確認核准加班', description: '確認核准此加班申請？', confirmLabel: '確認核准' })
                            if (ok) approveOvertime.mutate(id)
                        }}
                        onReject={async (id, reason) => {
                            if (dialogState.open) return
                            const ok = await confirm({ title: '確認駁回加班', description: '確認駁回此加班申請？', variant: 'destructive', confirmLabel: '確認駁回' })
                            if (ok) rejectOvertime.mutate({ id, reason })
                        }}
                        isApproving={approveOvertime.isPending}
                        isRejecting={rejectOvertime.isPending}
                    />
                </PageTabContent>

                {canViewAll && (
                    <PageTabContent value="all-records" className="space-y-4">
                        <AllRecordsTabContent
                            isActive={canViewAll && activeTab === 'all-records'}
                            staffList={staffList}
                        />
                    </PageTabContent>
                )}
            </PageTabs>
            <ConfirmDialog state={dialogState} />
        </div>
    )
}

export default HrOvertimePage
