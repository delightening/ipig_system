import { Button } from '@/components/ui/button'
import { Plus, Download } from 'lucide-react'
import { useUserManagement } from './hooks/useUserManagement'
import { UserTable } from './components/UserTable'
import {
  UserCreateDialog,
  UserEditDialog,
  UserRolesDialog,
  UserDeleteDialog,
  UserResetPasswordDialog,
} from './components/UserFormDialogs'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { confirmPassword } from '@/lib/api'

export function UsersPage() {
  const mgmt = useUserManagement()

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">使用者管理</h1>
          <p className="text-muted-foreground">管理系統使用者帳號與角色</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={mgmt.handleExportUsers} disabled={!mgmt.sortedUsers?.length || mgmt.isLoading}>
            <Download className="h-4 w-4 mr-2" />
            匯出現在的使用者
          </Button>
          <Button onClick={() => mgmt.setShowCreateDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            新增使用者
          </Button>
        </div>
      </div>

      <UserTable
        users={mgmt.users}
        isLoading={mgmt.isLoading}
        sortRole={mgmt.sortRole}
        sortStatus={mgmt.sortStatus}
        currentPage={mgmt.currentPage}
        totalPages={mgmt.totalPages}
        sortedUsersLength={mgmt.sortedUsers.length}
        currentUserId={mgmt.currentUser?.id}
        onToggleSortRole={mgmt.toggleSortRole}
        onToggleSortStatus={mgmt.toggleSortStatus}
        onPrevPage={() => mgmt.setCurrentPage((p) => Math.max(1, p - 1))}
        onNextPage={() => mgmt.setCurrentPage((p) => Math.min(mgmt.totalPages, p + 1))}
        onEdit={mgmt.handleEdit}
        onManageRoles={mgmt.handleManageRoles}
        onResetPassword={mgmt.openResetPasswordDialog}
        onToggleActive={mgmt.handleToggleActive}
        onDelete={(user) => {
          mgmt.setUserToDelete(user)
          mgmt.setShowDeleteDialog(true)
        }}
        onImpersonate={(user) => {
          mgmt.setUserToImpersonate(user)
          mgmt.setShowReauthForImpersonate(true)
        }}
      />

      <UserCreateDialog
        open={mgmt.showCreateDialog}
        onOpenChange={mgmt.setShowCreateDialog}
        formData={mgmt.formData}
        setFormData={mgmt.setFormData}
        roles={mgmt.roles}
        isPending={mgmt.createMutation.isPending}
        onSubmit={mgmt.handleCreate}
        toggleRole={mgmt.toggleRole}
      />

      <UserEditDialog
        open={mgmt.showEditDialog}
        onOpenChange={mgmt.setShowEditDialog}
        formData={mgmt.formData}
        setFormData={mgmt.setFormData}
        isPending={mgmt.updateMutation.isPending}
        onSubmit={mgmt.handleUpdate}
      />

      <UserRolesDialog
        open={mgmt.showRolesDialog}
        onOpenChange={mgmt.setShowRolesDialog}
        selectedUser={mgmt.selectedUser}
        formData={mgmt.formData}
        roles={mgmt.roles}
        isPending={mgmt.updateMutation.isPending}
        onSubmit={mgmt.handleUpdateRoles}
        toggleRole={mgmt.toggleRole}
      />

      <UserDeleteDialog
        open={mgmt.showDeleteDialog}
        onOpenChange={(o) => {
          mgmt.setShowDeleteDialog(o)
          if (!o) mgmt.setUserToDelete(null)
        }}
        userToDelete={mgmt.userToDelete}
        showReauth={mgmt.showReauthForDelete}
        onReauthOpenChange={(o) => {
          mgmt.setShowReauthForDelete(o)
          if (!o) mgmt.setUserToDelete(null)
        }}
        onConfirmDelete={() => mgmt.setShowReauthForDelete(true)}
        onReauthSubmit={async (password) => {
          const { reauth_token } = await confirmPassword(password)
          if (!mgmt.userToDelete) return
          await mgmt.deleteUserWithReauth(mgmt.userToDelete.id, reauth_token)
          mgmt.setShowDeleteDialog(false)
          mgmt.setUserToDelete(null)
        }}
      />

      <ConfirmPasswordModal
        open={mgmt.showReauthForImpersonate}
        onOpenChange={(o) => {
          mgmt.setShowReauthForImpersonate(o)
          if (!o) mgmt.setUserToImpersonate(null)
        }}
        title="模擬登入確認"
        description={
          mgmt.userToImpersonate
            ? `確定要以「${mgmt.userToImpersonate.display_name}」的身分登入？請輸入您的登入密碼以確認。`
            : ''
        }
        onSubmit={async (password) => {
          const { reauth_token } = await confirmPassword(password)
          if (!mgmt.userToImpersonate) return
          await mgmt.handleImpersonate(reauth_token)
        }}
      />

      <UserResetPasswordDialog
        open={mgmt.showResetPasswordDialog}
        onOpenChange={mgmt.setShowResetPasswordDialog}
        userToResetPassword={mgmt.userToResetPassword}
        reauthPassword={mgmt.reauthPassword}
        setReauthPassword={mgmt.setReauthPassword}
        newPassword={mgmt.newPassword}
        setNewPassword={mgmt.setNewPassword}
        confirmNewPassword={mgmt.confirmNewPassword}
        setConfirmNewPassword={mgmt.setConfirmNewPassword}
        isPending={mgmt.resetPasswordMutation.isPending}
        onSubmit={mgmt.handleResetPassword}
        onClose={() => {
          mgmt.setUserToResetPassword(null)
          mgmt.setNewPassword('')
          mgmt.setConfirmNewPassword('')
          mgmt.setReauthPassword('')
        }}
      />
    </div>
  )
}
