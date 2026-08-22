import { Fragment, useState } from 'react'
import { LowStockTotal } from '@/lib/api'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, Package, ChevronRight } from 'lucide-react'
import { formatNumber, formatUom, cn } from '@/lib/utils'

const COL_COUNT = 5

/** 低庫存狀態 Badge：缺貨 / 低於安全 */
function StatusBadge({ status }: { status: string }) {
  if (status === 'out_of_stock') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-destructive/15 text-destructive text-xs font-medium">
        缺貨
      </span>
    )
  }
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-status-warning-bg text-status-warning-text text-xs font-medium">
      低於安全庫存
    </span>
  )
}

/** 展開列：該品項在各倉庫的庫存分布 */
function WarehouseBreakdownRows({ item }: { item: LowStockTotal }) {
  if (item.warehouse_breakdown.length === 0) {
    return (
      <TableRow className="bg-muted/20">
        <TableCell colSpan={COL_COUNT} className="py-3 pl-12 text-sm text-muted-foreground">
          各倉庫皆無此品項庫存
        </TableCell>
      </TableRow>
    )
  }
  return (
    <>
      {item.warehouse_breakdown.map((w) => (
        <TableRow key={w.warehouse_id} className="bg-muted/20 text-sm">
          <TableCell className="pl-12 text-muted-foreground">{w.warehouse_name}</TableCell>
          <TableCell className="text-right font-medium">{formatNumber(w.qty_on_hand, 0)}</TableCell>
          <TableCell>
            <span className="text-xs px-1.5 py-0.5 border rounded-md bg-background">
              {formatUom(item.base_uom)}
            </span>
          </TableCell>
          <TableCell className="hidden md:table-cell" />
          <TableCell />
        </TableRow>
      ))}
    </>
  )
}

/**
 * 低庫存彙總表：一品項一列（全公司總量 vs 安全庫存），可展開看各倉分布。
 * 取代舊版逐倉庫低庫存清單（避免同品項在多倉重複虛報）。
 */
export function LowStockTotalTable({
  data,
  isLoading,
}: {
  data: LowStockTotal[] | undefined
  isLoading: boolean
}) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  const toggle = (id: string) =>
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })

  return (
    <div className="rounded-xl border bg-card shadow-xs overflow-hidden">
      <div className="overflow-x-auto">
        <Table>
          <TableHeader className="bg-muted/30">
            <TableRow>
              <TableHead className="font-semibold">品項</TableHead>
              <TableHead className="text-right font-semibold">全公司現有量</TableHead>
              <TableHead className="font-semibold">單位</TableHead>
              <TableHead className="text-right font-semibold hidden md:table-cell">安全庫存</TableHead>
              <TableHead className="font-semibold">狀態</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={COL_COUNT} className="text-center py-24">
                  <div className="flex flex-col items-center gap-2">
                    <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    <p className="text-sm text-muted-foreground animate-pulse">正在調度庫存數據...</p>
                  </div>
                </TableCell>
              </TableRow>
            ) : data && data.length > 0 ? (
              data.map((item) => {
                const isExpanded = expanded.has(item.product_id)
                return (
                  <Fragment key={item.product_id}>
                    <TableRow
                      className={cn(
                        'group cursor-pointer transition-colors hover:bg-muted/50',
                        isExpanded && 'bg-muted/30',
                      )}
                      onClick={() => toggle(item.product_id)}
                    >
                      <TableCell className="font-medium">
                        <div className="flex items-center gap-1.5">
                          <ChevronRight
                            className={cn(
                              'h-4 w-4 text-muted-foreground/50 transition-transform shrink-0',
                              isExpanded && 'rotate-90',
                            )}
                          />
                          <div className="flex flex-col">
                            <span className="font-semibold text-foreground">{item.product_name}</span>
                            <span className="text-xs text-muted-foreground/70 font-mono italic">
                              {item.product_sku}
                            </span>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell
                        className={cn(
                          'text-right font-bold',
                          item.stock_status === 'out_of_stock' ? 'text-destructive' : 'text-status-warning-text',
                        )}
                      >
                        {formatNumber(item.total_on_hand, 0)}
                      </TableCell>
                      <TableCell>
                        <span className="text-xs px-1.5 py-0.5 border rounded-md bg-background">
                          {formatUom(item.base_uom)}
                        </span>
                      </TableCell>
                      <TableCell className="text-right hidden md:table-cell">
                        {item.safety_stock ? formatNumber(item.safety_stock, 0) : '-'}
                      </TableCell>
                      <TableCell>
                        <StatusBadge status={item.stock_status} />
                      </TableCell>
                    </TableRow>
                    {isExpanded && <WarehouseBreakdownRows item={item} />}
                  </Fragment>
                )
              })
            ) : (
              <TableRow>
                <TableCell colSpan={COL_COUNT} className="text-center py-20">
                  <div className="flex flex-col items-center max-w-[280px] mx-auto">
                    <div className="h-20 w-20 rounded-full bg-muted flex items-center justify-center mb-4">
                      <Package className="h-10 w-10 text-muted-foreground/40" />
                    </div>
                    <h3 className="text-lg font-semibold mb-1">沒有低庫存品項</h3>
                    <p className="text-sm text-muted-foreground text-center">
                      目前所有已設定安全庫存的品項，全公司總量都在安全水位之上。
                    </p>
                  </div>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
