import { Loader2, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

import { useNotificationRouting } from './NotificationRouting/hooks/useNotificationRouting'
import { RoutingTable } from './NotificationRouting/components/RoutingTable'
import { CreateRoutingDialog } from './NotificationRouting/components/CreateRoutingDialog'
import { EditRoutingDialog } from './NotificationRouting/components/EditRoutingDialog'

export function NotificationRoutingPage() {
    const {
        isLoading,
        rulesByGroup,
        eventNameMap,
        roleNameMap,
        eventCategories,
        roles,
        dialogState,

        showCreateDialog,
        setShowCreateDialog,
        createForm,
        setCreateForm,
        handleCreate,
        isCreating,

        showEditDialog,
        setShowEditDialog,
        selectedRule,
        editForm,
        setEditForm,
        handleEdit,
        handleUpdate,
        isUpdating,

        handleDelete,
        handleToggleActive,
    } = useNotificationRouting()

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        )
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">通知路由設定</h1>
                    <p className="text-muted-foreground">
                        管理事件觸發時的通知對象與通知方式
                    </p>
                </div>
                <Button onClick={() => setShowCreateDialog(true)}>
                    <Plus className="h-4 w-4 mr-2" />
                    新增規則
                </Button>
            </div>

            <RoutingTable
                rulesByGroup={rulesByGroup}
                eventNameMap={eventNameMap}
                roleNameMap={roleNameMap}
                onEdit={handleEdit}
                onDelete={handleDelete}
                onToggleActive={handleToggleActive}
            />

            <CreateRoutingDialog
                open={showCreateDialog}
                onOpenChange={setShowCreateDialog}
                form={createForm}
                onFormChange={setCreateForm}
                onSubmit={handleCreate}
                isPending={isCreating}
                eventCategories={eventCategories}
                roles={roles}
            />

            <EditRoutingDialog
                open={showEditDialog}
                onOpenChange={setShowEditDialog}
                selectedRule={selectedRule}
                form={editForm}
                onFormChange={setEditForm}
                onSubmit={handleUpdate}
                isPending={isUpdating}
                eventNameMap={eventNameMap}
                roleNameMap={roleNameMap}
            />

            <ConfirmDialog state={dialogState} />
        </div>
    )
}
