import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { LogIn, LogOut } from 'lucide-react'

import { useAuthIsGuest } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { formatTime, uiLocale } from '@/lib/utils'
import type { AttendanceWithUser } from '@/types/hr'

function formatHours(t: TFunction, hours: number | string | null) {
    if (hours === null || hours === undefined) return '-'
    const numHours = typeof hours === 'string' ? parseFloat(hours) : hours
    if (isNaN(numHours)) return '-'
    return t('hrPages.shared.hoursValue', { hours: numHours.toFixed(1) })
}

function getStatusBadge(t: TFunction, status: string) {
    switch (status) {
        case 'normal': return <StatusBadge variant="success">{t('hrPages.shared.attendanceStatus.normal')}</StatusBadge>
        case 'late': return <StatusBadge variant="error">{t('hrPages.shared.attendanceStatus.late')}</StatusBadge>
        case 'early_leave': return <StatusBadge variant="warning">{t('hrPages.shared.attendanceStatus.early_leave')}</StatusBadge>
        case 'absent': return <StatusBadge variant="error">{t('hrPages.shared.attendanceStatus.absent')}</StatusBadge>
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
    const { t } = useTranslation()
    // R49 follow-up：guest 看到完整頁面 + 按鈕，但按鈕 disabled 並提示 demo 限制。
    // 採 Gemini review：() 寫在 selector 內讓 Zustand 訂閱 boolean 結果而非函式 ref，
    // 避免使用者狀態變動時組件不 re-render。
    const isGuest = useAuthIsGuest()
    const guestTitle = isGuest ? t('hrPages.attendance.today.guestDemoTitle') : undefined
    return (
        <Card>
            <CardHeader>
                <CardTitle>{new Date().toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit', weekday: 'long' })}</CardTitle>
                <CardDescription>{t('hrPages.attendance.today.statusDescription')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
                <div className="grid gap-4 md:grid-cols-2">
                    <Card>
                        <CardContent className="pt-6">
                            <div className="text-center space-y-4">
                                <LogIn className="h-12 w-12 mx-auto text-status-success-text" />
                                <div>
                                    <div className="text-sm text-muted-foreground">{t('hrPages.attendance.today.clockInLabel')}</div>
                                    <div className="text-3xl font-bold">
                                        {todayAttendance?.clock_in_time
                                            ? formatTime(todayAttendance.clock_in_time)
                                            : '--:--:--'}
                                    </div>
                                </div>
                                <Button
                                    size="lg"
                                    className="w-full"
                                    disabled={isGuest || !!todayAttendance?.clock_in_time || clockInPending}
                                    onClick={onClockIn}
                                    title={guestTitle}
                                >
                                    <LogIn className="h-4 w-4 mr-2" />
                                    {todayAttendance?.clock_in_time ? t('hrPages.attendance.today.alreadyClocked') : t('hrPages.attendance.today.clockInButton')}
                                </Button>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardContent className="pt-6">
                            <div className="text-center space-y-4">
                                <LogOut className="h-12 w-12 mx-auto text-status-error-text" />
                                <div>
                                    <div className="text-sm text-muted-foreground">{t('hrPages.attendance.today.clockOutLabel')}</div>
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
                                        isGuest ||
                                        !todayAttendance?.clock_in_time ||
                                        !!todayAttendance?.clock_out_time ||
                                        clockOutPending
                                    }
                                    onClick={onClockOut}
                                    title={guestTitle}
                                >
                                    <LogOut className="h-4 w-4 mr-2" />
                                    {todayAttendance?.clock_out_time ? t('hrPages.attendance.today.alreadyClocked') : t('hrPages.attendance.today.clockOutButton')}
                                </Button>
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {todayAttendance && (
                    <div className="grid gap-4 md:grid-cols-3">
                        <div className="text-center">
                            <div className="text-sm text-muted-foreground">{t('hrPages.attendance.today.regularHours')}</div>
                            <div className="text-xl font-semibold">{formatHours(t, todayAttendance.regular_hours)}</div>
                        </div>
                        <div className="text-center">
                            <div className="text-sm text-muted-foreground">{t('hrPages.attendance.today.overtimeHours')}</div>
                            <div className="text-xl font-semibold">{formatHours(t, todayAttendance.overtime_hours)}</div>
                        </div>
                        <div className="text-center">
                            <div className="text-sm text-muted-foreground">{t('hrPages.shared.col.status')}</div>
                            <div className="text-xl">{getStatusBadge(t, todayAttendance.status)}</div>
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
