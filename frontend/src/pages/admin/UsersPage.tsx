import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
import { Plus, Download, Mail } from 'lucide-react'
import { useAuthHasPermission, useAuthIsAdmin } from '@/stores/auth'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { useUserManagement } from './hooks/useUserManagement'
import { UserTable } from './components/UserTable'
import { StaffAffiliationAudit } from './components/StaffAffiliationAudit'
import {
  UserCreateDialog,
  UserEditDialog,
  UserRolesDialog,
  UserDeleteDialog,
  UserResetPasswordDialog,
} from './components/UserFormDialogs'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { confirmPassword, isAxiosError } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import type { ApiErrorPayload } from '@/types/error'

export function UsersPage() {
  const { t } = useTranslation()
  const mgmt = useUserManagement()
  const { toast } = useToast()
  // 邀請使用者：重用既有「邀請管理」流程（email + 名稱 + 角色 → 寄信 → 設密碼登入）
  const canInvite = useAuthHasPermission()('invitation.view')
  // 強制移除角色僅系統管理員可用（後端亦擋）；非 admin 不顯示該按鈕以免誤導
  const isAdmin = useAuthIsAdmin()

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('nav.adminUsers')}
        description={t('adminUsers.users.description')}
        actions={
          <GuestHide>
            <div className="flex items-center gap-2">
              <label htmlFor="show-inactive-switch" className="flex items-center gap-2 text-sm text-muted-foreground cursor-pointer select-none">
                <Switch
                  id="show-inactive-switch"
                  checked={mgmt.showInactive}
                  onCheckedChange={mgmt.setShowInactive}
                />
                {t('adminUsers.users.showInactive')}
              </label>
              <Button variant="outline" size="sm" onClick={mgmt.handleExportUsers} disabled={!mgmt.sortedUsers?.length || mgmt.isLoading}>
                <Download className="h-4 w-4 mr-2" />
                {mgmt.showInactive ? t('adminUsers.users.exportAll') : t('adminUsers.users.exportActive')}
              </Button>
              {canInvite && (
                <Button variant="outline" size="sm" asChild>
                  <Link to="/hr/invitations">
                    <Mail className="h-4 w-4 mr-2" />
                    {t('adminUsers.users.inviteButton')}
                  </Link>
                </Button>
              )}
              <Button size="sm" data-testid="add-user-button" onClick={() => mgmt.setShowCreateDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                {t('admin.userCreateDialog.title')}
              </Button>
            </div>
          </GuestHide>
        }
      />

      {/* 身分旗標與角色矛盾的稽核清單。用 sortedUsers（全部）而非 users
          （當頁），否則矛盾的人翻到第二頁就看不到警告了。 */}
      <StaffAffiliationAudit users={mgmt.sortedUsers} onEdit={mgmt.handleEdit} />

      <UserTable
        users={mgmt.users}
        isLoading={mgmt.isLoading}
        currentUserId={mgmt.currentUser?.id}
        actions={{
          onEdit: mgmt.handleEdit,
          onManageRoles: mgmt.handleManageRoles,
          onResetPassword: mgmt.openResetPasswordDialog,
          onToggleActive: mgmt.handleToggleActive,
          onDelete: (user) => {
            mgmt.setUserToDelete(user)
            mgmt.setShowDeleteDialog(true)
          },
          onImpersonate: (user) => {
            mgmt.setUserToImpersonate(user)
            mgmt.setShowReauthForImpersonate(true)
          },
        }}
        sorting={{
          sortRole: mgmt.sortRole,
          sortStatus: mgmt.sortStatus,
          sortLastLogin: mgmt.sortLastLogin,
          onToggleSortRole: mgmt.toggleSortRole,
          onToggleSortStatus: mgmt.toggleSortStatus,
          onToggleSortLastLogin: mgmt.toggleSortLastLogin,
        }}
        pagination={{
          currentPage: mgmt.currentPage,
          totalPages: mgmt.totalPages,
          sortedUsersLength: mgmt.sortedUsers.length,
          onPrevPage: () => mgmt.setCurrentPage((p) => Math.max(1, p - 1)),
          onNextPage: () => mgmt.setCurrentPage((p) => Math.min(mgmt.totalPages, p + 1)),
        }}
      />

      <UserCreateDialog
        open={mgmt.showCreateDialog}
        onOpenChange={mgmt.setShowCreateDialog}
        roles={mgmt.roles}
        isPending={mgmt.createMutation.isPending}
        onSubmit={mgmt.handleCreateWithData}
      />

      <UserEditDialog
        open={mgmt.showEditDialog}
        onOpenChange={mgmt.setShowEditDialog}
        initialData={{
          email: mgmt.formData.email,
          display_name: mgmt.formData.display_name,
          is_internal: mgmt.formData.is_internal ?? true,
          entry_date: mgmt.formData.entry_date || '',
          aup_roles: mgmt.formData.aup_roles || [],
          trainings: mgmt.formData.trainings || [],
        }}
        isPending={mgmt.updateMutation.isPending}
        onSubmit={mgmt.handleUpdateWithData}
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

      {/* 移除角色被未結清事項擋下：列出卡住的單據，僅系統管理員可強制移除 */}
      <AlertDialog
        open={!!mgmt.unsettledConflict}
        onOpenChange={(open) => {
          if (!open) mgmt.setUnsettledConflict(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('adminUsers.users.unsettled.title')}</AlertDialogTitle>
            <AlertDialogDescription>{mgmt.unsettledConflict?.message}</AlertDialogDescription>
          </AlertDialogHeader>
          <ul className="max-h-60 space-y-1 overflow-y-auto rounded-md border bg-muted/40 p-3 text-sm">
            {mgmt.unsettledConflict?.items.map((item) => (
              <li key={item} className="break-words">
                {item}
              </li>
            ))}
          </ul>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            {isAdmin && (
              <AlertDialogAction onClick={mgmt.handleForceUpdateRoles}>
                {t('adminUsers.users.unsettled.forceRemove')}
              </AlertDialogAction>
            )}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

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
          // 密碼驗證失敗（401）→ 讓錯誤冒泡到 ConfirmPasswordModal 顯示「密碼錯誤」
          const { reauth_token } = await confirmPassword(password)
          if (!mgmt.userToDelete) return
          const targetId = mgmt.userToDelete.id
          try {
            await mgmt.deleteUserWithReauth(targetId, reauth_token)
            mgmt.setShowDeleteDialog(false)
            mgmt.setUserToDelete(null)
          } catch (err) {
            // 刪除失敗（非密碼問題），關閉刪除對話框並 toast 實際錯誤
            mgmt.setShowDeleteDialog(false)
            const message = isAxiosError(err)
              ? (err.response?.data as ApiErrorPayload | undefined)?.error?.message ?? t('adminUsers.users.deleteUserFailed')
              : t('adminUsers.users.deleteUserFailed')
            toast({ title: t('adminUsers.shared.deleteFailed'), description: message, variant: 'destructive' })
            // 不 re-throw：密碼已確認，讓 ConfirmPasswordModal 正常關閉
          }
        }}
      />

      <ConfirmPasswordModal
        open={mgmt.showReauthForImpersonate}
        onOpenChange={(o) => {
          mgmt.setShowReauthForImpersonate(o)
          if (!o) mgmt.setUserToImpersonate(null)
        }}
        title={t('adminUsers.users.impersonate.title')}
        description={
          mgmt.userToImpersonate
            ? t('adminUsers.users.impersonate.description', { name: mgmt.userToImpersonate.display_name })
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
        isPending={mgmt.resetPasswordMutation.isPending || mgmt.confirmPasswordMutation.isPending}
        onSubmit={mgmt.handleResetPasswordWithData}
        onClose={() => {
          mgmt.setUserToResetPassword(null)
        }}
      />
    </div>
  )
}
