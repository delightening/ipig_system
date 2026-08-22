import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, {
  confirmPassword,
  deleteResource,
  getSystemFeatures,
  Role,
  Permission,
  type MutationSignaturePayload,
} from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { useToast } from '@/components/ui/use-toast'

export interface CreateRoleData {
  code: string
  name: string
  permission_ids: string[]
}

const defaultFormData: CreateRoleData = { code: '', name: '', permission_ids: [] }

export function useRolesMutations() {
  const queryClient = useQueryClient()
  const { toast } = useToast()

  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [showDetailDialog, setShowDetailDialog] = useState(false)
  const [roleForDetail, setRoleForDetail] = useState<Role | null>(null)
  const [showReauthForDeleteRole, setShowReauthForDeleteRole] = useState(false)
  const [roleToDelete, setRoleToDelete] = useState<Role | null>(null)
  const [selectedRole, setSelectedRole] = useState<Role | null>(null)
  const [formData, setFormData] = useState<CreateRoleData>(defaultFormData)

  // R30-27b：簽章 dialog 狀態 — 區分 create / update / delete 階段
  const [signaturePrompt, setSignaturePrompt] = useState<
    | null
    | { mode: 'create' }
    | { mode: 'update'; roleId: string }
    | { mode: 'delete'; role: Role; reauthToken: string }
  >(null)

  const { data: roles, isLoading } = useQuery({
    queryKey: ['roles'],
    queryFn: async () => (await api.get<Role[]>('/roles')).data,
  })

  const { data: permissions } = useQuery({
    queryKey: ['permissions'],
    queryFn: async () => (await api.get<Permission[]>('/permissions')).data,
  })

  // R30-27b：feature flag — 決定是否要簽章（cache 5 min，避免每次 hover 都打）
  const { data: features } = useQuery({
    queryKey: ['system', 'features'],
    queryFn: getSystemFeatures,
    staleTime: 5 * 60 * 1000,
  })
  const requireSignature = features?.role_signature_required ?? false

  const createMutation = useMutation({
    mutationFn: async (
      data: CreateRoleData & { mutation_signature?: MutationSignaturePayload },
    ) => (await api.post('/roles', data)).data,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
      setShowCreateDialog(false)
      setFormData(defaultFormData)
      toast({ title: '成功', description: '角色已創建' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '創建失敗',
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async ({
      id,
      data,
    }: {
      id: string
      data: Partial<CreateRoleData> & { mutation_signature?: MutationSignaturePayload }
    }) => (await api.put(`/roles/${id}`, data)).data,
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
        variant: 'destructive',
      })
    },
  })

  // R30-27b：與 create/update 對齊，delete 也走 useMutation 統一錯誤 / cache 處理
  const deleteMutation = useMutation({
    mutationFn: async (input: {
      id: string
      reauthToken: string
      is_system: boolean
      signature?: MutationSignaturePayload
    }) => {
      const body = input.signature ? { mutation_signature: input.signature } : undefined
      await deleteResource(`/roles/${input.id}`, {
        headers: { 'X-Reauth-Token': input.reauthToken },
        data: body,
      })
      return { is_system: input.is_system }
    },
    onSuccess: ({ is_system }) => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
      toast({
        title: '成功',
        description: is_system ? '系統角色已停用' : '角色已刪除',
      })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const handleCreate = () => {
    if (!formData.code || !formData.name) {
      toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
      return
    }
    if (requireSignature) {
      // 開簽章 dialog；payload 在 dialog onSubmit 取得後再呼叫 createMutation
      setSignaturePrompt({ mode: 'create' })
      return
    }
    createMutation.mutate(formData)
  }

  const handleEdit = (role: Role) => {
    setSelectedRole(role)
    setFormData({
      code: role.code,
      name: role.name,
      permission_ids: role.permissions.map((p) => p.id),
    })
    setShowEditDialog(true)
  }

  const handleUpdate = () => {
    if (!selectedRole) return
    if (requireSignature) {
      setSignaturePrompt({ mode: 'update', roleId: selectedRole.id })
      return
    }
    updateMutation.mutate({
      id: selectedRole.id,
      data: { name: formData.name, permission_ids: formData.permission_ids },
    })
  }

  const togglePermission = (permId: string) => {
    setFormData((prev) => ({
      ...prev,
      permission_ids: prev.permission_ids.includes(permId)
        ? prev.permission_ids.filter((id) => id !== permId)
        : [...prev.permission_ids, permId],
    }))
  }

  const handleDeleteClick = (role: Role) => {
    setRoleToDelete(role)
    setShowReauthForDeleteRole(true)
  }

  const handleDeleteConfirm = async (password: string) => {
    const { reauth_token } = await confirmPassword(password)
    if (!roleToDelete) return
    if (requireSignature) {
      // reauth 完成 → 開簽章 dialog；payload 取得後才真正 delete
      setSignaturePrompt({ mode: 'delete', role: roleToDelete, reauthToken: reauth_token })
      return
    }
    await deleteMutation.mutateAsync({
      id: roleToDelete.id,
      reauthToken: reauth_token,
      is_system: roleToDelete.is_system,
    })
    setRoleToDelete(null)
  }

  const handleSignatureSubmit = async (payload: MutationSignaturePayload) => {
    if (!signaturePrompt) return
    // 失敗時由 mutation onError toast；error 自然向上拋讓 RoleSignatureDialog
    // 維持開啟並顯示錯誤訊息（onSubmit 在 dialog 內以 try/catch 接住）。
    if (signaturePrompt.mode === 'create') {
      await createMutation.mutateAsync({ ...formData, mutation_signature: payload })
    } else if (signaturePrompt.mode === 'update') {
      await updateMutation.mutateAsync({
        id: signaturePrompt.roleId,
        data: {
          name: formData.name,
          permission_ids: formData.permission_ids,
          mutation_signature: payload,
        },
      })
    } else if (signaturePrompt.mode === 'delete') {
      await deleteMutation.mutateAsync({
        id: signaturePrompt.role.id,
        reauthToken: signaturePrompt.reauthToken,
        is_system: signaturePrompt.role.is_system,
        signature: payload,
      })
      setRoleToDelete(null)
    }
    setSignaturePrompt(null)
  }

  const handleViewDetail = (role: Role) => {
    setRoleForDetail(role)
    setShowDetailDialog(true)
  }

  return {
    roles,
    isLoading,
    permissions,
    formData,
    setFormData,
    selectedRole,
    roleForDetail,
    roleToDelete,
    showCreateDialog,
    setShowCreateDialog,
    showEditDialog,
    setShowEditDialog,
    showDetailDialog,
    setShowDetailDialog,
    showReauthForDeleteRole,
    setShowReauthForDeleteRole,
    setRoleForDetail,
    setRoleToDelete,
    createMutation,
    updateMutation,
    deleteMutation,
    handleCreate,
    handleEdit,
    handleUpdate,
    handleDeleteClick,
    handleDeleteConfirm,
    handleViewDetail,
    togglePermission,
    // R30-27b：簽章 dialog 控制
    signaturePrompt,
    setSignaturePrompt,
    handleSignatureSubmit,
    requireSignature,
  }
}
