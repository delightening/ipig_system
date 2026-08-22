import { type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowDown, ArrowUp, ArrowUpDown, CheckSquare } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { UseMutationResult } from '@tanstack/react-query'
import type { AxiosResponse } from 'axios'

import { onActivateKey } from '@/lib/a11y'
import { uiLocale } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import type { SecurityAlert } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import type { AlertSortConfig, AlertSortField } from '../types/audit'
import { AuditPagination } from './AuditPagination'

import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Search, ShieldAlert } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { alertTypeLabels, severityLabels } from '../constants/auditLogs'

function getSeverityColor(severity: string) {
    switch (severity) {
        case 'critical':
        case 'high':
            return 'destructive' as const
        case 'warning':
            return 'warning' as const
        case 'medium':
        case 'info':
            return 'default' as const
        default:
            return 'secondary' as const
    }
}

function getSortIcon(sortConfig: AlertSortConfig, field: AlertSortField): ReactNode {
    if (sortConfig.field !== field) return <ArrowUpDown className="ml-1 h-3 w-3" />
    return sortConfig.order === 'asc' ? (
        <ArrowUp className="ml-1 h-3 w-3 text-primary" />
    ) : (
        <ArrowDown className="ml-1 h-3 w-3 text-primary" />
    )
}

interface AuditAlertsTabProps {
    alerts: PaginatedResponse<SecurityAlert> | undefined
    sortedAlerts: SecurityAlert[]
    isLoading: boolean
    currentPage: number
    onPageChange: (page: number) => void
    sortConfig: AlertSortConfig
    onSort: (field: AlertSortField) => void
    onSelectAlert: (alert: SecurityAlert) => void
    resolveAlertMutation: UseMutationResult<AxiosResponse, Error, string>
    bulkResolveAlertsMutation: UseMutationResult<AxiosResponse, Error, string[]>
    selectedAlertIds: string[]
    onAlertSelect: (id: string, checked: boolean) => void
    onSelectAllAlerts: (checked: boolean) => void
    search: string
    onSearchChange: (search: string) => void
    statusFilter: string
    onStatusFilterChange: (status: string) => void
}

export function AuditAlertsTab({
    alerts,
    sortedAlerts,
    isLoading,
    currentPage,
    onPageChange,
    sortConfig,
    onSort,
    onSelectAlert,
    resolveAlertMutation,
    bulkResolveAlertsMutation,
    selectedAlertIds,
    onAlertSelect,
    onSelectAllAlerts,
    search,
    onSearchChange,
    statusFilter,
    onStatusFilterChange,
}: AuditAlertsTabProps) {
    const { t } = useTranslation()
    const openAlerts = sortedAlerts.filter(a => a.status !== 'resolved')
    const allOpenSelected = openAlerts.length > 0 && openAlerts.every(a => selectedAlertIds.includes(a.id))
    const someSelected = selectedAlertIds.length > 0

    return (
        <Card className="overflow-hidden">
            <CardHeader className="flex flex-col space-y-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
                <div>
                    <CardTitle>{t('admin.auditAlertsTab.title')}</CardTitle>
                    <CardDescription>{t('admin.auditAlertsTab.description')}</CardDescription>
                </div>
                <div className="flex flex-col space-y-2 sm:flex-row sm:items-center sm:space-x-2 sm:space-y-0">
                    {someSelected && (
                        <Button
                            size="sm"
                            variant="default"
                            onClick={() => bulkResolveAlertsMutation.mutate(selectedAlertIds)}
                            disabled={bulkResolveAlertsMutation.isPending}
                        >
                            <CheckSquare className="h-4 w-4 mr-2" />
                            {t('admin.auditAlertsTab.markResolvedCount', { count: selectedAlertIds.length })}
                        </Button>
                    )}
                    <div className="relative w-full sm:w-64">
                        <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder={t('admin.auditAlertsTab.searchPlaceholder')}
                            className="pl-9"
                            value={search}
                            onChange={(e) => onSearchChange(e.target.value)}
                        />
                    </div>
                    <Select value={statusFilter} onValueChange={onStatusFilterChange}>
                        <SelectTrigger className="w-full sm:w-32">
                            <SelectValue placeholder={t('admin.auditAlertsTab.statusFilterPlaceholder')} />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">{t('common.allStatus')}</SelectItem>
                            <SelectItem value="open">{t('admin.auditAlertsTab.statusPending')}</SelectItem>
                            <SelectItem value="resolved">{t('admin.auditAlertsTab.statusResolved')}</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/50 hover:bg-muted/50">
                            <TableHead className="w-10">
                                <Checkbox
                                    checked={allOpenSelected}
                                    onCheckedChange={(v) => onSelectAllAlerts(!!v)}
                                    aria-label={t('admin.auditAlertsTab.selectAllAriaLabel')}
                                />
                            </TableHead>
                            <SortableHead label={t('admin.auditAlertsTab.colTime')} field="created_at" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label={t('admin.auditAlertsTab.colType')} field="alert_type" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label={t('admin.auditAlertsTab.colSeverity')} field="severity" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead>{t('admin.auditAlertsTab.colTitle')}</TableHead>
                            <TableHead>{t('admin.auditAlertsTab.colDescription')}</TableHead>
                            <SortableHead label={t('admin.auditAlertsTab.colStatus')} field="status" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead className="text-right">{t('common.actions')}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={8} cols={8} /></TableCell></TableRow>
                        ) : alerts?.data?.length === 0 ? (
                            <TableEmptyRow colSpan={8} icon={ShieldAlert} title={t('admin.auditAlertsTab.emptyTitle')} />
                        ) : (
                            sortedAlerts.map((alert) => (
                                <AlertRow
                                    key={alert.id}
                                    alert={alert}
                                    isSelected={selectedAlertIds.includes(alert.id)}
                                    onCheckChange={(checked) => onAlertSelect(alert.id, checked)}
                                    onSelect={onSelectAlert}
                                    onResolve={() => resolveAlertMutation.mutate(alert.id)}
                                    isResolving={resolveAlertMutation.isPending}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
                {alerts && (
                    <AuditPagination
                        total={alerts.total}
                        totalPages={alerts.total_pages}
                        currentPage={currentPage}
                        onPageChange={onPageChange}
                    />
                )}
            </CardContent>
        </Card>
    )
}

interface SortableHeadProps {
    label: string
    field: AlertSortField
    sortConfig: AlertSortConfig
    onSort: (field: AlertSortField) => void
}

function SortableHead({ label, field, sortConfig, onSort }: SortableHeadProps) {
    const isSorted = sortConfig.field === field
    const ariaSort = !isSorted ? 'none' : sortConfig.order === 'asc' ? 'ascending' : 'descending'
    return (
        <TableHead
            className="cursor-pointer hover:bg-muted/50 transition-colors"
            tabIndex={0}
            aria-sort={ariaSort}
            onClick={() => onSort(field)}
            onKeyDown={onActivateKey(() => onSort(field))}
        >
            <div className="flex items-center">
                {label}
                {getSortIcon(sortConfig, field)}
            </div>
        </TableHead>
    )
}

interface AlertRowProps {
    alert: SecurityAlert
    isSelected: boolean
    onCheckChange: (checked: boolean) => void
    onSelect: (alert: SecurityAlert) => void
    onResolve: () => void
    isResolving: boolean
}

function AlertRow({ alert, isSelected, onCheckChange, onSelect, onResolve, isResolving }: AlertRowProps) {
    const { t } = useTranslation()
    return (
        <TableRow
            className="cursor-pointer hover:bg-muted/50 transition-colors"
            tabIndex={0}
            aria-label={t('admin.auditAlertsTab.viewAlertAriaLabel', { title: alert.title })}
            onClick={() => onSelect(alert)}
            onKeyDown={onActivateKey(() => onSelect(alert))}
            data-selected={isSelected}
        >
            <TableCell onClick={(e) => e.stopPropagation()}>
                {alert.status !== 'resolved' && (
                    <Checkbox
                        checked={isSelected}
                        onCheckedChange={(v) => onCheckChange(!!v)}
                        aria-label={t('admin.auditAlertsTab.selectAlertAriaLabel')}
                    />
                )}
            </TableCell>
            <TableCell>
                <span className="block">
                    {new Date(alert.created_at).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit' })}
                </span>
                <span className="block text-muted-foreground text-sm">
                    {new Date(alert.created_at).toLocaleTimeString(uiLocale(), { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit' })}
                </span>
            </TableCell>
            <TableCell><Badge variant="outline">{alertTypeLabels[alert.alert_type] ?? alert.alert_type}</Badge></TableCell>
            <TableCell><Badge variant={getSeverityColor(alert.severity)}>{severityLabels[alert.severity] ?? alert.severity}</Badge></TableCell>
            <TableCell>{alert.title}</TableCell>
            <TableCell className="text-sm text-muted-foreground max-w-[300px] whitespace-normal break-words" title={alert.description || ''}>
                <AlertDescription alert={alert} t={t} />
            </TableCell>
            <TableCell>
                <Badge variant={alert.status === 'resolved' ? 'secondary' : 'default'}>
                    {alert.status === 'open' ? t('admin.auditAlertsTab.statusPending') : t('admin.auditAlertsTab.statusResolved')}
                </Badge>
            </TableCell>
            <TableCell>
                <div className="flex items-center justify-end gap-1">
                    {alert.status !== 'resolved' && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={(e) => { e.stopPropagation(); onResolve() }}
                            disabled={isResolving}
                        >
                            {t('admin.auditAlertsTab.markResolved')}
                        </Button>
                    )}
                </div>
            </TableCell>
        </TableRow>
    )
}

function AlertDescription({ alert, t }: { alert: SecurityAlert; t: TFunction }) {
    if (alert.alert_type === 'global_mass_login' && alert.context_data) {
        const hasCount = typeof alert.context_data === 'object' && 'account_count' in alert.context_data
        return (
            <span>
                {alert.description}
                {hasCount ? ` ${t('admin.auditAlertsTab.accountCount', { count: (alert.context_data as Record<string, unknown>).account_count as number })}` : ''}
            </span>
        )
    }
    return <>{alert.description || '-'}</>
}
