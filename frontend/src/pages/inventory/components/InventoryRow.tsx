import { useQuery } from '@tanstack/react-query'
import api, { InventoryOnHand } from '@/lib/api'
import { TableCell, TableRow } from '@/components/ui/table'
import { Loader2, ChevronRight, AlertTriangle } from 'lucide-react'
import { formatNumber, formatCurrency, formatDate, formatUom, cn } from '@/lib/utils'
import type { UnassignedInventoryItem } from '@/types/erp'

/** 效期日期 Badge：依剩餘天數顯示不同顏色 */
function ExpiryDateBadge({ date }: { date: string }) {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const expiry = new Date(date)
  const diffDays = Math.ceil((expiry.getTime() - today.getTime()) / 86_400_000)

  if (diffDays < 0) {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-destructive/15 text-destructive text-xs font-medium">
        已過期 ({date})
      </span>
    )
  }
  if (diffDays <= 30) {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-status-warning-bg0/15 text-status-warning-text dark:text-orange-400 text-xs font-medium">
        {diffDays}天後到期 ({date})
      </span>
    )
  }
  return <span className="text-sm">{date}</span>
}

/** 全倉庫概覽模式下，展開行顯示批號明細 */
function BatchDetailRows({
  warehouseId,
  productId,
  batchFilter,
  colSpan,
}: {
  warehouseId: string
  productId: string
  batchFilter?: string
  colSpan: number
}) {
  const params: Record<string, string> = { warehouse_id: warehouseId, product_id: productId }
  if (batchFilter) params.batch_no = batchFilter

  const { data, isLoading } = useQuery({
    queryKey: ['inventory', 'batch-detail', warehouseId, productId, batchFilter],
    queryFn: async () => {
      const res = await api.get<InventoryOnHand[]>('/inventory/on-hand', { params })
      return res.data
    },
  })

  const { data: unassigned } = useQuery({
    queryKey: ['inventory', 'unassigned', warehouseId, productId],
    queryFn: async () => {
      const res = await api.get<UnassignedInventoryItem[]>('/inventory/unassigned', {
        params: { warehouse_id: warehouseId, product_id: productId },
      })
      return res.data
    },
  })

  const unassignedQty = unassigned?.[0] ? parseFloat(unassigned[0].qty_unassigned) : 0

  if (isLoading) {
    return (
      <TableRow className="bg-muted/20">
        <TableCell colSpan={colSpan} className="py-3 pl-12">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
            載入批號明細...
          </div>
        </TableCell>
      </TableRow>
    )
  }

  if (!data || data.length === 0) {
    return (
      <TableRow className="bg-muted/20">
        <TableCell colSpan={colSpan} className="py-3 pl-12 text-sm text-muted-foreground">
          無批號明細資料
        </TableCell>
      </TableRow>
    )
  }

  return (
    <>
      {data.map((d, i) => (
        <TableRow key={`${d.storage_location_id ?? 'none'}-${d.batch_no ?? i}`} className="bg-muted/20 text-sm">
          <TableCell className="pl-12 text-muted-foreground">
            {d.storage_location_name ?? d.storage_location_code ?? '—'}
          </TableCell>
          <TableCell>
            <div className="flex items-center gap-2">
              <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-mono font-medium">
                {d.batch_no || '無批號'}
              </span>
              {d.expiry_date && (
                <span className="text-xs text-muted-foreground">
                  效期 {d.expiry_date}
                </span>
              )}
            </div>
          </TableCell>
          <TableCell className="text-right font-medium">
            {formatNumber(d.qty_on_hand, 0)}
          </TableCell>
          <TableCell>
            <span className="text-xs px-1.5 py-0.5 border rounded-md bg-background">
              {formatUom(d.base_uom)}
            </span>
          </TableCell>
          <TableCell className="text-right text-muted-foreground hidden md:table-cell">—</TableCell>
          <TableCell className="text-right text-muted-foreground hidden md:table-cell">—</TableCell>
          <TableCell className="text-right text-muted-foreground hidden md:table-cell">—</TableCell>
          <TableCell className="text-sm text-muted-foreground/60 italic hidden lg:table-cell">
            {d.last_updated_at ? formatDate(d.last_updated_at) : '-'}
          </TableCell>
        </TableRow>
      ))}
      {unassignedQty > 0 && (
        <TableRow className="bg-status-warning-bg/50 dark:bg-amber-950/20 text-sm">
          <TableCell className="pl-12" colSpan={2}>
            <div className="flex items-center gap-1.5 text-status-warning-text dark:text-amber-400 italic">
              <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
              未分配庫存（尚未上架至儲位）
            </div>
          </TableCell>
          <TableCell className="text-right font-medium text-status-warning-text dark:text-amber-400">
            {formatNumber(unassignedQty, 0)}
          </TableCell>
          <TableCell colSpan={colSpan - 3} />
        </TableRow>
      )}
    </>
  )
}

/** 單一庫存列（含可展開批號明細） */
export function InventoryRow({
  item,
  isShelfQuery,
  showBatchColumns,
  isOverviewMode,
  isExpanded,
  onToggleExpand,
  colCount,
  batchFilter,
}: {
  item: InventoryOnHand
  isShelfQuery: boolean
  showBatchColumns: boolean
  isOverviewMode: boolean
  isExpanded: boolean
  onToggleExpand: () => void
  colCount: number
  batchFilter?: string
}) {
  return (
    <>
      <TableRow
        className={cn(
          'group transition-colors',
          isOverviewMode ? 'cursor-pointer hover:bg-muted/50' : 'hover:bg-muted/50',
          isExpanded && 'bg-muted/30',
        )}
        onClick={isOverviewMode ? onToggleExpand : undefined}
      >
        <TableCell className="font-medium">
          <div className="flex items-center gap-1.5">
            {isOverviewMode && (
              <ChevronRight
                className={cn(
                  'h-4 w-4 text-muted-foreground/50 transition-transform shrink-0',
                  isExpanded && 'rotate-90',
                )}
              />
            )}
            {item.warehouse_name}
          </div>
        </TableCell>
        {isShelfQuery && (
          <TableCell>
            <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-secondary text-secondary-foreground text-xs font-medium">
              {item.storage_location_name ?? item.storage_location_code ?? '-'}
            </span>
          </TableCell>
        )}
        <TableCell>
          <div className="flex flex-col">
            <span className="font-semibold text-foreground">{item.product_name}</span>
            <span className="text-xs text-muted-foreground/70 font-mono italic">{item.product_sku}</span>
          </div>
        </TableCell>
        {showBatchColumns && (
          <>
            <TableCell>
              {item.batch_no ? (
                <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-mono font-medium">
                  {item.batch_no}
                </span>
              ) : (
                <span className="text-muted-foreground/50">—</span>
              )}
            </TableCell>
            <TableCell>
              {item.expiry_date ? (
                <ExpiryDateBadge date={item.expiry_date} />
              ) : (
                <span className="text-muted-foreground/50">—</span>
              )}
            </TableCell>
          </>
        )}
        <TableCell className="text-right font-bold text-primary">
          {formatNumber(item.qty_on_hand, 0)}
        </TableCell>
        <TableCell>
          <span className="text-xs px-1.5 py-0.5 border rounded-md bg-background">
            {formatUom(item.base_uom)}
          </span>
        </TableCell>
        <TableCell className="text-right text-muted-foreground hidden md:table-cell">
          {item.avg_cost ? formatCurrency(item.avg_cost) : '-'}
        </TableCell>
        <TableCell className="text-right font-medium hidden md:table-cell">
          {item.avg_cost
            ? formatCurrency(parseFloat(item.qty_on_hand) * parseFloat(item.avg_cost))
            : '-'}
        </TableCell>
        <TableCell className="text-right hidden md:table-cell">
          {item.safety_stock ? (
            <span className={cn(
              "px-2 py-0.5 rounded text-xs",
              parseFloat(item.qty_on_hand) <= parseFloat(item.safety_stock)
                ? "bg-destructive/10 text-destructive font-bold"
                : "text-muted-foreground"
            )}>
              {formatNumber(item.safety_stock, 0)}
            </span>
          ) : '-'}
        </TableCell>
        <TableCell className="text-sm text-muted-foreground/60 italic hidden lg:table-cell">
          {item.last_updated_at ? formatDate(item.last_updated_at) : '-'}
        </TableCell>
      </TableRow>

      {isOverviewMode && isExpanded && (
        <BatchDetailRows
          warehouseId={item.warehouse_id}
          productId={item.product_id}
          batchFilter={batchFilter}
          colSpan={colCount}
        />
      )}
    </>
  )
}
