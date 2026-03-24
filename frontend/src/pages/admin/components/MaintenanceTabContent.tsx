/**
 * 維修/保養紀錄分頁內容：表格、分頁
 */
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Trash2, Loader2 } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS, MAINTENANCE_STATUS_LABELS } from '../types'

interface MaintenanceTabContentProps {
  canManage: boolean
  records: MaintenanceRecordWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onDelete: (id: string) => void
}

const TYPE_COLORS: Record<string, string> = {
  repair: 'bg-red-100 text-red-800',
  maintenance: 'bg-blue-100 text-blue-800',
}

const STATUS_COLORS: Record<string, string> = {
  pending: 'bg-gray-100 text-gray-800',
  in_progress: 'bg-yellow-100 text-yellow-800',
  completed: 'bg-green-100 text-green-800',
  unrepairable: 'bg-red-100 text-red-800',
}

function truncateText(text: string | null, maxLength: number): string {
  if (!text) return '—'
  return text.length > maxLength ? `${text.slice(0, maxLength)}...` : text
}

export function MaintenanceTabContent({
  canManage,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onDelete,
}: MaintenanceTabContentProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>維修/保養紀錄</CardTitle>
        <CardDescription>查看設備維修與定期保養紀錄</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-md border">
          {isLoading ? (
            <div className="flex justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : records.length === 0 ? (
            <div className="py-12 text-center text-muted-foreground">尚無紀錄</div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>設備</TableHead>
                  <TableHead>類型</TableHead>
                  <TableHead>狀態</TableHead>
                  <TableHead>報修日期</TableHead>
                  <TableHead>完修日期</TableHead>
                  <TableHead>問題描述/保養項目</TableHead>
                  {canManage && <TableHead className="w-[100px] text-right">操作</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.equipment_name}</TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        className={TYPE_COLORS[r.maintenance_type] || ''}
                      >
                        {MAINTENANCE_TYPE_LABELS[r.maintenance_type]}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        className={STATUS_COLORS[r.status] || ''}
                      >
                        {MAINTENANCE_STATUS_LABELS[r.status]}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {format(new Date(r.reported_at), 'yyyy/MM/dd', { locale: zhTW })}
                    </TableCell>
                    <TableCell>
                      {r.completed_at
                        ? format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW })
                        : '—'}
                    </TableCell>
                    <TableCell className="max-w-[200px]">
                      {r.maintenance_type === 'repair'
                        ? truncateText(r.problem_description, 30)
                        : truncateText(r.maintenance_items, 30)}
                    </TableCell>
                    {canManage && (
                      <TableCell>
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-destructive hover:text-destructive"
                            onClick={() => onDelete(r.id)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </div>
        {totalPages > 1 && (
          <div className="flex justify-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page <= 1}
              onClick={() => onPageChange(page - 1)}
            >
              上一頁
            </Button>
            <span className="flex items-center px-4 text-sm text-muted-foreground">
              第 {page} / {totalPages} 頁
            </span>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages}
              onClick={() => onPageChange(page + 1)}
            >
              下一頁
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
