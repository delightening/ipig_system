import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Role, Permission } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
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
import { Loader2, Shield, Plus, Pencil, Trash2, Check } from 'lucide-react'

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
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '創建失敗', variant: 'destructive' })
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
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '更新失敗', variant: 'destructive' })
    },
  })

  // 刪除角色
  const deleteMutation = useMutation({
    mutationFn: async ({ id }: { id: string; is_system: boolean }) => {
      await api.delete(`/roles/${id}`)
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
      toast({
        title: '成功',
        description: variables.is_system ? '系統角色已停用' : '角色已刪除',
      })
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '刪除失敗', variant: 'destructive' })
    },
  })

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

  // 按類別分組權限
  const groupedPermissions = permissions?.reduce((acc, perm) => {
    const category = perm.code.split('.')[0]
    if (!acc[category]) acc[category] = []
    acc[category].push(perm)
    return acc
  }, {} as Record<string, Permission[]>)

  const categoryNames: Record<string, string> = {
    user: '用戶管理',
    role: '角色管理',
    warehouse: '倉庫管理',
    product: '產品管理',
    partner: '夥伴管理',
    document: '單據管理',
    po: '採購單',
    grn: '採購入庫',
    pr: '採購退貨',
    so: '銷售單',
    do: '銷售出庫',
    sr: '銷售退貨',
    tr: '調撥單',
    stk: '盤點單',
    adj: '調整單',
    stock: '庫存管理',
    report: '報表',
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">角色權限</h1>
          <p className="text-muted-foreground">管理系統角色與權限設定</p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="h-4 w-4 mr-2" />
          新增角色
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {roles?.map((role) => (
          <Card key={role.id}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Shield className="h-5 w-5 text-blue-500" />
                  {role.name}
                  {role.is_system && (
                    <Badge variant="secondary" className="text-xs">
                      System
                    </Badge>
                  )}
                </CardTitle>
                <div className="flex gap-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(role)}>
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => {
                      const message = role.is_system
                        ? '確定要停用此系統角色嗎？'
                        : '確定要刪除此角色嗎？'
                      if (confirm(message)) {
                        deleteMutation.mutate({ id: role.id, is_system: role.is_system })
                      }
                    }}
                  >
                    <Trash2 className="h-4 w-4 text-red-500" />
                  </Button>
                </div>
              </div>
              <p className="text-sm text-muted-foreground font-mono">{role.code}</p>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-1">
                {role.permissions.slice(0, 8).map((perm) => (
                  <Badge key={perm.id} variant="outline" className="text-xs">
                    {perm.name}
                  </Badge>
                ))}
                {role.permissions.length > 8 && (
                  <Badge variant="secondary" className="text-xs">
                    +{role.permissions.length - 8} 更多
                  </Badge>
                )}
                {role.permissions.length === 0 && (
                  <span className="text-sm text-muted-foreground">無權限</span>
                )}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* 創建角色對話框 */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
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
              <div className="border rounded-md p-4 space-y-4 max-h-[400px] overflow-y-auto">
                {groupedPermissions && Object.entries(groupedPermissions).map(([category, perms]) => (
                  <div key={category}>
                    <h4 className="font-medium text-sm mb-2">{categoryNames[category] || category}</h4>
                    <div className="flex flex-wrap gap-2">
                      {perms.map((perm) => (
                        <Badge
                          key={perm.id}
                          variant={formData.permission_ids.includes(perm.id) ? 'default' : 'outline'}
                          className="cursor-pointer"
                          onClick={() => togglePermission(perm.id)}
                        >
                          {formData.permission_ids.includes(perm.id) && (
                            <Check className="h-3 w-3 mr-1" />
                          )}
                          {perm.name}
                        </Badge>
                      ))}
                    </div>
                  </div>
                ))}
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
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
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
              <div className="border rounded-md p-4 space-y-4 max-h-[400px] overflow-y-auto">
                {groupedPermissions && Object.entries(groupedPermissions).map(([category, perms]) => (
                  <div key={category}>
                    <h4 className="font-medium text-sm mb-2">{categoryNames[category] || category}</h4>
                    <div className="flex flex-wrap gap-2">
                      {perms.map((perm) => (
                        <Badge
                          key={perm.id}
                          variant={formData.permission_ids.includes(perm.id) ? 'default' : 'outline'}
                          className="cursor-pointer"
                          onClick={() => togglePermission(perm.id)}
                        >
                          {formData.permission_ids.includes(perm.id) && (
                            <Check className="h-3 w-3 mr-1" />
                          )}
                          {perm.name}
                        </Badge>
                      ))}
                    </div>
                  </div>
                ))}
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
    </div>
  )
}
