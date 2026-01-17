import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    Calendar,
    Clock,
    LogIn,
    LogOut,
    RefreshCw,
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
import { toast } from '@/components/ui/use-toast'
import type { AttendanceWithUser } from '@/types/hr'

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

export function HrAttendancePage() {
    const [activeTab, setActiveTab] = useState('today')
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
    const queryClient = useQueryClient()

    // 今日打卡狀態
    const { data: todayAttendance, refetch: refetchToday } = useQuery({
        queryKey: ['hr-today-attendance'],
        queryFn: async () => {
            const today = format(new Date(), 'yyyy-MM-dd')
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                `/hr/attendance?from=${today}&to=${today}`
            )
            return res.data.data[0] || null
        },
    })

    // 歷史記錄
    const { data: attendanceHistory, isLoading: loadingHistory } = useQuery({
        queryKey: ['hr-attendance-history', dateFrom, dateTo],
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(
                `/hr/attendance?${params}`
            )
            return res.data
        },
        enabled: activeTab === 'history',
    })

    // 打卡上班
    const clockInMutation = useMutation({
        mutationFn: async () => {
            return api.post('/hr/attendance/clock-in', { source: 'web' })
        },
        onSuccess: (res) => {
            refetchToday()
            toast({
                title: '打卡成功',
                description: `上班打卡時間：${format(new Date(), 'HH:mm:ss')}`,
            })
        },
        onError: (error: any) => {
            toast({
                title: '打卡失敗',
                description: error?.response?.data?.error?.message || '請稍後再試',
                variant: 'destructive',
            })
        },
    })

    // 打卡下班
    const clockOutMutation = useMutation({
        mutationFn: async () => {
            return api.post('/hr/attendance/clock-out', { source: 'web' })
        },
        onSuccess: (res) => {
            refetchToday()
            queryClient.invalidateQueries({ queryKey: ['hr-attendance-history'] })
            toast({
                title: '打卡成功',
                description: `下班打卡時間：${format(new Date(), 'HH:mm:ss')}`,
            })
        },
        onError: (error: any) => {
            toast({
                title: '打卡失敗',
                description: error?.response?.data?.error?.message || '請稍後再試',
                variant: 'destructive',
            })
        },
    })

    const formatTime = (dateStr: string | null) => {
        if (!dateStr) return '-'
        return format(new Date(dateStr), 'HH:mm:ss', { locale: zhTW })
    }

    const formatDate = (dateStr: string) => {
        return format(new Date(dateStr), 'yyyy/MM/dd (EEEE)', { locale: zhTW })
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
                            <CardTitle>{format(new Date(), 'yyyy年MM月dd日 EEEE', { locale: zhTW })}</CardTitle>
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
                        <Button
                            variant="outline"
                            onClick={() => queryClient.invalidateQueries({ queryKey: ['hr-attendance-history'] })}
                        >
                            <RefreshCw className="h-4 w-4 mr-2" />
                            重新整理
                        </Button>
                    </div>

                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>日期</TableHead>
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
                                        <TableCell colSpan={7} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : attendanceHistory?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                            沒有出勤記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    attendanceHistory?.data?.map((record) => (
                                        <TableRow key={record.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDate(record.work_date)}
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
