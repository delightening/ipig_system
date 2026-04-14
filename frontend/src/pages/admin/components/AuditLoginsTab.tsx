import { Input } from '@/components/ui/input'
import { Card } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { formatDateTime } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import type { LoginEventWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { AuditPagination } from './AuditPagination'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { LogIn } from 'lucide-react'

interface AuditLoginsTabProps {
    dateFrom: string
    dateTo: string
    onDateFromChange: (val: string) => void
    onDateToChange: (val: string) => void
    loginEvents: PaginatedResponse<LoginEventWithUser> | undefined
    isLoading: boolean
    currentPage: number
    onPageChange: (page: number) => void
    eventTypeFilter: string
    onEventTypeChange: (val: string) => void
}

export function AuditLoginsTab({
    dateFrom,
    dateTo,
    onDateFromChange,
    onDateToChange,
    loginEvents,
    isLoading,
    currentPage,
    onPageChange,
    eventTypeFilter,
    onEventTypeChange,
}: AuditLoginsTabProps) {
    const { sortedData, sort, toggleSort } = useTableSort(loginEvents?.data)

    return (
        <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-4">
                <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => onDateFromChange(e.target.value)}
                    className="max-w-[150px]"
                />
                <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => onDateToChange(e.target.value)}
                    className="max-w-[150px]"
                />
                <Select value={eventTypeFilter} onValueChange={onEventTypeChange}>
                    <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder="事件類型" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部</SelectItem>
                        <SelectItem value="login_success">登入成功</SelectItem>
                        <SelectItem value="login_failure">登入失敗</SelectItem>
                        <SelectItem value="2fa_failure">2FA 失敗</SelectItem>
                        <SelectItem value="logout">登出</SelectItem>
                    </SelectContent>
                </Select>
            </div>
            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
                            <SortableTableHead sortKey="email" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>Email</SortableTableHead>
                            <SortableTableHead sortKey="event_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>事件</SortableTableHead>
                            <SortableTableHead sortKey="device_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>裝置</SortableTableHead>
                            <SortableTableHead sortKey="browser" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>瀏覽器</SortableTableHead>
                            <SortableTableHead sortKey="ip_address" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>IP</SortableTableHead>
                            <TableHead>異常</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8">載入中...</TableCell>
                            </TableRow>
                        ) : sortedData?.length === 0 ? (
                            <TableEmptyRow colSpan={7} icon={LogIn} title="沒有登入事件" />
                        ) : (
                            sortedData?.map((event) => (
                                <LoginEventRow key={event.id} event={event} />
                            ))
                        )}
                    </TableBody>
                </Table>
                {loginEvents && (
                    <AuditPagination
                        total={loginEvents.total}
                        totalPages={loginEvents.total_pages}
                        currentPage={currentPage}
                        onPageChange={onPageChange}
                    />
                )}
            </Card>
        </div>
    )
}

function LoginEventRow({ event }: { event: LoginEventWithUser }) {
    return (
        <TableRow>
            <TableCell className="whitespace-nowrap">{formatDateTime(event.created_at)}</TableCell>
            <TableCell>{event.email}</TableCell>
            <TableCell>
                <Badge variant={event.event_type === 'login_success' ? 'default' : 'destructive'}>
                    {event.event_type === 'login_success'
                        ? '成功'
                        : event.event_type === 'login_failure'
                            ? '登入失敗'
                            : event.event_type === '2fa_failure'
                                ? '2FA 失敗'
                                : '登出'}
                </Badge>
            </TableCell>
            <TableCell>{event.device_type || '-'}</TableCell>
            <TableCell>{event.browser || '-'}</TableCell>
            <TableCell className="text-muted-foreground text-sm">{event.ip_address || '-'}</TableCell>
            <TableCell>
                <AnomalyBadges event={event} />
            </TableCell>
        </TableRow>
    )
}

function AnomalyBadges({ event }: { event: LoginEventWithUser }) {
    const anomalyClass = "bg-status-warning-bg text-status-warning-text hover:bg-status-warning-bg border-status-warning-border"
    const hasAnomaly = event.is_unusual_time || event.is_unusual_location || event.is_new_device || event.is_mass_login

    if (!hasAnomaly) return <span className="text-muted-foreground">-</span>

    return (
        <div className="flex flex-wrap gap-1">
            {event.is_unusual_time && <Badge variant="secondary" className={anomalyClass}>時間異常</Badge>}
            {event.is_unusual_location && <Badge variant="secondary" className={anomalyClass}>IP 異常</Badge>}
            {event.is_new_device && <Badge variant="secondary" className={anomalyClass}>新裝置</Badge>}
            {event.is_mass_login && <Badge variant="secondary" className={anomalyClass}>同時大量登入</Badge>}
        </div>
    )
}
