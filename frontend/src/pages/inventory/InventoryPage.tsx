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
import { formatNumber, formatCurrency, formatDate, formatUom } from '@/lib/utils'

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
      <div>
        <h1 className="text-3xl font-bold tracking-tight">庫存查詢</h1>
        <p className="text-muted-foreground">查看各倉庫的庫存現況</p>
      </div>

      <div className="flex flex-wrap gap-4 items-center">
        <div className="relative flex-1 min-w-[200px] max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋品項..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <WarehouseShelfTreeSelect
          value={locationFilter}
          onValueChange={(v) => setLocationFilter(v)}
        />
        <div className="relative w-48">
          <Input
            placeholder="搜尋批號..."
            value={batchFilter}
            onChange={(e) => setBatchFilter(e.target.value)}
          />
        </div>
        {hasFilters && (
          <Button variant="ghost" size="sm" onClick={clearFilters}>
            <X className="h-4 w-4 mr-1" />
            清除篩選
          </Button>
        )}
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>倉庫</TableHead>
              {isShelfQuery && <TableHead>貨架</TableHead>}
              <TableHead>品項</TableHead>
              <TableHead className="text-right">現有量</TableHead>
              <TableHead>單位</TableHead>
              <TableHead className="text-right">平均成本</TableHead>
              <TableHead className="text-right">庫存價值</TableHead>
              <TableHead className="text-right">安全庫存</TableHead>
              <TableHead>最後異動時間</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell
                  colSpan={isShelfQuery ? 9 : 8}
                  className="text-center py-8"
                >
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : inventory && inventory.length > 0 ? (
              inventory.map((item) => (
                <TableRow
                  key={`${item.warehouse_id}-${item.storage_location_id ?? 'wh'}-${item.product_id}`}
                >
                  <TableCell>{item.warehouse_name}</TableCell>
                  {isShelfQuery && (
                    <TableCell>
                      {item.storage_location_name ?? item.storage_location_code ?? '-'}
                    </TableCell>
                  )}
                  <TableCell>
                    <div>
                      <div className="font-medium">{item.product_name}</div>
                      <div className="text-xs text-muted-foreground">{item.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(item.qty_on_hand, 0)}
                  </TableCell>
                  <TableCell>{formatUom(item.base_uom)}</TableCell>
                  <TableCell className="text-right">
                    {item.avg_cost ? formatCurrency(item.avg_cost) : '-'}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.avg_cost
                      ? formatCurrency(parseFloat(item.qty_on_hand) * parseFloat(item.avg_cost))
                      : '-'}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.safety_stock ? formatNumber(item.safety_stock, 0) : '-'}
                  </TableCell>
                  <TableCell className="text-sm text-muted-foreground">
                    {item.last_updated_at ? formatDate(item.last_updated_at) : '-'}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={isShelfQuery ? 9 : 8}
                  className="text-center py-8"
                >
                  <Package className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">
                    {hasFilters ? '找不到符合條件的庫存資料' : '尚無庫存資料'}
                  </p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
