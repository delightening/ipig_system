import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Clock, Download, RefreshCw, Users } from 'lucide-react'

import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useAuthStore } from '@/stores/auth'
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
import { formatDate, formatTime } from '@/lib/utils'
import type { AttendanceWithUser, StaffInfo } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { useAttendanceMutations } from '../hooks/useAttendanceMutations'

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
    const { hasPermission } = useAuthStore()
    const canViewAll = hasPermission('hr.attendance.view_all')

    const { data: staffList } = useQuery({
        queryKey: queryKeys.hr.staffForAttendance,
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
        enabled: canViewAll,
    })

    const { data: attendanceHistory, isLoading: loadingHistory } = useQuery({
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
                <Button variant="outline" onClick={() => exportExcelMutation.mutate()} disabled={exportExcelMutation.isPending}>
                    <Download className="h-4 w-4 mr-2" />
                    {exportExcelMutation.isPending ? '匯出中...' : '匯出 Excel'}
                </Button>
            </div>

            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead sortKey="work_date" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>日期</SortableTableHead>
                            <SortableTableHead sortKey="user_name" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>人員名稱</SortableTableHead>
                            <SortableTableHead sortKey="clock_in_time" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>上班</SortableTableHead>
                            <SortableTableHead sortKey="clock_out_time" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>下班</SortableTableHead>
                            <SortableTableHead sortKey="regular_hours" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>工作時數</SortableTableHead>
                            <SortableTableHead sortKey="overtime_hours" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>加班時數</SortableTableHead>
                            <SortableTableHead sortKey="status" currentSort={historySort.column} currentDirection={historySort.direction} onSort={toggleHistorySort}>狀態</SortableTableHead>
                            <TableHead>備註</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {loadingHistory ? (
                            <TableRow>
                                <TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell>
                            </TableRow>
                        ) : sortedHistory?.length === 0 ? (
                            <TableEmptyRow colSpan={8} icon={Clock} title="沒有出勤記錄" />
                        ) : (
                            sortedHistory?.map((record) => (
                                <TableRow key={record.id}>
                                    <TableCell className="whitespace-nowrap">{formatDate(record.work_date, { weekday: true })}</TableCell>
                                    <TableCell className="font-medium">{record.user_name}</TableCell>
                                    <TableCell>{formatTime(record.clock_in_time)}</TableCell>
                                    <TableCell>{formatTime(record.clock_out_time)}</TableCell>
                                    <TableCell>{formatHours(record.regular_hours)}</TableCell>
                                    <TableCell>{formatHours(record.overtime_hours)}</TableCell>
                                    <TableCell>{getStatusBadge(record.status)}</TableCell>
                                    <TableCell>
                                        {record.is_corrected && <Badge variant="outline">已更正</Badge>}
                                        {record.remark && <span className="text-muted-foreground text-sm">{record.remark}</span>}
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </Card>
        </div>
    )
}
