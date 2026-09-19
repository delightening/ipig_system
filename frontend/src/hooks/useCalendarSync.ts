/**
 * 日曆同步相關的自訂 Hook
 * 封裝所有 React Query 與 mutation，供 CalendarSyncSettingsPage 子元件使用
 */
import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { CalendarSyncHistory, CalendarSyncStatus, ConflictWithDetails, CalendarConfig, UpdateCalendarConfig } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

/**
 * 日曆同步狀態、連接、手動同步、歷史、衝突等操作
 * @param activeTab 當前啟用的分頁，用於條件查詢
 */
export function useCalendarSync(activeTab: string) {
    const { t } = useTranslation()
    const [showConnectDialog, setShowConnectDialog] = useState(false)
    const [calendarId, setCalendarId] = useState('')
    const [authEmail, setAuthEmail] = useState('')
    const [historyPage, setHistoryPage] = useState(1)
    const [conflictsPage, setConflictsPage] = useState(1)
    const queryClient = useQueryClient()
    const { user, isAdmin: isAdminFn, hasPermission } = useAuthStore()
    // 用 store 的 isAdmin()（SYSTEM_ADMIN || legacy admin，比照後端 is_admin()），
    // 不用 hasRole('admin')——後者是精確字串比對，會把 SYSTEM_ADMIN 排除在外，
    // 導致真正的管理員看不到同步相關分頁。
    const isAdmin = isAdminFn()
    const canViewCalendarConfig = hasPermission('hr.calendar.config')
    const canViewCalendar = hasPermission('hr.calendar.view')
    // 原生請假行事曆用獨立權限碼：它揭露的是「誰哪天不在、誰代理」，
    // 與 Google 事件檢視（hr.calendar.view）是不同的資料來源與範圍。
    const canViewLeaveCalendar = isAdmin || hasPermission(PERMISSIONS.HR_LEAVE_VIEW_CALENDAR)

    // 當打開連接對話框時，預設授權 Email 為當前用戶的 Email
    useEffect(() => {
        if (showConnectDialog && user?.email) {
            setAuthEmail(user.email)
        }
    }, [showConnectDialog, user?.email])

    // 切換分頁時重置分頁數
    useEffect(() => {
        setHistoryPage(1)
        setConflictsPage(1)
    }, [activeTab])

    // 同步設定（僅在「連線狀態」分頁且已連接時查詢）
    const { data: calendarConfig, isLoading: loadingConfig } = useQuery({
        queryKey: ['calendar-config'],
        queryFn: async () => {
            const res = await api.get<CalendarConfig>('/hr/calendar/config')
            return res.data
        },
        enabled: activeTab === 'status' && isAdmin,
    })

    // 更新同步設定
    const updateConfigMutation = useMutation({
        mutationFn: async (data: UpdateCalendarConfig) => {
            return api.put('/hr/calendar/config', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['calendar-config'] })
            queryClient.invalidateQueries({ queryKey: ['calendar-status'] })
            toast({ title: t('common.success'), description: t('calendarSync.configSaved') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('calendarSync.saveFailed'),
                description: getApiErrorMessage(error, t('errors.tryAgainLater')),
                variant: 'destructive',
            })
        },
    })

    // 同步狀態（僅限有 hr.calendar.config 權限的使用者）
    const { data: syncStatus, isLoading: loadingStatus } = useQuery({
        queryKey: ['calendar-status'],
        queryFn: async () => {
            const res = await api.get<CalendarSyncStatus>('/hr/calendar/status')
            return res.data
        },
        enabled: canViewCalendarConfig,
    })

    // 同步歷史（支援分頁）
    const { data: syncHistory, isLoading: loadingHistory } = useQuery({
        queryKey: ['calendar-history', historyPage],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<CalendarSyncHistory>>(
                `/hr/calendar/history?page=${historyPage}&per_page=20`
            )
            return res.data
        },
        enabled: activeTab === 'history',
    })

    // 衝突列表（支援分頁）
    const { data: conflicts, isLoading: loadingConflicts } = useQuery({
        queryKey: ['calendar-conflicts', conflictsPage],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<ConflictWithDetails>>(
                `/hr/calendar/conflicts?status=pending&page=${conflictsPage}&per_page=20`
            )
            return res.data
        },
        enabled: activeTab === 'conflicts',
    })

    // 連接日曆
    const connectMutation = useMutation({
        mutationFn: async (data: { calendar_id: string; auth_email: string }) => {
            return api.post('/hr/calendar/connect', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['calendar-status'], refetchType: 'all' })
            setShowConnectDialog(false)
            setCalendarId('')
            setAuthEmail('')
            toast({ title: t('common.success'), description: t('calendarSync.connected') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('calendarSync.connectFailed'),
                description: getApiErrorMessage(error, t('calendarSync.checkSettings')),
                variant: 'destructive',
            })
        },
    })

    // 斷開連接
    const disconnectMutation = useMutation({
        mutationFn: async () => {
            return api.post('/hr/calendar/disconnect')
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['calendar-status'] })
            toast({ title: t('common.success'), description: t('calendarSync.disconnected') })
        },
    })

    // 手動觸發同步
    const syncMutation = useMutation({
        mutationFn: async () => {
            return api.post('/hr/calendar/sync')
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['calendar-status'] })
            queryClient.invalidateQueries({ queryKey: ['calendar-history'] })
            toast({ title: t('common.success'), description: t('calendarSync.syncStarted') })
        },
    })

    // 解決衝突（樂觀更新：點擊後立即從列表移除，失敗時回滾）
    const resolveConflictMutation = useMutation({
        mutationFn: async ({ id, resolution }: { id: string; resolution: string }) => {
            return api.post(`/hr/calendar/conflicts/${id}/resolve`, { resolution })
        },
        onMutate: async ({ id }) => {
            // 取消正在進行的查詢，避免覆蓋樂觀更新
            await queryClient.cancelQueries({ queryKey: ['calendar-conflicts', conflictsPage] })
            // 保存當前快照以備回滾
            const previous = queryClient.getQueryData<PaginatedResponse<ConflictWithDetails>>(['calendar-conflicts', conflictsPage])
            // 樂觀更新：移除已解決的衝突
            queryClient.setQueryData<PaginatedResponse<ConflictWithDetails>>(['calendar-conflicts', conflictsPage], old => {
                if (!old) return old
                return {
                    ...old,
                    data: old.data.filter(c => c.id !== id),
                    total: Math.max(0, old.total - 1),
                }
            })
            return { previous }
        },
        onError: (error: unknown, _vars, context) => {
            // 失敗時回滾到之前的資料
            if (context?.previous) {
                queryClient.setQueryData(['calendar-conflicts', conflictsPage], context.previous)
            }
            toast({
                title: t('calendarSync.resolveFailed'),
                description: getApiErrorMessage(error, t('errors.tryAgainLater')),
                variant: 'destructive',
            })
        },
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('calendarSync.conflictResolved') })
        },
        onSettled: () => {
            // 無論成功/失敗，最終都重新查詢確保資料一致
            queryClient.invalidateQueries({ queryKey: ['calendar-conflicts'] })
            queryClient.invalidateQueries({ queryKey: ['calendar-status'] })
        },
    })

    return {
        // 狀態
        syncStatus,
        loadingStatus,
        syncHistory,
        loadingHistory,
        conflicts,
        loadingConflicts,
        isAdmin,
        canViewCalendarConfig,
        canViewCalendar,
        canViewLeaveCalendar,

        // 分頁
        historyPage,
        setHistoryPage,
        conflictsPage,
        setConflictsPage,

        // 連接對話框
        showConnectDialog,
        setShowConnectDialog,
        calendarId,
        setCalendarId,
        authEmail,
        setAuthEmail,

        // 同步設定
        calendarConfig,
        loadingConfig,

        // Mutations
        connectMutation,
        disconnectMutation,
        syncMutation,
        resolveConflictMutation,
        updateConfigMutation,
    }
}
