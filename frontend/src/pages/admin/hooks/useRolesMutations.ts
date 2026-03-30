import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, { confirmPassword, deleteResource, Role, Permission } from '@/lib/api'
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

    const { data: roles, isLoading } = useQuery({
        queryKey: ['roles'],
        queryFn: async () => (await api.get<Role[]>('/roles')).data,
    })

    const { data: permissions } = useQuery({
        queryKey: ['permissions'],
        queryFn: async () => (await api.get<Permission[]>('/permissions')).data,
    })

    const createMutation = useMutation({
        mutationFn: async (data: CreateRoleData) => (await api.post('/roles', data)).data,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['roles'] })
            setShowCreateDialog(false)
            setFormData(defaultFormData)
            toast({ title: '成功', description: '角色已創建' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getErrorMessage(error) || '創建失敗', variant: 'destructive' })
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: Partial<CreateRoleData> }) => (await api.put(`/roles/${id}`, data)).data,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['roles'] })
            setShowEditDialog(false)
            setSelectedRole(null)
            toast({ title: '成功', description: '角色已更新' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getErrorMessage(error) || '更新失敗', variant: 'destructive' })
        },
    })

    const deleteRoleWithReauth = async (id: string, reauthToken: string, is_system: boolean) => {
        await deleteResource(`/roles/${id}`, { headers: { 'X-Reauth-Token': reauthToken } })
        queryClient.invalidateQueries({ queryKey: ['roles'] })
        toast({ title: '成功', description: is_system ? '系統角色已停用' : '角色已刪除' })
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
        setFormData({ code: role.code, name: role.name, permission_ids: role.permissions.map(p => p.id) })
        setShowEditDialog(true)
    }

    const handleUpdate = () => {
        if (!selectedRole) return
        updateMutation.mutate({ id: selectedRole.id, data: { name: formData.name, permission_ids: formData.permission_ids } })
    }

    const togglePermission = (permId: string) => {
        setFormData(prev => ({
            ...prev,
            permission_ids: prev.permission_ids.includes(permId)
                ? prev.permission_ids.filter(id => id !== permId)
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
        await deleteRoleWithReauth(roleToDelete.id, reauth_token, roleToDelete.is_system)
        setRoleToDelete(null)
    }

    const handleViewDetail = (role: Role) => {
        setRoleForDetail(role)
        setShowDetailDialog(true)
    }

    return {
        roles, isLoading, permissions, formData, setFormData,
        selectedRole, roleForDetail, roleToDelete,
        showCreateDialog, setShowCreateDialog,
        showEditDialog, setShowEditDialog,
        showDetailDialog, setShowDetailDialog,
        showReauthForDeleteRole, setShowReauthForDeleteRole,
        setRoleForDetail, setRoleToDelete,
        createMutation, updateMutation,
        handleCreate, handleEdit, handleUpdate,
        handleDeleteClick, handleDeleteConfirm,
        handleViewDetail, togglePermission,
    }
}
