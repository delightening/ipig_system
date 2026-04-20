import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { History, Eye, ChevronLeft, ChevronRight } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import type { UserActivityLog } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { categoryLabels, eventTypeLabels, entityTypeLabels } from '../constants/auditLogs'

interface AuditLogTableProps {
  activityLogs: PaginatedResponse<UserActivityLog> | undefined
  isLoading: boolean
  currentPage: number
  totalPages: number
  onPageChange: (page: number) => void
  onSelectLog: (log: UserActivityLog) => void
}

function formatDateTimeDisplay(dateStr: string) {
  const d = new Date(dateStr)
  const datePart = d.toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit' })
  const timePart = d.toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit' })
  return (
    <div className="block leading-tight">
      <div>{datePart}</div>
      <div>{timePart}</div>
    </div>
  )
}

function getEventBadge(eventType: string) {
  const config = eventTypeLabels[eventType] || { label: eventType, color: 'bg-muted0' }
  return (
    <Badge className={`${config.color} text-white`}>
      {config.label}
    </Badge>
  )
}

export function AuditLogTable({
  activityLogs,
  isLoading,
  currentPage,
  totalPages,
  onPageChange,
  onSelectLog,
}: AuditLogTableProps) {
  const { sortedData, sort, toggleSort } = useTableSort(activityLogs?.data)

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
            <SortableTableHead sortKey="actor_display_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>操作者</SortableTableHead>
            <SortableTableHead sortKey="event_category" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>類別</SortableTableHead>
            <SortableTableHead sortKey="event_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>事件</SortableTableHead>
            <SortableTableHead sortKey="entity_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>實體類型</SortableTableHead>
            <SortableTableHead sortKey="entity_display_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>實體名稱</SortableTableHead>
            <TableHead className="text-right">詳情</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={7} className="p-0">
                <TableSkeleton rows={8} cols={7} />
              </TableCell>
            </TableRow>
          ) : sortedData && sortedData.length > 0 ? (
            sortedData.map((log) => (
              <TableRow key={log.id}>
                <TableCell className="text-sm">
                  {formatDateTimeDisplay(log.created_at)}
                </TableCell>
                <TableCell>
                  <div>
                    <p className="font-medium">{log.actor_display_name || '-'}</p>
                    <p className="text-xs text-muted-foreground">{log.actor_email || ''}</p>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline">
                    {categoryLabels[log.event_category] || log.event_category}
                  </Badge>
                </TableCell>
                <TableCell>{getEventBadge(log.event_type)}</TableCell>
                <TableCell>
                  <Badge variant="secondary">
                    {entityTypeLabels[log.entity_type || ''] || log.entity_type || '-'}
                  </Badge>
                </TableCell>
                <TableCell className="text-sm">
                  {log.entity_display_name || (log.entity_id ? `${log.entity_id.slice(0, 8)}...` : '-')}
                </TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onSelectLog(log)}
                    aria-label="查看詳情"
                  >
                    <Eye className="h-4 w-4" />
                  </Button>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableEmptyRow colSpan={7} icon={History} title="尚無操作日誌" />
          )}
        </TableBody>
      </Table>

      {activityLogs && totalPages > 0 && (
        <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
          <p className="text-sm text-muted-foreground">
            共 {activityLogs.total} 筆，第 {currentPage} / {totalPages} 頁
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage <= 1}
              onClick={() => onPageChange(Math.max(1, currentPage - 1))}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              上一頁
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage >= totalPages}
              onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
            >
              下一頁
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
