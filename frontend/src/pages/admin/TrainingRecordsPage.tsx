/**
 * 人員訓練紀錄管理頁 — GLP 合規
 *
 * 版面參考「特休額度管理」：
 * - 統計卡片（人員數、訓練紀錄數、證照即將到期）
 * - Tab 分頁（員工訓練紀錄、證照即將到期）
 * - Card 內搜尋 + 員工標籤選擇
 */

import { useTabState } from '@/hooks/useTabState'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Plus, User, AlertTriangle } from 'lucide-react'

import { useTrainingRecords } from './hooks/useTrainingRecords'
import { TrainingStatsCards } from './components/TrainingStatsCards'
import { TrainingRecordsTab } from './components/TrainingRecordsTab'
import { TrainingExpiringTab } from './components/TrainingExpiringTab'
import { TrainingFormDialog } from './components/TrainingFormDialog'

export function TrainingRecordsPage() {
  const { activeTab, setActiveTab } = useTabState<'records' | 'expiring'>('records')
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">人員訓練紀錄</h1>
          <p className="text-muted-foreground">GLP 合規：管理人員訓練與證照有效期限</p>
        </div>
        {canManage && (
          <Button onClick={openCreateDialog}>
            <Plus className="h-4 w-4 mr-2" />
            新增訓練紀錄
          </Button>
        )}
      </div>

      <TrainingStatsCards totalRecords={totalRecords} expiringSoonCount={expiringSoonCount} />

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="records" className="flex items-center gap-2">
            <User className="h-4 w-4" />
            員工訓練紀錄
          </TabsTrigger>
          <TabsTrigger value="expiring" className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            證照即將到期
            {expiringSoonCount > 0 && (
              <Badge variant="destructive" className="ml-1">
                {expiringSoonCount}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="records" className="space-y-4">
          <TrainingRecordsTab
            canManage={canManage}
            canManageAll={canManageAll}
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
        </TabsContent>

        <TabsContent value="expiring" className="space-y-4">
          <TrainingExpiringTab records={expiringSoonRecords} />
        </TabsContent>
      </Tabs>

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
