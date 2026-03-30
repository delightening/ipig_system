import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { Loader2, Shield, Plus, Pencil, Trash2, Eye } from 'lucide-react'
import { PermissionTree } from '@/components/admin/PermissionTree'
import { groupPermissionsByModule } from '@/hooks/usePermissionManager'
import { useRolesMutations } from './hooks/useRolesMutations'

export function RolesPage() {
  const rm = useRolesMutations()

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
        title="角色權限"
        description="管理系統角色與權限設定"
        actions={
          <Button size="sm" onClick={() => rm.setShowCreateDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            新增角色
          </Button>
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
                <div className="flex gap-1">
                  <Button variant="ghost" size="icon" onClick={() => rm.handleEdit(role)} aria-label="編輯">
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="icon" onClick={() => rm.handleDeleteClick(role)} aria-label="刪除">
                    <Trash2 className="h-4 w-4 text-destructive" />
                  </Button>
                </div>
              </div>
              <p className="text-sm text-muted-foreground font-mono">{role.code}</p>
            </CardHeader>
            <CardContent className="space-y-3">
              {role.permissions.length === 0 ? (
                <span className="text-sm text-muted-foreground">無權限</span>
              ) : (
                <>
                  <div className="text-sm text-muted-foreground">
                    {groupPermissionsByModule(role.permissions)
                      .map(({ moduleName, count }) => `${moduleName} ${count} 項`)
                      .join(' · ')}
                  </div>
                  <Button variant="outline" size="sm" className="w-full" onClick={() => rm.handleViewDetail(role)}>
                    <Eye className="h-4 w-4 mr-2" />
                    查看詳情 ({role.permissions.length} 個權限)
                  </Button>
                </>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* 創建角色對話框 */}
      <Dialog open={rm.showCreateDialog} onOpenChange={rm.setShowCreateDialog}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>新增角色</DialogTitle>
            <DialogDescription>創建新的系統角色並設定權限</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">角色代碼 *</Label>
                <Input id="code" value={rm.formData.code} onChange={(e) => rm.setFormData({ ...rm.formData, code: e.target.value })} placeholder="例如: manager" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">角色名稱 *</Label>
                <Input id="name" value={rm.formData.name} onChange={(e) => rm.setFormData({ ...rm.formData, name: e.target.value })} placeholder="例如: 經理" />
              </div>
            </div>
            <div className="space-y-2">
              <Label>權限設定</Label>
              <div className="border rounded-md p-4">
                <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.formData.permission_ids} onTogglePermission={rm.togglePermission} showSearch={true} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => rm.setShowCreateDialog(false)}>取消</Button>
            <Button onClick={rm.handleCreate} disabled={rm.createMutation.isPending}>
              {rm.createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              創建
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯角色對話框 */}
      <Dialog open={rm.showEditDialog} onOpenChange={rm.setShowEditDialog}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>編輯角色</DialogTitle>
            <DialogDescription>修改角色 {rm.selectedRole?.name} 的設定</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">角色名稱</Label>
              <Input id="edit-name" value={rm.formData.name} onChange={(e) => rm.setFormData({ ...rm.formData, name: e.target.value })} />
            </div>
            <div className="space-y-2">
              <Label>權限設定</Label>
              <div className="border rounded-md p-4">
                <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.formData.permission_ids} onTogglePermission={rm.togglePermission} showSearch={true} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => rm.setShowEditDialog(false)}>取消</Button>
            <Button onClick={rm.handleUpdate} disabled={rm.updateMutation.isPending}>
              {rm.updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 查看權限詳情對話框 */}
      <Dialog open={rm.showDetailDialog} onOpenChange={(open) => { rm.setShowDetailDialog(open); if (!open) rm.setRoleForDetail(null) }}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>{rm.roleForDetail?.name} — 權限詳情</DialogTitle>
            <DialogDescription>共 {rm.roleForDetail?.permissions.length ?? 0} 個權限（唯讀）</DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto border rounded-md p-4 min-h-0">
            <PermissionTree permissions={rm.permissions} selectedPermissionIds={rm.roleForDetail?.permissions.map((p) => p.id) ?? []} showSearch={true} readOnly={true} />
          </div>
        </DialogContent>
      </Dialog>

      {/* SEC-33：刪除角色前重新輸入密碼 */}
      <ConfirmPasswordModal
        open={rm.showReauthForDeleteRole}
        onOpenChange={(open) => { rm.setShowReauthForDeleteRole(open); if (!open) rm.setRoleToDelete(null) }}
        title={rm.roleToDelete?.is_system ? '確認停用系統角色' : '確認刪除角色'}
        description={rm.roleToDelete ? `確定要${rm.roleToDelete.is_system ? '停用' : '刪除'}角色「${rm.roleToDelete.name}」？請輸入您的登入密碼以確認。` : ''}
        onSubmit={rm.handleDeleteConfirm}
      />
    </div>
  )
}
