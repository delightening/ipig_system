/**
 * 報廢紀錄分頁內容：表格、分頁、核准/駁回操作、申請報廢
 */
import { useMemo } from 'react'
import { truncateText } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Check, Plus, X, FileText, RotateCcw } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { DisposalWithDetails, DisposalStatus } from '../types'
import { DISPOSAL_STATUS_LABELS } from '../types'

interface DisposalTabContentProps {
  canApprove: boolean
  canRequest?: boolean
  records: DisposalWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onApprove: (id: string, approved: boolean) => void
  onRestore?: (id: string) => void
  onRequestDisposal?: () => void
}

const STATUS_VARIANT: Record<DisposalStatus, 'warning' | 'success' | 'error'> = {
  pending: 'warning',
  approved: 'success',
  rejected: 'error',
}


export function DisposalTabContent({
  canApprove,
  canRequest,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onApprove,
  onRestore,
  onRequestDisposal,
}: DisposalTabContentProps) {
  const columns = useMemo<ColumnDef<DisposalWithDetails>[]>(() => [
    { key: 'equipment', header: '設備', cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
    {
      key: 'status', header: '狀態',
      cell: (r) => (
        <StatusBadge variant={STATUS_VARIANT[r.status]}>
          {DISPOSAL_STATUS_LABELS[r.status]}
        </StatusBadge>
      ),
    },
    {
      key: 'date', header: '報廢日期',
      cell: (r) => r.disposal_date ? format(new Date(r.disposal_date), 'yyyy/MM/dd', { locale: zhTW }) : '—',
    },
    { key: 'reason', header: '原因', cell: (r) => <span title={r.reason}>{truncateText(r.reason, 20)}</span> },
    { key: 'applicant', header: '申請人', cell: (r) => r.applicant_name },
    {
      key: 'appliedAt', header: '申請時間',
      cell: (r) => format(new Date(r.applied_at), 'yyyy/MM/dd', { locale: zhTW }),
    },
    { key: 'approver', header: '核准人', cell: (r) => r.approver_name || '—' },
    {
      key: 'actions', header: '操作', className: 'w-[120px] text-right',
      cell: (r) => (
        <div className="flex items-center justify-end gap-1">
          {canApprove && r.status === 'pending' && (
            <>
              <Button variant="ghost" size="icon" className="text-status-success-text hover:text-status-success-text/80" onClick={() => onApprove(r.id, true)} aria-label="核准">
                <Check className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onApprove(r.id, false)} aria-label="駁回">
                <X className="h-4 w-4" />
              </Button>
            </>
          )}
          {canApprove && r.status === 'approved' && onRestore && (
            <Button variant="ghost" size="icon" className="text-primary hover:text-primary/80" onClick={() => onRestore(r.id)} aria-label="恢復設備" title="恢復設備為啟用">
              <RotateCcw className="h-4 w-4" />
            </Button>
          )}
        </div>
      ),
    },
  ], [canApprove, onApprove, onRestore])

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>報廢紀錄</CardTitle>
            <CardDescription>設備報廢申請與核准紀錄</CardDescription>
          </div>
          {canRequest && onRequestDisposal && (
            <Button size="sm" onClick={onRequestDisposal}>
              <Plus className="h-4 w-4 mr-2" />
              申請報廢
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <DataTable
          columns={columns}
          data={records}
          isLoading={isLoading}
          emptyIcon={FileText}
          emptyTitle="尚無紀錄"
          rowKey={(r) => r.id}
          page={page}
          totalPages={totalPages}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  )
}
