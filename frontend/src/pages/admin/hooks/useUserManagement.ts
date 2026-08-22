import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { isAxiosError, deleteResource, User, Role, ResetPasswordRequest } from '@/lib/api'
import { confirmPassword } from '@/lib/api/client'
import { getErrorMessage, ApiErrorPayload } from '@/types/error'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/components/ui/use-toast'
import { getPasswordError, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'

// === 表單驗證（R58: 改為原生檢查避免 Zod 4 Function 探測觸發 CSP） ===
const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

function validateCreateUserForm(data: { email: string; password: string; display_name: string; role_ids: string[] }): string | null {
  if (!data.email) return '請輸入 Email'
  if (!EMAIL_PATTERN.test(data.email)) return '請輸入有效的 Email'
  if (data.password.length < PASSWORD_MIN_LENGTH) return `密碼至少需要 ${PASSWORD_MIN_LENGTH} 個字元`
  const name = data.display_name.trim()
  if (name.length < 2) return '至少 2 個字元'
  if (name.length > 50) return '最多 50 個字元'
  if (data.role_ids.length < 1) return '請選擇角色'
  return null
}

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
  /** 內部員工旗標。建立時省略即可——後端預設 true（models/user.rs default_is_internal） */
  is_internal?: boolean
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
  /**
   * 內部員工旗標。後端是 `is_internal = COALESCE($11, is_internal)`，
   * 所以**送 undefined 等於不改**——要關掉必須明確送 `false`，不能靠
   * 其他欄位那種 `值 || undefined` 的寫法（那會讓 false 變成 undefined）。
   */
  is_internal?: boolean
  role_ids?: string[]
  entry_date?: string
  position?: string
  aup_roles?: string[]
  years_experience?: number
  trainings?: UserTrainingInput[]
  /** 明知對方仍有未結清事項仍要移除角色（僅系統管理員；後端會把那些待辦釋出） */
  force_role_change?: boolean
}

/** 未結清事項衝突的明細；非此類錯誤回 null（依 warning_type 判定，不比對訊息字串）。 */
function getUnsettledItems(error: unknown): { message: string; items: string[] } | null {
  if (!isAxiosError(error)) return null
  const payload = error.response?.data as ApiErrorPayload | undefined
  if (payload?.error?.warning_type !== 'unsettled_items') return null
  return { message: payload.error.message, items: payload.error.items ?? [] }
}

const defaultFormData: CreateUserData = {
  email: '',
  password: '',
  display_name: '',
  role_ids: [],
  is_internal: true,
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
  const [sortLastLogin, setSortLastLogin] = useState<'asc' | 'desc' | null>(null)
  // 預設不顯示停用帳號（僅列啟用中）；需要時由「顯示停用帳號」開關開啟
  const [showInactive, setShowInactiveState] = useState(false)
  const setShowInactive = (show: boolean) => {
    setShowInactiveState(show)
    setCurrentPage(1)
    if (!show) setSortStatus(null)
  }
  const perPage = 50
  const [currentPage, setCurrentPage] = useState(1)

  const { data: users, isLoading } = useQuery({
    // queryKey 帶入 include_inactive 參數避免與其他預期「只取在職」的組件共用快取；
    // 既有 invalidateQueries(['users']) 以 prefix 比對仍會命中此 key
    queryKey: ['users', { include_inactive: true }],
    queryFn: async () => {
      // include_inactive=true 一併取回停用帳號，交由下方 showInactive 開關前端過濾；
      // 後端預設只回在職帳號，若不帶此參數停用者永遠不會出現，開關形同虛設
      const response = await api.get<User[]>('/users', {
        params: { include_inactive: true },
      })
      return response.data
    },
    staleTime: 60_000,
  })

  const sortedUsers = useMemo(() => {
    if (!users) return []
    const sorted = showInactive ? [...users] : users.filter((u) => u.is_active)
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
    if (sortLastLogin) {
      sorted.sort((a, b) => {
        const at = a.last_login_at ? new Date(a.last_login_at).getTime() : 0
        const bt = b.last_login_at ? new Date(b.last_login_at).getTime() : 0
        return sortLastLogin === 'asc' ? at - bt : bt - at
      })
    }
    return sorted
  }, [users, showInactive, sortRole, sortStatus, sortLastLogin])

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
      if (isAxiosError(error)) {
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

  // 移除角色被未結清事項擋下時的明細；非 null 即開啟強制移除確認對話框。
  const [unsettledConflict, setUnsettledConflict] = useState<{
    message: string
    items: string[]
  } | null>(null)

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateUserData }) => {
      const response = await api.put(`/users/${id}`, data)
      return response.data
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['users'] })
      // 停用使用者時，後端會刪除其 refresh_token，需刷新 session 相關查詢
      if (variables.data.is_active === false) {
        queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
        queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
      }
      setShowEditDialog(false)
      setShowRolesDialog(false)
      setSelectedUser(null)
      toast({ title: '成功', description: '用戶已更新' })
    },
    onError: (error: unknown) => {
      // 未結清事項衝突改由 handleUpdateRoles 用專屬對話框列出明細，不另外跳 toast
      if (getUnsettledItems(error)) return
      toast({ title: '錯誤', description: getErrorMessage(error) || '更新失敗', variant: 'destructive' })
    },
  })

  const deleteUserWithReauth = async (id: string, reauthToken: string) => {
    await deleteResource(`/users/${id}`, { headers: { 'X-Reauth-Token': reauthToken } })
    queryClient.invalidateQueries({ queryKey: ['users'] })
    // 刪除使用者後，其 session 已失效，需刷新 session 相關查詢
    queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
    queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
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
    const validationError = validateCreateUserForm(formData)
    if (validationError) {
      toast({ title: '錯誤', description: validationError, variant: 'destructive' })
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

  /** RHF-validated create: Zod 已驗證，僅需檢查密碼複雜度 */
  const handleCreateWithData = (data: CreateUserData) => {
    const pwError = getPasswordError(data.password)
    if (pwError) {
      toast({ title: '錯誤', description: pwError, variant: 'destructive' })
      return
    }
    createMutation.mutate({
      ...data,
      entry_date: data.entry_date || undefined,
      position: data.position || undefined,
    })
  }

  const handleEdit = (user: User) => {
    setSelectedUser(user)
    setFormData({
      email: user.email,
      password: '',
      display_name: user.display_name,
      role_ids: [],
      // 後端 UserResponse 一定會給這個欄位；型別上的 optional 只是 TS 寬鬆。
      // 讀不到時以 true 為準，避免把既有內部員工靜默降級成外部人員。
      is_internal: user.is_internal ?? true,
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

  /** RHF-validated update: dialog 已透過 Zod 驗證，直接提交 */
  const handleUpdateWithData = (data: { email: string; display_name: string; is_internal: boolean; entry_date: string; aup_roles: string[]; trainings: UserTrainingInput[] }) => {
    if (!selectedUser) return
    updateMutation.mutate({
      id: selectedUser.id,
      data: {
        email: data.email || undefined,
        display_name: data.display_name || undefined,
        // 直接送布林值，不能寫成 `data.is_internal || undefined`——
        // 那會讓「關掉」變成 undefined，而後端是 COALESCE，等於永遠關不掉。
        is_internal: data.is_internal,
        entry_date: data.entry_date || undefined,
        aup_roles: data.aup_roles,
        trainings: data.trainings,
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

  const handleUpdateRoles = async () => {
    if (!selectedUser) return
    try {
      await updateMutation.mutateAsync({
        id: selectedUser.id,
        data: { role_ids: formData.role_ids },
      })
    } catch (error) {
      // 對方身上還有卡住的單據 → 列出明細讓管理員決定是否強制移除。
      // 其他錯誤已由 updateMutation.onError 統一 toast，這裡不重複處理。
      const conflict = getUnsettledItems(error)
      if (conflict) setUnsettledConflict(conflict)
    }
  }

  /** 強制移除角色：後端會把待他確認的代理假單退回草稿、待他簽核的單交還該關，並記稽核。 */
  const handleForceUpdateRoles = () => {
    if (!selectedUser) return
    setUnsettledConflict(null)
    updateMutation.mutate({
      id: selectedUser.id,
      data: { role_ids: formData.role_ids, force_role_change: true },
    })
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

  /** RHF-validated reset password: Zod 已驗證基本格式，僅需密碼複雜度檢查 */
  const handleResetPasswordWithData = (data: { reauth_password: string; new_password: string; confirm_password: string }) => {
    if (!userToResetPassword) return
    const resetPwError = getPasswordError(data.new_password)
    if (resetPwError) {
      toast({ title: '錯誤', description: resetPwError, variant: 'destructive' })
      return
    }
    setNewPassword(data.new_password)
    confirmPasswordMutation.mutate(data.reauth_password)
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

  const toggleSortLastLogin = () => {
    setSortLastLogin((prev) => (prev === null ? 'asc' : prev === 'asc' ? 'desc' : null))
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
    sortLastLogin,
    showInactive,
    setShowInactive,
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
    unsettledConflict,
    setUnsettledConflict,
    handleForceUpdateRoles,
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
    handleCreateWithData,
    handleEdit,
    handleUpdateWithData,
    handleToggleActive,
    handleManageRoles,
    handleUpdateRoles,
    handleResetPassword,
    handleResetPasswordWithData,
    toggleRole,
    openResetPasswordDialog,
    toggleSortRole,
    toggleSortStatus,
    toggleSortLastLogin,
    handleImpersonate,
    handleExportUsers,
  }
}
