import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import api, { StockLedgerDetail } from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { PageHeader } from '@/components/ui/page-header'
import { FileText, Download, ExternalLink } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { formatDateTime, formatNumber, formatCurrency } from '@/lib/utils'

/** 異動方向顯示名稱；未知方向原樣顯示。 */
function directionLabel(t: TFunction, direction: string): string {
  switch (direction) {
    case 'in':
      return t('erpDocs.inventory.direction.in')
    case 'out':
      return t('erpDocs.inventory.direction.out')
    case 'transfer_in':
      return t('erpDocs.inventory.direction.transferIn')
    case 'transfer_out':
      return t('erpDocs.inventory.direction.transferOut')
    case 'adjust_in':
      return t('erpDocs.inventory.direction.adjustIn')
    case 'adjust_out':
      return t('erpDocs.inventory.direction.adjustOut')
    default:
      return direction
  }
}

export function StockLedgerPage() {
  const { t } = useTranslation()
  const { data: ledger, isLoading } = useQuery({
    queryKey: ['stock-ledger'],
    queryFn: async () => {
      const response = await api.get<StockLedgerDetail[]>('/inventory/ledger')
      return response.data
    },
  })

  const { sortedData: sortedLedger, sort, toggleSort } = useTableSort(ledger)

  const exportToCSV = () => {
    if (!ledger) return

    const headers = [
      t('erpDocs.shared.time'),
      t('erpDocs.shared.warehouse'),
      t('erpDocs.inventory.ledger.itemCode'),
      t('erpDocs.shared.itemName'),
      t('erpDocs.inventory.ledger.docNo'),
      t('erpDocs.shared.directionLabel'),
      t('erpDocs.shared.quantity'),
      t('erpDocs.inventory.ledger.unitCost'),
      t('erpDocs.shared.batchNo'),
    ]
    const rows = ledger.map(item => [
      item.trx_date,
      item.warehouse_name,
      item.product_sku,
      item.product_name,
      item.doc_no,
      directionLabel(t, item.direction),
      item.qty_base,
      item.unit_cost || '',
      item.batch_no || '',
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `stock_ledger_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
  }

  const getDirectionBadge = (direction: string) => {
    const isInbound = ['in', 'transfer_in', 'adjust_in'].includes(direction)
    return (
      <Badge variant={isInbound ? 'success' : 'destructive'}>
        {directionLabel(t, direction)}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('nav.erpInventoryLedger')}
        description={t('erpDocs.inventory.ledger.description')}
        actions={
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" asChild>
              <Link to="/stock-ledger">
                <ExternalLink className="mr-2 h-4 w-4" />
                {t('erpDocs.inventory.ledger.reportCenter')}
              </Link>
            </Button>
            <Button size="sm" onClick={exportToCSV} disabled={!ledger?.length}>
              <Download className="mr-2 h-4 w-4" />
              {t('erpDocs.inventory.ledger.exportCsv')}
            </Button>
          </div>
        }
      />

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="trx_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.time')}</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.warehouse')}</SortableTableHead>
              <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.item')}</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.doc')}</SortableTableHead>
              <SortableTableHead sortKey="direction" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.directionLabel')}</SortableTableHead>
              <SortableTableHead sortKey="qty_base" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('erpDocs.shared.quantity')}</SortableTableHead>
              <SortableTableHead sortKey="unit_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('erpDocs.inventory.ledger.unitCost')}</SortableTableHead>
              <SortableTableHead sortKey="batch_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.batchNo')}</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="p-0">
                  <TableSkeleton rows={8} cols={8} />
                </TableCell>
              </TableRow>
            ) : sortedLedger && sortedLedger.length > 0 ? (
              sortedLedger.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="text-sm">{formatDateTime(item.trx_date)}</TableCell>
                  <TableCell>{item.warehouse_name}</TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{item.product_name}</div>
                      <div className="text-xs text-muted-foreground">{item.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    <Link to={`/documents/${item.doc_id}`} className="text-primary hover:underline">
                      {item.doc_no}
                    </Link>
                  </TableCell>
                  <TableCell>{getDirectionBadge(item.direction)}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(item.qty_base, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.unit_cost ? formatCurrency(item.unit_cost) : '-'}
                  </TableCell>
                  <TableCell>{item.batch_no || '-'}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={8} icon={FileText} title={t('erpDocs.inventory.ledger.empty')} />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
