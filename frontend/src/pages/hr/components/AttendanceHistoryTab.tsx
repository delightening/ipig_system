import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Clock, Download, Pencil, Plus, RefreshCw, Users } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_ATTENDANCE } from '@/lib/guest-demo'
import { useAuthHasPermission } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { StatusBadge } from '@/components/ui/status-badge'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import {
    Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { GuestHide } from '@/components/ui/guest-hide'
import { formatDate, formatTime } from '@/lib/utils'
import type { AttendanceWithUser, StaffInfo } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { useAttendanceMutations } from '../hooks/useAttendanceMutations'
import { AttendanceCorrectionDialog } from './AttendanceCorrectionDialog'

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

export function AttendanceHistoryTab() {
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
    const [viewAll, setViewAll] = useState(false)
    const [filterUserId, setFilterUserId] = useState<string>('')
    const queryClient = useQueryClient()
    const hasPermission = useAuthHasPermission()
    const canViewAll = hasPermission('hr.attendance.view_all')
    // 補卡權限（`hr.attendance.correct`）：無權限者整個「操作」欄不渲染，不佔欄寬。
    // 後端 `require_permission!` 才是真正的閘門，這裡只是不給按不動的按鈕。
    const canCorrect = hasPermission('hr.attendance.correct')

    // 對話框：`editingRecord === null` 且 dialogOpen ＝ 補登模式；有值＝更正該列
    const [dialogOpen, setDialogOpen] = useState(false)
    const [editingRecord, setEditingRecord] = useState<AttendanceWithUser | null>(null)

    const openBackfill = () => {
        setEditingRecord(null)
        setDialogOpen(true)
    }
    const openCorrection = (record: AttendanceWithUser) => {
        setEditingRecord(record)
        setDialogOpen(true)
    }

    const columnCount = canCorrect ? 9 : 8

    const { data: staffList } = useQuery({
        queryKey: queryKeys.hr.staffForAttendance,
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
        // 補卡對話框的人員下拉也吃這份清單，故 canCorrect 也要拉
        enabled: canViewAll || canCorrect,
    })

    const { data: attendanceHistory, isLoading: loadingHistory } = useGuestQuery(DEMO_ATTENDANCE, {
        queryKey: queryKeys.hr.attendanceHistory({ dateFrom, dateTo, viewAll, filterUserId }),
        queryFn: async () => {
            const params = new URLSearchParams()
            if (dateFrom) params.set('from', dateFrom)
            if (dateTo) params.set('to', dateTo)
            if (canViewAll && viewAll) params.set('view_all', 'true')
            if (canViewAll && viewAll && filterUserId) params.set('user_id', filterUserId)
            const res = await api.get<PaginatedResponse<AttendanceWithUser>>(`/hr/attendance?${params}`)
            return res.data
        },
    })

    const { exportExcelMutation } = useAttendanceMutations({
        refetchToday: () => {},
        canViewAll,
        dateFrom,
        dateTo,
        viewAll,
        filterUserId,
    })

    const { sortedData: sortedHistory, sort: historySort, toggleSort: toggleHistorySort } = useTableSort(attendanceHistory?.data)

    return (
        <div className="space-y-4">
            <div className="flex flex-wrap items-end gap-4">
                <div className="flex flex-col gap-2">
                    <Label>開始日期</Label>
                    <Input type="date" value={dateFrom} onChange={(e) => setDateFrom(e.target.value)} placeholder="開始日期" />
                </div>
                <div className="flex flex-col gap-2">
                    <Label>結束日期</Label>
                    <Input type="date" value={dateTo} onChange={(e) => setDateTo(e.target.value)} placeholder="結束日期" />
                </div>
                {canViewAll && (
                    <>
                        <div className="flex items-center gap-2">
                            <Switch id="view-all" checked={viewAll} onCheckedChange={(checked) => { setViewAll(checked); if (!checked) setFilterUserId('') }} />
                            <Label htmlFor="view-all" className="cursor-pointer flex items-center gap-2">
                                <Users className="h-4 w-4" />
                                查看所有人
                            </Label>
                        </div>
                        {viewAll && staffList && (
                            <div className="flex flex-col gap-2">
                                <Label>篩選人員</Label>
                                <Select value={filterUserId || 'all'} onValueChange={(v) => setFilterUserId(v === 'all' ? '' : v)}>
                                    <SelectTrigger className="w-[200px]"><SelectValue placeholder="全部人員" /></SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="all">全部人員</SelectItem>
                                        {staffList.map((s) => (
                                            <SelectItem key={s.id} value={s.id}>{s.display_name}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        )}
                    </>
                )}
                <Button variant="outline" onClick={() => queryClient.invalidateQueries({ queryKey: queryKeys.hr.allAttendanceHistory })}>
                    <RefreshCw className="h-4 w-4 mr-2" />
                    重新整理
                </Button>
                <GuestHide>
                    <Button variant="outline" onClick={() => exportExcelMutation.mutate()} disabled={exportExcelMutation.isPending}>
                        <Download className="h-4 w-4 mr-2" />
                        {exportExcelMutation.isPending ? '匯出中...' : '匯出 Excel'}
                    </Button>
                </GuestHide>
                {canCorrect && (
                    <GuestHide>
                        <Button onClick={openBackfill}>
                            <Plus className="h-4 w-4 mr-2" />
                            補卡
                        </Button>
                    </GuestHide>
                )}
            </div>

            <Card className="@container overflow-hidden">
                <div className="hidden @[600px]:block">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <SortableTableHead sortKey="work_date" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>日期</SortableTableHead>
                                <SortableTableHead sortKey="user_name" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>人員名稱</SortableTableHead>
                                <SortableTableHead className="hidden @[750px]:table-cell" sortKey="clock_in_time" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>上班</SortableTableHead>
                                <SortableTableHead className="hidden @[750px]:table-cell" sortKey="clock_out_time" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>下班</SortableTableHead>
                                <SortableTableHead className="hidden @[900px]:table-cell" sortKey="regular_hours" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>工作時數</SortableTableHead>
                                <SortableTableHead className="hidden @[900px]:table-cell" sortKey="overtime_hours" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>加班時數</SortableTableHead>
                                <SortableTableHead sortKey="status" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>狀態</SortableTableHead>
                                <TableHead className="hidden @[1050px]:table-cell">備註</TableHead>
                                {canCorrect && <TableHead className="w-20">操作</TableHead>}
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {loadingHistory ? (
                                <TableRow>
                                    <TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={5} cols={columnCount} /></TableCell>
                                </TableRow>
                            ) : sortedHistory?.length === 0 ? (
                                <TableEmptyRow colSpan={columnCount} icon={Clock} title="沒有出勤記錄" />
                            ) : (
                                sortedHistory?.map((record) => (
                                    <TableRow key={record.id}>
                                        <TableCell className="whitespace-nowrap">{formatDate(record.work_date, { weekday: true })}</TableCell>
                                        <TableCell className="font-medium">{record.user_name}</TableCell>
                                        <TableCell className="hidden @[750px]:table-cell">{formatTime(record.clock_in_time)}</TableCell>
                                        <TableCell className="hidden @[750px]:table-cell">{formatTime(record.clock_out_time)}</TableCell>
                                        <TableCell className="hidden @[900px]:table-cell">{formatHours(record.regular_hours)}</TableCell>
                                        <TableCell className="hidden @[900px]:table-cell">{formatHours(record.overtime_hours)}</TableCell>
                                        <TableCell>{getStatusBadge(record.status)}</TableCell>
                                        <TableCell className="hidden @[1050px]:table-cell">
                                            {record.is_corrected && <Badge variant="outline">已更正</Badge>}
                                            {record.remark && <span className="text-muted-foreground text-sm">{record.remark}</span>}
                                        </TableCell>
                                        {canCorrect && (
                                            <TableCell>
                                                <GuestHide>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        aria-label={`更正 ${record.user_name} ${record.work_date} 的出勤記錄`}
                                                        onClick={() => openCorrection(record)}
                                                    >
                                                        <Pencil className="h-4 w-4" />
                                                    </Button>
                                                </GuestHide>
                                            </TableCell>
                                        )}
                                    </TableRow>
                                ))
                            )}
                        </TableBody>
                    </Table>
                </div>

                <div className="@[600px]:hidden divide-y">
                    {loadingHistory ? (
                        <div className="p-3"><TableSkeleton rows={3} cols={1} /></div>
                    ) : sortedHistory?.length === 0 ? (
                        <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                            <Clock className="h-8 w-8" />
                            <p className="text-sm">沒有出勤記錄</p>
                        </div>
                    ) : (
                        sortedHistory?.map((record) => (
                            <div key={record.id} className="p-3 space-y-1">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="min-w-0">
                                        <div className="font-medium break-words">{record.user_name}</div>
                                        <div className="text-xs text-muted-foreground">{formatDate(record.work_date, { weekday: true })}</div>
                                    </div>
                                    {getStatusBadge(record.status)}
                                </div>
                                <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3">
                                    <span>上 {formatTime(record.clock_in_time)}</span>
                                    <span>下 {formatTime(record.clock_out_time)}</span>
                                    <span>工時 {formatHours(record.regular_hours)}</span>
                                    {record.overtime_hours && Number(record.overtime_hours) > 0 && (
                                        <span>加班 {formatHours(record.overtime_hours)}</span>
                                    )}
                                </div>
                                {(record.is_corrected || record.remark) && (
                                    <div className="flex items-center gap-2 text-xs">
                                        {record.is_corrected && <Badge variant="outline">已更正</Badge>}
                                        {record.remark && <span className="text-muted-foreground break-words">{record.remark}</span>}
                                    </div>
                                )}
                                {canCorrect && (
                                    <GuestHide>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            aria-label={`更正 ${record.user_name} ${record.work_date} 的出勤記錄`}
                                            onClick={() => openCorrection(record)}
                                        >
                                            <Pencil className="h-4 w-4" />
                                        </Button>
                                    </GuestHide>
                                )}
                            </div>
                        ))
                    )}
                </div>
            </Card>

            {canCorrect && (
                <AttendanceCorrectionDialog
                    open={dialogOpen}
                    onOpenChange={setDialogOpen}
                    record={editingRecord}
                    staffList={staffList}
                />
            )}
        </div>
    )
}
