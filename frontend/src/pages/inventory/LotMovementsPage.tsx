import { useQuery } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router-dom'
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

const directionNames: Record<string, string> = {
  in: '入庫',
  out: '出庫',
  transfer_in: '調入',
  transfer_out: '調出',
  adjust_in: '調增',
  adjust_out: '調減',
}

/** 對帳卡片外框：僅「品項總量也對不上」才用 destructive，批號歸屬差異用 warning */
function reconciliationCardClass(status: LotReconciliationStatus): string | undefined {
  if (status === 'unbalanced') return 'border-destructive'
  if (status === 'attribution_only') return 'border-status-warning-border'
  return undefined
}

/** 批號完整生命週期查詢頁（R84-6）：時間軸 + 數量對帳，跨倉彙總。見 ERP流程.md §6.2.2 */
export function LotMovementsPage() {
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
        {directionNames[direction] || direction}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="批號生命週期追溯"
        description={`${productSku ? productSku + ' — ' : ''}批號 ${batchNo}${expiryDate ? `（效期 ${expiryDate}）` : ''}`}
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
              數量對帳
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6 text-sm">
              <div>
                <div className="text-muted-foreground">收到</div>
                <div className="font-medium">{formatNumber(r.received, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">客戶退貨收到</div>
                <div className="font-medium">{formatNumber(r.customer_returned, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">內部領用</div>
                <div className="font-medium">{formatNumber(r.internal_consumed, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">退回供應商</div>
                <div className="font-medium">{formatNumber(r.returned_to_supplier, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">調整增減（淨額）</div>
                <div className="font-medium">{formatNumber(r.adjusted_net, 0)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">目前剩餘</div>
                <div className="font-bold text-primary">{formatNumber(r.remaining, 0)}</div>
              </div>
            </div>
            {r.status === 'attribution_only' && (
              <p className="mt-3 text-sm text-status-warning-text">
                批號歸屬待確認：本批號依分類加總反推為 {formatNumber(r.derived_remaining, 0)}，
                儲位實際庫存 {formatNumber(r.remaining, 0)}。
                但本品項全部批號合計帳實相符（{formatNumber(r.product_remaining_total, 0)}），
                <span className="font-medium">總量無虞</span>，差異僅是批號之間的歸屬。
                多半源自 2026-06-10 前的歷史補帳——當時的沖銷未帶批號
                {Number(r.unattributed_adjust_net) !== 0 && (
                  <>（本品項未歸批調整淨額 {formatNumber(r.unattributed_adjust_net, 0)}）</>
                )}
                ，待全倉盤點重立基準後歸零。
              </p>
            )}
            {r.status === 'unbalanced' && (
              <p className="mt-3 text-sm text-destructive">
                ⚠️ 帳實不符：依分類加總反推的剩餘量為 {formatNumber(r.derived_remaining, 0)}，
                與儲位實際庫存 {formatNumber(r.remaining, 0)} 不一致；本品項全部批號合計也對不上
                （帳上 {formatNumber(r.product_derived_total, 0)} vs 實際 {formatNumber(r.product_remaining_total, 0)}），
                需人工查證。
              </p>
            )}
          </CardContent>
        </Card>
      )}

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="trx_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>倉庫</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單據</SortableTableHead>
              <SortableTableHead sortKey="direction" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>方向</SortableTableHead>
              <SortableTableHead sortKey="qty_base" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">數量</SortableTableHead>
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
              <TableEmptyRow colSpan={5} icon={FileText} title="尚無此批號的異動紀錄" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
