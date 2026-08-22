/**
 * 設備閒置申請分頁內容：表格、分頁、核准/駁回操作
 */
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Check, X, FileText } from 'lucide-react'
import { format } from 'date-fns'
import { getDateFnsLocale } from '@/lib/utils'

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

const REQUEST_TYPE_LABEL_KEYS: Record<string, string> = {
  idle: 'admin.idleTabContent.requestTypeIdle',
  restore: 'admin.idleTabContent.requestTypeRestore',
}

interface IdleTabContentProps {
  canApprove: boolean
  records: IdleRequestWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onApprove: (id: string, approved: boolean) => void
  /** R71-9：核准/駁回進行中時停用按鈕，防連點重複送出 */
  isApproving: boolean
}

export function IdleTabContent({
  canApprove,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onApprove,
  isApproving,
}: IdleTabContentProps) {
  const { t } = useTranslation()
  const columns = useMemo<ColumnDef<IdleRequestWithDetails>[]>(() => [
    { key: 'equipment', header: t('admin.idleTabContent.colEquipment'), cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
    {
      key: 'type', header: t('admin.idleTabContent.colType'),
      cell: (r) => REQUEST_TYPE_LABEL_KEYS[r.request_type] ? t(REQUEST_TYPE_LABEL_KEYS[r.request_type]) : r.request_type,
    },
    {
      key: 'status', header: t('admin.idleTabContent.colStatus'),
      cell: (r) => (
        <StatusBadge variant={STATUS_VARIANT[r.status]}>
          {t(DISPOSAL_STATUS_LABELS[r.status])}
        </StatusBadge>
      ),
    },
    { key: 'reason', header: t('admin.idleTabContent.colReason'), className: 'max-w-[240px] whitespace-normal break-words', cell: (r) => r.reason },
    { key: 'applicant', header: t('admin.idleTabContent.colApplicant'), cell: (r) => r.applicant_name },
    {
      key: 'appliedAt', header: t('admin.idleTabContent.colAppliedAt'),
      cell: (r) => format(new Date(r.applied_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }),
    },
    { key: 'approver', header: t('admin.idleTabContent.colApprover'), cell: (r) => r.approver_name || '—' },
    {
      key: 'actions', header: t('common.actions'), className: 'w-[100px] text-right',
      cell: (r) => (
        <div className="flex items-center justify-end gap-1">
          {canApprove && r.status === 'pending' && (
            <>
              <Button variant="ghost" size="icon" className="text-status-success-text hover:text-status-success-text/80" onClick={() => onApprove(r.id, true)} disabled={isApproving} aria-label={t('admin.idleTabContent.approve')}>
                <Check className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onApprove(r.id, false)} disabled={isApproving} aria-label={t('admin.idleTabContent.reject')}>
                <X className="h-4 w-4" />
              </Button>
            </>
          )}
        </div>
      ),
    },
  ], [canApprove, onApprove, isApproving, t])

  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle>{t('admin.idleTabContent.title')}</CardTitle>
          <CardDescription>{t('admin.idleTabContent.description')}</CardDescription>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <DataTable
          columns={columns}
          data={records}
          isLoading={isLoading}
          emptyIcon={FileText}
          emptyTitle={t('admin.idleTabContent.empty')}
          rowKey={(r) => r.id}
          page={page}
          totalPages={totalPages}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  )
}
