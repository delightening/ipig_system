import { useTranslation } from 'react-i18next'

import { Partner } from '@/lib/api'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useAuthStore } from '@/stores/auth'
import { useTableSort } from '@/hooks/useTableSort'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { Edit, Trash2, Users } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { cn } from '@/lib/utils'

interface PartnerTableProps {
  partners: Partner[] | undefined
  isLoading: boolean
  onEdit: (partner: Partner) => void
  onDelete: (partner: Partner, hard: boolean) => void
  confirm: (opts: {
    title: string
    description: string
    variant?: 'default' | 'destructive'
    confirmLabel?: string
  }) => Promise<boolean>
}

export function PartnerTable({
  partners,
  isLoading,
  onEdit,
  onDelete,
  confirm,
}: PartnerTableProps) {
  const { t } = useTranslation()
  const { sortedData, sort, toggleSort } = useTableSort(partners)

  const handleDeleteClick = async (partner: Partner) => {
    const isAdmin = useAuthStore.getState().user?.roles.includes('admin')
    const ok = await confirm({
      title: isAdmin ? t('erpMaster.partners.table.hardDeleteTitle') : t('erpMaster.partners.table.deleteTitle'),
      description: isAdmin
        ? t('erpMaster.partners.table.hardDeleteDescription')
        : t('erpMaster.partners.table.deleteDescription'),
      variant: 'destructive',
      confirmLabel: isAdmin ? t('erpMaster.partners.table.hardDeleteConfirm') : t('common.confirmDelete'),
    })
    if (ok) {
      onDelete(partner, !!isAdmin)
    }
  }

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            <SortableTableHead sortKey="partner_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.partners.table.type')}</SortableTableHead>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.common.code')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.common.name')}</SortableTableHead>
            <SortableTableHead sortKey="tax_id" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.partners.taxId')}</SortableTableHead>
            <SortableTableHead sortKey="phone" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.partners.phone')}</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.common.status')}</SortableTableHead>
            <TableHead className="text-right">{t('common.actions')}</TableHead>
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
            sortedData.map((partner) => (
              <PartnerRow
                key={partner.id}
                partner={partner}
                onEdit={onEdit}
                onDelete={handleDeleteClick}
              />
            ))
          ) : (
            <TableEmptyRow colSpan={7} icon={Users} title={t('erpMaster.partners.table.empty')} />
          )}
        </TableBody>
      </Table>
    </div>
  )
}

function PartnerRow({
  partner,
  onEdit,
  onDelete,
}: {
  partner: Partner
  onEdit: (p: Partner) => void
  onDelete: (p: Partner) => void
}) {
  const { t } = useTranslation()

  return (
    <TableRow className={cn(!partner.is_active && 'bg-muted/40')}>
      <TableCell>
        <Badge variant={partner.partner_type === 'supplier' ? 'default' : 'secondary'}>
          {partner.partner_type === 'supplier' ? t('erpMaster.partners.type.supplier') : t('erpMaster.partners.type.customer')}
        </Badge>
      </TableCell>
      <TableCell className="font-mono">{partner.code}</TableCell>
      <TableCell className="font-medium">{partner.name}</TableCell>
      <TableCell>{partner.tax_id || '-'}</TableCell>
      <TableCell>
        {partner.phone || '-'}
        {partner.phone_ext ? ` #${partner.phone_ext}` : ''}
      </TableCell>
      <TableCell>
        {partner.is_active ? (
          <Badge variant="success">{t('erpMaster.common.active')}</Badge>
        ) : (
          <Badge variant="destructive">{t('erpMaster.common.inactive')}</Badge>
        )}
      </TableCell>
      <TableCell className="text-right">
        <Can permission={PERMISSIONS.ERP_PARTNER_EDIT}>
          <Button variant="ghost" size="icon" onClick={() => onEdit(partner)} aria-label={t('common.edit')}>
            <Edit className="h-4 w-4" />
          </Button>
        </Can>
        {/* 後端 partner.rs:156 要求 erp.partner.delete。該碼原本是死碼（不在 permissions
            表裡，has_permission 永遠 false、只靠 is_admin() bypass 才過），所以 PR #57
            刻意留白等它補齊；#58 已把碼補進目錄，這裡才補上閘。
            目前沒有任何角色被授予此碼 → 實際只有管理員看得到，與後端行為一致。 */}
        <Can permission={PERMISSIONS.ERP_PARTNER_DELETE}>
          <Button variant="ghost" size="icon" onClick={() => onDelete(partner)} aria-label={t('common.delete')}>
            <Trash2 className="h-4 w-4" />
          </Button>
        </Can>
      </TableCell>
    </TableRow>
  )
}
