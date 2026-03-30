import { LogIn, LogOut } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { formatTime } from '@/lib/utils'
import type { AttendanceWithUser } from '@/types/hr'

function formatHours(hours: number | string | null) {
    if (hours === null || hours === undefined) return '-'
    const numHours = typeof hours === 'string' ? parseFloat(hours) : hours
    if (isNaN(numHours)) return '-'
    return `${numHours.toFixed(1)} 小時`
}

function getStatusBadge(status: string) {
    switch (status) {
        case 'normal': return <StatusBadge variant="success">正常</StatusBadge>
        case 'late': return <StatusBadge variant="error">遲到</StatusBadge>
        case 'early_leave': return <StatusBadge variant="warning">早退</StatusBadge>
        case 'absent': return <StatusBadge variant="error">缺勤</StatusBadge>
        default: return <StatusBadge variant="neutral">{status}</StatusBadge>
    }
}

interface TodayClockTabProps {
    todayAttendance: AttendanceWithUser | null | undefined
    clockInPending: boolean
    clockOutPending: boolean
    onClockIn: () => void
    onClockOut: () => void
}

export function TodayClockTab({ todayAttendance, clockInPending, clockOutPending, onClockIn, onClockOut }: TodayClockTabProps) {
    return (
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
                                <LogIn className="h-12 w-12 mx-auto text-status-success-text" />
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
                                    disabled={!!todayAttendance?.clock_in_time || clockInPending}
                                    onClick={onClockIn}
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
                                <LogOut className="h-12 w-12 mx-auto text-status-error-text" />
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
                                        clockOutPending
                                    }
                                    onClick={onClockOut}
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
                            <div className="text-xl font-semibold">{formatHours(todayAttendance.regular_hours)}</div>
                        </div>
                        <div className="text-center">
                            <div className="text-sm text-muted-foreground">加班時數</div>
                            <div className="text-xl font-semibold">{formatHours(todayAttendance.overtime_hours)}</div>
                        </div>
                        <div className="text-center">
                            <div className="text-sm text-muted-foreground">狀態</div>
                            <div className="text-xl">{getStatusBadge(todayAttendance.status)}</div>
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
