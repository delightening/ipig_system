import { useDialogSet } from '@/hooks/useDialogSet'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, FileText, Plus, Users } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { toast } from '@/components/ui/use-toast'
import { LEAVE_TYPE_NAMES } from '@/types/hr'
import type { BalanceSummary, LeaveRequestWithUser, StaffInfo } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { useLeaveRequestForm } from './hooks/useLeaveRequestForm'
import { useLeaveMutations } from './hooks/useLeaveMutations'
import { LeaveBalanceSummary } from './components/LeaveBalanceSummary'
import { CreateLeaveDialog } from './components/CreateLeaveDialog'
import { MyLeavesTabContent } from './components/MyLeavesTabContent'
import { LeavePendingApprovalsTab } from './components/LeavePendingApprovalsTab'
import { AllLeaveRecordsTabContent } from './components/AllLeaveRecordsTabContent'

export function HrLeavePage() {
    const dialogs = useDialogSet(['create'] as const)
    const { hasRole } = useAuthStore()
    const canViewAll = hasRole('admin') || hasRole('ADMIN_STAFF')

    const leaveForm = useLeaveRequestForm()

    const mutations = useLeaveMutations({
        onCreateSuccess: () => {
            dialogs.close('create')
            leaveForm.resetForm()
        },
    })

    // 我的餘額
    const { data: balanceSummary } = useQuery({
        queryKey: queryKeys.hr.balanceSummary,
        queryFn: async () => {
            const res = await api.get<BalanceSummary>('/hr/balances/summary')
            return res.data
        },
    })

    // 我的請假記錄
    const { data: myLeaves, isLoading: loadingLeaves } = useQuery({
        queryKey: queryKeys.hr.myLeaves,
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>('/hr/leaves')
            return res.data
        },
    })

    // 待審核的請假
    const { data: pendingLeaves, isLoading: loadingPending } = useQuery({
        queryKey: queryKeys.hr.pendingLeaves,
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>('/hr/leaves?pending_approval=true')
            return res.data
        },
    })

    // 工作人員列表（供代理人選擇）
    const { data: staffList } = useQuery({
        queryKey: queryKeys.hr.staffForProxy,
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
    })

    const handlePrefillLastLeave = () => {
        const last = myLeaves?.data?.[0]
        if (!last) {
            toast({ title: '無歷史紀錄', description: '找不到之前的請假記錄', variant: 'destructive' })
            return
        }
        leaveForm.updateField('leaveType', last.leave_type)
        leaveForm.updateField('reason', last.reason ?? '')
        leaveForm.updateField('proxyUserId', last.proxy_user_id ?? '')
        toast({ title: '已預填', description: `已套用上次「${LEAVE_TYPE_NAMES[last.leave_type] ?? last.leave_type}」假別資訊` })
    }

    const handleCreateLeave = () => {
        if (!leaveForm.form.leaveType || !leaveForm.form.startDate || !leaveForm.form.endDate) {
            toast({ title: '錯誤', description: '請填寫必填欄位', variant: 'destructive' })
            return
        }
        if (!leaveForm.isAnnualLeave && !leaveForm.form.reason.trim()) {
            toast({ title: '錯誤', description: '請填寫請假事由', variant: 'destructive' })
            return
        }
        const hours = parseFloat(leaveForm.form.totalHours) || 0
        if (hours < 0.5) {
            toast({ title: '錯誤', description: '請假時數至少 0.5 小時，且須為 0.5 的倍數', variant: 'destructive' })
            return
        }
        mutations.createLeaveMutation.mutate(leaveForm.buildSubmitPayload())
    }

    return (
        <div className="space-y-6">
            <PageHeader
                title="請假管理"
                description="申請請假與查看假期餘額"
                actions={
                    <Button size="sm" onClick={() => dialogs.open('create')}>
                        <Plus className="h-4 w-4 mr-2" />
                        新增請假
                    </Button>
                }
            />

            <CreateLeaveDialog
                open={dialogs.isOpen('create')}
                onOpenChange={dialogs.setOpen('create')}
                leaveForm={leaveForm}
                staffList={staffList}
                hasHistory={(myLeaves?.data?.length ?? 0) > 0}
                onPrefillLastLeave={handlePrefillLastLeave}
                onSubmit={handleCreateLeave}
                isPending={mutations.createLeaveMutation.isPending}
            />

            <LeaveBalanceSummary balanceSummary={balanceSummary} />

            <PageTabs
                tabs={[
                    { value: 'my-leaves', label: '我的請假', icon: FileText },
                    { value: 'approvals', label: '待我審核', icon: CheckCircle, badge: pendingLeaves?.total },
                    { value: 'all-records', label: '請假紀錄', icon: Users, hidden: !canViewAll },
                ]}
                defaultTab="my-leaves"
            >
                <PageTabContent value="my-leaves" className="space-y-4">
                    <MyLeavesTabContent
                        leaves={myLeaves?.data}
                        isLoading={loadingLeaves}
                        onSubmit={(id) => mutations.submitLeaveMutation.mutate(id)}
                        onCancel={(id) => mutations.cancelLeaveMutation.mutate(id)}
                        submitPending={mutations.submitLeaveMutation.isPending}
                        cancelPending={mutations.cancelLeaveMutation.isPending}
                    />
                </PageTabContent>

                <PageTabContent value="approvals" className="space-y-4">
                    <LeavePendingApprovalsTab
                        leaves={pendingLeaves?.data}
                        isLoading={loadingPending}
                        onApprove={(id) => mutations.approveLeaveMutation.mutate(id)}
                        onReject={(id, reason) => mutations.rejectLeaveMutation.mutate({ id, reason })}
                        approvePending={mutations.approveLeaveMutation.isPending}
                        rejectPending={mutations.rejectLeaveMutation.isPending}
                    />
                </PageTabContent>

                {canViewAll && (
                    <PageTabContent value="all-records" className="space-y-4">
                        <AllLeaveRecordsTabContent />
                    </PageTabContent>
                )}
            </PageTabs>
        </div>
    )
}

export default HrLeavePage
