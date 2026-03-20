import { useState } from 'react'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useQuery } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import api from '@/lib/api'
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
        queryKey: ['hr-all-leaves', filterStatus, filterLeaveType, filterFrom, filterTo],
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
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>申請人</TableHead>
                            <TableHead>假別</TableHead>
                            <TableHead>日期</TableHead>
                            <TableHead>時數</TableHead>
                            <TableHead>事由</TableHead>
                            <TableHead>狀態</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center py-8">
                                    載入中...
                                </TableCell>
                            </TableRow>
                        ) : allLeaves?.data?.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                    沒有符合條件的請假紀錄
                                </TableCell>
                            </TableRow>
                        ) : (
                            allLeaves?.data?.map((leave) => {
                                const status = getLeaveStatusVariant(leave.status)
                                return (
                                    <TableRow key={leave.id}>
                                        <TableCell>
                                            <div>
                                                <div className="font-medium">{leave.user_name}</div>
                                                <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                                            </div>
                                        </TableCell>
                                        <TableCell>{LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type}</TableCell>
                                        <TableCell className="whitespace-nowrap">
                                            {formatDate(leave.start_date)}
                                            {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                        </TableCell>
                                        <TableCell>{formatLeaveHours(leave)}</TableCell>
                                        <TableCell className="max-w-[200px] truncate">{leave.reason}</TableCell>
                                        <TableCell>
                                            <Badge variant={status.variant} className={status.className}>
                                                {status.label}
                                            </Badge>
                                        </TableCell>
                                    </TableRow>
                                )
                            })
                        )}
                    </TableBody>
                </Table>
                {allLeaves && allLeaves.total > 0 && (
                    <div className="text-sm text-muted-foreground">
                        共 {allLeaves.total} 筆紀錄
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
