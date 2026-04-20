import { type ReactNode } from 'react'
import { ArrowDown, ArrowUp, ArrowUpDown, CheckSquare } from 'lucide-react'
import type { UseMutationResult } from '@tanstack/react-query'
import type { AxiosResponse } from 'axios'

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
    const openAlerts = sortedAlerts.filter(a => a.status !== 'resolved')
    const allOpenSelected = openAlerts.length > 0 && openAlerts.every(a => selectedAlertIds.includes(a.id))
    const someSelected = selectedAlertIds.length > 0

    return (
        <Card className="overflow-hidden">
            <CardHeader className="flex flex-col space-y-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
                <div>
                    <CardTitle>安全警報</CardTitle>
                    <CardDescription>需要關注的安全事件</CardDescription>
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
                            標記解決（{selectedAlertIds.length}）
                        </Button>
                    )}
                    <div className="relative w-full sm:w-64">
                        <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="搜尋標題或描述..."
                            className="pl-9"
                            value={search}
                            onChange={(e) => onSearchChange(e.target.value)}
                        />
                    </div>
                    <Select value={statusFilter} onValueChange={onStatusFilterChange}>
                        <SelectTrigger className="w-full sm:w-32">
                            <SelectValue placeholder="狀態篩選" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">全部狀態</SelectItem>
                            <SelectItem value="open">待處理</SelectItem>
                            <SelectItem value="resolved">已解決</SelectItem>
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
                                    aria-label="全選待處理警報"
                                />
                            </TableHead>
                            <SortableHead label="時間" field="created_at" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label="類型" field="alert_type" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label="嚴重程度" field="severity" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead>標題</TableHead>
                            <TableHead>描述</TableHead>
                            <SortableHead label="狀態" field="status" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead className="text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={8} cols={8} /></TableCell></TableRow>
                        ) : alerts?.data?.length === 0 ? (
                            <TableEmptyRow colSpan={8} icon={ShieldAlert} title="沒有安全警報" />
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
    return (
        <TableHead
            className="cursor-pointer hover:bg-muted/50 transition-colors"
            onClick={() => onSort(field)}
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
    return (
        <TableRow
            className="cursor-pointer hover:bg-muted/50 transition-colors"
            onClick={() => onSelect(alert)}
            data-selected={isSelected}
        >
            <TableCell onClick={(e) => e.stopPropagation()}>
                {alert.status !== 'resolved' && (
                    <Checkbox
                        checked={isSelected}
                        onCheckedChange={(v) => onCheckChange(!!v)}
                        aria-label="選取此警報"
                    />
                )}
            </TableCell>
            <TableCell>
                <span className="block">
                    {new Date(alert.created_at).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit' })}
                </span>
                <span className="block text-muted-foreground text-sm">
                    {new Date(alert.created_at).toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit' })}
                </span>
            </TableCell>
            <TableCell><Badge variant="outline">{alert.alert_type}</Badge></TableCell>
            <TableCell><Badge variant={getSeverityColor(alert.severity)}>{alert.severity}</Badge></TableCell>
            <TableCell>{alert.title}</TableCell>
            <TableCell className="text-sm text-muted-foreground max-w-[300px] whitespace-normal break-words" title={alert.description || ''}>
                <AlertDescription alert={alert} />
            </TableCell>
            <TableCell>
                <Badge variant={alert.status === 'resolved' ? 'secondary' : 'default'}>
                    {alert.status === 'open' ? '待處理' : '已解決'}
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
                            標記解決
                        </Button>
                    )}
                </div>
            </TableCell>
        </TableRow>
    )
}

function AlertDescription({ alert }: { alert: SecurityAlert }) {
    if (alert.alert_type === 'global_mass_login' && alert.context_data) {
        const hasCount = typeof alert.context_data === 'object' && 'account_count' in alert.context_data
        return (
            <span>
                {alert.description}
                {hasCount ? ` (數量: ${(alert.context_data as Record<string, unknown>).account_count})` : ''}
            </span>
        )
    }
    return <>{alert.description || '-'}</>
}
