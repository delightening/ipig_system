import { useState } from 'react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CalendarDays, Download, RefreshCw } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_MONTHLY_REPORT } from '@/lib/guest-demo'
import { useAuthHasPermission } from '@/stores/auth'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import {
    Table, TableBody, TableCell, TableHeader, TableRow,
} from '@/components/ui/table'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { GuestHide } from '@/components/ui/guest-hide'
import type { MonthlyAttendanceSummary, StaffInfo } from '@/types/hr'

/** 台灣時區的當月 `yyyy-MM`，供 `<input type="month">` 的預設值 */
function taipeiCurrentMonth(): string {
    return new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' }).slice(0, 7)
}

/** `yyyy-MM` → `{ year, month }`；格式不正確時回 null（月份輸入框可被清空） */
function parseMonthInput(value: string): { year: number; month: number } | null {
    const match = /^(\d{4})-(\d{2})$/.exec(value)
    if (!match) return null
    const year = Number(match[1])
    const month = Number(match[2])
    if (month < 1 || month > 12) return null
    return { year, month }
}

/**
 * 後端的工時欄位在 SQL 端已 `::float8`，回來就是 JSON number（不是 rust_decimal
 * 預設的字串），所以這裡不需要 parseFloat 的防禦分支——`AttendanceHistoryTab`
 * 那份有，是因為它吃的 `regular_hours` 仍是 Decimal。見 `MonthlyAttendanceSummary`
 * 的後端註解（CodeRabbit PR #35）。
 */
function formatHours(t: TFunction, hours: number | null) {
    if (hours === null || hours === undefined || Number.isNaN(hours)) return '-'
    return t('hrPages.shared.hoursValue', { hours: hours.toFixed(1) })
}

/** 合計列：以顯示中的資料相加，與後端匯出的 Excel 合計列同語意（含每一個數值欄） */
function sumBy(rows: MonthlyAttendanceSummary[], pick: (r: MonthlyAttendanceSummary) => number) {
    return rows.reduce((acc, r) => acc + (pick(r) || 0), 0)
}

/**
 * 工時月報：某年月每人一列的工時合計。
 *
 * RWD 依 `/system_table_chats` 2026-08-26 裁定：斷點 1000 / 600。
 * ≥1000px 全 6 欄（710px）；600–1000px 隱藏「打卡不完整」「補登／更正」（450px）；
 * <600px 改 card layout。Email 依裁定不進表格，只出現在匯出的 Excel。
 */
export function MonthlyReportTab() {
    const { t } = useTranslation()
    const [monthInput, setMonthInput] = useState(taipeiCurrentMonth())
    const [filterUserId, setFilterUserId] = useState<string>('')
    const queryClient = useQueryClient()
    const hasPermission = useAuthHasPermission()
    const canViewAll = hasPermission('hr.attendance.view_all')

    const period = parseMonthInput(monthInput)

    const { data: staffList } = useQuery({
        queryKey: queryKeys.hr.staffForAttendance,
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
        enabled: canViewAll,
    })

    const { data: rows, isLoading } = useGuestQuery(DEMO_MONTHLY_REPORT, {
        queryKey: queryKeys.hr.monthlyReport({ monthInput, filterUserId }),
        queryFn: async () => {
            if (!period) return []
            const params = new URLSearchParams({
                year: String(period.year),
                month: String(period.month),
            })
            if (canViewAll && filterUserId) params.set('user_id', filterUserId)
            const res = await api.get<MonthlyAttendanceSummary[]>(`/hr/attendance/monthly-report?${params}`)
            return res.data
        },
        enabled: period !== null,
    })

    const exportMutation = useMutation({
        mutationFn: async () => {
            if (!period) throw new Error(t('hrPages.attendance.monthly.selectMonthFirst'))
            const params = new URLSearchParams({
                year: String(period.year),
                month: String(period.month),
            })
            if (canViewAll && filterUserId) params.set('user_id', filterUserId)
            const res = await api.get(`/hr/attendance/monthly-report/export?${params}`, {
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
            a.download = `attendance_monthly_${monthInput}.xlsx`
            a.click()
            URL.revokeObjectURL(url)
            toast({ title: t('common.exportSuccess'), description: t('hrPages.attendance.monthly.exportDownloaded') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.exportFailed'),
                description: getApiErrorMessage(error, t('hrPages.shared.tryAgainLater')),
                variant: 'destructive',
            })
        },
    })

    const { sortedData, sort, toggleSort } = useTableSort(rows)
    const visibleRows = sortedData ?? []

    return (
        <div className="space-y-4">
            <div className="flex flex-wrap items-end gap-4">
                <div className="flex flex-col gap-2">
                    <Label htmlFor="report-month">{t('hrPages.attendance.monthly.month')}</Label>
                    <Input
                        id="report-month"
                        type="month"
                        value={monthInput}
                        max={taipeiCurrentMonth()}
                        onChange={(e) => setMonthInput(e.target.value)}
                    />
                </div>
                {canViewAll && staffList && (
                    <div className="flex flex-col gap-2">
                        <Label>{t('hrPages.shared.filter.filterStaff')}</Label>
                        <Select
                            value={filterUserId || 'all'}
                            onValueChange={(v) => setFilterUserId(v === 'all' ? '' : v)}
                        >
                            <SelectTrigger className="w-[200px]">
                                <SelectValue placeholder={t('hrPages.shared.filter.allStaff')} />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">{t('hrPages.shared.filter.allStaff')}</SelectItem>
                                {staffList.map((s) => (
                                    <SelectItem key={s.id} value={s.id}>{s.display_name}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                )}
                <Button
                    variant="outline"
                    onClick={() => queryClient.invalidateQueries({ queryKey: queryKeys.hr.allMonthlyReport })}
                >
                    <RefreshCw className="h-4 w-4 mr-2" />
                    {t('hrPages.shared.action.refresh')}
                </Button>
                <GuestHide>
                    <Button
                        variant="outline"
                        onClick={() => exportMutation.mutate()}
                        disabled={exportMutation.isPending || !period}
                    >
                        <Download className="h-4 w-4 mr-2" />
                        {exportMutation.isPending ? t('hrPages.attendance.history.exporting') : t('hrPages.attendance.history.exportExcel')}
                    </Button>
                </GuestHide>
            </div>

            <Card className="@container overflow-hidden">
                <div className="hidden @[600px]:block">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.shared.col.staffName')}</SortableTableHead>
                                <SortableTableHead sortKey="work_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.attendance.monthly.workDays')}</SortableTableHead>
                                <SortableTableHead sortKey="total_regular_hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.attendance.monthly.totalRegularHours')}</SortableTableHead>
                                <SortableTableHead sortKey="total_overtime_hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.attendance.monthly.totalOvertimeHours')}</SortableTableHead>
                                <SortableTableHead className="hidden @[1000px]:table-cell" sortKey="incomplete_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.attendance.monthly.incompleteDays')}</SortableTableHead>
                                <SortableTableHead className="hidden @[1000px]:table-cell" sortKey="corrected_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('hrPages.attendance.monthly.correctedDays')}</SortableTableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={6} className="p-0"><TableSkeleton rows={5} cols={6} /></TableCell>
                                </TableRow>
                            ) : visibleRows.length === 0 ? (
                                <TableEmptyRow colSpan={6} icon={CalendarDays} title={t('hrPages.attendance.monthly.empty')} />
                            ) : (
                                <>
                                    {visibleRows.map((r) => (
                                        <TableRow key={r.user_id}>
                                            <TableCell className="font-medium">{r.user_name}</TableCell>
                                            <TableCell>{t('hrPages.shared.daysValue', { days: r.work_days })}</TableCell>
                                            <TableCell>{formatHours(t, r.total_regular_hours)}</TableCell>
                                            <TableCell>{formatHours(t, r.total_overtime_hours)}</TableCell>
                                            <TableCell className="hidden @[1000px]:table-cell">{t('hrPages.shared.daysValue', { days: r.incomplete_days })}</TableCell>
                                            <TableCell className="hidden @[1000px]:table-cell">{t('hrPages.shared.daysValue', { days: r.corrected_days })}</TableCell>
                                        </TableRow>
                                    ))}
                                    <TableRow className="font-medium">
                                        <TableCell>{t('hrPages.attendance.monthly.total')}</TableCell>
                                        <TableCell>{t('hrPages.shared.daysValue', { days: sumBy(visibleRows, (r) => r.work_days) })}</TableCell>
                                        <TableCell>{formatHours(t, sumBy(visibleRows, (r) => r.total_regular_hours))}</TableCell>
                                        <TableCell>{formatHours(t, sumBy(visibleRows, (r) => r.total_overtime_hours))}</TableCell>
                                        <TableCell className="hidden @[1000px]:table-cell">{t('hrPages.shared.daysValue', { days: sumBy(visibleRows, (r) => r.incomplete_days) })}</TableCell>
                                        <TableCell className="hidden @[1000px]:table-cell">{t('hrPages.shared.daysValue', { days: sumBy(visibleRows, (r) => r.corrected_days) })}</TableCell>
                                    </TableRow>
                                </>
                            )}
                        </TableBody>
                    </Table>
                </div>

                <div className="@[600px]:hidden divide-y">
                    {isLoading ? (
                        <div className="p-3"><TableSkeleton rows={3} cols={1} /></div>
                    ) : visibleRows.length === 0 ? (
                        <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                            <CalendarDays className="h-8 w-8" />
                            <p className="text-sm">{t('hrPages.attendance.monthly.empty')}</p>
                        </div>
                    ) : (
                        visibleRows.map((r) => (
                            <div key={r.user_id} className="p-3 space-y-1">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="font-medium break-words">{r.user_name}</div>
                                    <div className="text-sm whitespace-nowrap">{t('hrPages.shared.daysValue', { days: r.work_days })}</div>
                                </div>
                                <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3">
                                    <span>{t('hrPages.attendance.history.workHoursShort', { hours: formatHours(t, r.total_regular_hours) })}</span>
                                    <span>{t('hrPages.attendance.history.overtimeShort', { hours: formatHours(t, r.total_overtime_hours) })}</span>
                                    {/* 標籤與桌機表頭一致：這個數字含「更正既有紀錄」，
                                        只寫「補登」會漏掉一半語意（CodeRabbit PR #35） */}
                                    <span>{t('hrPages.attendance.monthly.incompleteShort', { days: r.incomplete_days })}</span>
                                    <span>{t('hrPages.attendance.monthly.correctedShort', { days: r.corrected_days })}</span>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </Card>

            <p className="text-xs text-muted-foreground">
                {t('hrPages.attendance.monthly.footnote')}
            </p>
        </div>
    )
}
