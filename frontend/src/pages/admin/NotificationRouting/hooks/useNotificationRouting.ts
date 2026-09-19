import { useState, useMemo, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { deleteResource } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { getApiErrorMessage } from '@/lib/apiError'

import type {
    NotificationRouting,
    EventTypeCategory,
    RoleInfo,
    CreateRoutingData,
    UpdateRoutingData,
    RoutingRecipientsPreview,
    RoutingRuleRecipients,
    FixedNotification,
} from '../types'
import type { GroupKey } from '../constants'
import { recipientLabel } from '../constants'

const QUERY_KEY = ['notification-routing']

export function useNotificationRouting() {
    const { t } = useTranslation()
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

    // 具體收件人預覽（巢狀收合展開用）。key 設為 QUERY_KEY 子鍵，讓 CRUD 後的
    // invalidateQueries(QUERY_KEY) 以前綴比對自動刷新本查詢。
    const { data: recipientsPreview } = useQuery({
        queryKey: ['notification-routing', 'recipients'],
        queryFn: async () => {
            const res = await api.get<RoutingRecipientsPreview[]>(
                '/admin/notification-routing/recipients',
            )
            return res.data
        },
    })

    // 固定通知目錄（流程決定、不可調整）—— 唯讀 notes。靜態資料，少變動。
    const { data: fixedNotifications } = useQuery({
        queryKey: ['notification-routing', 'fixed'],
        queryFn: async () => {
            const res = await api.get<FixedNotification[]>('/admin/notification-routing/fixed')
            return res.data
        },
        staleTime: Infinity,
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
            Equipment: [],
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

    const recipientsByRuleId = useMemo(() => {
        const map: Record<string, RoutingRuleRecipients> = {}
        recipientsPreview?.forEach((ev) => {
            ev.rules.forEach((r) => {
                map[r.rule_id] = r
            })
        })
        return map
    }, [recipientsPreview])

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
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.toast.created') })
        },
        onError: (error: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('adminOps.notificationRouting.toast.createFailed')), variant: 'destructive' })
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
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.toast.updated') })
        },
        onError: (error: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('adminOps.shared.updateFailed')), variant: 'destructive' })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            await deleteResource(`/admin/notification-routing/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: QUERY_KEY })
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.toast.deleted') })
        },
        onError: (error: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('adminOps.shared.deleteFailed')), variant: 'destructive' })
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
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('adminOps.notificationRouting.toast.toggleFailed')), variant: 'destructive' })
        },
    })

    // ---- Handlers ----

    const handleCreate = useCallback(() => {
        if (!createForm.event_type || !createForm.role_code) {
            toast({ title: t('common.error'), description: t('adminOps.notificationRouting.toast.selectEventAndRole'), variant: 'destructive' })
            return
        }
        createMutation.mutate(createForm)
    }, [createForm, createMutation, toast, t])

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
            const recipient = recipientLabel(rule, roleNameMap, t)
            const ok = await confirm({
                title: t('adminOps.notificationRouting.deleteDialog.title'),
                description: t('adminOps.notificationRouting.deleteDialog.description', { event: eventName, recipient }),
                variant: 'destructive',
                confirmLabel: t('common.confirmDelete'),
            })
            if (ok) {
                deleteMutation.mutate(rule.id)
            }
        },
        [eventNameMap, roleNameMap, confirm, deleteMutation, t],
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
        recipientsByRuleId,
        fixedNotifications,
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
