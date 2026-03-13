import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, Clock, Users } from 'lucide-react'

import { useTabState } from '@/hooks/useTabState'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CreateOvertimeDialog } from './components/CreateOvertimeDialog'
import { MyOvertimeTabContent } from './components/MyOvertimeTabContent'
import { PendingApprovalsTabContent } from './components/PendingApprovalsTabContent'
import { AllRecordsTabContent } from './components/AllRecordsTabContent'
import { useMyOvertime, usePendingOvertime, useOvertimeMutations } from './hooks/useOvertimeMutations'
import type { CreateOvertimeData } from './constants'

export function HrOvertimePage() {
    const { activeTab, setActiveTab } = useTabState<'my-overtime' | 'pending' | 'all-records'>('my-overtime')
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const { hasRole } = useAuthStore()

    const canViewAll = hasRole('admin') || hasRole('ADMIN_STAFF')

    // 取得員工清單（供申請人篩選用）
    const { data: staffList } = useQuery({
        queryKey: ['hr-internal-users'],
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

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">加班管理</h1>
                    <p className="text-muted-foreground">申請加班與累積補休時數</p>
                </div>
                <CreateOvertimeDialog
                    open={showCreateDialog}
                    onOpenChange={setShowCreateDialog}
                    onSubmit={handleCreateOvertime}
                    isPending={createOvertime.isPending}
                />
            </div>

            {/* Tabs */}
            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="my-overtime" className="flex items-center gap-2">
                        <Clock className="h-4 w-4" />
                        我的加班
                    </TabsTrigger>
                    <TabsTrigger value="approvals" className="flex items-center gap-2">
                        <CheckCircle className="h-4 w-4" />
                        待我審核
                        {pendingOvertime && pendingOvertime.total > 0 && (
                            <Badge variant="destructive" className="ml-1">
                                {pendingOvertime.total}
                            </Badge>
                        )}
                    </TabsTrigger>
                    {canViewAll && (
                        <TabsTrigger value="all-records" className="flex items-center gap-2">
                            <Users className="h-4 w-4" />
                            加班紀錄
                        </TabsTrigger>
                    )}
                </TabsList>

                <MyOvertimeTabContent
                    overtimeData={myOvertime}
                    isLoading={loadingOvertime}
                    onSubmit={(id) => submitOvertime.mutate(id)}
                    onDelete={(id) => deleteOvertime.mutate(id)}
                    isSubmitting={submitOvertime.isPending}
                    isDeleting={deleteOvertime.isPending}
                />

                <PendingApprovalsTabContent
                    pendingData={pendingOvertime}
                    isLoading={loadingPending}
                    onApprove={(id) => approveOvertime.mutate(id)}
                    onReject={(id, reason) => rejectOvertime.mutate({ id, reason })}
                    isApproving={approveOvertime.isPending}
                    isRejecting={rejectOvertime.isPending}
                />

                {canViewAll && (
                    <AllRecordsTabContent
                        isActive={canViewAll && activeTab === 'all-records'}
                        staffList={staffList}
                    />
                )}
            </Tabs>
        </div>
    )
}

export default HrOvertimePage
