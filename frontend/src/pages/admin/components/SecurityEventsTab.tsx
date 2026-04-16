import { Badge } from '@/components/ui/badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { PaginatedResponse } from '@/types/common'
import type { UserActivityLog } from '@/types/hr'

const SECURITY_EVENT_TYPES = [
    { value: 'all', label: '全部' },
    { value: 'RATE_LIMIT_AUTH', label: '認證速率限制' },
    { value: 'RATE_LIMIT_API', label: 'API 速率限制' },
    { value: 'RATE_LIMIT_WRITE', label: '寫入速率限制' },
    { value: 'RATE_LIMIT_UPLOAD', label: '上傳速率限制' },
    { value: 'RATE_LIMIT_AI_KEY', label: 'AI Key 速率限制' },
    { value: 'AI_KEY_DEACTIVATED', label: 'AI Key 停用' },
    { value: 'AI_KEY_EXPIRED', label: 'AI Key 過期' },
    { value: 'PERMISSION_DENIED', label: '權限拒絕 (403)' },
    { value: 'ACCOUNT_LOCKOUT', label: '帳號鎖定' },
    { value: 'HONEYPOT_HIT', label: '蜜罐觸發' },
]

function severityBadge(severity: string) {
    switch (severity) {
        case 'critical':
            return <Badge variant="destructive">Critical</Badge>
        case 'warning':
            return <Badge variant="outline" className="border-yellow-500 text-yellow-600">Warning</Badge>
        default:
            return <Badge variant="secondary">{severity}</Badge>
    }
}

function eventTypeLabel(eventType: string) {
    return SECURITY_EVENT_TYPES.find(t => t.value === eventType)?.label ?? eventType
}

interface SecurityEventsTabProps {
    dateFrom: string
    dateTo: string
    onDateFromChange: (val: string) => void
    onDateToChange: (val: string) => void
    securityEvents: PaginatedResponse<UserActivityLog> | undefined
    isLoading: boolean
    currentPage: number
    onPageChange: (page: number) => void
    eventTypeFilter: string
    onEventTypeChange: (val: string) => void
}

const columns: ColumnDef<UserActivityLog>[] = [
    {
        key: 'created_at',
        header: '時間',
        cell: (row: UserActivityLog) => new Date(row.created_at).toLocaleString('zh-TW'),
    },
    {
        key: 'event_type',
        header: '事件類型',
        cell: (row: UserActivityLog) => eventTypeLabel(row.event_type),
    },
    {
        key: 'event_severity',
        header: '嚴重度',
        cell: (row: UserActivityLog) => severityBadge(row.event_severity),
    },
    {
        key: 'ip_address',
        header: 'IP',
        cell: (row: UserActivityLog) => <span className="font-mono text-sm">{row.ip_address ?? '-'}</span>,
    },
    {
        key: 'request_path',
        header: '路徑',
        cell: (row: UserActivityLog) => <span className="font-mono text-sm">{row.request_path ?? '-'}</span>,
    },
    {
        key: 'suspicious_reason',
        header: '說明',
        cell: (row: UserActivityLog) => row.suspicious_reason ?? '-',
    },
]

export function SecurityEventsTab({
    dateFrom,
    dateTo,
    onDateFromChange,
    onDateToChange,
    securityEvents,
    isLoading,
    currentPage,
    onPageChange,
    eventTypeFilter,
    onEventTypeChange,
}: SecurityEventsTabProps) {
    return (
        <>
            <div className="flex flex-wrap gap-3 items-center">
                <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => onDateFromChange(e.target.value)}
                    className="w-40"
                />
                <span className="text-muted-foreground">~</span>
                <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => onDateToChange(e.target.value)}
                    className="w-40"
                />
                <Select value={eventTypeFilter} onValueChange={onEventTypeChange}>
                    <SelectTrigger className="w-48">
                        <SelectValue placeholder="事件類型" />
                    </SelectTrigger>
                    <SelectContent>
                        {SECURITY_EVENT_TYPES.map((t) => (
                            <SelectItem key={t.value} value={t.value}>
                                {t.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            <DataTable
                columns={columns}
                data={securityEvents?.data ?? []}
                isLoading={isLoading}
                page={currentPage}
                totalPages={securityEvents?.total_pages ?? 1}
                onPageChange={onPageChange}
                rowKey={(row: UserActivityLog) => row.id}
                emptyTitle="尚無安全事件記錄"
            />
        </>
    )
}
