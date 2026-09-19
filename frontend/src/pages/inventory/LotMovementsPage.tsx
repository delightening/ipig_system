import { useQuery } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router-dom'
import { Trans, useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import api, { LotMovementsResponse, LotReconciliationStatus } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { PageHeader } from '@/components/ui/page-header'
import { AlertTriangle, CheckCircle2, FileText } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { formatDateTime, formatNumber } from '@/lib/utils'

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

/** 對帳卡片外框：僅「品項總量也對不上」才用 destructive，批號歸屬差異用 warning */
function reconciliationCardClass(status: LotReconciliationStatus): string | undefined {
  if (status === 'unbalanced') return 'border-destructive'
  if (status === 'attribution_only') return 'border-status-warning-border'
  return undefined
}

/** 批號完整生命週期查詢頁（R84-6）：時間軸 + 數量對帳，跨倉彙總。見 ERP流程.md §6.2.2 */
export function LotMovementsPage() {
  const { t } = useTranslation()
  const [searchParams] = useSearchParams()
  const productId = searchParams.get('product_id') ?? ''
  const batchNo = searchParams.get('batch_no') ?? ''
  const expiryDate = searchParams.get('expiry_date') ?? undefined
  const productSku = searchParams.get('sku') ?? undefined

  const { data, isLoading } = useQuery({
    queryKey: ['lot-movements', productId, batchNo, expiryDate],
    queryFn: async () => {
      const response = await api.get<LotMovementsResponse>('/inventory/lot-movements', {
        params: { product_id: productId, batch_no: batchNo, expiry_date: expiryDate },
      })
      return response.data
    },
    enabled: !!productId && !!batchNo,
  })

  const { sortedData: sortedMovements, sort, toggleSort } = useTableSort(data?.movements)
  const r = data?.reconciliation

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
        title={t('erpDocs.inventory.lot.title')}
        description={`${productSku ? productSku + ' — ' : ''}${
          expiryDate
            ? t('erpDocs.inventory.lot.descriptionWithExpiry', { batchNo, expiry: expiryDate })
            : t('erpDocs.inventory.lot.description', { batchNo })
        }`}
      />

      {r && (
        <Card className={reconciliationCardClass(r.status)}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              {r.status === 'balanced' ? (
                <CheckCircle2 className="h-4 w-4 text-status-success-text" />
              ) : (
                <AlertTriangle
                  className={
                    r.status === 'attribution_only'
                      ? 'h-4 w-4 text-status-warning-text'
                      : 'h-4 w-4 text-destructive'
                  }
                />
              )}
              {t('erpDocs.inventory.lot.reconciliation')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6 text-sm">
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.received')}</div>
                <div className="font-medium">{formatNumber(r.received, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.customerReturned')}</div>
                <div className="font-medium">{formatNumber(r.customer_returned, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.internalConsumed')}</div>
                <div className="font-medium">{formatNumber(r.internal_consumed, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.returnedToSupplier')}</div>
                <div className="font-medium">{formatNumber(r.returned_to_supplier, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.adjustedNet')}</div>
                <div className="font-medium">{formatNumber(r.adjusted_net, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">{t('erpDocs.inventory.lot.remaining')}</div>
                <div className="font-bold text-primary">{formatNumber(r.remaining, 0)}</div>
              </div>
            </div>
            {r.status === 'attribution_only' && (
              <p className="mt-3 text-sm text-status-warning-text">
                <Trans
                  i18nKey={
                    Number(r.unattributed_adjust_net) !== 0
                      ? 'erpDocs.inventory.lot.attributionOnlyWithNet'
                      : 'erpDocs.inventory.lot.attributionOnly'
                  }
                  values={{
                    derived: formatNumber(r.derived_remaining, 0),
                    actual: formatNumber(r.remaining, 0),
                    total: formatNumber(r.product_remaining_total, 0),
                    net: formatNumber(r.unattributed_adjust_net, 0),
                  }}
                  components={{ bold: <span className="font-medium" /> }}
                />
              </p>
            )}
            {r.status === 'unbalanced' && (
              <p className="mt-3 text-sm text-destructive">
                {t('erpDocs.inventory.lot.unbalanced', {
                  derived: formatNumber(r.derived_remaining, 0),
                  actual: formatNumber(r.remaining, 0),
                  productDerived: formatNumber(r.product_derived_total, 0),
                  productActual: formatNumber(r.product_remaining_total, 0),
                })}
              </p>
            )}
          </CardContent>
        </Card>
      )}

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="trx_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.time')}</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.warehouse')}</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.doc')}</SortableTableHead>
              <SortableTableHead sortKey="direction" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.directionLabel')}</SortableTableHead>
              <SortableTableHead sortKey="qty_base" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('erpDocs.shared.quantity')}</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="p-0">
                  <TableSkeleton rows={6} cols={5} />
                </TableCell>
              </TableRow>
            ) : sortedMovements && sortedMovements.length > 0 ? (
              sortedMovements.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="text-sm">{formatDateTime(item.trx_date)}</TableCell>
                  <TableCell>{item.warehouse_name}</TableCell>
                  <TableCell className="font-mono text-sm">
                    <Link to={`/documents/${item.doc_id}`} className="text-primary hover:underline">
                      {item.doc_no}
                    </Link>
                  </TableCell>
                  <TableCell>{getDirectionBadge(item.direction)}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(item.qty_base, 0)}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={5} icon={FileText} title={t('erpDocs.inventory.lot.empty')} />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
