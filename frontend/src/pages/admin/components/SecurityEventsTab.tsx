import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

import { Badge } from '@/components/ui/badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { uiLocale } from '@/lib/utils'
import type { PaginatedResponse } from '@/types/common'
import type { UserActivityLog } from '@/types/hr'

const SECURITY_EVENT_TYPE_VALUES = [
    'all',
    'RATE_LIMIT_AUTH',
    'RATE_LIMIT_API',
    'RATE_LIMIT_WRITE',
    'RATE_LIMIT_UPLOAD',
    'RATE_LIMIT_AI_KEY',
    'AI_KEY_DEACTIVATED',
    'AI_KEY_EXPIRED',
    'PERMISSION_DENIED',
    'ACCOUNT_LOCKOUT',
    'HONEYPOT_HIT',
] as const

function eventTypeLabel(t: TFunction, eventType: string) {
    const key = `admin.securityEventsTab.eventType.${eventType}`
    const label = t(key)
    return label === key ? eventType : label
}

function severityBadge(t: TFunction, severity: string) {
    switch (severity) {
        case 'critical':
            return <Badge variant="destructive">{t('admin.securityEventsTab.severity.critical')}</Badge>
        case 'warning':
            return <Badge variant="outline" className="border-yellow-500 text-yellow-600">{t('admin.securityEventsTab.severity.warning')}</Badge>
        default:
            return <Badge variant="secondary">{severity}</Badge>
    }
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
    const { t } = useTranslation()

    const eventTypeOptions = useMemo(
        () =>
            SECURITY_EVENT_TYPE_VALUES.map((value) => ({
                value,
                label: eventTypeLabel(t, value),
            })),
        [t],
    )

    const columns: ColumnDef<UserActivityLog>[] = useMemo(
        () => [
            {
                key: 'created_at',
                header: t('admin.securityEventsTab.column.time'),
                cell: (row: UserActivityLog) => new Date(row.created_at).toLocaleString(uiLocale()),
            },
            {
                key: 'event_type',
                header: t('admin.securityEventsTab.column.eventType'),
                cell: (row: UserActivityLog) => eventTypeLabel(t, row.event_type),
            },
            {
                key: 'event_severity',
                header: t('admin.securityEventsTab.column.severity'),
                cell: (row: UserActivityLog) => severityBadge(t, row.event_severity),
            },
            {
                key: 'ip_address',
                header: t('admin.securityEventsTab.column.ip'),
                cell: (row: UserActivityLog) => <span className="font-mono text-sm">{row.ip_address ?? '-'}</span>,
            },
            {
                key: 'request_path',
                header: t('admin.securityEventsTab.column.path'),
                cell: (row: UserActivityLog) => <span className="font-mono text-sm">{row.request_path ?? '-'}</span>,
            },
            {
                key: 'suspicious_reason',
                header: t('admin.securityEventsTab.column.reason'),
                cell: (row: UserActivityLog) => row.suspicious_reason ?? '-',
            },
        ],
        [t],
    )

    return (
        <>
            <div className="flex flex-wrap gap-3 items-center">
                <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => onDateFromChange(e.target.value)}
                    className="w-40"
                    aria-label={t('admin.securityEventsTab.dateFromLabel')}
                />
                <span className="text-muted-foreground">~</span>
                <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => onDateToChange(e.target.value)}
                    className="w-40"
                    aria-label={t('admin.securityEventsTab.dateToLabel')}
                />
                <Select value={eventTypeFilter} onValueChange={onEventTypeChange}>
                    <SelectTrigger className="w-48" aria-label={t('admin.securityEventsTab.eventTypeFilterLabel')}>
                        <SelectValue placeholder={t('admin.securityEventsTab.column.eventType')} />
                    </SelectTrigger>
                    <SelectContent>
                        {eventTypeOptions.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                                {option.label}
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
                emptyTitle={t('admin.securityEventsTab.emptyTitle')}
            />
        </>
    )
}
