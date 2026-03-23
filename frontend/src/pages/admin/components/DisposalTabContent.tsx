/**
 * 報廢紀錄分頁內容：表格、分頁、核准/駁回操作
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
import { Loader2, Check, X } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { DisposalWithDetails, DisposalStatus } from '../types'
import { DISPOSAL_STATUS_LABELS } from '../types'

interface DisposalTabContentProps {
  canApprove: boolean
  records: DisposalWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onApprove: (id: string, approved: boolean) => void
}

const STATUS_COLORS: Record<DisposalStatus, string> = {
  pending: 'bg-yellow-100 text-yellow-800',
  approved: 'bg-green-100 text-green-800',
  rejected: 'bg-red-100 text-red-800',
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text
  }
  return `${text.slice(0, maxLength)}...`
}

export function DisposalTabContent({
  canApprove,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onApprove,
}: DisposalTabContentProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>報廢紀錄</CardTitle>
        <CardDescription>設備報廢申請與核准紀錄</CardDescription>
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
                  <TableHead>狀態</TableHead>
                  <TableHead>報廢日期</TableHead>
                  <TableHead>原因</TableHead>
                  <TableHead>申請人</TableHead>
                  <TableHead>申請時間</TableHead>
                  <TableHead>核准人</TableHead>
                  <TableHead className="w-[120px]">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.equipment_name}</TableCell>
                    <TableCell>
                      <Badge variant="outline" className={STATUS_COLORS[r.status]}>
                        {DISPOSAL_STATUS_LABELS[r.status]}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {r.disposal_date
                        ? format(new Date(r.disposal_date), 'yyyy/MM/dd', { locale: zhTW })
                        : '—'}
                    </TableCell>
                    <TableCell title={r.reason}>{truncateText(r.reason, 20)}</TableCell>
                    <TableCell>{r.applicant_name}</TableCell>
                    <TableCell>
                      {format(new Date(r.applied_at), 'yyyy/MM/dd', { locale: zhTW })}
                    </TableCell>
                    <TableCell>{r.approver_name || '—'}</TableCell>
                    <TableCell>
                      {canApprove && r.status === 'pending' && (
                        <div className="flex gap-2">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-green-600 hover:text-green-700"
                            onClick={() => onApprove(r.id, true)}
                          >
                            <Check className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-destructive hover:text-destructive"
                            onClick={() => onApprove(r.id, false)}
                          >
                            <X className="h-4 w-4" />
                          </Button>
                        </div>
                      )}
                    </TableCell>
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
