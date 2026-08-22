import { format } from 'date-fns'
import { Search } from 'lucide-react'

import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { parseDecimal } from '@/lib/utils'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import type { OvertimeWithUser } from '@/types/hr'

import { OVERTIME_TYPE_NAMES, formatDate } from '../constants'
import { OvertimeStatusBadge } from './OvertimeStatusBadge'
import { OvertimeVoidButton } from './OvertimeVoidButton'

const EMPTY_TITLE = '沒有符合條件的加班紀錄'

// 桌機表格與窄版卡片共用的欄位格式化——兩份版面各寫一套會在改欄位時漂移。
const timeRange = (ot: OvertimeWithUser) =>
    `${format(new Date(ot.start_time), 'HH:mm')} ~ ${format(new Date(ot.end_time), 'HH:mm')}`
const typeName = (ot: OvertimeWithUser) => OVERTIME_TYPE_NAMES[ot.overtime_type] || ot.overtime_type
const hoursText = (ot: OvertimeWithUser) => `${parseDecimal(ot.hours).toFixed(1)}h`
const compHoursText = (ot: OvertimeWithUser) =>
    parseDecimal(ot.comp_time_hours) > 0 ? `${parseDecimal(ot.comp_time_hours).toFixed(1)}h` : '-'

interface AllRecordsTableProps {
    data: OvertimeWithUser[] | undefined
    isLoading: boolean
}

/** 全部加班紀錄：桌機表格（容器 ≥600px）與窄版卡片兩種版面 */
export function AllRecordsTable({ data, isLoading }: AllRecordsTableProps) {
    const { sortedData, sort, toggleSort } = useTableSort(data)

    return (
        <div className="@container">
            <div className="hidden @[600px]:block">
                <DesktopOvertimeTable
                    rows={sortedData}
                    isLoading={isLoading}
                    sort={sort}
                    onSort={toggleSort}
                />
            </div>
            <div className="@[600px]:hidden divide-y rounded-lg border">
                <MobileOvertimeCards rows={sortedData} isLoading={isLoading} />
            </div>
        </div>
    )
}

interface DesktopOvertimeTableProps {
    rows: OvertimeWithUser[] | undefined
    isLoading: boolean
    sort: { column: string | null; direction: 'asc' | 'desc' }
    onSort: (column: string) => void
}

function DesktopOvertimeTable({ rows, isLoading, sort, onSort }: DesktopOvertimeTableProps) {
    const sortProps = { currentSort: sort.column, currentDirection: sort.direction, onSort }
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <SortableTableHead sortKey="user_name" {...sortProps}>申請人</SortableTableHead>
                    <SortableTableHead sortKey="overtime_date" {...sortProps}>日期</SortableTableHead>
                    <SortableTableHead className="hidden @[850px]:table-cell" sortKey="start_time" {...sortProps}>時間</SortableTableHead>
                    <SortableTableHead className="hidden @[750px]:table-cell" sortKey="overtime_type" {...sortProps}>類型</SortableTableHead>
                    <SortableTableHead sortKey="hours" {...sortProps}>時數</SortableTableHead>
                    <SortableTableHead className="hidden @[900px]:table-cell" sortKey="comp_time_hours" {...sortProps}>補休</SortableTableHead>
                    <TableHead className="hidden @[1000px]:table-cell">事由</TableHead>
                    <SortableTableHead sortKey="status" {...sortProps}>狀態</SortableTableHead>
                    <TableHead className="text-right">操作</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {isLoading ? (
                    <TableRow>
                        <TableCell colSpan={9} className="p-0">
                            <TableSkeleton rows={8} cols={9} />
                        </TableCell>
                    </TableRow>
                ) : rows?.length === 0 ? (
                    <TableEmptyRow colSpan={9} icon={Search} title={EMPTY_TITLE} />
                ) : (
                    rows?.map((ot) => <DesktopRow key={ot.id} ot={ot} />)
                )}
            </TableBody>
        </Table>
    )
}

function DesktopRow({ ot }: { ot: OvertimeWithUser }) {
    return (
        <TableRow>
            <TableCell>
                <div className="font-medium">{ot.user_name}</div>
                <div className="text-sm text-muted-foreground">{ot.user_email}</div>
            </TableCell>
            <TableCell className="whitespace-nowrap">{formatDate(ot.overtime_date)}</TableCell>
            <TableCell className="hidden @[850px]:table-cell whitespace-nowrap">{timeRange(ot)}</TableCell>
            <TableCell className="hidden @[750px]:table-cell">{typeName(ot)}</TableCell>
            <TableCell>{hoursText(ot)}</TableCell>
            <TableCell className="hidden @[900px]:table-cell">{compHoursText(ot)}</TableCell>
            <TableCell className="hidden @[1000px]:table-cell max-w-[200px] whitespace-normal break-words">{ot.reason}</TableCell>
            <TableCell>
                <OvertimeStatusBadge status={ot.status} />
                {ot.void_reason && (
                    <div className="mt-1 text-xs text-muted-foreground whitespace-normal break-words">
                        作廢原因：{ot.void_reason}
                    </div>
                )}
            </TableCell>
            <TableCell className="text-right">
                <OvertimeVoidButton overtime={ot} />
            </TableCell>
        </TableRow>
    )
}

interface MobileOvertimeCardsProps {
    rows: OvertimeWithUser[] | undefined
    isLoading: boolean
}

function MobileOvertimeCards({ rows, isLoading }: MobileOvertimeCardsProps) {
    if (isLoading) {
        return <div className="p-6 text-center text-muted-foreground">載入中...</div>
    }
    if (rows?.length === 0) {
        return (
            <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                <Search className="h-8 w-8" />
                <p className="text-sm">{EMPTY_TITLE}</p>
            </div>
        )
    }
    return <>{rows?.map((ot) => <MobileCard key={ot.id} ot={ot} />)}</>
}

function MobileCard({ ot }: { ot: OvertimeWithUser }) {
    return (
        <div className="p-3 space-y-1">
            <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                    <div className="font-medium break-words">{ot.user_name}</div>
                    <div className="text-xs text-muted-foreground break-words">{ot.user_email}</div>
                </div>
                <OvertimeStatusBadge status={ot.status} />
            </div>
            <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3">
                <span>{formatDate(ot.overtime_date)}</span>
                <span>{timeRange(ot)}</span>
                <span>{typeName(ot)}</span>
                <span>
                    {hoursText(ot)}
                    {parseDecimal(ot.comp_time_hours) > 0 && ` (補 ${compHoursText(ot)})`}
                </span>
            </div>
            {ot.reason && <div className="text-xs text-muted-foreground break-words">事由：{ot.reason}</div>}
            {ot.void_reason && (
                <div className="text-xs text-muted-foreground break-words">作廢原因：{ot.void_reason}</div>
            )}
            <div className="flex justify-end">
                <OvertimeVoidButton overtime={ot} />
            </div>
        </div>
    )
}
