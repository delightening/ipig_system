import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { InventoryOnHand } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { WarehouseShelfTreeSelect } from '@/components/inventory/WarehouseShelfTreeSelect'
import { Search, Loader2, Package, X } from 'lucide-react'
import { formatNumber, formatCurrency, formatDate, formatUom, cn } from '@/lib/utils'

export function InventoryPage() {
  const [search, setSearch] = useState('')
  const [locationFilter, setLocationFilter] = useState<string>('all')
  const [batchFilter, setBatchFilter] = useState('')

  const { data: inventory, isLoading } = useQuery({
    queryKey: ['inventory', locationFilter, search, batchFilter],
    queryFn: async () => {
      let params = ''
      if (locationFilter && locationFilter !== 'all') {
        if (locationFilter.startsWith('wh:')) {
          params += `warehouse_id=${encodeURIComponent(locationFilter.slice(3))}&`
        } else if (locationFilter.startsWith('loc:')) {
          params += `storage_location_id=${encodeURIComponent(locationFilter.slice(4))}&`
        }
      }
      if (search) params += `keyword=${encodeURIComponent(search)}&`
      if (batchFilter) params += `batch_no=${encodeURIComponent(batchFilter)}&`
      const response = await api.get<InventoryOnHand[]>(`/inventory/on-hand?${params}`)
      return response.data
    },
  })

  const clearFilters = () => {
    setSearch('')
    setLocationFilter('all')
    setBatchFilter('')
  }

  const hasFilters =
    search || (locationFilter && locationFilter !== 'all') || batchFilter
  const isShelfQuery = locationFilter.startsWith('loc:')

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center justify-between pb-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
            庫存查詢
          </h1>
          <p className="text-muted-foreground mt-1">查看各倉庫與貨架的即時庫存現況</p>
        </div>
        {hasFilters && (
          <Button variant="outline" size="sm" onClick={clearFilters} className="h-9 rounded-full px-4 border-dashed hover:border-destructive hover:text-destructive transition-colors">
            <X className="h-4 w-4 mr-2" />
            清除所有篩選
          </Button>
        )}
      </div>

      <div className="grid gap-4 md:flex md:items-center bg-card p-4 rounded-xl border shadow-sm items-stretch">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60" />
          <Input
            placeholder="搜尋品項名稱、SKU..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9 h-10 border-muted-foreground/20 focus-visible:ring-primary/30"
          />
        </div>
        <WarehouseShelfTreeSelect
          value={locationFilter}
          onValueChange={(v) => setLocationFilter(v)}
          className="h-10"
        />
        <div className="relative w-full md:w-56">
          <Input
            placeholder="搜尋批號..."
            value={batchFilter}
            onChange={(e) => setBatchFilter(e.target.value)}
            className="h-10 border-muted-foreground/20 focus-visible:ring-primary/30"
          />
        </div>
      </div>

      <div className="rounded-xl border bg-card shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <Table>
            <TableHeader className="bg-muted/30">
              <TableRow>
                <TableHead className="font-semibold">倉庫</TableHead>
                {isShelfQuery && <TableHead className="font-semibold">貨架</TableHead>}
                <TableHead className="font-semibold">品項</TableHead>
                <TableHead className="text-right font-semibold">現有量</TableHead>
                <TableHead className="font-semibold">單位</TableHead>
                <TableHead className="text-right font-semibold">平均成本</TableHead>
                <TableHead className="text-right font-semibold">庫存價值</TableHead>
                <TableHead className="text-right font-semibold">安全庫存</TableHead>
                <TableHead className="font-semibold">最後異動時間</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell
                    colSpan={isShelfQuery ? 9 : 8}
                    className="text-center py-24"
                  >
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-8 w-8 animate-spin text-primary" />
                      <p className="text-sm text-muted-foreground animate-pulse">正在調度庫存數據...</p>
                    </div>
                  </TableCell>
                </TableRow>
              ) : inventory && inventory.length > 0 ? (
                inventory.map((item) => (
                  <TableRow
                    key={`${item.warehouse_id}-${item.storage_location_id ?? 'wh'}-${item.product_id}`}
                    className="group hover:bg-muted/50 transition-colors"
                  >
                    <TableCell className="font-medium">{item.warehouse_name}</TableCell>
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
                    <TableCell className="text-right font-bold text-primary">
                      {formatNumber(item.qty_on_hand, 0)}
                    </TableCell>
                    <TableCell>
                      <span className="text-xs px-1.5 py-0.5 border rounded-md bg-background">
                        {formatUom(item.base_uom)}
                      </span>
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {item.avg_cost ? formatCurrency(item.avg_cost) : '-'}
                    </TableCell>
                    <TableCell className="text-right font-medium">
                      {item.avg_cost
                        ? formatCurrency(parseFloat(item.qty_on_hand) * parseFloat(item.avg_cost))
                        : '-'}
                    </TableCell>
                    <TableCell className="text-right">
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
                    <TableCell className="text-sm text-muted-foreground/60 italic">
                      {item.last_updated_at ? formatDate(item.last_updated_at) : '-'}
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={isShelfQuery ? 9 : 8}
                    className="text-center py-20"
                  >
                    <div className="flex flex-col items-center max-w-[280px] mx-auto">
                      <div className="h-20 w-20 rounded-full bg-muted flex items-center justify-center mb-4">
                        <Package className="h-10 w-10 text-muted-foreground/40" />
                      </div>
                      <h3 className="text-lg font-semibold mb-1">
                        {hasFilters ? '找不到符合條件的庫存' : '尚無庫存資料'}
                      </h3>
                      <p className="text-sm text-muted-foreground text-center">
                        {hasFilters
                          ? '請嘗試調整搜尋關鍵字或篩選條件。'
                          : '目前系統中沒有任何庫存記錄，若已入庫請檢查進貨單狀態。'}
                      </p>
                      {hasFilters && (
                        <Button variant="link" onClick={clearFilters} className="mt-2 text-primary">
                          重置所有篩選
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>
      </div>
    </div>
  )
}
