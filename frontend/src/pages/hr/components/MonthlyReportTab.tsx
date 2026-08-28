import { useState } from 'react'
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
function formatHours(hours: number | null) {
    if (hours === null || hours === undefined || Number.isNaN(hours)) return '-'
    return `${hours.toFixed(1)} 小時`
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
            if (!period) throw new Error('請先選擇月份')
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
            toast({ title: '匯出成功', description: '工時月報已下載' })
        },
        onError: (error: unknown) => {
            toast({
                title: '匯出失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
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
                    <Label htmlFor="report-month">月份</Label>
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
                        <Label>篩選人員</Label>
                        <Select
                            value={filterUserId || 'all'}
                            onValueChange={(v) => setFilterUserId(v === 'all' ? '' : v)}
                        >
                            <SelectTrigger className="w-[200px]">
                                <SelectValue placeholder="全部人員" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">全部人員</SelectItem>
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
                    重新整理
                </Button>
                <GuestHide>
                    <Button
                        variant="outline"
                        onClick={() => exportMutation.mutate()}
                        disabled={exportMutation.isPending || !period}
                    >
                        <Download className="h-4 w-4 mr-2" />
                        {exportMutation.isPending ? '匯出中...' : '匯出 Excel'}
                    </Button>
                </GuestHide>
            </div>

            <Card className="@container overflow-hidden">
                <div className="hidden @[600px]:block">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>人員名稱</SortableTableHead>
                                <SortableTableHead sortKey="work_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>出勤天數</SortableTableHead>
                                <SortableTableHead sortKey="total_regular_hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>總工時</SortableTableHead>
                                <SortableTableHead sortKey="total_overtime_hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>總加班時數</SortableTableHead>
                                <SortableTableHead className="hidden @[1000px]:table-cell" sortKey="incomplete_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>打卡不完整天數</SortableTableHead>
                                <SortableTableHead className="hidden @[1000px]:table-cell" sortKey="corrected_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>補登／更正天數</SortableTableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={6} className="p-0"><TableSkeleton rows={5} cols={6} /></TableCell>
                                </TableRow>
                            ) : visibleRows.length === 0 ? (
                                <TableEmptyRow colSpan={6} icon={CalendarDays} title="當月沒有出勤資料" />
                            ) : (
                                <>
                                    {visibleRows.map((r) => (
                                        <TableRow key={r.user_id}>
                                            <TableCell className="font-medium">{r.user_name}</TableCell>
                                            <TableCell>{r.work_days} 天</TableCell>
                                            <TableCell>{formatHours(r.total_regular_hours)}</TableCell>
                                            <TableCell>{formatHours(r.total_overtime_hours)}</TableCell>
                                            <TableCell className="hidden @[1000px]:table-cell">{r.incomplete_days} 天</TableCell>
                                            <TableCell className="hidden @[1000px]:table-cell">{r.corrected_days} 天</TableCell>
                                        </TableRow>
                                    ))}
                                    <TableRow className="font-medium">
                                        <TableCell>合計</TableCell>
                                        <TableCell>{sumBy(visibleRows, (r) => r.work_days)} 天</TableCell>
                                        <TableCell>{formatHours(sumBy(visibleRows, (r) => r.total_regular_hours))}</TableCell>
                                        <TableCell>{formatHours(sumBy(visibleRows, (r) => r.total_overtime_hours))}</TableCell>
                                        <TableCell className="hidden @[1000px]:table-cell">{sumBy(visibleRows, (r) => r.incomplete_days)} 天</TableCell>
                                        <TableCell className="hidden @[1000px]:table-cell">{sumBy(visibleRows, (r) => r.corrected_days)} 天</TableCell>
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
                            <p className="text-sm">當月沒有出勤資料</p>
                        </div>
                    ) : (
                        visibleRows.map((r) => (
                            <div key={r.user_id} className="p-3 space-y-1">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="font-medium break-words">{r.user_name}</div>
                                    <div className="text-sm whitespace-nowrap">{r.work_days} 天</div>
                                </div>
                                <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3">
                                    <span>工時 {formatHours(r.total_regular_hours)}</span>
                                    <span>加班 {formatHours(r.total_overtime_hours)}</span>
                                    {/* 標籤與桌機表頭一致：這個數字含「更正既有紀錄」，
                                        只寫「補登」會漏掉一半語意（CodeRabbit PR #35） */}
                                    <span>打卡不完整 {r.incomplete_days} 天</span>
                                    <span>補登／更正 {r.corrected_days} 天</span>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </Card>

            <p className="text-xs text-muted-foreground">
                「打卡不完整」指當日只有上班或只有下班卡，那些日子的工時不完整，是補卡的待辦清單。
                加班時數來自加班卡的已核准紀錄，與總工時分開計算；匯出的 Excel 會分成
                「正常出勤」與「加班」兩個工作表。
                本報表不含遲到／早退計數：系統目前沒有上下班時間基準，那兩個數字會恆為 0。
            </p>
        </div>
    )
}
