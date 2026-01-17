import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    Activity,
    AlertTriangle,
    Clock,
    LogIn,
    LogOut,
    RefreshCw,
    Search,
    Shield,
    Users,
} from 'lucide-react'
import api from '@/lib/api'
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
import { toast } from '@/components/ui/use-toast'
import type {
    AuditDashboardStats,
    LoginEventWithUser,
    SecurityAlert,
    SessionWithUser,
    UserActivityLog,
} from '@/types/hr'

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

export function AdminAuditPage() {
    const [activeTab, setActiveTab] = useState('dashboard')
    const [searchTerm, setSearchTerm] = useState('')
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
    const queryClient = useQueryClient()

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
        queryKey: ['audit-activities', dateFrom, dateTo],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            const res = await api.get<PaginatedResponse<UserActivityLog>>(
                `/admin/audit/activities?${params}`
            )
            return res.data
        },
        enabled: activeTab === 'activities',
    })

    // Login Events
    const { data: loginEvents, isLoading: loadingLogins } = useQuery({
        queryKey: ['audit-logins', dateFrom, dateTo],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            const res = await api.get<PaginatedResponse<LoginEventWithUser>>(
                `/admin/audit/logins?${params}`
            )
            return res.data
        },
        enabled: activeTab === 'logins',
    })

    // Sessions
    const { data: sessions, isLoading: loadingSessions } = useQuery({
        queryKey: ['audit-sessions'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<SessionWithUser>>('/admin/audit/sessions')
            return res.data
        },
        enabled: activeTab === 'sessions',
    })

    // Security Alerts
    const { data: alerts, isLoading: loadingAlerts } = useQuery({
        queryKey: ['audit-alerts'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<SecurityAlert>>('/admin/audit/alerts')
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
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['audit-sessions'] })
            toast({ title: '成功', description: '已強制登出該 Session' })
        },
    })

    // Resolve Alert Mutation
    const resolveAlertMutation = useMutation({
        mutationFn: async (alertId: string) => {
            return api.post(`/admin/audit/alerts/${alertId}/resolve`, {
                resolution_notes: '已確認並解決',
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['audit-alerts'] })
            queryClient.invalidateQueries({ queryKey: ['audit-dashboard'] })
            toast({ title: '成功', description: '已解決警報' })
        },
    })

    const formatDateTime = (dateStr: string) => {
        return format(new Date(dateStr), 'yyyy/MM/dd HH:mm', { locale: zhTW })
    }

    const getSeverityColor = (severity: string) => {
        switch (severity) {
            case 'critical':
                return 'destructive'
            case 'high':
                return 'destructive'
            case 'medium':
                return 'default'
            default:
                return 'secondary'
        }
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
                <TabsList className="grid w-full grid-cols-5">
                    <TabsTrigger value="dashboard" className="flex items-center gap-2">
                        <Shield className="h-4 w-4" />
                        總覽
                    </TabsTrigger>
                    <TabsTrigger value="activities" className="flex items-center gap-2">
                        <Activity className="h-4 w-4" />
                        活動記錄
                    </TabsTrigger>
                    <TabsTrigger value="logins" className="flex items-center gap-2">
                        <LogIn className="h-4 w-4" />
                        登入事件
                    </TabsTrigger>
                    <TabsTrigger value="sessions" className="flex items-center gap-2">
                        <Users className="h-4 w-4" />
                        活躍 Sessions
                    </TabsTrigger>
                    <TabsTrigger value="alerts" className="flex items-center gap-2">
                        <AlertTriangle className="h-4 w-4" />
                        安全警報
                        {dashboardStats && dashboardStats.open_alerts > 0 && (
                            <Badge variant="destructive" className="ml-1">
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

                {/* Activities Tab */}
                <TabsContent value="activities" className="space-y-4">
                    <div className="flex gap-4">
                        <Input
                            type="date"
                            value={dateFrom}
                            onChange={(e) => setDateFrom(e.target.value)}
                            placeholder="開始日期"
                        />
                        <Input
                            type="date"
                            value={dateTo}
                            onChange={(e) => setDateTo(e.target.value)}
                            placeholder="結束日期"
                        />
                        <Input
                            placeholder="搜尋..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            className="max-w-sm"
                        />
                    </div>
                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>時間</TableHead>
                                    <TableHead>使用者</TableHead>
                                    <TableHead>類別</TableHead>
                                    <TableHead>事件</TableHead>
                                    <TableHead>實體</TableHead>
                                    <TableHead>IP</TableHead>
                                    <TableHead>可疑</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingActivities ? (
                                    <TableRow>
                                        <TableCell colSpan={7} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : activityLogs?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                            沒有活動記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    activityLogs?.data?.map((log) => (
                                        <TableRow key={log.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDateTime(log.created_at)}
                                            </TableCell>
                                            <TableCell>{log.actor_display_name || log.actor_email || '-'}</TableCell>
                                            <TableCell>
                                                <Badge variant="outline">{log.event_category}</Badge>
                                            </TableCell>
                                            <TableCell>{log.event_type}</TableCell>
                                            <TableCell>
                                                {log.entity_type ? `${log.entity_type}` : '-'}
                                            </TableCell>
                                            <TableCell className="text-muted-foreground text-sm">
                                                {log.ip_address || '-'}
                                            </TableCell>
                                            <TableCell>
                                                {log.is_suspicious && (
                                                    <Badge variant="destructive">可疑</Badge>
                                                )}
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                    </Card>
                </TabsContent>

                {/* Logins Tab */}
                <TabsContent value="logins" className="space-y-4">
                    <div className="flex gap-4">
                        <Input
                            type="date"
                            value={dateFrom}
                            onChange={(e) => setDateFrom(e.target.value)}
                        />
                        <Input
                            type="date"
                            value={dateTo}
                            onChange={(e) => setDateTo(e.target.value)}
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
                                                {(event.is_unusual_time || event.is_unusual_location || event.is_new_device) && (
                                                    <Badge variant="secondary">異常</Badge>
                                                )}
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
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
                                        <TableHead>時間</TableHead>
                                        <TableHead>類型</TableHead>
                                        <TableHead>嚴重程度</TableHead>
                                        <TableHead>標題</TableHead>
                                        <TableHead>狀態</TableHead>
                                        <TableHead>操作</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingAlerts ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : alerts?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                                沒有安全警報
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        alerts?.data?.map((alert) => (
                                            <TableRow key={alert.id}>
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
                                                            onClick={() => resolveAlertMutation.mutate(alert.id)}
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
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}

export default AdminAuditPage
