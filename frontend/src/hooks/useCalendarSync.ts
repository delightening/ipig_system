/**
 * 日曆同步相關的自訂 Hook
 * 封裝所有 React Query 與 mutation，供 CalendarSyncSettingsPage 子元件使用
 */
import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { CalendarSyncHistory, CalendarSyncStatus, ConflictWithDetails, CalendarConfig, UpdateCalendarConfig } from '@/types/hr'

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

/**
 * 日曆同步狀態、連接、手動同步、歷史、衝突等操作
 * @param activeTab 當前啟用的分頁，用於條件查詢
 */
export function useCalendarSync(activeTab: string) {
    const [showConnectDialog, setShowConnectDialog] = useState(false)
    const [calendarId, setCalendarId] = useState('')
    const [authEmail, setAuthEmail] = useState('')
    const [historyPage, setHistoryPage] = useState(1)
    const [conflictsPage, setConflictsPage] = useState(1)
    const queryClient = useQueryClient()
    const { user, hasRole } = useAuthStore()
    const isAdmin = hasRole('admin')

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
            toast({ title: '成功', description: '同步設定已儲存' })
        },
        onError: (error: unknown) => {
            toast({
                title: '儲存失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    // 同步狀態
    const { data: syncStatus, isLoading: loadingStatus } = useQuery({
        queryKey: ['calendar-status'],
        queryFn: async () => {
            const res = await api.get<CalendarSyncStatus>('/hr/calendar/status')
            return res.data
        },
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
            toast({ title: '成功', description: '已連接 Google Calendar' })
        },
        onError: (error: unknown) => {
            toast({
                title: '連接失敗',
                description: getApiErrorMessage(error, '請檢查設定'),
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
            toast({ title: '成功', description: '已斷開 Google Calendar 連接' })
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
            toast({ title: '成功', description: '同步已開始' })
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
                title: '解決失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
        onSuccess: () => {
            toast({ title: '成功', description: '衝突已解決' })
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
