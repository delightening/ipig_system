import { useState, useMemo, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { getApiErrorMessage } from '@/lib/validation'

import type {
    NotificationRouting,
    EventTypeCategory,
    RoleInfo,
    CreateRoutingData,
    UpdateRoutingData,
} from '../types'
import type { GroupKey } from '../constants'

const QUERY_KEY = ['notification-routing']

export function useNotificationRouting() {
    const queryClient = useQueryClient()
    const { toast } = useToast()
    const { dialogState, confirm } = useConfirmDialog()

    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const [showEditDialog, setShowEditDialog] = useState(false)
    const [selectedRule, setSelectedRule] = useState<NotificationRouting | null>(null)
    const [createForm, setCreateForm] = useState<CreateRoutingData>({
        event_type: '',
        role_code: '',
        channel: 'both',
        description: '',
    })
    const [editForm, setEditForm] = useState<UpdateRoutingData>({})

    // ---- Queries ----

    const { data: rules, isLoading } = useQuery({
        queryKey: QUERY_KEY,
        queryFn: async () => {
            const res = await api.get<NotificationRouting[]>('/admin/notification-routing')
            return res.data
        },
    })

    const { data: eventCategories } = useQuery({
        queryKey: ['notification-routing-event-types'],
        queryFn: async () => {
            const res = await api.get<EventTypeCategory[]>('/admin/notification-routing/event-types')
            return res.data
        },
    })

    const { data: roles } = useQuery({
        queryKey: ['notification-routing-roles'],
        queryFn: async () => {
            const res = await api.get<RoleInfo[]>('/admin/notification-routing/roles')
            return res.data
        },
    })

    // ---- Derived Data ----

    const eventNameMap = useMemo(() => {
        const map: Record<string, string> = {}
        eventCategories?.forEach((cat) => {
            cat.event_types.forEach((et) => {
                map[et.code] = et.name
            })
        })
        return map
    }, [eventCategories])

    const eventGroupMap = useMemo(() => {
        const map: Record<string, string> = {}
        eventCategories?.forEach((cat) => {
            cat.event_types.forEach((et) => {
                map[et.code] = cat.group
            })
        })
        return map
    }, [eventCategories])

    const rulesByGroup = useMemo(() => {
        const groups: Record<GroupKey, NotificationRouting[]> = {
            AUP: [],
            Animal: [],
            ERP: [],
            HR: [],
        }
        if (!rules) return groups
        rules.forEach((rule) => {
            const group = eventGroupMap[rule.event_type] as GroupKey | undefined
            if (group && groups[group]) {
                groups[group].push(rule)
            } else {
                groups.AUP.push(rule)
            }
        })
        return groups
    }, [rules, eventGroupMap])

    const roleNameMap = useMemo(() => {
        const map: Record<string, string> = {}
        roles?.forEach((r) => {
            map[r.code] = r.name
        })
        return map
    }, [roles])

    // ---- Mutations ----

    const createMutation = useMutation({
        mutationFn: async (data: CreateRoutingData) => {
            const res = await api.post('/admin/notification-routing', data)
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: QUERY_KEY })
            setShowCreateDialog(false)
            setCreateForm({ event_type: '', role_code: '', channel: 'both', description: '' })
            toast({ title: '成功', description: '通知路由規則已建立' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '建立失敗'), variant: 'destructive' })
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: UpdateRoutingData }) => {
            const res = await api.put(`/admin/notification-routing/${id}`, data)
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: QUERY_KEY })
            setShowEditDialog(false)
            setSelectedRule(null)
            toast({ title: '成功', description: '通知路由規則已更新' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            await deleteResource(`/admin/notification-routing/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: QUERY_KEY })
            toast({ title: '成功', description: '通知路由規則已刪除' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '刪除失敗'), variant: 'destructive' })
        },
    })

    const toggleActiveMutation = useMutation({
        mutationFn: async ({ id, is_active }: { id: string; is_active: boolean }) => {
            const res = await api.put(`/admin/notification-routing/${id}`, { is_active })
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: QUERY_KEY })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '切換失敗'), variant: 'destructive' })
        },
    })

    // ---- Handlers ----

    const handleCreate = useCallback(() => {
        if (!createForm.event_type || !createForm.role_code) {
            toast({ title: '錯誤', description: '請選擇事件類型與角色', variant: 'destructive' })
            return
        }
        createMutation.mutate(createForm)
    }, [createForm, createMutation, toast])

    const handleEdit = useCallback((rule: NotificationRouting) => {
        setSelectedRule(rule)
        setEditForm({
            channel: rule.channel,
            is_active: rule.is_active,
            description: rule.description || '',
            frequency: rule.frequency,
            hour_of_day: rule.hour_of_day,
            day_of_week: rule.day_of_week,
        })
        setShowEditDialog(true)
    }, [])

    const handleUpdate = useCallback(() => {
        if (!selectedRule) return
        updateMutation.mutate({ id: selectedRule.id, data: editForm })
    }, [selectedRule, editForm, updateMutation])

    const handleDelete = useCallback(
        async (rule: NotificationRouting) => {
            const eventName = eventNameMap[rule.event_type] || rule.event_type
            const roleName = roleNameMap[rule.role_code] || rule.role_code
            const ok = await confirm({
                title: '刪除路由規則',
                description: `確定要刪除「${eventName} → ${roleName}」的路由規則嗎？`,
                variant: 'destructive',
                confirmLabel: '確認刪除',
            })
            if (ok) {
                deleteMutation.mutate(rule.id)
            }
        },
        [eventNameMap, roleNameMap, confirm, deleteMutation],
    )

    const handleToggleActive = useCallback(
        (id: string, isActive: boolean) => {
            toggleActiveMutation.mutate({ id, is_active: isActive })
        },
        [toggleActiveMutation],
    )

    return {
        // State
        isLoading,
        rulesByGroup,
        eventNameMap,
        roleNameMap,
        eventCategories,
        roles,
        dialogState,

        // Create dialog
        showCreateDialog,
        setShowCreateDialog,
        createForm,
        setCreateForm,
        handleCreate,
        isCreating: createMutation.isPending,

        // Edit dialog
        showEditDialog,
        setShowEditDialog,
        selectedRule,
        editForm,
        setEditForm,
        handleEdit,
        handleUpdate,
        isUpdating: updateMutation.isPending,

        // Actions
        handleDelete,
        handleToggleActive,
    }
}
