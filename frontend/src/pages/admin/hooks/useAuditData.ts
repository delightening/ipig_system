import { useState, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { toast } from '@/components/ui/use-toast'
import type {
    AuditDashboardStats,
    LoginEventWithUser,
    SecurityAlert,
    SessionWithUser,
} from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import type { AuditLog, AlertSortConfig, AlertSortField } from '../types/audit'

type AuditTab = 'dashboard' | 'activities' | 'logins' | 'sessions' | 'alerts'

// 嚴重程度排序優先級（數值越小越嚴重）
const severityOrder: Record<string, number> = {
    critical: 0,
    warning: 1,
    info: 2,
}

// 狀態排序優先級（數值越小越優先）
const statusOrder: Record<string, number> = {
    open: 0,
    acknowledged: 1,
    investigating: 2,
    resolved: 3,
}

const PER_PAGE = 50

function getDefaultDateFrom() {
    const now = new Date()
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`
}

function getDefaultDateTo() {
    const now = new Date()
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}

export function useAuditData() {
    const { user: currentUser, logout } = useAuthStore()
    const queryClient = useQueryClient()
    const [searchParams] = useSearchParams()
    const activeTab = (searchParams.get('tab') ?? 'dashboard') as AuditTab

    // 分頁狀態
    const [activitiesPage, setActivitiesPage] = useState(1)
    const [loginsPage, setLoginsPage] = useState(1)
    const [sessionsPage, setSessionsPage] = useState(1)
    const [alertsPage, setAlertsPage] = useState(1)

    // 安全警報排序狀態
    const [alertSortConfig, setAlertSortConfig] = useState<AlertSortConfig>({
        field: 'created_at',
        order: 'desc',
    })

    const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter({
        initialFrom: getDefaultDateFrom,
        initialTo: getDefaultDateTo,
    })

    // 登入事件篩選狀態
    const [loginEventType, setLoginEventType] = useState<string>('all')

    const handleLoginEventTypeChange = (val: string) => {
        setLoginEventType(val)
        setLoginsPage(1)
    }

    // 安全警報篩選狀態
    const [alertSearch, setAlertSearch] = useState('')
    const [alertStatusFilter, setAlertStatusFilter] = useState<string>('all')

    const handleAlertSearchChange = (val: string) => {
        setAlertSearch(val)
        setAlertsPage(1)
    }

    const handleAlertStatusFilterChange = (val: string) => {
        setAlertStatusFilter(val)
        setAlertsPage(1)
    }

    const handleDateFromChange = (val: string) => {
        setDateFrom(val)
        setActivitiesPage(1)
        setLoginsPage(1)
    }

    const handleDateToChange = (val: string) => {
        setDateTo(val)
        setActivitiesPage(1)
        setLoginsPage(1)
    }

    // Dashboard Stats
    const { data: dashboardStats } = useQuery({
        queryKey: ['audit-dashboard'],
        queryFn: async () => {
            const res = await api.get<AuditDashboardStats>('/admin/audit/dashboard')
            return res.data
        },
    })

    // Activity Logs
    const { data: activityLogs, isLoading: loadingActivities } = useQuery({
        queryKey: ['audit-user-activities', dateFrom, dateTo, activitiesPage],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('start_date', dateFrom)
            if (dateTo) params.set('end_date', dateTo)
            params.set('entity_type', 'user')
            params.set('page', String(activitiesPage))
            params.set('perPage', String(PER_PAGE))
            const res = await api.get<PaginatedResponse<AuditLog>>(`/audit-logs?${params}`)
            return res.data
        },
        enabled: activeTab === 'activities',
    })

    // Login Events
    const { data: loginEvents, isLoading: loadingLogins } = useQuery({
        queryKey: ['audit-logins', dateFrom, dateTo, loginsPage, loginEventType],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            if (loginEventType !== 'all') params.set('event_type', loginEventType)
            params.set('page', String(loginsPage))
            params.set('per_page', String(PER_PAGE))
            const res = await api.get<PaginatedResponse<LoginEventWithUser>>(`/admin/audit/logins?${params}`)
            return res.data
        },
        enabled: activeTab === 'logins',
    })

    // Sessions
    const { data: sessions, isLoading: loadingSessions } = useQuery({
        queryKey: ['audit-sessions', sessionsPage],
        queryFn: async () => {
            const params = new URLSearchParams()
            params.set('page', String(sessionsPage))
            params.set('per_page', String(PER_PAGE))
            const res = await api.get<PaginatedResponse<SessionWithUser>>(`/admin/audit/sessions?${params}`)
            return res.data
        },
        enabled: activeTab === 'sessions',
    })

    // Security Alerts
    const { data: alerts, isLoading: loadingAlerts } = useQuery({
        queryKey: ['audit-alerts', alertsPage, alertSearch, alertStatusFilter],
        queryFn: async () => {
            const params = new URLSearchParams()
            params.set('page', String(alertsPage))
            params.set('per_page', String(PER_PAGE))
            if (alertSearch) params.set('query', alertSearch)
            if (alertStatusFilter !== 'all') {
                params.set('status', alertStatusFilter)
            }
            const res = await api.get<PaginatedResponse<SecurityAlert>>(`/admin/audit/alerts?${params}`)
            return res.data
        },
        enabled: activeTab === 'alerts',
    })

    // Force Logout Mutation
    const forceLogoutMutation = useMutation({
        mutationFn: async (sessionId: string) => {
            return api.post(`/admin/audit/sessions/${sessionId}/logout`, {
                reason: '管理員強制登出',
            })
        },
        onSuccess: (_data, sessionId) => {
            const loggedOutSession = sessions?.data?.find(s => s.id === sessionId)
            if (loggedOutSession && currentUser && loggedOutSession.user_id === currentUser.id) {
                toast({ title: '已登出', description: '您的 Session 已被強制登出，即將返回登入頁面' })
                logout()
                return
            }
            queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
            toast({ title: '成功', description: '已強制登出該 Session' })
        },
    })

    // Resolve Alert Mutation
    const resolveAlertMutation = useMutation({
        mutationFn: async (alertId: string) => {
            return api.post(`/admin/audit/alerts/${alertId}/resolve`, {
                resolution: 'resolved',
                resolution_notes: '已確認並解決',
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['audit-alerts'] })
            queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
            toast({ title: '成功', description: '已解決警報' })
        },
    })

    // 安全警報排序邏輯
    const sortedAlerts = useMemo(() => {
        if (!alerts?.data) return []
        const sorted = [...alerts.data]
        sorted.sort((a, b) => {
            let valA: number | string
            let valB: number | string

            if (alertSortConfig.field === 'severity') {
                valA = severityOrder[a.severity] ?? 99
                valB = severityOrder[b.severity] ?? 99
            } else if (alertSortConfig.field === 'status') {
                valA = statusOrder[a.status] ?? 99
                valB = statusOrder[b.status] ?? 99
            } else {
                valA = a[alertSortConfig.field] || ''
                valB = b[alertSortConfig.field] || ''
            }

            if (valA < valB) return alertSortConfig.order === 'asc' ? -1 : 1
            if (valA > valB) return alertSortConfig.order === 'asc' ? 1 : -1
            return 0
        })
        return sorted
    }, [alerts?.data, alertSortConfig])

    const handleAlertSort = (field: AlertSortField) => {
        setAlertSortConfig(prev => ({
            field,
            order: prev.field === field && prev.order === 'asc' ? 'desc' : 'asc',
        }))
    }

    const refreshAll = () => {
        queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
        queryClient.invalidateQueries({ queryKey: ['audit-activities'] })
        queryClient.invalidateQueries({ queryKey: ['audit-logins'] })
        queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
        queryClient.invalidateQueries({ queryKey: ['audit-alerts'] })
    }

    return {
        // Tab (read-only, synced via URL ?tab=)
        activeTab,

        // Date
        dateFrom,
        dateTo,
        handleDateFromChange,
        handleDateToChange,

        // Dashboard
        dashboardStats,

        // Activities
        activityLogs,
        loadingActivities,
        activitiesPage,
        setActivitiesPage,

        // Logins
        loginEvents,
        loadingLogins,
        loginsPage,
        setLoginsPage,
        loginEventType,
        handleLoginEventTypeChange,

        // Sessions
        sessions,
        loadingSessions,
        sessionsPage,
        setSessionsPage,
        forceLogoutMutation,

        // Alerts
        sortedAlerts,
        loadingAlerts,
        alertsPage,
        setAlertsPage,
        alerts,
        alertSortConfig,
        handleAlertSort,
        resolveAlertMutation,
        alertSearch,
        handleAlertSearchChange,
        alertStatusFilter,
        handleAlertStatusFilterChange,

        // Actions
        refreshAll,
    }
}
