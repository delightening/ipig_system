import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Eye, FileText } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'

import { Button } from '@/components/ui/button'
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
import { formatDateTime } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import type { PaginatedResponse } from '@/types/common'
import type { AuditLog } from '../types/audit'
import { AuditPagination } from './AuditPagination'

const ACTION_VARIANT_MAP: Record<string, 'default' | 'destructive' | 'secondary'> = {
    'CREATE': 'default',
    'UPDATE': 'default',
    'DELETE': 'destructive',
    'PASSWORD_RESET': 'destructive',
    'PASSWORD_CHANGE': 'destructive',
    'IMPERSONATE': 'destructive',
    'STOP_IMPERSONATE': 'secondary',
    'STATUS_CHANGE': 'destructive',
    'ASSIGN': 'destructive',
    'UNASSIGN': 'destructive',
    'force_logout': 'destructive',
}

const ACTION_LABEL_KEY_MAP: Record<string, string> = {
    'CREATE': 'admin.auditActivitiesTab.action.create',
    'UPDATE': 'admin.auditActivitiesTab.action.update',
    'DELETE': 'admin.auditActivitiesTab.action.delete',
    'PASSWORD_RESET': 'admin.auditActivitiesTab.action.passwordReset',
    'PASSWORD_CHANGE': 'admin.auditActivitiesTab.action.passwordChange',
    'IMPERSONATE': 'admin.auditActivitiesTab.action.impersonate',
    'STOP_IMPERSONATE': 'admin.auditActivitiesTab.action.stopImpersonate',
    'STATUS_CHANGE': 'admin.auditActivitiesTab.action.statusChange',
    'ASSIGN': 'admin.auditActivitiesTab.action.assign',
    'UNASSIGN': 'admin.auditActivitiesTab.action.unassign',
    'force_logout': 'admin.auditActivitiesTab.action.forceLogout',
}

const FIELD_LABEL_KEY_MAP: Record<string, string> = {
    display_name: 'admin.auditActivitiesTab.field.displayName',
    email: 'admin.auditActivitiesTab.field.email',
    is_active: 'admin.auditActivitiesTab.field.isActive',
    roles: 'admin.auditActivitiesTab.field.roles',
    phone: 'admin.auditActivitiesTab.field.phone',
    organization: 'admin.auditActivitiesTab.field.organization',
}

const SUMMARY_LABEL_KEY_MAP: Record<string, string> = {
    'CREATE': 'admin.auditActivitiesTab.summary.create',
    'UPDATE': 'admin.auditActivitiesTab.summary.update',
    'DELETE': 'admin.auditActivitiesTab.summary.delete',
    'PASSWORD_RESET': 'admin.auditActivitiesTab.summary.passwordReset',
    'PASSWORD_CHANGE': 'admin.auditActivitiesTab.summary.passwordChange',
    'IMPERSONATE': 'admin.auditActivitiesTab.summary.impersonate',
    'STOP_IMPERSONATE': 'admin.auditActivitiesTab.summary.stopImpersonate',
    'STATUS_CHANGE': 'admin.auditActivitiesTab.summary.statusChange',
    'ASSIGN': 'admin.auditActivitiesTab.summary.assign',
    'UNASSIGN': 'admin.auditActivitiesTab.summary.unassign',
    'force_logout': 'admin.auditActivitiesTab.summary.forceLogout',
}

function getActionSummary(log: AuditLog, t: TFunction): string {
    const summaryKey = SUMMARY_LABEL_KEY_MAP[log.action]
    const base = summaryKey ? t(summaryKey) : log.action

    if (log.action === 'IMPERSONATE' && log.after_data) {
        const data = log.after_data as Record<string, string>
        const target = data.impersonated_email || data.impersonated_user_id?.slice(0, 8)
        return target ? `${base} → ${target}` : base
    }

    if (log.action === 'UPDATE' && log.after_data) {
        const keys = Object.keys(log.after_data)
        const changed = keys.map(k => (FIELD_LABEL_KEY_MAP[k] ? t(FIELD_LABEL_KEY_MAP[k]) : k)).slice(0, 3)
        return t('admin.auditActivitiesTab.summary.updateFields', {
            base,
            fields: changed.join('、'),
            more: keys.length > 3 ? t('admin.auditActivitiesTab.summary.more') : '',
        })
    }

    if (log.action === 'CREATE' && log.after_data) {
        const data = log.after_data as Record<string, string>
        if (data.email) return t('admin.auditActivitiesTab.summary.withValue', { base, value: data.email })
    }

    if (log.action === 'force_logout' && log.after_data) {
        const data = log.after_data as Record<string, string>
        if (data.reason) return t('admin.auditActivitiesTab.summary.withValue', { base, value: data.reason })
    }

    return base
}

interface AuditActivitiesTabProps {
    dateFrom: string
    dateTo: string
    onDateFromChange: (val: string) => void
    onDateToChange: (val: string) => void
    activityLogs: PaginatedResponse<AuditLog> | undefined
    isLoading: boolean
    currentPage: number
    onPageChange: (page: number) => void
    onSelectLog: (log: AuditLog) => void
}

export function AuditActivitiesTab({
    dateFrom,
    dateTo,
    onDateFromChange,
    onDateToChange,
    activityLogs,
    isLoading,
    currentPage,
    onPageChange,
    onSelectLog,
}: AuditActivitiesTabProps) {
    const { t } = useTranslation()
    const { sortedData, sort, toggleSort } = useTableSort(activityLogs?.data)

    return (
        <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-4">
                <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => onDateFromChange(e.target.value)}
                    placeholder={t('admin.auditActivitiesTab.startDate')}
                    className="max-w-[150px]"
                />
                <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => onDateToChange(e.target.value)}
                    placeholder={t('admin.auditActivitiesTab.endDate')}
                    className="max-w-[150px]"
                />
            </div>
            <Card className="overflow-hidden">
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/50 hover:bg-muted/50">
                            <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditActivitiesTab.colTime')}</SortableTableHead>
                            <SortableTableHead sortKey="actor_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditActivitiesTab.colOperator')}</SortableTableHead>
                            <SortableTableHead sortKey="action" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditActivitiesTab.colAction')}</SortableTableHead>
                            <SortableTableHead sortKey="entity_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.auditActivitiesTab.colTargetUser')}</SortableTableHead>
                            <TableHead>{t('admin.auditActivitiesTab.colSummary')}</TableHead>
                            <TableHead className="w-[60px]">{t('admin.auditActivitiesTab.colDetail')}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
                        ) : !sortedData || sortedData.length === 0 ? (
                            <TableEmptyRow colSpan={6} icon={FileText} title={t('admin.auditActivitiesTab.empty')} />
                        ) : (
                            sortedData.map((log) => (
                                <ActivityRow key={log.id} log={log} onSelect={onSelectLog} t={t} />
                            ))
                        )}
                    </TableBody>
                </Table>
                {activityLogs && (
                    <AuditPagination
                        total={activityLogs.total}
                        totalPages={activityLogs.total_pages}
                        currentPage={currentPage}
                        onPageChange={onPageChange}
                    />
                )}
            </Card>
        </div>
    )
}

function ActivityRow({ log, onSelect, t }: { log: AuditLog; onSelect: (log: AuditLog) => void; t: TFunction }) {
    return (
        <TableRow>
            <TableCell className="whitespace-nowrap">{formatDateTime(log.created_at)}</TableCell>
            <TableCell>
                <div>
                    <div className="font-medium">{log.actor_name || '-'}</div>
                    <div className="text-sm text-muted-foreground">{log.actor_email || ''}</div>
                </div>
            </TableCell>
            <TableCell>
                <Badge variant={ACTION_VARIANT_MAP[log.action] || 'outline'}>
                    {ACTION_LABEL_KEY_MAP[log.action] ? t(ACTION_LABEL_KEY_MAP[log.action]) : log.action}
                </Badge>
            </TableCell>
            <TableCell>
                {log.entity_name || log.entity_email ? (
                    <div>
                        <div className="font-medium">{log.entity_name || '-'}</div>
                        <div className="text-xs text-muted-foreground">{log.entity_email || ''}</div>
                    </div>
                ) : (
                    <span className="font-mono text-xs text-muted-foreground">{log.entity_id?.slice(0, 8)}...</span>
                )}
            </TableCell>
            <TableCell className="text-sm max-w-[250px]">
                <span className="text-muted-foreground">{getActionSummary(log, t)}</span>
            </TableCell>
            <TableCell>
                <Button variant="ghost" size="icon" onClick={() => onSelect(log)} title={t('admin.auditActivitiesTab.viewDetail')} aria-label={t('admin.auditActivitiesTab.viewDetail')}>
                    <Eye className="h-4 w-4" />
                </Button>
            </TableCell>
        </TableRow>
    )
}
