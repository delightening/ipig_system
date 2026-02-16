import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    Activity,
    AlertTriangle,
    ArrowDown,
    ArrowUp,
    ArrowUpDown,
    ChevronLeft,
    ChevronRight,
    Clock,
    Eye,
    LogIn,
    LogOut,
    RefreshCw,
    Search,
    Shield,
    Users,
} from 'lucide-react'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'

import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import type {
    AuditDashboardStats,
    LoginEventWithUser,
    SecurityAlert,
    SessionWithUser,
} from '@/types/hr'

interface AuditLog {
    id: string
    actor_user_id: string
    actor_email: string
    actor_name: string
    action: string
    entity_type: string
    entity_id: string
    entity_email?: string
    entity_name?: string
    before_data?: Record<string, unknown>
    after_data?: Record<string, unknown>
    created_at: string
}

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

type AlertSortField = 'created_at' | 'alert_type' | 'severity' | 'status'
type AlertSortOrder = 'asc' | 'desc'

interface AlertSortConfig {
    field: AlertSortField
    order: AlertSortOrder
}

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

export function AdminAuditPage() {
    const { user: currentUser, logout } = useAuthStore()
    const [activeTab, setActiveTab] = useState('dashboard')
    const [searchTerm, setSearchTerm] = useState('')
    const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null)
    const [selectedAlert, setSelectedAlert] = useState<SecurityAlert | null>(null)

    // 分頁狀態
    const perPage = 50
    const [activitiesPage, setActivitiesPage] = useState(1)
    const [loginsPage, setLoginsPage] = useState(1)
    const [sessionsPage, setSessionsPage] = useState(1)
    const [alertsPage, setAlertsPage] = useState(1)

    // 安全警報排序狀態
    const [alertSortConfig, setAlertSortConfig] = useState<AlertSortConfig>({
        field: 'created_at',
        order: 'desc',
    })

    // Default date range: first day of current month to today
    const getDefaultDateFrom = () => {
        const now = new Date()
        return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`
    }
    const getDefaultDateTo = () => {
        const now = new Date()
        return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
    }

    const [dateFrom, setDateFrom] = useState(getDefaultDateFrom)
    const [dateTo, setDateTo] = useState(getDefaultDateTo)

    // 日期變更時重設分頁
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
    const queryClient = useQueryClient()

    // Dashboard Stats
    const { data: dashboardStats } = useQuery({
        queryKey: ['audit-dashboard'],
        queryFn: async () => {
            const res = await api.get<AuditDashboardStats>('/admin/audit/dashboard')
            return res.data
        },
    })

    // Activity Logs (audit_logs - 使用者管理操作)
    const { data: activityLogs, isLoading: loadingActivities } = useQuery({
        queryKey: ['audit-user-activities', dateFrom, dateTo, activitiesPage],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('start_date', dateFrom)
            if (dateTo) params.set('end_date', dateTo)
            params.set('entity_type', 'user')
            params.set('page', String(activitiesPage))
            params.set('perPage', String(perPage))
            const res = await api.get<PaginatedResponse<AuditLog>>(
                `/audit-logs?${params}`
            )
            return res.data
        },
        enabled: activeTab === 'activities',
    })

    // Login Events
    const { data: loginEvents, isLoading: loadingLogins } = useQuery({
        queryKey: ['audit-logins', dateFrom, dateTo, loginsPage],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            params.set('page', String(loginsPage))
            params.set('per_page', String(perPage))
            const res = await api.get<PaginatedResponse<LoginEventWithUser>>(
                `/admin/audit/logins?${params}`
            )
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
            params.set('per_page', String(perPage))
            const res = await api.get<PaginatedResponse<SessionWithUser>>(`/admin/audit/sessions?${params}`)
            return res.data
        },
        enabled: activeTab === 'sessions',
    })

    // Security Alerts
    const { data: alerts, isLoading: loadingAlerts } = useQuery({
        queryKey: ['audit-alerts', alertsPage],
        queryFn: async () => {
            const params = new URLSearchParams()
            params.set('page', String(alertsPage))
            params.set('per_page', String(perPage))
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
            // 檢查被登出的 session 是否屬於當前使用者
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

    const formatDateTime = (dateStr: string) =>
        format(new Date(dateStr), 'yyyy/MM/dd HH:mm', { locale: zhTW })

    const getSeverityColor = (severity: string) => {
        switch (severity) {
            case 'critical':
                return 'destructive' as const
            case 'high':
                return 'destructive' as const
            case 'warning':
                return 'warning' as const
            case 'medium':
                return 'default' as const
            case 'info':
                return 'default' as const
            default:
                return 'secondary' as const
        }
    }

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

    const getAlertSortIcon = (field: AlertSortField) => {
        if (alertSortConfig.field !== field)
            return <ArrowUpDown className="ml-1 h-3 w-3" />
        return alertSortConfig.order === 'asc' ? (
            <ArrowUp className="ml-1 h-3 w-3 text-primary" />
        ) : (
            <ArrowDown className="ml-1 h-3 w-3 text-primary" />
        )
    }

    // 根據操作類型與資料產生簡短中文摘要
    const getActionSummary = (log: AuditLog): string => {
        const actionLabels: Record<string, string> = {
            'CREATE': '建立使用者',
            'UPDATE': '更新使用者資料',
            'DELETE': '刪除使用者',
            'PASSWORD_RESET': '重設密碼',
            'IMPERSONATE': '模擬登入',
            'force_logout': '強制登出 Session',
        }

        const base = actionLabels[log.action] || log.action

        if (log.action === 'IMPERSONATE' && log.after_data) {
            const data = log.after_data as Record<string, string>
            const target = data.impersonated_email || data.impersonated_user_id?.slice(0, 8)
            return target ? `${base} → ${target}` : base
        }

        if (log.action === 'UPDATE' && log.after_data) {
            const keys = Object.keys(log.after_data)
            const fieldLabels: Record<string, string> = {
                display_name: '顯示名稱',
                email: '信箱',
                is_active: '啟用狀態',
                roles: '角色',
                phone: '電話',
                organization: '組織',
            }
            const changed = keys.map(k => fieldLabels[k] || k).slice(0, 3)
            return `${base}：${changed.join('、')}${keys.length > 3 ? ' 等' : ''}`
        }

        if (log.action === 'CREATE' && log.after_data) {
            const data = log.after_data as Record<string, string>
            if (data.email) return `${base}：${data.email}`
        }

        if (log.action === 'force_logout' && log.after_data) {
            const data = log.after_data as Record<string, string>
            if (data.reason) return `${base}：${data.reason}`
        }

        return base
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">安全審計</h1>
                    <p className="text-muted-foreground">監控系統活動與安全事件</p>
                </div>
                <Button
                    variant="outline"
                    onClick={() => {
                        queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
                        queryClient.invalidateQueries({ queryKey: ['audit-activities'] })
                        queryClient.invalidateQueries({ queryKey: ['audit-logins'] })
                        queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
                        queryClient.invalidateQueries({ queryKey: ['audit-alerts'] })
                    }}
                >
                    <RefreshCw className="h-4 w-4 mr-2" />
                    重新整理
                </Button>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList className="flex w-full overflow-x-auto sm:grid sm:grid-cols-5">
                    <TabsTrigger value="dashboard" className="flex items-center gap-1.5 whitespace-nowrap text-xs sm:text-sm sm:gap-2">
                        <Shield className="h-4 w-4 hidden sm:inline-block" />
                        總覽
                    </TabsTrigger>
                    <TabsTrigger value="activities" className="flex items-center gap-1.5 whitespace-nowrap text-xs sm:text-sm sm:gap-2">
                        <Activity className="h-4 w-4 hidden sm:inline-block" />
                        活動記錄
                    </TabsTrigger>
                    <TabsTrigger value="logins" className="flex items-center gap-1.5 whitespace-nowrap text-xs sm:text-sm sm:gap-2">
                        <LogIn className="h-4 w-4 hidden sm:inline-block" />
                        登入事件
                    </TabsTrigger>
                    <TabsTrigger value="sessions" className="flex items-center gap-1.5 whitespace-nowrap text-xs sm:text-sm sm:gap-2">
                        <Users className="h-4 w-4 hidden sm:inline-block" />
                        活躍 Sessions
                    </TabsTrigger>
                    <TabsTrigger value="alerts" className="flex items-center gap-1.5 whitespace-nowrap text-xs sm:text-sm sm:gap-2">
                        <AlertTriangle className="h-4 w-4 hidden sm:inline-block" />
                        安全警報
                        {dashboardStats && dashboardStats.open_alerts > 0 && (
                            <Badge variant="destructive" className="ml-0.5 sm:ml-1 text-[10px] sm:text-xs px-1 sm:px-1.5">
                                {dashboardStats.open_alerts}
                            </Badge>
                        )}
                    </TabsTrigger>
                </TabsList>

                {/* Dashboard Tab */}
                <TabsContent value="dashboard" className="space-y-4">
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium">今日活躍用戶</CardTitle>
                                <Users className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="text-2xl font-bold">{dashboardStats?.active_users_today ?? 0}</div>
                                <p className="text-xs text-muted-foreground">
                                    本週: {dashboardStats?.active_users_week ?? 0} / 本月:{' '}
                                    {dashboardStats?.active_users_month ?? 0}
                                </p>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium">今日登入次數</CardTitle>
                                <LogIn className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="text-2xl font-bold">{dashboardStats?.total_logins_today ?? 0}</div>
                                <p className="text-xs text-muted-foreground">
                                    失敗: {dashboardStats?.failed_logins_today ?? 0} 次
                                </p>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium">活躍 Sessions</CardTitle>
                                <Clock className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="text-2xl font-bold">{dashboardStats?.active_sessions ?? 0}</div>
                                <p className="text-xs text-muted-foreground">目前線上</p>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium">未解決警報</CardTitle>
                                <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="text-2xl font-bold">{dashboardStats?.open_alerts ?? 0}</div>
                                <p className="text-xs text-muted-foreground">
                                    嚴重: {dashboardStats?.critical_alerts ?? 0}
                                </p>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                {/* Activities Tab (使用者管理操作) */}
                <TabsContent value="activities" className="space-y-4">
                    <div className="flex flex-wrap items-center gap-4">
                        <Input
                            type="date"
                            value={dateFrom}
                            onChange={(e) => handleDateFromChange(e.target.value)}
                            placeholder="開始日期"
                            className="max-w-[150px]"
                        />
                        <Input
                            type="date"
                            value={dateTo}
                            onChange={(e) => handleDateToChange(e.target.value)}
                            placeholder="結束日期"
                            className="max-w-[150px]"
                        />
                    </div>
                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>時間</TableHead>
                                    <TableHead>操作者</TableHead>
                                    <TableHead>操作</TableHead>
                                    <TableHead>目標使用者</TableHead>
                                    <TableHead>摘要</TableHead>
                                    <TableHead className="w-[60px]">詳情</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingActivities ? (
                                    <TableRow>
                                        <TableCell colSpan={6} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : !activityLogs?.data || activityLogs.data.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                            沒有使用者管理活動記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    activityLogs.data.map((log) => (
                                        <TableRow key={log.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDateTime(log.created_at)}
                                            </TableCell>
                                            <TableCell>
                                                <div>
                                                    <div className="font-medium">{log.actor_name || '-'}</div>
                                                    <div className="text-sm text-muted-foreground">{log.actor_email || ''}</div>
                                                </div>
                                            </TableCell>
                                            <TableCell>
                                                <Badge variant={{
                                                    'CREATE': 'default',
                                                    'UPDATE': 'default',
                                                    'DELETE': 'destructive',
                                                    'PASSWORD_RESET': 'secondary',
                                                    'IMPERSONATE': 'secondary',
                                                    'force_logout': 'destructive',
                                                }[log.action] as 'default' | 'destructive' | 'secondary' || 'outline'}>
                                                    {{
                                                        'CREATE': '建立使用者',
                                                        'UPDATE': '更新使用者',
                                                        'DELETE': '刪除使用者',
                                                        'PASSWORD_RESET': '重設密碼',
                                                        'IMPERSONATE': '模擬登入',
                                                        'force_logout': '強制登出',
                                                    }[log.action] || log.action}
                                                </Badge>
                                            </TableCell>
                                            <TableCell>
                                                {log.entity_name || log.entity_email ? (
                                                    <div>
                                                        <div className="font-medium">{log.entity_name || '-'}</div>
                                                        <div className="text-xs text-muted-foreground">{log.entity_email || ''}</div>
                                                    </div>
                                                ) : (
                                                    <span className="font-mono text-xs text-muted-foreground">{log.entity_id?.slice(0, 8)}...</span>
                                                )}
                                            </TableCell>
                                            <TableCell className="text-sm max-w-[250px]">
                                                <span className="text-muted-foreground">{getActionSummary(log)}</span>
                                            </TableCell>
                                            <TableCell>
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    onClick={() => setSelectedLog(log)}
                                                    title="查看詳情"
                                                >
                                                    <Eye className="h-4 w-4" />
                                                </Button>
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                        {/* 活動記錄分頁 */}
                        {activityLogs && activityLogs.total_pages > 0 && (
                            <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
                                <p className="text-sm text-muted-foreground">
                                    共 {activityLogs.total} 筆，第 {activitiesPage} / {activityLogs.total_pages} 頁
                                </p>
                                <div className="flex items-center gap-2">
                                    <Button variant="outline" size="sm" disabled={activitiesPage <= 1} onClick={() => setActivitiesPage(p => Math.max(1, p - 1))}>
                                        <ChevronLeft className="h-4 w-4 mr-1" />上一頁
                                    </Button>
                                    <Button variant="outline" size="sm" disabled={activitiesPage >= activityLogs.total_pages} onClick={() => setActivitiesPage(p => Math.min(activityLogs.total_pages, p + 1))}>
                                        下一頁<ChevronRight className="h-4 w-4 ml-1" />
                                    </Button>
                                </div>
                            </div>
                        )}
                    </Card>
                </TabsContent>

                {/* Logins Tab */}
                <TabsContent value="logins" className="space-y-4">
                    <div className="flex flex-wrap items-center gap-4">
                        <Input
                            type="date"
                            value={dateFrom}
                            onChange={(e) => handleDateFromChange(e.target.value)}
                            className="max-w-[150px]"
                        />
                        <Input
                            type="date"
                            value={dateTo}
                            onChange={(e) => handleDateToChange(e.target.value)}
                            className="max-w-[150px]"
                        />
                        <Select defaultValue="all">
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="事件類型" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">全部</SelectItem>
                                <SelectItem value="login_success">登入成功</SelectItem>
                                <SelectItem value="login_failure">登入失敗</SelectItem>
                                <SelectItem value="logout">登出</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>時間</TableHead>
                                    <TableHead>Email</TableHead>
                                    <TableHead>事件</TableHead>
                                    <TableHead>裝置</TableHead>
                                    <TableHead>瀏覽器</TableHead>
                                    <TableHead>IP</TableHead>
                                    <TableHead>異常</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingLogins ? (
                                    <TableRow>
                                        <TableCell colSpan={7} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : loginEvents?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                            沒有登入事件
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    loginEvents?.data?.map((event) => (
                                        <TableRow key={event.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDateTime(event.created_at)}
                                            </TableCell>
                                            <TableCell>{event.email}</TableCell>
                                            <TableCell>
                                                <Badge
                                                    variant={event.event_type === 'login_success' ? 'default' : 'destructive'}
                                                >
                                                    {event.event_type === 'login_success'
                                                        ? '成功'
                                                        : event.event_type === 'login_failure'
                                                            ? '失敗'
                                                            : '登出'}
                                                </Badge>
                                            </TableCell>
                                            <TableCell>{event.device_type || '-'}</TableCell>
                                            <TableCell>{event.browser || '-'}</TableCell>
                                            <TableCell className="text-muted-foreground text-sm">
                                                {event.ip_address || '-'}
                                            </TableCell>
                                            <TableCell>
                                                <div className="flex flex-wrap gap-1">
                                                    {event.is_unusual_time && (
                                                        <Badge variant="secondary" className="bg-amber-100 text-amber-800 hover:bg-amber-100 border-amber-200">時間異常</Badge>
                                                    )}
                                                    {event.is_unusual_location && (
                                                        <Badge variant="secondary" className="bg-amber-100 text-amber-800 hover:bg-amber-100 border-amber-200">IP 異常</Badge>
                                                    )}
                                                    {event.is_new_device && (
                                                        <Badge variant="secondary" className="bg-amber-100 text-amber-800 hover:bg-amber-100 border-amber-200">新裝置</Badge>
                                                    )}
                                                    {event.is_mass_login && (
                                                        <Badge variant="secondary" className="bg-amber-100 text-amber-800 hover:bg-amber-100 border-amber-200">同時大量登入</Badge>
                                                    )}
                                                    {!(event.is_unusual_time || event.is_unusual_location || event.is_new_device || event.is_mass_login) && (
                                                        <span className="text-muted-foreground">-</span>
                                                    )}
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                        {/* 登入事件分頁 */}
                        {loginEvents && loginEvents.total_pages > 0 && (
                            <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
                                <p className="text-sm text-muted-foreground">
                                    共 {loginEvents.total} 筆，第 {loginsPage} / {loginEvents.total_pages} 頁
                                </p>
                                <div className="flex items-center gap-2">
                                    <Button variant="outline" size="sm" disabled={loginsPage <= 1} onClick={() => setLoginsPage(p => Math.max(1, p - 1))}>
                                        <ChevronLeft className="h-4 w-4 mr-1" />上一頁
                                    </Button>
                                    <Button variant="outline" size="sm" disabled={loginsPage >= loginEvents.total_pages} onClick={() => setLoginsPage(p => Math.min(loginEvents.total_pages, p + 1))}>
                                        下一頁<ChevronRight className="h-4 w-4 ml-1" />
                                    </Button>
                                </div>
                            </div>
                        )}
                    </Card>
                </TabsContent>

                {/* Sessions Tab */}
                <TabsContent value="sessions" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>活躍 Sessions</CardTitle>
                            <CardDescription>目前線上的使用者 Session</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>使用者</TableHead>
                                        <TableHead>開始時間</TableHead>
                                        <TableHead>最後活動</TableHead>
                                        <TableHead>IP</TableHead>
                                        <TableHead>頁面瀏覽</TableHead>
                                        <TableHead>操作次數</TableHead>
                                        <TableHead>操作</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingSessions ? (
                                        <TableRow>
                                            <TableCell colSpan={7} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : sessions?.data?.filter(s => s.is_active).length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                                沒有活躍的 Session
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        sessions?.data?.filter(s => s.is_active).map((session) => (
                                            <TableRow key={session.id}>
                                                <TableCell>
                                                    <div>
                                                        <div className="font-medium">{session.user_name}</div>
                                                        <div className="text-sm text-muted-foreground">{session.user_email}</div>
                                                    </div>
                                                </TableCell>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDateTime(session.started_at)}
                                                </TableCell>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDateTime(session.last_activity_at)}
                                                </TableCell>
                                                <TableCell className="text-muted-foreground text-sm">
                                                    {session.ip_address || '-'}
                                                </TableCell>
                                                <TableCell>{session.page_view_count}</TableCell>
                                                <TableCell>{session.action_count}</TableCell>
                                                <TableCell>
                                                    <Button
                                                        variant="destructive"
                                                        size="sm"
                                                        onClick={() => forceLogoutMutation.mutate(session.id)}
                                                        disabled={forceLogoutMutation.isPending}
                                                    >
                                                        <LogOut className="h-4 w-4 mr-1" />
                                                        強制登出
                                                    </Button>
                                                </TableCell>
                                            </TableRow>
                                        ))
                                    )}
                                </TableBody>
                            </Table>
                            {/* Sessions 分頁 */}
                            {sessions && sessions.total_pages > 0 && (
                                <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
                                    <p className="text-sm text-muted-foreground">
                                        共 {sessions.total} 筆，第 {sessionsPage} / {sessions.total_pages} 頁
                                    </p>
                                    <div className="flex items-center gap-2">
                                        <Button variant="outline" size="sm" disabled={sessionsPage <= 1} onClick={() => setSessionsPage(p => Math.max(1, p - 1))}>
                                            <ChevronLeft className="h-4 w-4 mr-1" />上一頁
                                        </Button>
                                        <Button variant="outline" size="sm" disabled={sessionsPage >= sessions.total_pages} onClick={() => setSessionsPage(p => Math.min(sessions.total_pages, p + 1))}>
                                            下一頁<ChevronRight className="h-4 w-4 ml-1" />
                                        </Button>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* Alerts Tab */}
                <TabsContent value="alerts" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>安全警報</CardTitle>
                            <CardDescription>需要關注的安全事件</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead
                                            className="cursor-pointer hover:bg-muted/50 transition-colors"
                                            onClick={() => handleAlertSort('created_at')}
                                        >
                                            <div className="flex items-center">
                                                時間
                                                {getAlertSortIcon('created_at')}
                                            </div>
                                        </TableHead>
                                        <TableHead
                                            className="cursor-pointer hover:bg-muted/50 transition-colors"
                                            onClick={() => handleAlertSort('alert_type')}
                                        >
                                            <div className="flex items-center">
                                                類型
                                                {getAlertSortIcon('alert_type')}
                                            </div>
                                        </TableHead>
                                        <TableHead
                                            className="cursor-pointer hover:bg-muted/50 transition-colors"
                                            onClick={() => handleAlertSort('severity')}
                                        >
                                            <div className="flex items-center">
                                                嚴重程度
                                                {getAlertSortIcon('severity')}
                                            </div>
                                        </TableHead>
                                        <TableHead>標題</TableHead>
                                        <TableHead>描述</TableHead>
                                        <TableHead
                                            className="cursor-pointer hover:bg-muted/50 transition-colors"
                                            onClick={() => handleAlertSort('status')}
                                        >
                                            <div className="flex items-center">
                                                狀態
                                                {getAlertSortIcon('status')}
                                            </div>
                                        </TableHead>
                                        <TableHead>操作</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingAlerts ? (
                                        <TableRow>
                                            <TableCell colSpan={7} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : alerts?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                                沒有安全警報
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        sortedAlerts.map((alert) => (
                                            <TableRow
                                                key={alert.id}
                                                className="cursor-pointer hover:bg-muted/50 transition-colors"
                                                onClick={() => setSelectedAlert(alert)}
                                            >
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDateTime(alert.created_at)}
                                                </TableCell>
                                                <TableCell>
                                                    <Badge variant="outline">{alert.alert_type}</Badge>
                                                </TableCell>
                                                <TableCell>
                                                    <Badge variant={getSeverityColor(alert.severity)}>
                                                        {alert.severity}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell>{alert.title}</TableCell>
                                                <TableCell className="text-sm text-muted-foreground max-w-[300px] truncate" title={alert.description || ''}>
                                                    {alert.alert_type === 'global_mass_login' && alert.context_data ? (
                                                        <span>
                                                            {alert.description}
                                                            {typeof alert.context_data === 'object' && 'account_count' in alert.context_data ?
                                                                ` (數量: ${alert.context_data.account_count})` : ''}
                                                        </span>
                                                    ) : (
                                                        alert.description || '-'
                                                    )}
                                                </TableCell>
                                                <TableCell>
                                                    <Badge
                                                        variant={alert.status === 'resolved' ? 'secondary' : 'default'}
                                                    >
                                                        {alert.status === 'open' ? '待處理' : '已解決'}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell>
                                                    {alert.status !== 'resolved' && (
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            onClick={(e) => {
                                                                e.stopPropagation()
                                                                resolveAlertMutation.mutate(alert.id)
                                                            }}
                                                            disabled={resolveAlertMutation.isPending}
                                                        >
                                                            標記解決
                                                        </Button>
                                                    )}
                                                </TableCell>
                                            </TableRow>
                                        ))
                                    )}
                                </TableBody>
                            </Table>
                            {/* 安全警報分頁 */}
                            {alerts && alerts.total_pages > 0 && (
                                <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
                                    <p className="text-sm text-muted-foreground">
                                        共 {alerts.total} 筆，第 {alertsPage} / {alerts.total_pages} 頁
                                    </p>
                                    <div className="flex items-center gap-2">
                                        <Button variant="outline" size="sm" disabled={alertsPage <= 1} onClick={() => setAlertsPage(p => Math.max(1, p - 1))}>
                                            <ChevronLeft className="h-4 w-4 mr-1" />上一頁
                                        </Button>
                                        <Button variant="outline" size="sm" disabled={alertsPage >= alerts.total_pages} onClick={() => setAlertsPage(p => Math.min(alerts.total_pages, p + 1))}>
                                            下一頁<ChevronRight className="h-4 w-4 ml-1" />
                                        </Button>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>

            {/* 活動記錄詳情 Dialog */}
            <Dialog open={!!selectedLog} onOpenChange={() => setSelectedLog(null)}>
                <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <Eye className="h-5 w-5" />
                            活動記錄詳情
                        </DialogTitle>
                    </DialogHeader>
                    {selectedLog && (
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <Label className="text-muted-foreground">操作時間</Label>
                                    <p className="font-medium">{formatDateTime(selectedLog.created_at)}</p>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">操作者</Label>
                                    <p className="font-medium">{selectedLog.actor_name || '-'}</p>
                                    <p className="text-sm text-muted-foreground">{selectedLog.actor_email || ''}</p>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">操作類型</Label>
                                    <div className="mt-1">
                                        <Badge variant={{
                                            'CREATE': 'default',
                                            'UPDATE': 'default',
                                            'DELETE': 'destructive',
                                            'PASSWORD_RESET': 'secondary',
                                            'IMPERSONATE': 'secondary',
                                            'force_logout': 'destructive',
                                        }[selectedLog.action] as 'default' | 'destructive' | 'secondary' || 'outline'}>
                                            {{
                                                'CREATE': '建立使用者',
                                                'UPDATE': '更新使用者',
                                                'DELETE': '刪除使用者',
                                                'PASSWORD_RESET': '重設密碼',
                                                'IMPERSONATE': '模擬登入',
                                                'force_logout': '強制登出',
                                            }[selectedLog.action] || selectedLog.action}
                                        </Badge>
                                    </div>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">目標使用者</Label>
                                    {selectedLog.entity_name || selectedLog.entity_email ? (
                                        <div>
                                            <p className="font-medium">{selectedLog.entity_name || '-'}</p>
                                            <p className="text-sm text-muted-foreground">{selectedLog.entity_email || ''}</p>
                                        </div>
                                    ) : (
                                        <p className="font-medium">
                                            <Badge variant="outline">{selectedLog.entity_type}</Badge>
                                        </p>
                                    )}
                                </div>
                                <div className="col-span-2">
                                    <Label className="text-muted-foreground">實體 ID</Label>
                                    <p className="font-mono text-sm">{selectedLog.entity_id || '-'}</p>
                                </div>
                            </div>

                            {selectedLog.before_data && (
                                <div>
                                    <Label className="text-muted-foreground">變更前資料</Label>
                                    <pre className="mt-1 p-3 bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800 rounded-md text-sm overflow-x-auto">
                                        {JSON.stringify(selectedLog.before_data, null, 2)}
                                    </pre>
                                </div>
                            )}

                            {selectedLog.after_data && (
                                <div>
                                    <Label className="text-muted-foreground">變更後資料</Label>
                                    <pre className="mt-1 p-3 bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 rounded-md text-sm overflow-x-auto">
                                        {JSON.stringify(selectedLog.after_data, null, 2)}
                                    </pre>
                                </div>
                            )}

                            {!selectedLog.before_data && !selectedLog.after_data && (
                                <p className="text-muted-foreground text-center py-4">無變更資料</p>
                            )}
                        </div>
                    )}
                </DialogContent>
            </Dialog>

            {/* 安全警報詳情 Dialog */}
            <Dialog open={!!selectedAlert} onOpenChange={() => setSelectedAlert(null)}>
                <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <AlertTriangle className="h-5 w-5" />
                            安全警報詳情
                        </DialogTitle>
                        <DialogDescription>
                            查看警報的完整資訊與上下文
                        </DialogDescription>
                    </DialogHeader>
                    {selectedAlert && (
                        <div className="space-y-4">
                            {/* 基本資訊 */}
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <Label className="text-muted-foreground">警報時間</Label>
                                    <p className="font-medium">{formatDateTime(selectedAlert.created_at)}</p>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">警報類型</Label>
                                    <div className="mt-1">
                                        <Badge variant="outline">{selectedAlert.alert_type}</Badge>
                                    </div>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">嚴重程度</Label>
                                    <div className="mt-1">
                                        <Badge variant={getSeverityColor(selectedAlert.severity)}>
                                            {selectedAlert.severity}
                                        </Badge>
                                    </div>
                                </div>
                                <div>
                                    <Label className="text-muted-foreground">狀態</Label>
                                    <div className="mt-1">
                                        <Badge
                                            variant={selectedAlert.status === 'resolved' ? 'secondary' : 'default'}
                                        >
                                            {selectedAlert.status === 'open' ? '待處理' : '已解決'}
                                        </Badge>
                                    </div>
                                </div>
                            </div>

                            <hr className="border-border" />

                            {/* 標題與描述 */}
                            <div>
                                <Label className="text-muted-foreground">標題</Label>
                                <p className="font-medium text-base">{selectedAlert.title}</p>
                            </div>
                            {selectedAlert.description && (
                                <div>
                                    <Label className="text-muted-foreground">描述</Label>
                                    <p className="text-sm mt-1 whitespace-pre-wrap">{selectedAlert.description}</p>
                                </div>
                            )}

                            {/* 相關使用者 */}
                            {selectedAlert.user_id && (
                                <div>
                                    <Label className="text-muted-foreground">相關使用者 ID</Label>
                                    <p className="font-mono text-sm">{selectedAlert.user_id}</p>
                                </div>
                            )}

                            {/* Context Data */}
                            {selectedAlert.context_data && Object.keys(selectedAlert.context_data).length > 0 && (
                                <div>
                                    <Label className="text-muted-foreground">詳細上下文資料</Label>
                                    <pre className="mt-1 p-3 bg-muted/50 border rounded-md text-sm overflow-x-auto">
                                        {JSON.stringify(selectedAlert.context_data, null, 2)}
                                    </pre>
                                </div>
                            )}

                            {/* 解決資訊 */}
                            {selectedAlert.status === 'resolved' && (
                                <>
                                    <hr className="border-border" />
                                    <div className="grid grid-cols-2 gap-4">
                                        {selectedAlert.resolved_at && (
                                            <div>
                                                <Label className="text-muted-foreground">解決時間</Label>
                                                <p className="font-medium">{formatDateTime(selectedAlert.resolved_at)}</p>
                                            </div>
                                        )}
                                        {selectedAlert.resolved_by && (
                                            <div>
                                                <Label className="text-muted-foreground">解決者</Label>
                                                <p className="font-medium">{selectedAlert.resolved_by}</p>
                                            </div>
                                        )}
                                    </div>
                                    {selectedAlert.resolution_notes && (
                                        <div>
                                            <Label className="text-muted-foreground">解決備註</Label>
                                            <p className="text-sm mt-1">{selectedAlert.resolution_notes}</p>
                                        </div>
                                    )}
                                </>
                            )}

                            {/* Dialog 內的操作按鈕 */}
                            {selectedAlert.status !== 'resolved' && (
                                <DialogFooter>
                                    <Button
                                        variant="outline"
                                        onClick={() => setSelectedAlert(null)}
                                    >
                                        關閉
                                    </Button>
                                    <Button
                                        onClick={() => {
                                            resolveAlertMutation.mutate(selectedAlert.id)
                                            setSelectedAlert(null)
                                        }}
                                        disabled={resolveAlertMutation.isPending}
                                    >
                                        標記為已解決
                                    </Button>
                                </DialogFooter>
                            )}
                        </div>
                    )}
                </DialogContent>
            </Dialog>
        </div>
    )
}

export default AdminAuditPage
