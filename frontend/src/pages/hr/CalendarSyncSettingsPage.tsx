import { useState, useEffect, useCallback, useMemo, lazy, Suspense } from 'react'
import { Loader2 } from 'lucide-react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format, startOfMonth, endOfMonth, addMonths } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    AlertTriangle,
    Calendar,
    CalendarDays,
    CheckCircle,
    Clock,
    Link2,
    RefreshCw,
    Settings,
    Unlink,
} from 'lucide-react'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { ErrorBoundary } from '@/components/ui/error-boundary'
import type { CalendarSyncHistory, CalendarSyncStatus, ConflictWithDetails, CalendarEvent } from '@/types/hr'

// 僅在日曆已連接時動態載入 FullCalendar，避免未設定時觸發 cssRules 等錯誤
const CalendarView = lazy(() =>
    import('./CalendarView').then((mod) => ({ default: mod.CalendarView })),
)

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

export function CalendarSyncSettingsPage() {
    const [activeTab, setActiveTab] = useState('calendar')
    const [showConnectDialog, setShowConnectDialog] = useState(false)
    const [calendarId, setCalendarId] = useState('')
    const [authEmail, setAuthEmail] = useState('')
    const [calendarDateRange, setCalendarDateRange] = useState(() => ({
        start: startOfMonth(new Date()),
        end: endOfMonth(new Date()),
    }))
    const queryClient = useQueryClient()
    const { user, hasRole } = useAuthStore()
    const isAdmin = hasRole('admin')
    // 延後掛載 FullCalendar，避免載入時同步 reflow 造成 Violation（message/setTimeout handler、forced reflow）
    const [calendarMounted, setCalendarMounted] = useState(false)

    // 當打開連接對話框時，預設授權 Email 為當前用戶的 Email
    useEffect(() => {
        if (showConnectDialog && user?.email) {
            setAuthEmail(user.email)
        }
    }, [showConnectDialog, user?.email])

    // 同步狀態
    const { data: syncStatus, isLoading: loadingStatus } = useQuery({
        queryKey: ['calendar-status'],
        queryFn: async () => {
            const res = await api.get<CalendarSyncStatus>('/hr/calendar/status')
            return res.data
        },
    })

    // 同步歷史
    const { data: syncHistory, isLoading: loadingHistory } = useQuery({
        queryKey: ['calendar-history'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<CalendarSyncHistory>>('/hr/calendar/history')
            return res.data
        },
        enabled: activeTab === 'history',
    })

    // 衝突列表
    const { data: conflicts, isLoading: loadingConflicts } = useQuery({
        queryKey: ['calendar-conflicts'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<ConflictWithDetails>>('/hr/calendar/conflicts?status=pending')
            return res.data
        },
        enabled: activeTab === 'conflicts',
    })

    // 日曆事件
    const { data: calendarEvents, isLoading: loadingEvents } = useQuery({
        queryKey: ['calendar-events', calendarDateRange],
        queryFn: async () => {
            const startDate = format(calendarDateRange.start, 'yyyy-MM-dd')
            const endDate = format(calendarDateRange.end, 'yyyy-MM-dd')
            const res = await api.get<CalendarEvent[]>(`/hr/calendar/events?start_date=${startDate}&end_date=${endDate}`)
            return res.data
        },
        enabled: activeTab === 'calendar' && syncStatus?.is_configured === true,
    })

    // 日曆已連接且切到日曆分頁時掛載 FullCalendar；有快取事件時即使 refetch 也保持掛載，避免切月後被卸載又重掛成「本月」
    const shouldShowCalendar = activeTab === 'calendar' && syncStatus?.is_configured === true && (calendarEvents != null || !loadingEvents)
    useEffect(() => {
        if (!shouldShowCalendar) {
            setCalendarMounted(false)
            return
        }
        let secondId: number | undefined
        const firstId = requestAnimationFrame(() => {
            secondId = requestAnimationFrame(() => {
                setCalendarMounted(true)
            })
        })
        return () => {
            cancelAnimationFrame(firstId)
            if (typeof secondId === 'number') cancelAnimationFrame(secondId)
        }
    }, [shouldShowCalendar])

    // FullCalendar 日期範圍變更處理：僅接受「往後」或「恰好上一個月」，其餘往前的 datesSet 忽略（見 docs/development/CALENDAR_PAGE_DESIGN_AND_IMPROVEMENT.md）
    const handleDatesSet = useCallback((dateInfo: { start: Date; end: Date }) => {
        setCalendarDateRange((prev) => {
            const newStart = dateInfo.start.getTime()
            const prevStart = prev.start.getTime()
            if (newStart === prevStart) return prev
            if (newStart > prevStart) return { start: dateInfo.start, end: dateInfo.end }
            const prevMonthStart = startOfMonth(addMonths(prev.start, -1)).getTime()
            const newMonthStart = startOfMonth(dateInfo.start).getTime()
            if (newMonthStart === prevMonthStart) return { start: dateInfo.start, end: dateInfo.end }
            return prev
        })
    }, [])

    // 轉換事件為 FullCalendar 格式。refetch 時不傳舊月份事件（loadingEvents 時 React Query 仍回傳上一筆），
    // 避免 FullCalendar 依事件範圍觸發 datesSet 把視圖拉回舊月，造成「下個月」點一次進、點一次退」的交替問題。
    const fullCalendarEvents = useMemo(() => {
        if (loadingEvents) return []
        return calendarEvents?.map(event => ({
            id: event.id,
            title: event.summary,
            start: event.start,
            end: event.end,
            allDay: event.all_day,
            extendedProps: {
                description: event.description,
                location: event.location,
                htmlLink: event.html_link,
            },
        })) ?? []
    }, [loadingEvents, calendarEvents])

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

    // 解決衝突
    const resolveConflictMutation = useMutation({
        mutationFn: async ({ id, resolution }: { id: string; resolution: string }) => {
            return api.post(`/hr/calendar/conflicts/${id}/resolve`, { resolution })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['calendar-conflicts'] })
            toast({ title: '成功', description: '衝突已解決' })
        },
    })

    const formatDateTime = (dateStr: string) => {
        return new Date(dateStr).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
    }

    const getStatusBadge = (status: string) => {
        switch (status) {
            case 'completed':
                return <Badge className="bg-green-500">完成</Badge>
            case 'running':
                return <Badge className="bg-blue-500">執行中</Badge>
            case 'failed':
                return <Badge variant="destructive">失敗</Badge>
            default:
                return <Badge variant="secondary">{status}</Badge>
        }
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">Google Calendar</h1>
                    <p className="text-muted-foreground">設定與管理請假日曆</p>
                </div>
                {syncStatus?.is_configured && (
                    <Button
                        onClick={() => syncMutation.mutate()}
                        disabled={syncMutation.isPending}
                    >
                        <RefreshCw className={`h-4 w-4 mr-2 ${syncMutation.isPending ? 'animate-spin' : ''}`} />
                        立即同步
                    </Button>
                )}
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="calendar" className="flex items-center gap-2">
                        <CalendarDays className="h-4 w-4" />
                        日曆
                    </TabsTrigger>
                    <TabsTrigger value="history" className="flex items-center gap-2">
                        <Clock className="h-4 w-4" />
                        同步歷史
                    </TabsTrigger>
                    {isAdmin && (
                        <TabsTrigger value="conflicts" className="flex items-center gap-2">
                            <AlertTriangle className="h-4 w-4" />
                            衝突處理
                            {conflicts && conflicts.total > 0 && (
                                <Badge variant="destructive" className="ml-1">
                                    {conflicts.total}
                                </Badge>
                            )}
                        </TabsTrigger>
                    )}
                    {isAdmin && (
                        <TabsTrigger value="status" className="flex items-center gap-2">
                            <Settings className="h-4 w-4" />
                            連線狀態
                        </TabsTrigger>
                    )}
                </TabsList>

                {/* 連線狀態 */}
                <TabsContent value="status" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>連線設定</CardTitle>
                            <CardDescription>
                                連接到共用的 Google Calendar 以同步請假事件
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            {loadingStatus ? (
                                <div className="text-center py-8">載入中...</div>
                            ) : syncStatus?.is_configured ? (
                                <>
                                    <div className="flex items-center gap-4">
                                        <div className="h-12 w-12 rounded-full bg-green-100 flex items-center justify-center">
                                            <CheckCircle className="h-6 w-6 text-green-600" />
                                        </div>
                                        <div className="flex-1">
                                            <div className="font-medium">已連接</div>
                                            <div className="text-sm text-muted-foreground">
                                                Calendar ID: {syncStatus.calendar_id}
                                            </div>
                                        </div>
                                        <Button
                                            variant="destructive"
                                            onClick={() => disconnectMutation.mutate()}
                                            disabled={disconnectMutation.isPending}
                                        >
                                            <Unlink className="h-4 w-4 mr-2" />
                                            斷開連接
                                        </Button>
                                    </div>

                                    <div className="grid gap-4 md:grid-cols-4">
                                        <Card>
                                            <CardHeader className="pb-2">
                                                <CardTitle className="text-sm">同步狀態</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="flex items-center gap-2">
                                                    {syncStatus.sync_enabled ? (
                                                        <Badge className="bg-green-500">啟用</Badge>
                                                    ) : (
                                                        <Badge variant="secondary">停用</Badge>
                                                    )}
                                                </div>
                                            </CardContent>
                                        </Card>
                                        <Card>
                                            <CardHeader className="pb-2">
                                                <CardTitle className="text-sm">最後同步</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="text-sm">
                                                    {syncStatus.last_sync_at
                                                        ? formatDateTime(syncStatus.last_sync_at)
                                                        : '尚未同步'}
                                                </div>
                                            </CardContent>
                                        </Card>
                                        <Card>
                                            <CardHeader className="pb-2">
                                                <CardTitle className="text-sm">待同步事件</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="text-2xl font-bold">{syncStatus.pending_syncs}</div>
                                            </CardContent>
                                        </Card>
                                        <Card>
                                            <CardHeader className="pb-2">
                                                <CardTitle className="text-sm">待處理衝突</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="text-2xl font-bold text-orange-500">
                                                    {syncStatus.pending_conflicts}
                                                </div>
                                            </CardContent>
                                        </Card>
                                    </div>
                                </>
                            ) : (
                                <div className="text-center py-8 space-y-4">
                                    <Calendar className="h-16 w-16 mx-auto text-muted-foreground" />
                                    <div>
                                        <div className="font-medium">尚未連接 Google Calendar</div>
                                        <div className="text-sm text-muted-foreground">
                                            連接後可自動同步核准的請假事件到共用日曆
                                        </div>
                                    </div>
                                    <Button onClick={() => setShowConnectDialog(true)}>
                                        <Link2 className="h-4 w-4 mr-2" />
                                        連接 Google Calendar
                                    </Button>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* 日曆視圖 */}
                <TabsContent value="calendar" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Google Calendar 事件</CardTitle>
                            <CardDescription>
                                從已連接的 Google Calendar 讀取的事件
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            {loadingStatus ? (
                                <div className="flex flex-col items-center justify-center py-12 gap-4">
                                    <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                                    <div className="text-sm text-muted-foreground">載入中...</div>
                                </div>
                            ) : !syncStatus?.is_configured ? (
                                <div className="text-center py-8 space-y-4">
                                    <Calendar className="h-16 w-16 mx-auto text-muted-foreground" />
                                    <div>
                                        <div className="font-medium">尚未連接 Google Calendar</div>
                                        {isAdmin ? (
                                            <div className="text-sm text-muted-foreground">
                                                請先在「連線狀態」分頁連接 Google Calendar
                                            </div>
                                        ) : (
                                            <div className="text-sm text-muted-foreground">
                                                請聯繫系統管理員設定 Google Calendar 連接
                                            </div>
                                        )}
                                    </div>
                                </div>
                            ) : loadingEvents && calendarEvents == null ? (
                                <div className="flex flex-col items-center justify-center py-12 gap-4">
                                    <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                                    <div className="text-sm text-muted-foreground">載入事件中...</div>
                                </div>
                            ) : !calendarMounted ? (
                                <div className="flex flex-col items-center justify-center py-12 gap-4">
                                    <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                                    <div className="text-sm text-muted-foreground">載入日曆中...</div>
                                </div>
                            ) : (
                                <ErrorBoundary
                                    fallback={
                                        <div className="flex flex-col items-center justify-center py-12 gap-4">
                                            <AlertTriangle className="h-12 w-12 text-destructive" />
                                            <div className="text-center space-y-2">
                                                <div className="font-medium">日曆載入失敗</div>
                                                <div className="text-sm text-muted-foreground max-w-md">
                                                    可能是瀏覽器環境限制，請重新整理頁面或聯繫系統管理員
                                                </div>
                                            </div>
                                            <Button variant="outline" onClick={() => window.location.reload()}>
                                                重新整理
                                            </Button>
                                        </div>
                                    }
                                >
                                    <Suspense
                                        fallback={
                                            <div className="flex flex-col items-center justify-center py-12 gap-4">
                                                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
                                                <div className="text-sm text-muted-foreground">載入日曆中...</div>
                                            </div>
                                        }
                                    >
                                        <CalendarView
                                            key={format(calendarDateRange.start, 'yyyy-MM')}
                                            initialDate={calendarDateRange.start}
                                            events={fullCalendarEvents}
                                            onDatesSet={handleDatesSet}
                                        />
                                    </Suspense>
                                </ErrorBoundary>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* 同步歷史 */}
                <TabsContent value="history" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>同步歷史記錄</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>時間</TableHead>
                                        <TableHead>類型</TableHead>
                                        <TableHead>狀態</TableHead>
                                        <TableHead>新增</TableHead>
                                        <TableHead>更新</TableHead>
                                        <TableHead>刪除</TableHead>
                                        <TableHead>衝突</TableHead>
                                        <TableHead>耗時</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingHistory ? (
                                        <TableRow>
                                            <TableCell colSpan={8} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : syncHistory?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                                沒有同步記錄
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        syncHistory?.data?.map((h) => (
                                            <TableRow key={h.id}>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDateTime(h.started_at)}
                                                </TableCell>
                                                <TableCell>
                                                    {h.job_type === 'manual' ? '手動' : '自動'}
                                                </TableCell>
                                                <TableCell>{getStatusBadge(h.status)}</TableCell>
                                                <TableCell>{h.events_created}</TableCell>
                                                <TableCell>{h.events_updated}</TableCell>
                                                <TableCell>{h.events_deleted}</TableCell>
                                                <TableCell>
                                                    {h.conflicts_detected > 0 && (
                                                        <Badge variant="secondary">{h.conflicts_detected}</Badge>
                                                    )}
                                                </TableCell>
                                                <TableCell>
                                                    {h.duration_ms ? `${(h.duration_ms / 1000).toFixed(1)}s` : '-'}
                                                </TableCell>
                                            </TableRow>
                                        ))
                                    )}
                                </TableBody>
                            </Table>
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* 衝突處理 */}
                <TabsContent value="conflicts" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>待處理衝突</CardTitle>
                            <CardDescription>
                                系統與 Google Calendar 之間的資料不一致需要手動處理
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>偵測時間</TableHead>
                                        <TableHead>員工</TableHead>
                                        <TableHead>假別</TableHead>
                                        <TableHead>衝突類型</TableHead>
                                        <TableHead>差異</TableHead>
                                        <TableHead>操作</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingConflicts ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : conflicts?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                                沒有待處理的衝突
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        conflicts?.data?.map((c) => (
                                            <TableRow key={c.id}>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDateTime(c.detected_at)}
                                                </TableCell>
                                                <TableCell>{c.user_name || '-'}</TableCell>
                                                <TableCell>{c.leave_type || '-'}</TableCell>
                                                <TableCell>
                                                    <Badge variant="outline">{c.conflict_type}</Badge>
                                                </TableCell>
                                                <TableCell className="max-w-[200px] truncate">
                                                    {c.difference_summary || '-'}
                                                </TableCell>
                                                <TableCell>
                                                    <div className="flex gap-2">
                                                        <Button
                                                            variant="default"
                                                            size="sm"
                                                            onClick={() =>
                                                                resolveConflictMutation.mutate({
                                                                    id: c.id,
                                                                    resolution: 'keep_ipig',
                                                                })
                                                            }
                                                            disabled={resolveConflictMutation.isPending}
                                                        >
                                                            保留 iPig
                                                        </Button>
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            onClick={() =>
                                                                resolveConflictMutation.mutate({
                                                                    id: c.id,
                                                                    resolution: 'dismiss',
                                                                })
                                                            }
                                                            disabled={resolveConflictMutation.isPending}
                                                        >
                                                            忽略
                                                        </Button>
                                                    </div>
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

            {/* 連接對話框 */}
            <Dialog open={showConnectDialog} onOpenChange={setShowConnectDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>連接 Google Calendar</DialogTitle>
                        <DialogDescription>
                            輸入共用日曆的 ID 和授權帳戶 Email
                        </DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label>Calendar ID *</Label>
                            <Input
                                placeholder="例如: company-leave@group.calendar.google.com"
                                value={calendarId}
                                onChange={(e) => setCalendarId(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                在 Google Calendar 設定中找到日曆 ID
                            </p>
                        </div>
                        <div className="grid gap-2">
                            <Label>授權 Email *</Label>
                            <Input
                                type="email"
                                placeholder="service-account@example.com"
                                value={authEmail}
                                onChange={(e) => setAuthEmail(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                擁有日曆編輯權限的 Google 帳戶
                            </p>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowConnectDialog(false)}>
                            取消
                        </Button>
                        <Button
                            onClick={() =>
                                connectMutation.mutate({
                                    calendar_id: calendarId,
                                    auth_email: authEmail,
                                })
                            }
                            disabled={!calendarId || !authEmail || connectMutation.isPending}
                        >
                            連接
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}

export default CalendarSyncSettingsPage
