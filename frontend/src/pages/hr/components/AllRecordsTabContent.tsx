import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
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
} from '../constants'
import { AllRecordsTable } from './AllRecordsTable'

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
        queryKey: queryKeys.hr.allOvertime({ filterStatus, filterOvertimeType, filterFrom, filterTo, filterApplicant }),
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
