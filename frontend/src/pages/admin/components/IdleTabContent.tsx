/**
 * 設備閒置申請分頁內容：表格、分頁、核准/駁回操作
 */
import { useMemo } from 'react'
import { truncateText } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Check, X, FileText } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { DisposalStatus } from '../types'
import { DISPOSAL_STATUS_LABELS } from '../types'

export interface IdleRequestWithDetails {
  id: string
  equipment_id: string
  equipment_name: string
  request_type: 'idle' | 'restore'
  reason: string
  status: DisposalStatus
  applied_by: string
  applicant_name: string
  applied_at: string
  approved_by: string | null
  approver_name: string | null
  approved_at: string | null
  rejection_reason: string | null
  notes: string | null
  created_at: string
}

const STATUS_VARIANT: Record<DisposalStatus, 'warning' | 'success' | 'error'> = {
  pending: 'warning',
  approved: 'success',
  rejected: 'error',
}

const REQUEST_TYPE_LABELS: Record<string, string> = {
  idle: '閒置申請',
  restore: '恢復申請',
}

interface IdleTabContentProps {
  canApprove: boolean
  records: IdleRequestWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onApprove: (id: string, approved: boolean) => void
}

export function IdleTabContent({
  canApprove,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onApprove,
}: IdleTabContentProps) {
  const columns = useMemo<ColumnDef<IdleRequestWithDetails>[]>(() => [
    { key: 'equipment', header: '設備', cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
    {
      key: 'type', header: '類型',
      cell: (r) => REQUEST_TYPE_LABELS[r.request_type] ?? r.request_type,
    },
    {
      key: 'status', header: '狀態',
      cell: (r) => (
        <StatusBadge variant={STATUS_VARIANT[r.status]}>
          {DISPOSAL_STATUS_LABELS[r.status]}
        </StatusBadge>
      ),
    },
    { key: 'reason', header: '原因', cell: (r) => <span title={r.reason}>{truncateText(r.reason, 20)}</span> },
    { key: 'applicant', header: '申請人', cell: (r) => r.applicant_name },
    {
      key: 'appliedAt', header: '申請時間',
      cell: (r) => format(new Date(r.applied_at), 'yyyy/MM/dd', { locale: zhTW }),
    },
    { key: 'approver', header: '核准人', cell: (r) => r.approver_name || '—' },
    {
      key: 'actions', header: '操作', className: 'w-[100px] text-right',
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
        </div>
      ),
    },
  ], [canApprove, onApprove])

  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle>閒置管理</CardTitle>
          <CardDescription>設備閒置/恢復申請與核准紀錄</CardDescription>
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
