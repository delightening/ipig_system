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
import { Loader2, History, Eye, ChevronLeft, ChevronRight } from 'lucide-react'
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
  const config = eventTypeLabels[eventType] || { label: eventType, color: 'bg-gray-500' }
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
  return (
    <div className="rounded-md border bg-white overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>時間</TableHead>
            <TableHead>操作者</TableHead>
            <TableHead>類別</TableHead>
            <TableHead>事件</TableHead>
            <TableHead>實體類型</TableHead>
            <TableHead>實體名稱</TableHead>
            <TableHead className="text-right">詳情</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center py-8">
                <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
              </TableCell>
            </TableRow>
          ) : activityLogs?.data && activityLogs.data.length > 0 ? (
            activityLogs.data.map((log) => (
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
                  >
                    <Eye className="h-4 w-4" />
                  </Button>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={7} className="text-center py-8">
                <History className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                <p className="text-muted-foreground">尚無操作日誌</p>
              </TableCell>
            </TableRow>
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
