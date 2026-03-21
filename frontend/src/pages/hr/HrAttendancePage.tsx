import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
    Calendar,
    Clock,
    Download,
    LogIn,
    LogOut,
    RefreshCw,
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
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { toast } from '@/components/ui/use-toast'
import { formatTime } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { AxiosError } from 'axios'
import type { AttendanceWithUser, StaffInfo } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

export function HrAttendancePage() {
    const [activeTab, setActiveTab] = useState('today')
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
    const [viewAll, setViewAll] = useState(false)
    const [filterUserId, setFilterUserId] = useState<string>('')
    const queryClient = useQueryClient()
    const { hasPermission } = useAuthStore()
    const canViewAll = hasPermission('hr.attendance.view_all')

    // 今日打卡狀態
    const { data: todayAttendance, refetch: refetchToday } = useQuery({
        queryKey: ['hr-today-attendance'],
        queryFn: async () => {
            const today = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                `/hr/attendance?from=${today}&to=${today}`
            )
            return res.data.data[0] || null
        },
    })

    // 人員列表（供 view_all 時篩選）
    const { data: staffList } = useQuery({
        queryKey: ['hr-staff-for-attendance'],
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
        enabled: canViewAll && activeTab === 'history',
    })

    // 歷史記錄
    const { data: attendanceHistory, isLoading: loadingHistory } = useQuery({
        queryKey: ['hr-attendance-history', dateFrom, dateTo, viewAll, filterUserId],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            if (canViewAll && viewAll) params.set('view_all', 'true')
            if (canViewAll && viewAll && filterUserId) params.set('user_id', filterUserId)
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                `/hr/attendance?${params}`
            )
            return res.data
        },
        enabled: activeTab === 'history',
    })

    // 取得 GPS 座標的工具函式
    const getGpsPosition = (): Promise<{ latitude: number; longitude: number } | null> => {
        return new Promise((resolve) => {
            if (!navigator.geolocation) {
                resolve(null)
                return
            }
            navigator.geolocation.getCurrentPosition(
                (pos) => resolve({ latitude: pos.coords.latitude, longitude: pos.coords.longitude }),
                () => resolve(null), // 使用者拒絕或逾時 → 不帶 GPS
                { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 }
            )
        })
    }

    // 打卡上班
    const clockInMutation = useMutation({
        mutationFn: async () => {
            const gps = await getGpsPosition()
            return api.post<{ success: boolean; clock_in_time: string }>('/hr/attendance/clock-in', {
                source: 'web',
                ...(gps && { latitude: gps.latitude, longitude: gps.longitude }),
            })
        },
        onSuccess: (res) => {
            const clockInTime = res.data.clock_in_time
            // 立即更新 cache，讓 UI 馬上顯示打卡時間
            queryClient.setQueryData(['hr-today-attendance'], (old: AttendanceWithUser | null) => ({
                ...old,
                clock_in_time: clockInTime,
            } as AttendanceWithUser))
            refetchToday()
            toast({
                title: '打卡成功',
                description: `上班打卡時間：${new Date(clockInTime).toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
            })
        },
        onError: (error: unknown) => {
            let description: string
            if (error instanceof AxiosError && error.response?.status === 403) {
                description = (error.response?.data as { error?: { message?: string } })?.error?.message || '請確認您已連接辦公室 WiFi 或允許定位權限'
            } else {
                description = getApiErrorMessage(error, '請稍後再試')
            }
            toast({
                title: '打卡失敗',
                description,
                variant: 'destructive',
            })
        },
    })

    // 打卡下班
    const clockOutMutation = useMutation({
        mutationFn: async () => {
            const gps = await getGpsPosition()
            return api.post<{ success: boolean; clock_out_time: string }>('/hr/attendance/clock-out', {
                source: 'web',
                ...(gps && { latitude: gps.latitude, longitude: gps.longitude }),
            })
        },
        onSuccess: (res) => {
            const clockOutTime = res.data.clock_out_time
            // 立即更新 cache，讓 UI 馬上顯示打卡時間
            queryClient.setQueryData(['hr-today-attendance'], (old: AttendanceWithUser | null) => ({
                ...old,
                clock_out_time: clockOutTime,
            } as AttendanceWithUser))
            refetchToday()
            queryClient.invalidateQueries({ queryKey: ['hr-attendance-history'] })
            toast({
                title: '打卡成功',
                description: `下班打卡時間：${new Date(clockOutTime).toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })}`,
            })
        },
        onError: (error: unknown) => {
            let description: string
            if (error instanceof AxiosError && error.response?.status === 403) {
                description = (error.response?.data as { error?: { message?: string } })?.error?.message || '請確認您已連接辦公室 WiFi 或允許定位權限'
            } else {
                description = getApiErrorMessage(error, '請稍後再試')
            }
            toast({
                title: '打卡失敗',
                description,
                variant: 'destructive',
            })
        },
    })

    // 匯出 Excel
    const exportExcelMutation = useMutation({
        mutationFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            if (canViewAll && viewAll) params.set('view_all', 'true')
            if (canViewAll && viewAll && filterUserId) params.set('user_id', filterUserId)
            const res = await api.get(`/hr/attendance/export?${params.toString()}`, {
                responseType: 'blob',
            })
            return res.data
        },
        onSuccess: (data) => {
            const blob = new Blob([data], {
                type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
            })
            const url = URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `attendance_records_${new Date().toISOString().slice(0, 10)}.xlsx`
            a.click()
            URL.revokeObjectURL(url)
            toast({ title: '匯出成功', description: '出勤記錄已下載' })
        },
        onError: (error: unknown) => {
            toast({
                title: '匯出失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit', weekday: 'long' })
    }

    const formatHours = (hours: number | string | null) => {
        if (hours === null || hours === undefined) return '-'
        const numHours = typeof hours === 'string' ? parseFloat(hours) : hours
        if (isNaN(numHours)) return '-'
        return `${numHours.toFixed(1)} 小時`
    }

    const getStatusBadge = (status: string) => {
        switch (status) {
            case 'normal':
                return <Badge className="bg-green-500">正常</Badge>
            case 'late':
                return <Badge variant="destructive">遲到</Badge>
            case 'early_leave':
                return <Badge className="bg-orange-500">早退</Badge>
            case 'absent':
                return <Badge variant="destructive">缺勤</Badge>
            default:
                return <Badge variant="secondary">{status}</Badge>
        }
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">出勤管理</h1>
                    <p className="text-muted-foreground">打卡與出勤記錄</p>
                </div>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="today" className="flex items-center gap-2">
                        <Clock className="h-4 w-4" />
                        今日打卡
                    </TabsTrigger>
                    <TabsTrigger value="history" className="flex items-center gap-2">
                        <Calendar className="h-4 w-4" />
                        出勤記錄
                    </TabsTrigger>
                </TabsList>

                {/* 今日打卡 */}
                <TabsContent value="today" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>{new Date().toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit', weekday: 'long' })}</CardTitle>
                            <CardDescription>今日出勤狀態</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <div className="grid gap-4 md:grid-cols-2">
                                <Card>
                                    <CardContent className="pt-6">
                                        <div className="text-center space-y-4">
                                            <LogIn className="h-12 w-12 mx-auto text-green-500" />
                                            <div>
                                                <div className="text-sm text-muted-foreground">上班打卡</div>
                                                <div className="text-3xl font-bold">
                                                    {todayAttendance?.clock_in_time
                                                        ? formatTime(todayAttendance.clock_in_time)
                                                        : '--:--:--'}
                                                </div>
                                            </div>
                                            <Button
                                                size="lg"
                                                className="w-full"
                                                disabled={!!todayAttendance?.clock_in_time || clockInMutation.isPending}
                                                onClick={() => clockInMutation.mutate()}
                                            >
                                                <LogIn className="h-4 w-4 mr-2" />
                                                {todayAttendance?.clock_in_time ? '已打卡' : '打卡上班'}
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>

                                <Card>
                                    <CardContent className="pt-6">
                                        <div className="text-center space-y-4">
                                            <LogOut className="h-12 w-12 mx-auto text-red-500" />
                                            <div>
                                                <div className="text-sm text-muted-foreground">下班打卡</div>
                                                <div className="text-3xl font-bold">
                                                    {todayAttendance?.clock_out_time
                                                        ? formatTime(todayAttendance.clock_out_time)
                                                        : '--:--:--'}
                                                </div>
                                            </div>
                                            <Button
                                                size="lg"
                                                variant="outline"
                                                className="w-full"
                                                disabled={
                                                    !todayAttendance?.clock_in_time ||
                                                    !!todayAttendance?.clock_out_time ||
                                                    clockOutMutation.isPending
                                                }
                                                onClick={() => clockOutMutation.mutate()}
                                            >
                                                <LogOut className="h-4 w-4 mr-2" />
                                                {todayAttendance?.clock_out_time ? '已打卡' : '打卡下班'}
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>

                            {todayAttendance && (
                                <div className="grid gap-4 md:grid-cols-3">
                                    <div className="text-center">
                                        <div className="text-sm text-muted-foreground">工作時數</div>
                                        <div className="text-xl font-semibold">
                                            {formatHours(todayAttendance.regular_hours)}
                                        </div>
                                    </div>
                                    <div className="text-center">
                                        <div className="text-sm text-muted-foreground">加班時數</div>
                                        <div className="text-xl font-semibold">
                                            {formatHours(todayAttendance.overtime_hours)}
                                        </div>
                                    </div>
                                    <div className="text-center">
                                        <div className="text-sm text-muted-foreground">狀態</div>
                                        <div className="text-xl">{getStatusBadge(todayAttendance.status)}</div>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* 出勤記錄 */}
                <TabsContent value="history" className="space-y-4">
                    <div className="flex flex-wrap items-end gap-4">
                        <div className="flex flex-col gap-2">
                            <Label>開始日期</Label>
                            <Input
                                type="date"
                                value={dateFrom}
                                onChange={(e) => setDateFrom(e.target.value)}
                                placeholder="開始日期"
                            />
                        </div>
                        <div className="flex flex-col gap-2">
                            <Label>結束日期</Label>
                            <Input
                                type="date"
                                value={dateTo}
                                onChange={(e) => setDateTo(e.target.value)}
                                placeholder="結束日期"
                            />
                        </div>
                        {canViewAll && (
                            <>
                                <div className="flex items-center gap-2">
                                    <Switch
                                        id="view-all"
                                        checked={viewAll}
                                        onCheckedChange={(checked) => {
                                            setViewAll(checked)
                                            if (!checked) setFilterUserId('')
                                        }}
                                    />
                                    <Label htmlFor="view-all" className="cursor-pointer flex items-center gap-2">
                                        <Users className="h-4 w-4" />
                                        查看所有人
                                    </Label>
                                </div>
                                {viewAll && staffList && (
                                    <div className="flex flex-col gap-2">
                                        <Label>篩選人員</Label>
                                        <Select value={filterUserId || 'all'} onValueChange={(v) => setFilterUserId(v === 'all' ? '' : v)}>
                                            <SelectTrigger className="w-[200px]">
                                                <SelectValue placeholder="全部人員" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="all">全部人員</SelectItem>
                                                {staffList.map((s) => (
                                                    <SelectItem key={s.id} value={s.id}>
                                                        {s.display_name}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                )}
                            </>
                        )}
                        <Button
                            variant="outline"
                            onClick={() => queryClient.invalidateQueries({ queryKey: ['hr-attendance-history'] })}
                        >
                            <RefreshCw className="h-4 w-4 mr-2" />
                            重新整理
                        </Button>
                        <Button variant="outline" onClick={() => exportExcelMutation.mutate()} disabled={exportExcelMutation.isPending}>
                            <Download className="h-4 w-4 mr-2" />
                            {exportExcelMutation.isPending ? '匯出中...' : '匯出 Excel'}
                        </Button>
                    </div>

                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>日期</TableHead>
                                    <TableHead>人員名稱</TableHead>
                                    <TableHead>上班</TableHead>
                                    <TableHead>下班</TableHead>
                                    <TableHead>工作時數</TableHead>
                                    <TableHead>加班時數</TableHead>
                                    <TableHead>狀態</TableHead>
                                    <TableHead>備註</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingHistory ? (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : attendanceHistory?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                            沒有出勤記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    attendanceHistory?.data?.map((record) => (
                                        <TableRow key={record.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDate(record.work_date)}
                                            </TableCell>
                                            <TableCell className="font-medium">
                                                {record.user_name}
                                            </TableCell>
                                            <TableCell>{formatTime(record.clock_in_time)}</TableCell>
                                            <TableCell>{formatTime(record.clock_out_time)}</TableCell>
                                            <TableCell>{formatHours(record.regular_hours)}</TableCell>
                                            <TableCell>{formatHours(record.overtime_hours)}</TableCell>
                                            <TableCell>{getStatusBadge(record.status)}</TableCell>
                                            <TableCell>
                                                {record.is_corrected && (
                                                    <Badge variant="outline">已更正</Badge>
                                                )}
                                                {record.remark && (
                                                    <span className="text-muted-foreground text-sm">{record.remark}</span>
                                                )}
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}

export default HrAttendancePage
