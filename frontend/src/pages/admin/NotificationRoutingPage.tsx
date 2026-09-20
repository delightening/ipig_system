import { Loader2, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

import { useNotificationRouting } from './NotificationRouting/hooks/useNotificationRouting'
import { RoutingTable } from './NotificationRouting/components/RoutingTable'
import { FixedNotificationsSection } from './NotificationRouting/components/FixedNotificationsSection'
import { CreateRoutingDialog } from './NotificationRouting/components/CreateRoutingDialog'
import { EditRoutingDialog } from './NotificationRouting/components/EditRoutingDialog'

export function NotificationRoutingPage() {
    const { t } = useTranslation()
    const {
        isLoading,
        rulesByGroup,
        eventNameMap,
        roleNameMap,
        recipientsByRuleId,
        fixedNotifications,
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
            <PageHeader
                title={t('adminOps.notificationRouting.page.title')}
                description={t('adminOps.notificationRouting.page.description')}
                actions={
                    <Button size="sm" onClick={() => setShowCreateDialog(true)}>
                        <Plus className="h-4 w-4 mr-2" />
                        {t('adminOps.notificationRouting.page.addRule')}
                    </Button>
                }
            />

            <RoutingTable
                rulesByGroup={rulesByGroup}
                eventNameMap={eventNameMap}
                roleNameMap={roleNameMap}
                recipientsByRuleId={recipientsByRuleId}
                onEdit={handleEdit}
                onDelete={handleDelete}
                onToggleActive={handleToggleActive}
            />

            <FixedNotificationsSection items={fixedNotifications} />

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
