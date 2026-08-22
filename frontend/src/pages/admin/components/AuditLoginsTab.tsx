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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { LogIn } from 'lucide-react'
import { useTranslation } from 'react-i18next'

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
    const { t } = useTranslation()
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
                        <SelectValue placeholder={t('admin.auditLoginsTab.eventTypePlaceholder')} />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">{t('admin.auditLoginsTab.eventTypeAll')}</SelectItem>
                        <SelectItem value="login_success">{t('admin.auditLoginsTab.eventLoginSuccess')}</SelectItem>
                        <SelectItem value="login_failure">{t('admin.auditLoginsTab.eventLoginFailure')}</SelectItem>
                        <SelectItem value="2fa_failure">{t('admin.auditLoginsTab.event2faFailure')}</SelectItem>
                        <SelectItem value="reauth_failure">{t('admin.auditLoginsTab.eventReauthFailure')}</SelectItem>
                        <SelectItem value="lockout_reset">{t('admin.auditLoginsTab.eventLockoutReset')}</SelectItem>
                        <SelectItem value="logout">{t('common.logout')}</SelectItem>
                    </SelectContent>
                </Select>
            </div>
            <Card className="overflow-hidden">
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/50 hover:bg-muted/50">
                            <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditLoginsTab.colTime')}</SortableTableHead>
                            <SortableTableHead sortKey="email" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('common.email')}</SortableTableHead>
                            <SortableTableHead sortKey="event_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditLoginsTab.colEvent')}</SortableTableHead>
                            <SortableTableHead sortKey="device_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditLoginsTab.colDevice')}</SortableTableHead>
                            <SortableTableHead sortKey="browser" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditLoginsTab.colBrowser')}</SortableTableHead>
                            <SortableTableHead sortKey="ip_address" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditLoginsTab.colIp')}</SortableTableHead>
                            <TableHead>{t('admin.auditLoginsTab.colAnomaly')}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={8} cols={7} /></TableCell></TableRow>
                        ) : sortedData?.length === 0 ? (
                            <TableEmptyRow colSpan={7} icon={LogIn} title={t('admin.auditLoginsTab.emptyTitle')} />
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

/** event_type → i18n key。未列出的類型直接顯示原始值，不再一律當成「登出」 */
const EVENT_LABEL_KEYS: Record<string, string> = {
    login_success: 'admin.auditLoginsTab.badgeSuccess',
    login_failure: 'admin.auditLoginsTab.eventLoginFailure',
    '2fa_failure': 'admin.auditLoginsTab.event2faFailure',
    reauth_failure: 'admin.auditLoginsTab.eventReauthFailure',
    lockout_reset: 'admin.auditLoginsTab.eventLockoutReset',
    logout: 'common.logout',
}

function LoginEventRow({ event }: { event: LoginEventWithUser }) {
    const { t } = useTranslation()
    const labelKey = EVENT_LABEL_KEYS[event.event_type]
    // lockout_reset 是密碼變更後的解鎖標記，不是失敗事件 → 不用警示色
    const variant = event.event_type === 'login_success'
        ? 'default'
        : event.event_type === 'lockout_reset'
            ? 'secondary'
            : 'destructive'
    return (
        <TableRow>
            <TableCell className="whitespace-nowrap">{formatDateTime(event.created_at)}</TableCell>
            <TableCell>{event.email}</TableCell>
            <TableCell>
                <Badge variant={variant}>
                    {labelKey ? t(labelKey) : event.event_type}
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
    const { t } = useTranslation()
    const anomalyClass = "bg-status-warning-bg text-status-warning-text hover:bg-status-warning-bg border-status-warning-border"
    const hasAnomaly = event.is_unusual_time || event.is_unusual_location || event.is_new_device || event.is_mass_login

    if (!hasAnomaly) return <span className="text-muted-foreground">-</span>

    return (
        <div className="flex flex-wrap gap-1">
            {event.is_unusual_time && <Badge variant="secondary" className={anomalyClass}>{t('admin.auditLoginsTab.anomalyUnusualTime')}</Badge>}
            {event.is_unusual_location && <Badge variant="secondary" className={anomalyClass}>{t('admin.auditLoginsTab.anomalyUnusualLocation')}</Badge>}
            {event.is_new_device && <Badge variant="secondary" className={anomalyClass}>{t('admin.auditLoginsTab.anomalyNewDevice')}</Badge>}
            {event.is_mass_login && <Badge variant="secondary" className={anomalyClass}>{t('admin.auditLoginsTab.anomalyMassLogin')}</Badge>}
        </div>
    )
}
