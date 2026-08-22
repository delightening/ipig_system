/**
 * 報廢紀錄分頁內容：表格、分頁、核准/駁回操作、申請報廢
 */
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Check, Plus, X, FileText, RotateCcw } from 'lucide-react'
import { format } from 'date-fns'
import { getDateFnsLocale } from '@/lib/utils'

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
  const { t } = useTranslation()
  const columns = useMemo<ColumnDef<DisposalWithDetails>[]>(() => [
    { key: 'equipment', header: t('admin.disposalTabContent.equipment'), cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
    {
      key: 'status', header: t('admin.disposalTabContent.status'),
      cell: (r) => (
        <StatusBadge variant={STATUS_VARIANT[r.status]}>
          {t(DISPOSAL_STATUS_LABELS[r.status])}
        </StatusBadge>
      ),
    },
    {
      key: 'date', header: t('admin.disposalTabContent.disposalDate'),
      cell: (r) => r.disposal_date ? format(new Date(r.disposal_date), 'yyyy/MM/dd', { locale: getDateFnsLocale() }) : '—',
    },
    { key: 'reason', header: t('admin.disposalTabContent.reason'), className: 'max-w-[240px] whitespace-normal break-words', cell: (r) => r.reason },
    { key: 'applicant', header: t('admin.disposalTabContent.applicant'), cell: (r) => r.applicant_name },
    {
      key: 'appliedAt', header: t('admin.disposalTabContent.appliedAt'),
      cell: (r) => format(new Date(r.applied_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }),
    },
    { key: 'approver', header: t('admin.disposalTabContent.approver'), cell: (r) => r.approver_name || '—' },
    {
      key: 'actions', header: t('common.actions'), className: 'w-[120px] text-right',
      cell: (r) => (
        <div className="flex items-center justify-end gap-1">
          {canApprove && r.status === 'pending' && (
            <>
              <Button variant="ghost" size="icon" className="text-status-success-text hover:text-status-success-text/80" onClick={() => onApprove(r.id, true)} aria-label={t('admin.disposalTabContent.approve')}>
                <Check className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onApprove(r.id, false)} aria-label={t('admin.disposalTabContent.reject')}>
                <X className="h-4 w-4" />
              </Button>
            </>
          )}
          {canApprove && r.status === 'approved' && onRestore && (
            <Button variant="ghost" size="icon" className="text-primary hover:text-primary/80" onClick={() => onRestore(r.id)} aria-label={t('admin.disposalTabContent.restoreEquipment')} title={t('admin.disposalTabContent.restoreEquipmentTitle')}>
              <RotateCcw className="h-4 w-4" />
            </Button>
          )}
        </div>
      ),
    },
  ], [canApprove, onApprove, onRestore, t])

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{t('admin.disposalTabContent.title')}</CardTitle>
            <CardDescription>{t('admin.disposalTabContent.description')}</CardDescription>
          </div>
          {canRequest && onRequestDisposal && (
            <Button size="sm" onClick={onRequestDisposal}>
              <Plus className="h-4 w-4 mr-2" />
              {t('admin.disposalTabContent.requestDisposal')}
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
          emptyTitle={t('admin.disposalTabContent.emptyTitle')}
          rowKey={(r) => r.id}
          page={page}
          totalPages={totalPages}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  )
}
