import { useState } from 'react'
import { FileText } from 'lucide-react'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useQuery } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { StatusBadge } from '@/components/ui/status-badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { formatDate } from '@/lib/utils'
import { LEAVE_STATUS_NAMES, LEAVE_TYPE_NAMES } from '@/types/hr'
import type { LeaveRequestWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatLeaveHours, getLeaveStatusVariant } from '../constants'

export function AllLeaveRecordsTabContent() {
    const [filterStatus, setFilterStatus] = useState<string>('all')
    const [filterLeaveType, setFilterLeaveType] = useState<string>('all')
    const { from: filterFrom, to: filterTo, setFrom: setFilterFrom, setTo: setFilterTo, reset: resetDateRange } =
        useDateRangeFilter()

    const { data: allLeaves, isLoading } = useQuery({
        queryKey: queryKeys.hr.allLeaves({ filterStatus, filterLeaveType, filterFrom, filterTo }),
        queryFn: async () => {
            const params = new URLSearchParams({ view_all: 'true' })
            if (filterStatus !== 'all') params.append('status', filterStatus)
            if (filterLeaveType !== 'all') params.append('leave_type', filterLeaveType)
            if (filterFrom) params.append('from', filterFrom)
            if (filterTo) params.append('to', filterTo)
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>(`/hr/leaves?${params.toString()}`)
            return res.data
        },
    })

    const hasFilters = filterStatus !== 'all' || filterLeaveType !== 'all' || filterFrom || filterTo

    return (
        <Card>
            <CardHeader>
                <CardTitle>全部請假紀錄</CardTitle>
                <CardDescription>查看所有員工的請假資料</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* 篩選列 */}
                <div className="flex flex-wrap gap-3 items-end">
                    <div className="grid gap-1">
                        <Label className="text-xs">狀態</Label>
                        <Select value={filterStatus} onValueChange={setFilterStatus}>
                            <SelectTrigger className="w-[140px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">全部狀態</SelectItem>
                                {Object.entries(LEAVE_STATUS_NAMES).map(([code, name]) => (
                                    <SelectItem key={code} value={code}>{name}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">假別</Label>
                        <Select value={filterLeaveType} onValueChange={setFilterLeaveType}>
                            <SelectTrigger className="w-[140px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">全部假別</SelectItem>
                                {Object.entries(LEAVE_TYPE_NAMES).map(([code, name]) => (
                                    <SelectItem key={code} value={code}>{name}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">起始日期</Label>
                        <Input
                            type="date"
                            value={filterFrom}
                            onChange={(e) => setFilterFrom(e.target.value)}
                            className="w-[160px]"
                        />
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">結束日期</Label>
                        <Input
                            type="date"
                            value={filterTo}
                            onChange={(e) => setFilterTo(e.target.value)}
                            className="w-[160px]"
                        />
                    </div>
                    {hasFilters && (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                                setFilterStatus('all')
                                setFilterLeaveType('all')
                                resetDateRange()
                            }}
                        >
                            清除篩選
                        </Button>
                    )}
                </div>

                {/* 表格 */}
                <DataTable<LeaveRequestWithUser>
                    columns={[
                        {
                            key: 'applicant',
                            header: '申請人',
                            cell: (leave) => (
                                <div>
                                    <div className="font-medium">{leave.user_name}</div>
                                    <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                                </div>
                            ),
                        },
                        {
                            key: 'leave_type',
                            header: '假別',
                            cell: (leave) => LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type,
                        },
                        {
                            key: 'date',
                            header: '日期',
                            cell: (leave) => (
                                <span className="whitespace-nowrap">
                                    {formatDate(leave.start_date)}
                                    {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                </span>
                            ),
                        },
                        {
                            key: 'hours',
                            header: '時數',
                            cell: (leave) => formatLeaveHours(leave),
                        },
                        {
                            key: 'reason',
                            header: '事由',
                            className: 'max-w-[200px] truncate',
                            cell: (leave) => leave.reason,
                        },
                        {
                            key: 'status',
                            header: '狀態',
                            cell: (leave) => {
                                const status = getLeaveStatusVariant(leave.status)
                                return (
                                    <StatusBadge variant={status.variant}>
                                        {status.label}
                                    </StatusBadge>
                                )
                            },
                        },
                    ]}
                    data={allLeaves?.data}
                    isLoading={isLoading}
                    emptyIcon={FileText}
                    emptyTitle="沒有符合條件的請假紀錄"
                    rowKey={(row) => row.id}
                />
                {allLeaves && allLeaves.total > 0 && (
                    <div className="text-sm text-muted-foreground">
                        共 {allLeaves.total} 筆紀錄
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
