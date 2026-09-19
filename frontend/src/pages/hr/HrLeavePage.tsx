import { useDialogSet } from '@/hooks/useDialogSet'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { CheckCircle, FileText, Plus, Users } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_LEAVES, DEMO_BALANCE_SUMMARY } from '@/lib/guest-demo'
import { useAuthHasPermission, useAuthHasRole, useAuthIsAdmin, useAuthUser } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { toast } from '@/components/ui/use-toast'
import type { BalanceSummary, LeaveRequestWithUser, StaffInfo } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { leaveTypeLabel } from './constants'
import { useLeaveRequestForm } from './hooks/useLeaveRequestForm'
import { useLeaveMutations } from './hooks/useLeaveMutations'
import { LeaveBalanceSummary } from './components/LeaveBalanceSummary'
import { CreateLeaveDialog } from './components/CreateLeaveDialog'
import { MyLeavesTabContent } from './components/MyLeavesTabContent'
import { LeavePendingApprovalsTab } from './components/LeavePendingApprovalsTab'
import { AllLeaveRecordsTabContent } from './components/AllLeaveRecordsTabContent'

export function HrLeavePage() {
    const { t } = useTranslation()
    const dialogs = useDialogSet(['create'] as const)
    const currentUser = useAuthUser()
    const hasRole = useAuthHasRole()
    const hasPermission = useAuthHasPermission()
    const isAdmin = useAuthIsAdmin()
    // 與後端 list_leaves 的判準對齊（`is_admin || has_permission("hr.leave.view_all")`）。
    // 原本硬編 admin/ADMIN_STAFF 兩個角色名，漏掉同樣持有該權限的 DIRECTOR——
    // 負責人因此看不到「請假紀錄」分頁。改看權限後，日後新角色只要給權限即自動生效。
    const canViewAll = isAdmin || hasPermission('hr.leave.view_all')
    // 負責人之上無人可代理其職務，代理人選填（未指定時後端走報備制），前端不擋
    const isDirector = hasRole('DIRECTOR')
    const { dialogState, confirm } = useConfirmDialog()

    const leaveForm = useLeaveRequestForm()

    const mutations = useLeaveMutations({
        onCreateSuccess: () => {
            dialogs.close('create')
            leaveForm.resetForm()
        },
    })

    // 我的餘額
    const { data: balanceSummary } = useGuestQuery(DEMO_BALANCE_SUMMARY, {
        queryKey: queryKeys.hr.balanceSummary,
        queryFn: async () => {
            const res = await api.get<BalanceSummary>('/hr/balances/summary')
            return res.data
        },
    })

    // 我的請假記錄
    const { data: myLeaves, isLoading: loadingLeaves } = useGuestQuery(DEMO_LEAVES, {
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

    // 「待我審核」列出當前使用者可處理的單：可核准（後端 can_approve）或
    // 身為代理人待確認（can_confirm_proxy）。其餘關卡不列入，避免清單與徽章灌爆。
    const actionablePending = pendingLeaves?.data?.filter((l) => l.can_approve || l.can_confirm_proxy)

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
            toast({
                title: t('hrPages.leaves.toast.noHistoryTitle'),
                description: t('hrPages.leaves.toast.noHistoryDescription'),
                variant: 'destructive',
            })
            return
        }
        leaveForm.updateField('leaveType', last.leave_type)
        leaveForm.updateField('reason', last.reason ?? '')
        leaveForm.updateField('proxyUserId', last.proxy_user_id ?? '')
        toast({
            title: t('hrPages.leaves.toast.prefilledTitle'),
            description: t('hrPages.leaves.toast.prefilledDescription', { type: leaveTypeLabel(t, last.leave_type) }),
        })
    }

    const handleCreateLeave = () => {
        if (!leaveForm.form.leaveType || !leaveForm.form.startDate || !leaveForm.form.endDate) {
            toast({ title: t('common.error'), description: t('hrPages.leaves.validation.requiredFields'), variant: 'destructive' })
            return
        }
        if (!leaveForm.isAnnualLeave && !leaveForm.form.reason.trim()) {
            toast({ title: t('common.error'), description: t('hrPages.leaves.validation.reasonRequired'), variant: 'destructive' })
            return
        }
        if (!isDirector && (!leaveForm.form.proxyUserId || leaveForm.form.proxyUserId === '__none__')) {
            toast({ title: t('common.error'), description: t('hrPages.leaves.validation.delegateRequired'), variant: 'destructive' })
            return
        }
        const hours = parseFloat(leaveForm.form.totalHours) || 0
        if (hours < 0.5) {
            toast({ title: t('common.error'), description: t('hrPages.leaves.validation.hoursMinimum'), variant: 'destructive' })
            return
        }
        mutations.createLeaveMutation.mutate(leaveForm.buildSubmitPayload())
    }

    return (
        <div className="space-y-6">
            <PageHeader
                title={t('nav.hrLeaves')}
                description={t('hrPages.leaves.page.description')}
                actions={
                    <GuestHide>
                        <Button size="sm" onClick={() => dialogs.open('create')}>
                            <Plus className="h-4 w-4 mr-2" />
                            {t('hrPages.leaves.page.addLeave')}
                        </Button>
                    </GuestHide>
                }
            />

            <CreateLeaveDialog
                open={dialogs.isOpen('create')}
                onOpenChange={dialogs.setOpen('create')}
                leaveForm={leaveForm}
                staffList={staffList?.filter((s) => s.id !== currentUser?.id)}
                hasHistory={(myLeaves?.data?.length ?? 0) > 0}
                onPrefillLastLeave={handlePrefillLastLeave}
                onSubmit={handleCreateLeave}
                isPending={mutations.createLeaveMutation.isPending}
            />

            <LeaveBalanceSummary balanceSummary={balanceSummary} />

            <PageTabs
                tabs={[
                    { value: 'my-leaves', label: t('hrPages.leaves.tabs.myLeaves'), icon: FileText },
                    { value: 'approvals', label: t('hrPages.shared.pendingMyReview'), icon: CheckCircle, badge: actionablePending?.length },
                    { value: 'all-records', label: t('hrPages.leaves.tabs.allRecords'), icon: Users, hidden: !canViewAll },
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
                        leaves={actionablePending}
                        isLoading={loadingPending}
                        onApprove={async (id) => {
                            // R72-2：核准前二次確認（已開啟確認框時忽略，避免並發覆寫狀態）
                            if (dialogState.open) return
                            const ok = await confirm({
                                title: t('hrPages.leaves.confirm.approveTitle'),
                                description: t('hrPages.leaves.confirm.approveDescription'),
                                confirmLabel: t('hrPages.shared.action.confirmApprove'),
                            })
                            if (ok) mutations.approveLeaveMutation.mutate(id)
                        }}
                        onReject={async (id, reason) => {
                            if (dialogState.open) return
                            const ok = await confirm({
                                title: t('hrPages.leaves.confirm.rejectTitle'),
                                description: t('hrPages.leaves.confirm.rejectDescription'),
                                variant: 'destructive',
                                confirmLabel: t('hrPages.shared.action.confirmReject'),
                            })
                            if (ok) mutations.rejectLeaveMutation.mutate({ id, reason })
                        }}
                        onProxyConfirm={async (id) => {
                            if (dialogState.open) return
                            const ok = await confirm({
                                title: t('hrPages.shared.action.confirmDelegate'),
                                description: t('hrPages.leaves.confirm.delegateDescription'),
                                confirmLabel: t('hrPages.shared.action.confirmDelegate'),
                            })
                            if (ok) mutations.proxyConfirmLeaveMutation.mutate(id)
                        }}
                        onProxyReject={async (id) => {
                            if (dialogState.open) return
                            const ok = await confirm({
                                title: t('hrPages.leaves.confirm.returnTitle'),
                                description: t('hrPages.leaves.confirm.returnDescription'),
                                variant: 'destructive',
                                confirmLabel: t('hrPages.leaves.confirm.returnConfirm'),
                            })
                            if (ok) mutations.proxyRejectLeaveMutation.mutate({ id })
                        }}
                        approvePending={mutations.approveLeaveMutation.isPending}
                        rejectPending={mutations.rejectLeaveMutation.isPending}
                        proxyConfirmPending={mutations.proxyConfirmLeaveMutation.isPending}
                        proxyRejectPending={mutations.proxyRejectLeaveMutation.isPending}
                    />
                </PageTabContent>

                {canViewAll && (
                    <PageTabContent value="all-records" className="space-y-4">
                        <AllLeaveRecordsTabContent />
                    </PageTabContent>
                )}
            </PageTabs>
            <ConfirmDialog state={dialogState} />
        </div>
    )
}

export default HrLeavePage
