import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import axios from 'axios'
import { z } from 'zod'
import api, { confirmPassword, deleteResource, User, Role, ResetPasswordRequest } from '@/lib/api'
import { getErrorMessage, ApiErrorPayload } from '@/types/error'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/components/ui/use-toast'
import { getPasswordError, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'

// === Zod 驗證 Schema ===

const createUserFormSchema = z.object({
  email: z.string().min(1, '請輸入 Email').email('請輸入有效的 Email'),
  password: z.string().min(PASSWORD_MIN_LENGTH, `密碼至少需要 ${PASSWORD_MIN_LENGTH} 個字元`),
  display_name: z.string().min(2, '至少 2 個字元').max(50, '最多 50 個字元').trim(),
  role_ids: z.array(z.string()).min(1, '請選擇角色'),
})

const updateUserFormSchema = z.object({
  email: z.string().email('請輸入有效的 Email').optional().or(z.literal('')),
  display_name: z.string().min(2, '至少 2 個字元').max(50, '最多 50 個字元').trim().optional().or(z.literal('')),
})

export interface UserTrainingInput {
  code: string
  certificate_no: string
  received_date: string
}

export interface CreateUserData {
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

export interface UpdateUserData {
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

const defaultFormData: CreateUserData = {
  email: '',
  password: '',
  display_name: '',
  role_ids: [],
  entry_date: '',
  position: '',
  aup_roles: [],
  years_experience: 0,
  trainings: [],
}

export function useUserManagement() {
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
  const [formData, setFormData] = useState<CreateUserData>(defaultFormData)
  const [sortRole, setSortRole] = useState<'asc' | 'desc' | null>(null)
  const [sortStatus, setSortStatus] = useState<'asc' | 'desc' | null>(null)
  const perPage = 50
  const [currentPage, setCurrentPage] = useState(1)

  const { data: users, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
    staleTime: 60_000,
  })

  const sortedUsers = useMemo(() => {
    if (!users) return []
    const sorted = [...users]
    if (sortStatus) {
      sorted.sort((a, b) => {
        if (sortStatus === 'asc') return (a.is_active ? 1 : 0) - (b.is_active ? 1 : 0)
        return (b.is_active ? 1 : 0) - (a.is_active ? 1 : 0)
      })
    }
    if (sortRole) {
      sorted.sort((a, b) => {
        const aRoles = a.roles.slice().sort().join(',')
        const bRoles = b.roles.slice().sort().join(',')
        return sortRole === 'asc' ? aRoles.localeCompare(bRoles) : bRoles.localeCompare(aRoles)
      })
    }
    return sorted
  }, [users, sortRole, sortStatus])

  const totalPages = Math.ceil(sortedUsers.length / perPage)
  const paginatedUsers = useMemo(() => {
    const start = (currentPage - 1) * perPage
    return sortedUsers.slice(start, start + perPage)
  }, [sortedUsers, currentPage, perPage])

  const { data: roles } = useQuery({
    queryKey: ['roles'],
    queryFn: async () => {
      const response = await api.get<Role[]>('/roles')
      return response.data
    },
    staleTime: 600_000,
  })

  const resetForm = () => {
    setFormData(defaultFormData)
  }

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
    onError: (error: unknown) => {
      let errorMessage: string
      let detailMessage = ''
      if (axios.isAxiosError(error)) {
        const backendMessage = (error.response?.data as ApiErrorPayload | undefined)?.error?.message
        const statusCode = error.response?.status
        const rawData = error.response?.data
        if (backendMessage) {
          if (backendMessage.includes('Password must be at least'))
            errorMessage = `密碼至少需要 ${PASSWORD_MIN_LENGTH} 個字元`
          else if (
            backendMessage.includes('Password must contain uppercase, lowercase, and numeric')
          )
            errorMessage = '密碼必須包含大寫字母、小寫字母和數字'
          else if (backendMessage.includes('Invalid email format')) errorMessage = 'Email 格式不正確'
          else if (backendMessage.includes('Display name is required'))
            errorMessage = '顯示名稱為必填欄位'
          else if (backendMessage.includes('Email already exists')) errorMessage = '此 Email 已被使用'
          else if (backendMessage.includes('Validation failed'))
            errorMessage = backendMessage.replace('Validation failed:', '驗證失敗:')
          else errorMessage = backendMessage
        } else if (typeof rawData === 'string' && statusCode === 422) {
          errorMessage = '資料格式錯誤 (422)'
          detailMessage = rawData
        } else if (statusCode === 500) {
          errorMessage = '伺服器內部錯誤，請稍後再試'
        } else if (statusCode === 403) {
          errorMessage = '權限不足'
        } else {
          errorMessage = `請求失敗 (${statusCode || 'Unknown'})`
          detailMessage = typeof rawData === 'object' ? JSON.stringify(rawData) : String(rawData)
        }
      } else {
        errorMessage = getErrorMessage(error) || '創建失敗'
      }
      toast({
        title: '錯誤',
        description: detailMessage
          ? `${errorMessage}\n\nDetail: ${detailMessage}`
          : errorMessage,
        variant: 'destructive',
      })
    },
  })

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
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(error) || '更新失敗', variant: 'destructive' })
    },
  })

  const deleteUserWithReauth = async (id: string, reauthToken: string) => {
    await deleteResource(`/users/${id}`, { headers: { 'X-Reauth-Token': reauthToken } })
    queryClient.invalidateQueries({ queryKey: ['users'] })
    toast({ title: '成功', description: '用戶已刪除' })
  }

  const resetPasswordMutation = useMutation({
    mutationFn: async ({
      id,
      data,
      reauthToken,
    }: {
      id: string
      data: ResetPasswordRequest
      reauthToken: string
    }) => {
      await api.put(`/users/${id}/password`, data, {
        headers: { 'X-Reauth-Token': reauthToken },
      })
    },
    onSuccess: () => {
      toast({ title: '成功', description: '密碼已重設' })
      setShowResetPasswordDialog(false)
      setUserToResetPassword(null)
      setNewPassword('')
      setConfirmNewPassword('')
      setReauthPassword('')
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '重設密碼失敗',
        variant: 'destructive',
      })
    },
  })

  const handleCreate = () => {
    const result = createUserFormSchema.safeParse(formData)
    if (!result.success) {
      const firstError = result.error.issues[0]?.message ?? '驗證失敗'
      toast({ title: '錯誤', description: firstError, variant: 'destructive' })
      return
    }
    // 密碼複雜度驗證（含大寫、小寫、數字、弱密碼黑名單）
    const pwError = getPasswordError(formData.password)
    if (pwError) {
      toast({ title: '錯誤', description: pwError, variant: 'destructive' })
      return
    }
    createMutation.mutate({
      ...formData,
      entry_date: formData.entry_date || undefined,
      position: formData.position || undefined,
    })
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
      trainings: (user.trainings || []).map((t) => ({
        code: t.code,
        certificate_no: t.certificate_no || '',
        received_date: t.received_date || '',
      })),
    })
    setShowEditDialog(true)
  }

  const handleUpdate = () => {
    if (!selectedUser) return
    const result = updateUserFormSchema.safeParse({
      email: formData.email,
      display_name: formData.display_name,
    })
    if (!result.success) {
      const firstError = result.error.issues[0]?.message ?? '驗證失敗'
      toast({ title: '錯誤', description: firstError, variant: 'destructive' })
      return
    }
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
    updateMutation.mutate({ id: user.id, data: { is_active: !user.is_active } })
  }

  const handleManageRoles = (user: User) => {
    setSelectedUser(user)
    const userRoleIds =
      roles?.filter((r) => user.roles.includes(r.code)).map((r) => r.id) || []
    setFormData((prev) => ({ ...prev, role_ids: userRoleIds }))
    setShowRolesDialog(true)
  }

  const handleUpdateRoles = () => {
    if (!selectedUser) return
    updateMutation.mutate({ id: selectedUser.id, data: { role_ids: formData.role_ids } })
  }

  const toggleRole = (roleId: string) => {
    setFormData((prev) => ({
      ...prev,
      role_ids: prev.role_ids.includes(roleId)
        ? prev.role_ids.filter((id) => id !== roleId)
        : [...prev.role_ids, roleId],
    }))
  }

  const confirmPasswordMutation = useMutation({
    mutationFn: (password: string) => confirmPassword(password),
    onSuccess: ({ reauth_token }) => {
      if (!userToResetPassword) return
      resetPasswordMutation.mutate({
        id: userToResetPassword.id,
        data: { new_password: newPassword },
        reauthToken: reauth_token,
      })
    },
    onError: () => {
      toast({ title: '錯誤', description: '密碼錯誤，請重新輸入您的登入密碼', variant: 'destructive' })
    },
  })

  const handleResetPassword = () => {
    if (!userToResetPassword) return
    if (!reauthPassword) {
      toast({ title: '錯誤', description: '請輸入您的登入密碼以確認身份', variant: 'destructive' })
      return
    }
    if (!newPassword || !confirmNewPassword) {
      toast({ title: '錯誤', description: '請填寫所有欄位', variant: 'destructive' })
      return
    }
    const resetPwError = getPasswordError(newPassword)
    if (resetPwError) {
      toast({ title: '錯誤', description: resetPwError, variant: 'destructive' })
      return
    }
    if (newPassword !== confirmNewPassword) {
      toast({ title: '錯誤', description: '兩次輸入的密碼不一致', variant: 'destructive' })
      return
    }
    confirmPasswordMutation.mutate(reauthPassword)
  }

  const openResetPasswordDialog = (user: User) => {
    setUserToResetPassword(user)
    setNewPassword('')
    setConfirmNewPassword('')
    setReauthPassword('')
    setShowResetPasswordDialog(true)
  }

  const toggleSortRole = () => {
    setSortRole((prev) => (prev === null ? 'asc' : prev === 'asc' ? 'desc' : null))
  }

  const toggleSortStatus = () => {
    setSortStatus((prev) => (prev === null ? 'asc' : prev === 'asc' ? 'desc' : null))
  }

  const handleImpersonate = async (reauthToken: string) => {
    if (userToImpersonate) await impersonate(userToImpersonate.id, reauthToken)
  }

  const handleExportUsers = () => {
    if (!sortedUsers || sortedUsers.length === 0) {
      toast({ title: '無資料可匯出', description: '目前沒有使用者', variant: 'destructive' })
      return
    }
    const headers = ['Email', '名稱', '電話', '組織', '角色', '職稱', '到職日', '狀態', 'AUP角色', '年資']
    const rows = sortedUsers.map((u) => [
      u.email,
      u.display_name,
      u.phone || '',
      u.organization || '',
      (u.roles || []).join('; '),
      u.position || '',
      u.entry_date || '',
      u.is_active ? '啟用' : '停用',
      (u.aup_roles || []).join('; '),
      String(u.years_experience ?? ''),
    ])
    const csvContent = [headers, ...rows]
      .map((row) => row.map((cell) => `"${String(cell).replace(/"/g, '""')}"`).join(','))
      .join('\n')
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `users_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
    URL.revokeObjectURL(link.href)
    toast({ title: '匯出成功', description: `已匯出 ${sortedUsers.length} 位使用者` })
  }

  return {
    currentUser,
    users: paginatedUsers,
    sortedUsers,
    isLoading,
    roles,
    formData,
    setFormData,
    selectedUser,
    userToDelete,
    userToResetPassword,
    userToImpersonate,
    newPassword,
    setNewPassword,
    confirmNewPassword,
    setConfirmNewPassword,
    reauthPassword,
    setReauthPassword,
    sortRole,
    sortStatus,
    currentPage,
    totalPages,
    perPage,
    setCurrentPage,
    showCreateDialog,
    setShowCreateDialog,
    showEditDialog,
    setShowEditDialog,
    showRolesDialog,
    setShowRolesDialog,
    showDeleteDialog,
    setShowDeleteDialog,
    showReauthForDelete,
    setShowReauthForDelete,
    showResetPasswordDialog,
    setShowResetPasswordDialog,
    showReauthForImpersonate,
    setShowReauthForImpersonate,
    setUserToImpersonate,
    setUserToDelete,
    setUserToResetPassword,
    createMutation,
    updateMutation,
    resetPasswordMutation,
    confirmPasswordMutation,
    deleteUserWithReauth,
    handleCreate,
    handleEdit,
    handleUpdate,
    handleToggleActive,
    handleManageRoles,
    handleUpdateRoles,
    handleResetPassword,
    toggleRole,
    openResetPasswordDialog,
    toggleSortRole,
    toggleSortStatus,
    handleImpersonate,
    handleExportUsers,
  }
}
