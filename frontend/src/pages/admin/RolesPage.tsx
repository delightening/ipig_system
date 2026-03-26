import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { confirmPassword, deleteResource, Role, Permission } from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { Loader2, Shield, Plus, Pencil, Trash2, Eye } from 'lucide-react'
import { PermissionTree } from '@/components/admin/PermissionTree'
import { groupPermissionsByModule } from '@/hooks/usePermissionManager'

interface CreateRoleData {
  code: string
  name: string
  permission_ids: string[]
}

export function RolesPage() {
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [showDetailDialog, setShowDetailDialog] = useState(false)
  const [roleForDetail, setRoleForDetail] = useState<Role | null>(null)
  const [showReauthForDeleteRole, setShowReauthForDeleteRole] = useState(false)
  const [roleToDelete, setRoleToDelete] = useState<Role | null>(null)
  const [selectedRole, setSelectedRole] = useState<Role | null>(null)
  const [formData, setFormData] = useState<CreateRoleData>({
    code: '',
    name: '',
    permission_ids: [],
  })

  // 獲取角色列表
  const { data: roles, isLoading } = useQuery({
    queryKey: ['roles'],
    queryFn: async () => {
      const response = await api.get<Role[]>('/roles')
      return response.data
    },
  })

  // 獲取權限列表
  const { data: permissions } = useQuery({
    queryKey: ['permissions'],
    queryFn: async () => {
      const response = await api.get<Permission[]>('/permissions')
      return response.data
    },
  })

  // 創建角色
  const createMutation = useMutation({
    mutationFn: async (data: CreateRoleData) => {
      const response = await api.post('/roles', data)
      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
      setShowCreateDialog(false)
      resetForm()
      toast({ title: '成功', description: '角色已創建' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '創建失敗',
        variant: 'destructive'
      })
    },
  })

  // 更新角色
  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<CreateRoleData> }) => {
      const response = await api.put(`/roles/${id}`, data)
      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
      setShowEditDialog(false)
      setSelectedRole(null)
      toast({ title: '成功', description: '角色已更新' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '更新失敗',
        variant: 'destructive'
      })
    },
  })

  // 刪除角色（SEC-33：需帶 X-Reauth-Token，在 ConfirmPasswordModal 內取得後呼叫）
  const deleteRoleWithReauth = async (id: string, reauthToken: string, is_system: boolean) => {
    await deleteResource(`/roles/${id}`, { headers: { 'X-Reauth-Token': reauthToken } })
    queryClient.invalidateQueries({ queryKey: ['roles'] })
    toast({
      title: '成功',
      description: is_system ? '系統角色已停用' : '角色已刪除',
    })
  }

  const resetForm = () => {
    setFormData({ code: '', name: '', permission_ids: [] })
  }

  const handleCreate = () => {
    if (!formData.code || !formData.name) {
      toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
      return
    }
    createMutation.mutate(formData)
  }

  const handleEdit = (role: Role) => {
    setSelectedRole(role)
    setFormData({
      code: role.code,
      name: role.name,
      permission_ids: role.permissions.map(p => p.id),
    })
    setShowEditDialog(true)
  }

  const handleUpdate = () => {
    if (!selectedRole) return
    updateMutation.mutate({
      id: selectedRole.id,
      data: {
        name: formData.name,
        permission_ids: formData.permission_ids,
      },
    })
  }

  const togglePermission = (permId: string) => {
    setFormData(prev => ({
      ...prev,
      permission_ids: prev.permission_ids.includes(permId)
        ? prev.permission_ids.filter(id => id !== permId)
        : [...prev.permission_ids, permId],
    }))
  }

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
        title="角色權限"
        description="管理系統角色與權限設定"
        actions={
          <Button size="sm" onClick={() => setShowCreateDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            新增角色
          </Button>
        }
      />

      {/* 角色列表：<640px 1 欄、<1280px 2 欄、<1920px 3 欄、≥1920px 4 欄 */}
      <div className="grid gap-4 grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 min-[1920px]:grid-cols-4">
        {roles?.map((role) => (
          <Card key={role.id}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Shield className="h-5 w-5 text-primary" />
                  {role.name}
                  {role.is_system && (
                    <Badge variant="secondary" className="text-xs">
                      System
                    </Badge>
                  )}
                </CardTitle>
                <div className="flex gap-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(role)} aria-label="編輯">
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => {
                      setRoleToDelete(role)
                      setShowReauthForDeleteRole(true)
                    }}
                    aria-label="刪除"
                  >
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
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => {
                      setRoleForDetail(role)
                      setShowDetailDialog(true)
                    }}
                  >
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
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>新增角色</DialogTitle>
            <DialogDescription>創建新的系統角色並設定權限</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">角色代碼 *</Label>
                <Input
                  id="code"
                  value={formData.code}
                  onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                  placeholder="例如: manager"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">角色名稱 *</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="例如: 經理"
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label>權限設定</Label>
              <div className="border rounded-md p-4">
                <PermissionTree
                  permissions={permissions}
                  selectedPermissionIds={formData.permission_ids}
                  onTogglePermission={togglePermission}
                  showSearch={true}
                />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
              取消
            </Button>
            <Button onClick={handleCreate} disabled={createMutation.isPending}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              創建
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯角色對話框 */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>編輯角色</DialogTitle>
            <DialogDescription>修改角色 {selectedRole?.name} 的設定</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">角色名稱</Label>
              <Input
                id="edit-name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>權限設定</Label>
              <div className="border rounded-md p-4">
                <PermissionTree
                  permissions={permissions}
                  selectedPermissionIds={formData.permission_ids}
                  onTogglePermission={togglePermission}
                  showSearch={true}
                />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEditDialog(false)}>
              取消
            </Button>
            <Button onClick={handleUpdate} disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 查看權限詳情對話框 */}
      <Dialog
        open={showDetailDialog}
        onOpenChange={(open) => {
          setShowDetailDialog(open)
          if (!open) setRoleForDetail(null)
        }}
      >
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>
              {roleForDetail?.name} — 權限詳情
            </DialogTitle>
            <DialogDescription>
              共 {roleForDetail?.permissions.length ?? 0} 個權限（唯讀）
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto border rounded-md p-4 min-h-0">
            <PermissionTree
              permissions={permissions}
              selectedPermissionIds={roleForDetail?.permissions.map((p) => p.id) ?? []}
              showSearch={true}
              readOnly={true}
            />
          </div>
        </DialogContent>
      </Dialog>

      {/* SEC-33：刪除角色前重新輸入密碼 */}
      <ConfirmPasswordModal
        open={showReauthForDeleteRole}
        onOpenChange={(open) => {
          setShowReauthForDeleteRole(open)
          if (!open) setRoleToDelete(null)
        }}
        title={roleToDelete?.is_system ? '確認停用系統角色' : '確認刪除角色'}
        description={roleToDelete ? `確定要${roleToDelete.is_system ? '停用' : '刪除'}角色「${roleToDelete.name}」？請輸入您的登入密碼以確認。` : ''}
        onSubmit={async (password) => {
          const { reauth_token } = await confirmPassword(password)
          if (!roleToDelete) return
          await deleteRoleWithReauth(roleToDelete.id, reauth_token, roleToDelete.is_system)
          setRoleToDelete(null)
        }}
      />
    </div>
  )
}
