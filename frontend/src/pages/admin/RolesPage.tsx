import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { RoleSignatureDialog } from '@/components/auth/RoleSignatureDialog'
import { Loader2, Shield, Plus, Pencil, Trash2, Eye } from 'lucide-react'
import { PermissionTree } from '@/components/admin/PermissionTree'
import { groupPermissionsByModule } from '@/hooks/usePermissionManager'
import { translateModuleName } from '@/hooks/permission/permissionConfig'
import { useRolesMutations } from './hooks/useRolesMutations'

export function RolesPage() {
  const { t } = useTranslation()
  const rm = useRolesMutations()

  // R30-27b：簽章 dialog 標題依當下 mode 推導，獨立成 helper 避免 JSX 巢狀三元
  const signatureDialogTitle = (() => {
    if (!rm.signaturePrompt) return t('adminUsers.roles.signature.defaultTitle')
    switch (rm.signaturePrompt.mode) {
      case 'create': return t('adminUsers.roles.signature.create')
      case 'update': return t('adminUsers.roles.signature.update')
      case 'delete':
        return rm.signaturePrompt.role.is_system
          ? t('adminUsers.roles.signature.deactivate')
          : t('adminUsers.roles.signature.delete')
    }
  })()

  // R30-27c-2：bridge purpose 字串（與 backend audit / role_signature 一致）
  const signatureDialogPurpose = (() => {
    if (!rm.signaturePrompt) return 'role.unknown'
    switch (rm.signaturePrompt.mode) {
      case 'create': return 'role.create'
      case 'update': return 'role.update'
      case 'delete': return 'role.delete'
    }
  })()

  if (rm.isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('nav.adminRoles')}
        description={t('adminUsers.roles.description')}
        actions={
          <GuestHide>
            <Button size="sm" onClick={() => rm.setShowCreateDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              {t('adminUsers.roles.addRole')}
            </Button>
          </GuestHide>
        }
      />

      <div className="grid gap-4 grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 min-[1920px]:grid-cols-4">
        {rm.roles?.map((role) => (
          <Card key={role.id}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Shield className="h-5 w-5 text-primary" />
                  {role.name}
                  {role.is_system && <Badge variant="secondary" className="text-xs">System</Badge>}
                </CardTitle>
                <GuestHide>
                  <div className="flex gap-1">
                    <Button variant="ghost" size="icon" onClick={() => rm.handleEdit(role)} aria-label={t('common.edit')}>
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" onClick={() => rm.handleDeleteClick(role)} aria-label={t('common.delete')}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </GuestHide>
              </div>
              <p className="text-sm text-muted-foreground font-mono">{role.code}</p>
            </CardHeader>
            <CardContent className="space-y-3">
              {/* R49 follow-up: guest mode 拿到的 role 可能無 permissions 欄位 — null-safe */}
              {(role.permissions?.length ?? 0) === 0 ? (
                <span className="text-sm text-muted-foreground">{t('adminUsers.roles.noPermissions')}</span>
              ) : (
                <>
                  <div className="text-sm text-muted-foreground">
                    {groupPermissionsByModule(role.permissions ?? [])
                      .map(({ moduleName, count }) =>
                        t('adminUsers.roles.moduleCount', { module: translateModuleName(t, moduleName), count }),
                      )
                      .join(' · ')}
                  </div>
                  <Button variant="outline" size="sm" className="w-full" onClick={() => rm.handleViewDetail(role)}>
                    <Eye className="h-4 w-4 mr-2" />
                    {t('adminUsers.roles.viewDetails', { count: role.permissions?.length ?? 0 })}
                  </Button>
                </>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* 創建角色對話框 */}
      <Dialog open={rm.showCreateDialog} onOpenChange={rm.setShowCreateDialog}>
        <DialogContent size="xl">
          <DialogHeader>
            <DialogTitle>{t('adminUsers.roles.addRole')}</DialogTitle>
            <DialogDescription>{t('adminUsers.roles.createDescription')}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">{t('adminUsers.roles.codeLabel')}</Label>
                <Input id="code" value={rm.formData.code} onChange={(e) => rm.setFormData({ ...rm.formData, code: e.target.value })} placeholder={t('adminUsers.roles.codePlaceholder')} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">{t('adminUsers.roles.nameLabel')}</Label>
                <Input id="name" value={rm.formData.name} onChange={(e) => rm.setFormData({ ...rm.formData, name: e.target.value })} placeholder={t('adminUsers.roles.namePlaceholder')} />
              </div>
            </div>
            <div className="space-y-2">
              <Label>{t('adminUsers.roles.permissionSettings')}</Label>
              <div className="border rounded-md p-4">
                <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.formData.permission_ids} onTogglePermission={rm.togglePermission} showSearch={true} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => rm.setShowCreateDialog(false)}>{t('common.cancel')}</Button>
            <Button onClick={rm.handleCreate} disabled={rm.createMutation.isPending}>
              {rm.createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('adminUsers.roles.createButton')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯角色對話框 */}
      <Dialog open={rm.showEditDialog} onOpenChange={rm.setShowEditDialog}>
        <DialogContent size="xl">
          <DialogHeader>
            <DialogTitle>{t('adminUsers.roles.editRole')}</DialogTitle>
            <DialogDescription>{t('adminUsers.roles.editDescription', { name: rm.selectedRole?.name })}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">{t('adminUsers.roles.editNameLabel')}</Label>
              <Input id="edit-name" value={rm.formData.name} onChange={(e) => rm.setFormData({ ...rm.formData, name: e.target.value })} />
            </div>
            <div className="space-y-2">
              <Label>{t('adminUsers.roles.permissionSettings')}</Label>
              <div className="border rounded-md p-4">
                <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.formData.permission_ids} onTogglePermission={rm.togglePermission} showSearch={true} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => rm.setShowEditDialog(false)}>{t('common.cancel')}</Button>
            <Button onClick={rm.handleUpdate} disabled={rm.updateMutation.isPending}>
              {rm.updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 查看權限詳情對話框 */}
      <Dialog open={rm.showDetailDialog} onOpenChange={(open) => { rm.setShowDetailDialog(open); if (!open) rm.setRoleForDetail(null) }}>
        <DialogContent size="xl" className="max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>{t('adminUsers.roles.detailTitle', { name: rm.roleForDetail?.name })}</DialogTitle>
            <DialogDescription>{t('adminUsers.roles.detailDescription', { count: rm.roleForDetail?.permissions?.length ?? 0 })}</DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto border rounded-md p-4 min-h-0">
            <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.roleForDetail?.permissions?.map((p) => p.id) ?? []} showSearch={true} readOnly={true} />
          </div>
        </DialogContent>
      </Dialog>

      {/* SEC-33：刪除角色前重新輸入密碼 */}
      <ConfirmPasswordModal
        open={rm.showReauthForDeleteRole}
        onOpenChange={(open) => { rm.setShowReauthForDeleteRole(open); if (!open) rm.setRoleToDelete(null) }}
        title={rm.roleToDelete?.is_system ? t('adminUsers.roles.confirmDeactivateTitle') : t('adminUsers.roles.confirmDeleteTitle')}
        description={rm.roleToDelete ? t(rm.roleToDelete.is_system ? 'adminUsers.roles.confirmDeactivateDescription' : 'adminUsers.roles.confirmDeleteDescription', { name: rm.roleToDelete.name }) : ''}
        onSubmit={rm.handleDeleteConfirm}
      />

      {/* R30-27b：role/permission 變更強制密碼 + 手寫雙因子簽章（21 CFR §11.10(d)） */}
      <RoleSignatureDialog
        open={rm.signaturePrompt !== null}
        onOpenChange={(open) => { if (!open) rm.setSignaturePrompt(null) }}
        title={signatureDialogTitle}
        description={t('adminUsers.roles.signature.description')}
        purpose={signatureDialogPurpose}
        onSubmit={rm.handleSignatureSubmit}
      />
    </div>
  )
}
