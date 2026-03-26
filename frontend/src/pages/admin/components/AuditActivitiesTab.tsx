import { Eye, FileText } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'

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

const ACTION_LABEL_MAP: Record<string, string> = {
    'CREATE': '建立使用者',
    'UPDATE': '更新使用者',
    'DELETE': '刪除使用者',
    'PASSWORD_RESET': '重設密碼',
    'PASSWORD_CHANGE': '變更密碼',
    'IMPERSONATE': '模擬登入',
    'STOP_IMPERSONATE': '停止模擬',
    'STATUS_CHANGE': '狀態變更',
    'ASSIGN': '角色指派',
    'UNASSIGN': '角色移除',
    'force_logout': '強制登出',
}

const FIELD_LABEL_MAP: Record<string, string> = {
    display_name: '顯示名稱',
    email: '信箱',
    is_active: '啟用狀態',
    roles: '角色',
    phone: '電話',
    organization: '組織',
}

function getActionSummary(log: AuditLog): string {
    const summaryLabels: Record<string, string> = {
        'CREATE': '建立使用者',
        'UPDATE': '更新使用者資料',
        'DELETE': '刪除使用者',
        'PASSWORD_RESET': '重設密碼',
        'PASSWORD_CHANGE': '變更密碼',
        'IMPERSONATE': '模擬登入',
        'STOP_IMPERSONATE': '停止模擬',
        'STATUS_CHANGE': '狀態變更',
        'ASSIGN': '指派角色',
        'UNASSIGN': '移除角色',
        'force_logout': '強制登出 Session',
    }

    const base = summaryLabels[log.action] || log.action

    if (log.action === 'IMPERSONATE' && log.after_data) {
        const data = log.after_data as Record<string, string>
        const target = data.impersonated_email || data.impersonated_user_id?.slice(0, 8)
        return target ? `${base} → ${target}` : base
    }

    if (log.action === 'UPDATE' && log.after_data) {
        const keys = Object.keys(log.after_data)
        const changed = keys.map(k => FIELD_LABEL_MAP[k] || k).slice(0, 3)
        return `${base}：${changed.join('、')}${keys.length > 3 ? ' 等' : ''}`
    }

    if (log.action === 'CREATE' && log.after_data) {
        const data = log.after_data as Record<string, string>
        if (data.email) return `${base}：${data.email}`
    }

    if (log.action === 'force_logout' && log.after_data) {
        const data = log.after_data as Record<string, string>
        if (data.reason) return `${base}：${data.reason}`
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
    const { sortedData, sort, toggleSort } = useTableSort(activityLogs?.data)

    return (
        <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-4">
                <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => onDateFromChange(e.target.value)}
                    placeholder="開始日期"
                    className="max-w-[150px]"
                />
                <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => onDateToChange(e.target.value)}
                    placeholder="結束日期"
                    className="max-w-[150px]"
                />
            </div>
            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
                            <SortableTableHead sortKey="actor_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>操作者</SortableTableHead>
                            <SortableTableHead sortKey="action" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>操作</SortableTableHead>
                            <SortableTableHead sortKey="entity_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>目標使用者</SortableTableHead>
                            <TableHead>摘要</TableHead>
                            <TableHead className="w-[60px]">詳情</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center py-8">載入中...</TableCell>
                            </TableRow>
                        ) : !sortedData || sortedData.length === 0 ? (
                            <TableEmptyRow colSpan={6} icon={FileText} title="沒有使用者管理活動記錄" />
                        ) : (
                            sortedData.map((log) => (
                                <ActivityRow key={log.id} log={log} onSelect={onSelectLog} />
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

function ActivityRow({ log, onSelect }: { log: AuditLog; onSelect: (log: AuditLog) => void }) {
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
                    {ACTION_LABEL_MAP[log.action] || log.action}
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
                <span className="text-muted-foreground">{getActionSummary(log)}</span>
            </TableCell>
            <TableCell>
                <Button variant="ghost" size="icon" onClick={() => onSelect(log)} title="查看詳情" aria-label="查看詳情">
                    <Eye className="h-4 w-4" />
                </Button>
            </TableCell>
        </TableRow>
    )
}
