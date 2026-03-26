import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import api from '@/lib/api'
import { parseDecimal } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import {
    OVERTIME_TYPE_NAMES,
    OVERTIME_STATUS_NAMES,
    formatDate,
} from '../constants'
import { OvertimeStatusBadge } from './OvertimeStatusBadge'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Search } from 'lucide-react'

interface StaffItem {
    id: string
    display_name: string
    email: string
}

interface AllRecordsTabContentProps {
    isActive: boolean
    staffList: StaffItem[] | undefined
}

export function AllRecordsTabContent({ isActive, staffList }: AllRecordsTabContentProps) {
    const [filterStatus, setFilterStatus] = useState<string>('all')
    const [filterOvertimeType, setFilterOvertimeType] = useState<string>('all')
    const [filterApplicant, setFilterApplicant] = useState<string>('all')
    const {
        from: filterFrom,
        to: filterTo,
        setFrom: setFilterFrom,
        setTo: setFilterTo,
        reset: resetDateRange,
    } = useDateRangeFilter()

    const { data: allOvertime, isLoading } = useQuery({
        queryKey: ['hr-all-overtime', filterStatus, filterOvertimeType, filterFrom, filterTo, filterApplicant],
        queryFn: async () => {
            const params = new URLSearchParams({ view_all: 'true' })
            if (filterStatus !== 'all') params.append('status', filterStatus)
            if (filterFrom) params.append('from', filterFrom)
            if (filterTo) params.append('to', filterTo)
            if (filterApplicant !== 'all') params.append('user_id', filterApplicant)
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>(`/hr/overtime?${params.toString()}`)
            return res.data
        },
        enabled: isActive,
    })

    const hasActiveFilters =
        filterStatus !== 'all' ||
        filterOvertimeType !== 'all' ||
        !!filterFrom ||
        !!filterTo ||
        filterApplicant !== 'all'

    const clearFilters = () => {
        setFilterStatus('all')
        setFilterOvertimeType('all')
        resetDateRange()
        setFilterApplicant('all')
    }

    const filteredData = allOvertime?.data?.filter(
        (ot) => filterOvertimeType === 'all' || ot.overtime_type === filterOvertimeType
    )

    return (
        <Card>
                <CardHeader>
                    <CardTitle>全部加班紀錄</CardTitle>
                    <CardDescription>查看所有員工的加班資料</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <AllRecordsFilterBar
                        filterApplicant={filterApplicant}
                        onApplicantChange={setFilterApplicant}
                        filterStatus={filterStatus}
                        onStatusChange={setFilterStatus}
                        filterOvertimeType={filterOvertimeType}
                        onOvertimeTypeChange={setFilterOvertimeType}
                        filterFrom={filterFrom}
                        onFromChange={setFilterFrom}
                        filterTo={filterTo}
                        onToChange={setFilterTo}
                        hasActiveFilters={hasActiveFilters}
                        onClear={clearFilters}
                        staffList={staffList}
                    />

                    <AllRecordsTable
                        data={filteredData}
                        isLoading={isLoading}
                    />

                    {allOvertime && allOvertime.total > 0 && (
                        <div className="text-sm text-muted-foreground">
                            共 {allOvertime.total} 筆紀錄
                        </div>
                    )}
                </CardContent>
            </Card>
    )
}

interface AllRecordsFilterBarProps {
    filterApplicant: string
    onApplicantChange: (v: string) => void
    filterStatus: string
    onStatusChange: (v: string) => void
    filterOvertimeType: string
    onOvertimeTypeChange: (v: string) => void
    filterFrom: string
    onFromChange: (v: string) => void
    filterTo: string
    onToChange: (v: string) => void
    hasActiveFilters: boolean
    onClear: () => void
    staffList: StaffItem[] | undefined
}

function AllRecordsFilterBar({
    filterApplicant,
    onApplicantChange,
    filterStatus,
    onStatusChange,
    filterOvertimeType,
    onOvertimeTypeChange,
    filterFrom,
    onFromChange,
    filterTo,
    onToChange,
    hasActiveFilters,
    onClear,
    staffList,
}: AllRecordsFilterBarProps) {
    return (
        <div className="flex flex-wrap gap-3 items-end">
            <div className="grid gap-1">
                <Label className="text-xs">申請人</Label>
                <Select value={filterApplicant} onValueChange={onApplicantChange}>
                    <SelectTrigger className="w-[180px]">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部人員</SelectItem>
                        {staffList?.map((staff) => (
                            <SelectItem key={staff.id} value={staff.id}>
                                {staff.display_name}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div className="grid gap-1">
                <Label className="text-xs">狀態</Label>
                <Select value={filterStatus} onValueChange={onStatusChange}>
                    <SelectTrigger className="w-[140px]">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部狀態</SelectItem>
                        {Object.entries(OVERTIME_STATUS_NAMES).map(([code, name]) => (
                            <SelectItem key={code} value={code}>{name}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div className="grid gap-1">
                <Label className="text-xs">加班類型</Label>
                <Select value={filterOvertimeType} onValueChange={onOvertimeTypeChange}>
                    <SelectTrigger className="w-[140px]">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部類型</SelectItem>
                        {Object.entries(OVERTIME_TYPE_NAMES).map(([code, name]) => (
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
                    onChange={(e) => onFromChange(e.target.value)}
                    className="w-[160px]"
                />
            </div>
            <div className="grid gap-1">
                <Label className="text-xs">結束日期</Label>
                <Input
                    type="date"
                    value={filterTo}
                    onChange={(e) => onToChange(e.target.value)}
                    className="w-[160px]"
                />
            </div>
            {hasActiveFilters && (
                <Button variant="ghost" size="sm" onClick={onClear}>
                    清除篩選
                </Button>
            )}
        </div>
    )
}

interface AllRecordsTableProps {
    data: OvertimeWithUser[] | undefined
    isLoading: boolean
}

function AllRecordsTable({ data, isLoading }: AllRecordsTableProps) {
    const { sortedData, sort, toggleSort } = useTableSort(data)

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>申請人</SortableTableHead>
                    <SortableTableHead sortKey="overtime_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                    <SortableTableHead sortKey="start_time" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
                    <SortableTableHead sortKey="overtime_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>類型</SortableTableHead>
                    <SortableTableHead sortKey="hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時數</SortableTableHead>
                    <SortableTableHead sortKey="comp_time_hours" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>補休</SortableTableHead>
                    <TableHead>事由</TableHead>
                    <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>狀態</SortableTableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {isLoading ? (
                    <TableRow>
                        <TableCell colSpan={8} className="text-center py-8">
                            載入中...
                        </TableCell>
                    </TableRow>
                ) : sortedData?.length === 0 ? (
                    <TableEmptyRow colSpan={8} icon={Search} title="沒有符合條件的加班紀錄" />
                ) : (
                    sortedData?.map((overtime) => (
                        <TableRow key={overtime.id}>
                            <TableCell>
                                <div>
                                    <div className="font-medium">{overtime.user_name}</div>
                                    <div className="text-sm text-muted-foreground">{overtime.user_email}</div>
                                </div>
                            </TableCell>
                            <TableCell className="whitespace-nowrap">
                                {formatDate(overtime.overtime_date)}
                            </TableCell>
                            <TableCell className="whitespace-nowrap">
                                {format(new Date(overtime.start_time), 'HH:mm')} ~ {format(new Date(overtime.end_time), 'HH:mm')}
                            </TableCell>
                            <TableCell>{OVERTIME_TYPE_NAMES[overtime.overtime_type] || overtime.overtime_type}</TableCell>
                            <TableCell>{parseDecimal(overtime.hours).toFixed(1)}h</TableCell>
                            <TableCell>{parseDecimal(overtime.comp_time_hours) > 0 ? `${parseDecimal(overtime.comp_time_hours).toFixed(1)}h` : '-'}</TableCell>
                            <TableCell className="max-w-[200px] truncate">{overtime.reason}</TableCell>
                            <TableCell><OvertimeStatusBadge status={overtime.status} /></TableCell>
                        </TableRow>
                    ))
                )}
            </TableBody>
        </Table>
    )
}
