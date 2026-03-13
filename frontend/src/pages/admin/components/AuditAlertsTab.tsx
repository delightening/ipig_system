import { type ReactNode } from 'react'
import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import type { UseMutationResult } from '@tanstack/react-query'
import type { AxiosResponse } from 'axios'

import { Button } from '@/components/ui/button'
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
}: AuditAlertsTabProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>安全警報</CardTitle>
                <CardDescription>需要關注的安全事件</CardDescription>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableHead label="時間" field="created_at" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label="類型" field="alert_type" sortConfig={sortConfig} onSort={onSort} />
                            <SortableHead label="嚴重程度" field="severity" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead>標題</TableHead>
                            <TableHead>描述</TableHead>
                            <SortableHead label="狀態" field="status" sortConfig={sortConfig} onSort={onSort} />
                            <TableHead>操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8">載入中...</TableCell>
                            </TableRow>
                        ) : alerts?.data?.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8 text-muted-foreground">
                                    沒有安全警報
                                </TableCell>
                            </TableRow>
                        ) : (
                            sortedAlerts.map((alert) => (
                                <AlertRow
                                    key={alert.id}
                                    alert={alert}
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
    onSelect: (alert: SecurityAlert) => void
    onResolve: () => void
    isResolving: boolean
}

function AlertRow({ alert, onSelect, onResolve, isResolving }: AlertRowProps) {
    return (
        <TableRow
            className="cursor-pointer hover:bg-muted/50 transition-colors"
            onClick={() => onSelect(alert)}
        >
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
            <TableCell className="text-sm text-muted-foreground max-w-[300px] truncate" title={alert.description || ''}>
                <AlertDescription alert={alert} />
            </TableCell>
            <TableCell>
                <Badge variant={alert.status === 'resolved' ? 'secondary' : 'default'}>
                    {alert.status === 'open' ? '待處理' : '已解決'}
                </Badge>
            </TableCell>
            <TableCell>
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
