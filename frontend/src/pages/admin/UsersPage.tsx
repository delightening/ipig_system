import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { confirmPassword, User, Role, ResetPasswordRequest } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
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
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import { Loader2, Users, Plus, Pencil, Trash2, Shield, UserCheck, UserX, AlertTriangle, Key, ArrowUpDown, ArrowUp, ArrowDown, LogIn, ChevronLeft, ChevronRight } from 'lucide-react'

interface UserTrainingInput {
  code: string
  certificate_no: string
  received_date: string
}

interface CreateUserData {
  email: string
  password: string
  display_name: string
  role_ids: string[]
  entry_date?: string
  position?: string
  aup_roles: string[]
  years_experience: number
  trainings: UserTrainingInput[]
}

interface UpdateUserData {
  email?: string
  display_name?: string
  is_active?: boolean
  role_ids?: string[]
  entry_date?: string
  position?: string
  aup_roles?: string[]
  years_experience?: number
  trainings?: UserTrainingInput[]
}

export function UsersPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const { user: currentUser, impersonate } = useAuthStore()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [showRolesDialog, setShowRolesDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showReauthForDelete, setShowReauthForDelete] = useState(false)
  const [showResetPasswordDialog, setShowResetPasswordDialog] = useState(false)
  const [showReauthForImpersonate, setShowReauthForImpersonate] = useState(false)
  const [userToImpersonate, setUserToImpersonate] = useState<User | null>(null)
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [userToDelete, setUserToDelete] = useState<User | null>(null)
  const [userToResetPassword, setUserToResetPassword] = useState<User | null>(null)
  const [newPassword, setNewPassword] = useState('')
  const [confirmNewPassword, setConfirmNewPassword] = useState('')
  const [reauthPassword, setReauthPassword] = useState('')
  const [formData, setFormData] = useState<CreateUserData>({
    email: '',
    password: '',
    display_name: '',
    role_ids: [],
    entry_date: '',
    position: '',
    aup_roles: [],
    years_experience: 0,
    trainings: [],
  })

  // 排序狀態: 'asc' | 'desc' | null
  const [sortRole, setSortRole] = useState<'asc' | 'desc' | null>(null)
  const [sortStatus, setSortStatus] = useState<'asc' | 'desc' | null>(null)

  // 分頁狀態
  const perPage = 50
  const [currentPage, setCurrentPage] = useState(1)

  // 獲取用戶列表
  const { data: users, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
  })

  // 排序後的用戶列表
  const sortedUsers = useMemo(() => {
    if (!users) return []
    let sorted = [...users]

    // 按狀態排序
    if (sortStatus) {
      sorted.sort((a, b) => {
        if (sortStatus === 'asc') {
          return (a.is_active ? 1 : 0) - (b.is_active ? 1 : 0) // 停用在前
        } else {
          return (b.is_active ? 1 : 0) - (a.is_active ? 1 : 0) // 啟用在前
        }
      })
    }

    // 按角色排序
    if (sortRole) {
      sorted.sort((a, b) => {
        const aRoles = a.roles.slice().sort().join(',')
        const bRoles = b.roles.slice().sort().join(',')
        if (sortRole === 'asc') {
          return aRoles.localeCompare(bRoles)
        } else {
          return bRoles.localeCompare(aRoles)
        }
      })
    }

    return sorted
  }, [users, sortRole, sortStatus])

  // 分頁計算
  const totalPages = Math.ceil(sortedUsers.length / perPage)
  const paginatedUsers = useMemo(() => {
    const start = (currentPage - 1) * perPage
    return sortedUsers.slice(start, start + perPage)
  }, [sortedUsers, currentPage, perPage])

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
      // 解析詳細的錯誤訊息
      let errorMessage = '創建失敗'
      let detailMessage = ''
      const backendMessage = error.response?.data?.error?.message
      const statusCode = error.response?.status
      const rawData = error.response?.data

      if (backendMessage) {
        // 翻譯常見的驗證錯誤訊息
        if (backendMessage.includes('Password must be at least')) {
          errorMessage = '密碼至少需要 6 個字元'
        } else if (backendMessage.includes('Password must contain uppercase, lowercase, and numeric')) {
          errorMessage = '密碼必須包含大寫字母、小寫字母和數字'
        } else if (backendMessage.includes('Invalid email format')) {
          errorMessage = 'Email 格式不正確'
        } else if (backendMessage.includes('Display name is required')) {
          errorMessage = '顯示名稱為必填欄位'
        } else if (backendMessage.includes('Email already exists')) {
          errorMessage = '此 Email 已被使用'
        } else if (backendMessage.includes('Validation failed')) {
          // 提取驗證失敗的詳細訊息
          errorMessage = backendMessage.replace('Validation failed:', '驗證失敗:')
        } else {
          errorMessage = backendMessage
        }
      } else if (typeof rawData === 'string' && statusCode === 422) {
        // 處理 Axum 預設的 422 錯誤（通常是 JSON 解析失敗）
        errorMessage = '資料格式錯誤 (422)'
        detailMessage = rawData
      } else if (statusCode === 500) {
        errorMessage = '伺服器內部錯誤，請稍後再試'
      } else if (statusCode === 403) {
        errorMessage = '權限不足'
      } else if (rawData?.error?.message) {
        errorMessage = rawData.error.message
      } else if (rawData?.message) {
        errorMessage = rawData.message
      } else {
        // 最後備案：顯示狀態碼與原始訊息
        errorMessage = `請求失敗 (${statusCode || 'Unknown'})`
        detailMessage = typeof rawData === 'object' ? JSON.stringify(rawData) : String(rawData)
      }

      toast({
        title: '錯誤',
        description: (
          <div className="space-y-1">
            <p>{errorMessage}</p>
            {detailMessage && (
              <p className="text-xs opacity-70 break-all font-mono bg-black/5 p-1 rounded">
                Detail: {detailMessage}
              </p>
            )}
          </div>
        ),
        variant: 'destructive',
      })
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

  // 刪除用戶（SEC-33：需先取得 reauth token，在 ConfirmPasswordModal onSubmit 內呼叫）
  const deleteUserWithReauth = async (id: string, reauthToken: string) => {
    await api.delete(`/users/${id}`, { headers: { 'X-Reauth-Token': reauthToken } })
    queryClient.invalidateQueries({ queryKey: ['users'] })
    toast({ title: '成功', description: '用戶已刪除' })
  }

  // 重設密碼（SEC-33：需帶 X-Reauth-Token）
  const resetPasswordMutation = useMutation({
    mutationFn: async ({ id, data, reauthToken }: { id: string; data: ResetPasswordRequest; reauthToken: string }) => {
      await api.put(`/users/${id}/password`, data, { headers: { 'X-Reauth-Token': reauthToken } })
    },
    onSuccess: () => {
      toast({ title: '成功', description: '密碼已重設' })
      setShowResetPasswordDialog(false)
      setUserToResetPassword(null)
      setNewPassword('')
      setConfirmNewPassword('')
      setReauthPassword('')
    },
    onError: (error: any) => {
      toast({ title: '錯誤', description: error.response?.data?.error?.message || '重設密碼失敗', variant: 'destructive' })
    },
  })

  const resetForm = () => {
    setFormData({
      email: '', password: '', display_name: '', role_ids: [],
      entry_date: '', position: '', aup_roles: [], years_experience: 0, trainings: []
    })
  }

  const handleCreate = () => {
    if (!formData.email || !formData.password || !formData.display_name) {
      toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
      return
    }
    if (formData.password.length < 6) {
      toast({ title: '錯誤', description: '密碼至少需要 6 個字元', variant: 'destructive' })
      return
    }

    // 處理空字串欄位，避免後端反序列化失敗 (422)
    const data = {
      ...formData,
      entry_date: formData.entry_date || undefined,
      position: formData.position || undefined,
    }
    createMutation.mutate(data)
  }

  const handleEdit = (user: User) => {
    setSelectedUser(user)
    setFormData({
      email: user.email,
      password: '',
      display_name: user.display_name,
      role_ids: [],
      entry_date: user.entry_date || '',
      position: user.position || '',
      aup_roles: user.aup_roles || [],
      years_experience: user.years_experience || 0,
      trainings: (user.trainings || []).map(t => ({
        code: t.code,
        certificate_no: t.certificate_no || '',
        received_date: t.received_date || ''
      })),
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
        entry_date: formData.entry_date || undefined,
        position: formData.position || undefined,
        aup_roles: formData.aup_roles,
        years_experience: formData.years_experience,
        trainings: formData.trainings,
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

  // 切換排序
  const toggleSortRole = () => {
    if (sortRole === null) {
      setSortRole('asc')
    } else if (sortRole === 'asc') {
      setSortRole('desc')
    } else {
      setSortRole(null)
    }
  }

  const toggleSortStatus = () => {
    if (sortStatus === null) {
      setSortStatus('asc')
    } else if (sortStatus === 'asc') {
      setSortStatus('desc')
    } else {
      setSortStatus(null)
    }
  }

  const getSortIcon = (sort: 'asc' | 'desc' | null) => {
    if (sort === 'asc') return <ArrowUp className="h-4 w-4" />
    if (sort === 'desc') return <ArrowDown className="h-4 w-4" />
    return <ArrowUpDown className="h-4 w-4 text-muted-foreground" />
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
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:text-foreground transition-colors"
                  onClick={toggleSortRole}
                >
                  角色
                  {getSortIcon(sortRole)}
                </button>
              </TableHead>
              <TableHead>
                <button
                  className="flex items-center gap-1 hover:text-foreground transition-colors"
                  onClick={toggleSortStatus}
                >
                  狀態
                  {getSortIcon(sortStatus)}
                </button>
              </TableHead>
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
            ) : paginatedUsers.length > 0 ? (
              paginatedUsers.map((user) => (
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
                      {/* 模擬登入（管理員專用） */}
                      {user.id !== currentUser?.id && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => impersonate(user.id)}
                          title="模擬登入 (Login As)"
                        >
                          <LogIn className="h-4 w-4 text-blue-500" />
                        </Button>
                      )}
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
        {/* 分頁控制列 */}
        {totalPages > 0 && (
          <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
            <p className="text-sm text-muted-foreground">
              共 {sortedUsers.length} 筆，第 {currentPage} / {totalPages} 頁
            </p>
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm" disabled={currentPage <= 1} onClick={() => setCurrentPage(p => Math.max(1, p - 1))}>
                <ChevronLeft className="h-4 w-4 mr-1" />上一頁
              </Button>
              <Button variant="outline" size="sm" disabled={currentPage >= totalPages} onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}>
                下一頁<ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            </div>
          </div>
        )}
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


            {/* 入職日期與訓練/資格欄位 */}
            <div className="border-t pt-4 mt-4">
              <h4 className="font-medium mb-3">AUP 人員資料</h4>

              <div className="space-y-2 mb-4">
                <Label htmlFor="edit-entry_date">入職日期 (Entry Date)</Label>
                <Input
                  id="edit-entry_date"
                  type="date"
                  value={formData.entry_date || ''}
                  onChange={(e) => setFormData({ ...formData, entry_date: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label>訓練/資格 (Trainings)</Label>
                <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                  {['A', 'B', 'C', 'D', 'E', 'F'].map((code) => (
                    <Badge
                      key={code}
                      variant={formData.trainings.some(t => t.code === code) ? 'default' : 'outline'}
                      className="cursor-pointer"
                      onClick={() => {
                        const exists = formData.trainings.some(t => t.code === code)
                        const newTrainings = exists
                          ? formData.trainings.filter(t => t.code !== code)
                          : [...formData.trainings, { code, certificate_no: '', received_date: '' }]
                        setFormData({ ...formData, trainings: newTrainings })
                      }}
                    >
                      {code}
                    </Badge>
                  ))}
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  A.IACUC訓練班 B.IACUC研討會 C.輻射安全 D.生醫產業研習 E.動物法規管理班 F.其他
                </p>

                {formData.trainings.length > 0 && (
                  <div className="space-y-2 mt-3">
                    {formData.trainings.map((training, idx) => (
                      <div key={training.code} className="flex gap-2 items-center">
                        <Badge variant="secondary">{training.code}</Badge>
                        <Input
                          placeholder="證書編號"
                          value={training.certificate_no}
                          onChange={(e) => {
                            const newTrainings = [...formData.trainings]
                            newTrainings[idx] = { ...training, certificate_no: e.target.value }
                            setFormData({ ...formData, trainings: newTrainings })
                          }}
                          className="w-32"
                        />
                        <Input
                          type="date"
                          value={training.received_date}
                          onChange={(e) => {
                            const newTrainings = [...formData.trainings]
                            newTrainings[idx] = { ...training, received_date: e.target.value }
                            setFormData({ ...formData, trainings: newTrainings })
                          }}
                          className="w-36"
                        />
                      </div>
                    ))}
                  </div>
                )}
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
                  className={`p-3 border rounded-md cursor-pointer transition-colors ${formData.role_ids.includes(role.id)
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
