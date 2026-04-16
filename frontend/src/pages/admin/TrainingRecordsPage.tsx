/**
 * 人員訓練紀錄管理頁 — GLP 合規
 *
 * 版面參考「特休額度管理」：
 * - 統計卡片（人員數、訓練紀錄數、證照即將到期）
 * - Tab 分頁（員工訓練紀錄、證照即將到期）
 * - Card 內搜尋 + 員工標籤選擇
 */

import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { Plus, User, AlertTriangle } from 'lucide-react'

import { useAuthStore } from '@/stores/auth'
import { useTrainingRecords } from './hooks/useTrainingRecords'
import { TrainingStatsCards } from './components/TrainingStatsCards'
import { TrainingRecordsTab } from './components/TrainingRecordsTab'
import { TrainingExpiringTab } from './components/TrainingExpiringTab'
import { TrainingFormDialog } from './components/TrainingFormDialog'

export function TrainingRecordsPage() {
  const isGuestUser = useAuthStore((s) => s.isGuest)()
  const {
    canManage,
    canManageAll,
    user,
    dialogs,
    users,
    filteredUsers,
    records,
    totalPages,
    totalRecords,
    isLoading,
    expiringSoonCount,
    expiringSoonRecords,
    form,
    setForm,
    editingRecord,
    selectedUserId,
    setSelectedUserId,
    searchQuery,
    setSearchQuery,
    keyword,
    setKeyword,
    page,
    setPage,
    openCreateDialog,
    openEditDialog,
    handleCreate,
    handleUpdate,
    handleDelete,
    createMutation,
    updateMutation,
  } = useTrainingRecords()

  return (
    <div className="space-y-6">
      <PageHeader
        title="人員訓練紀錄"
        description="GLP 合規：管理人員訓練與證照有效期限"
        actions={(canManage || isGuestUser) ? (
          <Button size="sm" onClick={isGuestUser ? undefined : openCreateDialog} disabled={isGuestUser} title={isGuestUser ? '訪客模式' : undefined}>
            <Plus className="h-4 w-4 mr-2" />
            新增訓練紀錄
          </Button>
        ) : undefined}
      />

      <TrainingStatsCards totalRecords={totalRecords} expiringSoonCount={expiringSoonCount} />

      <PageTabs
        tabs={[
          { value: 'records', label: '員工訓練紀錄', icon: User },
          { value: 'expiring', label: '證照即將到期', icon: AlertTriangle, badge: expiringSoonCount },
        ]}
        defaultTab="records"
      >
        <PageTabContent value="records" className="space-y-4">
          <TrainingRecordsTab
            canManage={canManage}
            canManageAll={canManageAll}
            isGuestUser={isGuestUser}
            records={records}
            isLoading={isLoading}
            totalPages={totalPages}
            page={page}
            setPage={setPage}
            selectedUserId={selectedUserId}
            setSelectedUserId={setSelectedUserId}
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
            keyword={keyword}
            setKeyword={setKeyword}
            filteredUsers={filteredUsers}
            onEdit={openEditDialog}
            onDelete={handleDelete}
          />
        </PageTabContent>

        <PageTabContent value="expiring" className="space-y-4">
          <TrainingExpiringTab records={expiringSoonRecords} />
        </PageTabContent>
      </PageTabs>

      <TrainingFormDialog
        mode="create"
        open={dialogs.isOpen('create')}
        onOpenChange={dialogs.setOpen('create')}
        form={form}
        setForm={setForm}
        onSubmit={handleCreate}
        isPending={createMutation.isPending}
        canManageAll={canManageAll}
        users={users}
        currentUser={user}
      />

      <TrainingFormDialog
        mode="edit"
        open={dialogs.isOpen('edit')}
        onOpenChange={(open) => {
          dialogs.setOpen('edit')(open)
        }}
        form={form}
        setForm={setForm}
        onSubmit={handleUpdate}
        isPending={updateMutation.isPending}
        canManageAll={canManageAll}
        users={users}
        editingRecord={editingRecord}
      />
    </div>
  )
}
