import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { User, Role, ResetPasswordRequest } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { Loader2, Users, Plus, Pencil, Trash2, Shield, UserCheck, UserX, AlertTriangle, Key } from 'lucide-react'

interface CreateUserData {
  email: string
  password: string
  display_name: string
  role_ids: string[]
}

interface UpdateUserData {
  email?: string
  display_name?: string
  is_active?: boolean
  role_ids?: string[]
}

export function UsersPage() {
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const { user: currentUser } = useAuthStore()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [showRolesDialog, setShowRolesDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showResetPasswordDialog, setShowResetPasswordDialog] = useState(false)
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [userToDelete, setUserToDelete] = useState<User | null>(null)
  const [userToResetPassword, setUserToResetPassword] = useState<User | null>(null)
  const [newPassword, setNewPassword] = useState('')
  const [confirmNewPassword, setConfirmNewPassword] = useState('')
  const [formData, setFormData] = useState<CreateUserData>({
    email: '',
    password: '',
    display_name: '',
    role_ids: [],
  })

  // 獲取用戶列表
  const { data: users, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
  })

  // 獲取角色列表
  const { data: roles } = useQuery({
    queryKey: ['roles'],
    queryFn: async () => {
      const response = await api.get<Role[]>('/roles')
      return response.data
    },
  })

  // 創建用戶
  const createMutation = useMutation({
    mutationFn: async (data: CreateUserData) => {
      const response = await api.post('/users', data)
      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
      setShowCreateDialog(false)
      resetForm()
      toast({ title: '成功', description: '用戶已創建' })
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '創建失敗', variant: 'destructive' })
    },
  })

  // 更新用戶
  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateUserData }) => {
      const response = await api.put(`/users/${id}`, data)
      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
      setShowEditDialog(false)
      setShowRolesDialog(false)
      setSelectedUser(null)
      toast({ title: '成功', description: '用戶已更新' })
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '更新失敗', variant: 'destructive' })
    },
  })

  // 刪除用戶
  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/users/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
      toast({ title: '成功', description: '用戶已刪除' })
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '刪除失敗', variant: 'destructive' })
    },
  })

  // 重設密碼
  const resetPasswordMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: ResetPasswordRequest }) => {
      await api.put(`/users/${id}/password`, data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '密碼已重設' })
      setShowResetPasswordDialog(false)
      setUserToResetPassword(null)
      setNewPassword('')
      setConfirmNewPassword('')
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '重設密碼失敗', variant: 'destructive' })
    },
  })

  const resetForm = () => {
    setFormData({ email: '', password: '', display_name: '', role_ids: [] })
  }

  const handleCreate = () => {
    if (!formData.email || !formData.password || !formData.display_name) {
      toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
      return
    }
    createMutation.mutate(formData)
  }

  const handleEdit = (user: User) => {
    setSelectedUser(user)
    setFormData({
      email: user.email,
      password: '',
      display_name: user.display_name,
      role_ids: [],
    })
    setShowEditDialog(true)
  }

  const handleUpdate = () => {
    if (!selectedUser) return
    updateMutation.mutate({
      id: selectedUser.id,
      data: {
        email: formData.email || undefined,
        display_name: formData.display_name || undefined,
      },
    })
  }

  const handleToggleActive = (user: User) => {
    updateMutation.mutate({
      id: user.id,
      data: { is_active: !user.is_active },
    })
  }

  const handleManageRoles = (user: User) => {
    setSelectedUser(user)
    // 獲取用戶當前的角色 ID
    const userRoleIds = roles?.filter(r => user.roles.includes(r.code)).map(r => r.id) || []
    setFormData(prev => ({ ...prev, role_ids: userRoleIds }))
    setShowRolesDialog(true)
  }

  const handleUpdateRoles = () => {
    if (!selectedUser) return
    updateMutation.mutate({
      id: selectedUser.id,
      data: { role_ids: formData.role_ids },
    })
  }

  const toggleRole = (roleId: string) => {
    setFormData(prev => ({
      ...prev,
      role_ids: prev.role_ids.includes(roleId)
        ? prev.role_ids.filter(id => id !== roleId)
        : [...prev.role_ids, roleId],
    }))
  }

  const handleResetPassword = () => {
    if (!userToResetPassword) return
    if (!newPassword || !confirmNewPassword) {
      toast({ title: '錯誤', description: '請填寫所有欄位', variant: 'destructive' })
      return
    }
    if (newPassword.length < 6) {
      toast({ title: '錯誤', description: '密碼至少需要 6 個字元', variant: 'destructive' })
      return
    }
    if (newPassword !== confirmNewPassword) {
      toast({ title: '錯誤', description: '兩次輸入的密碼不一致', variant: 'destructive' })
      return
    }
    resetPasswordMutation.mutate({
      id: userToResetPassword.id,
      data: { new_password: newPassword },
    })
  }

  const openResetPasswordDialog = (user: User) => {
    setUserToResetPassword(user)
    setNewPassword('')
    setConfirmNewPassword('')
    setShowResetPasswordDialog(true)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">使用者管理</h1>
          <p className="text-muted-foreground">管理系統使用者帳號與角色</p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="h-4 w-4 mr-2" />
          新增使用者
        </Button>
      </div>

      <div className="rounded-md border bg-white">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Email</TableHead>
              <TableHead>名稱</TableHead>
              <TableHead>角色</TableHead>
              <TableHead>狀態</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : users && users.length > 0 ? (
              users.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="font-medium">{user.email}</TableCell>
                  <TableCell>{user.display_name}</TableCell>
                  <TableCell>
                    <div className="flex gap-1 flex-wrap">
                      {user.roles.length > 0 ? (
                        user.roles.map((role) => (
                          <Badge key={role} variant="secondary">
                            {role}
                          </Badge>
                        ))
                      ) : (
                        <span className="text-muted-foreground text-sm">無角色</span>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    {user.is_active ? (
                      <Badge variant="success">啟用</Badge>
                    ) : (
                      <Badge variant="destructive">停用</Badge>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleManageRoles(user)}
                        title="管理角色"
                      >
                        <Shield className="h-4 w-4" />
                      </Button>
                      {/* 重設密碼按鈕（不能重設自己的密碼） */}
                      {user.id !== currentUser?.id && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => openResetPasswordDialog(user)}
                          title="重設密碼"
                        >
                          <Key className="h-4 w-4 text-orange-500" />
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleEdit(user)}
                        title="編輯"
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleToggleActive(user)}
                        title={user.is_active ? '停用' : '啟用'}
                      >
                        {user.is_active ? (
                          <UserX className="h-4 w-4 text-red-500" />
                        ) : (
                          <UserCheck className="h-4 w-4 text-green-500" />
                        )}
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => {
                          setUserToDelete(user)
                          setShowDeleteDialog(true)
                        }}
                        title="刪除"
                      >
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  <Users className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無使用者資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* 創建用戶對話框 */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增使用者</DialogTitle>
            <DialogDescription>創建新的系統使用者帳號</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email *</Label>
              <Input
                id="email"
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                placeholder="user@example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">密碼 *</Label>
              <Input
                id="password"
                type="password"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                placeholder="至少 6 個字元"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="display_name">顯示名稱 *</Label>
              <Input
                id="display_name"
                value={formData.display_name}
                onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
                placeholder="使用者名稱"
              />
            </div>
            <div className="space-y-2">
              <Label>指派角色</Label>
              <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                {roles?.map((role) => (
                  <Badge
                    key={role.id}
                    variant={formData.role_ids.includes(role.id) ? 'default' : 'outline'}
                    className="cursor-pointer"
                    onClick={() => toggleRole(role.id)}
                  >
                    {role.name}
                  </Badge>
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

      {/* 編輯用戶對話框 */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>編輯使用者</DialogTitle>
            <DialogDescription>修改使用者資訊</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-email">Email</Label>
              <Input
                id="edit-email"
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-display_name">顯示名稱</Label>
              <Input
                id="edit-display_name"
                value={formData.display_name}
                onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
              />
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

      {/* 管理角色對話框 */}
      <Dialog open={showRolesDialog} onOpenChange={setShowRolesDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>管理角色</DialogTitle>
            <DialogDescription>
              為 {selectedUser?.display_name} 指派角色
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <div className="space-y-2 max-h-[360px] overflow-y-auto">
              {roles?.map((role) => (
                <div
                  key={role.id}
                  className={`p-3 border rounded-md cursor-pointer transition-colors ${
                    formData.role_ids.includes(role.id)
                      ? 'border-primary bg-primary/5'
                      : 'hover:bg-muted'
                  }`}
                  onClick={() => toggleRole(role.id)}
                >
                  <div className="flex items-center gap-2">
                    <Shield className={`h-4 w-4 ${formData.role_ids.includes(role.id) ? 'text-primary' : 'text-muted-foreground'}`} />
                    <span className="font-medium">{role.name}</span>
                    <span className="text-xs text-muted-foreground">({role.code})</span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    {role.permissions.length} 個權限
                  </p>
                </div>
              ))}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowRolesDialog(false)}>
              取消
            </Button>
            <Button onClick={handleUpdateRoles} disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 刪除確認對話框 */}
      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-500" />
              確認刪除使用者
            </DialogTitle>
            <DialogDescription>
              此操作無法復原。確定要刪除使用者 <span className="font-medium">{userToDelete?.display_name}</span>（{userToDelete?.email}）嗎？
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <div className="p-3 bg-red-50 border border-red-200 rounded-md">
              <p className="text-sm text-red-800">
                刪除後，該使用者將無法再登入系統。如果只是暫時停用，建議使用「停用」功能。
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setShowDeleteDialog(false)
                setUserToDelete(null)
              }}
            >
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                if (userToDelete) {
                  deleteMutation.mutate(userToDelete.id)
                  setShowDeleteDialog(false)
                  setUserToDelete(null)
                }
              }}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認刪除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 重設密碼對話框 */}
      <Dialog open={showResetPasswordDialog} onOpenChange={(open) => {
        setShowResetPasswordDialog(open)
        if (!open) {
          setUserToResetPassword(null)
          setNewPassword('')
          setConfirmNewPassword('')
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Key className="h-5 w-5 text-orange-500" />
              重設使用者密碼
            </DialogTitle>
            <DialogDescription>
              為使用者 <span className="font-medium">{userToResetPassword?.display_name}</span>（{userToResetPassword?.email}）設定新密碼。
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="p-3 bg-orange-50 border border-orange-200 rounded-md">
              <p className="text-sm text-orange-800">
                重設密碼後，該使用者需要使用新密碼重新登入。建議通知該使用者密碼已變更。
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="reset-new-password">新密碼</Label>
              <Input
                id="reset-new-password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="至少 6 個字元"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="reset-confirm-password">確認新密碼</Label>
              <Input
                id="reset-confirm-password"
                type="password"
                value={confirmNewPassword}
                onChange={(e) => setConfirmNewPassword(e.target.value)}
                placeholder="再次輸入新密碼"
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setShowResetPasswordDialog(false)
                setUserToResetPassword(null)
                setNewPassword('')
                setConfirmNewPassword('')
              }}
              disabled={resetPasswordMutation.isPending}
            >
              取消
            </Button>
            <Button
              onClick={handleResetPassword}
              disabled={resetPasswordMutation.isPending}
            >
              {resetPasswordMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認重設
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
