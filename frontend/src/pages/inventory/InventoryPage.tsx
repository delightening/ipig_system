import { useState, useEffect } from 'react'
import { useSearchParams } from 'react-router-dom'
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
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { WarehouseShelfTreeSelect } from '@/components/inventory/WarehouseShelfTreeSelect'
import { PageHeader } from '@/components/ui/page-header'
import { Search, Loader2, Package, X, AlertTriangle } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'
import { InventoryRow } from './components/InventoryRow'

export function InventoryPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [search, setSearch] = useState('')
  const [locationFilter, setLocationFilter] = useState<string>('all')
  const [batchFilter, setBatchFilter] = useState('')
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set())
  const [expiryFilter, setExpiryFilter] = useState(
    () => searchParams.get('filter') === 'expiry_warning',
  )

  // R15-3: filter 變更時清除展開狀態
  useEffect(() => {
    setExpandedRows(new Set())
  }, [locationFilter, search, batchFilter, expiryFilter])

  const { data: inventory, isLoading } = useQuery({
    queryKey: ['inventory', locationFilter, search, batchFilter, expiryFilter],
    queryFn: async () => {
      let params = ''
      if (expiryFilter) params += 'expiry_within_days=60&'
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

  const { sortedData, sort, toggleSort } = useTableSort(inventory)

  const clearFilters = () => {
    setSearch('')
    setLocationFilter('all')
    setBatchFilter('')
    setExpiryFilter(false)
    setExpandedRows(new Set())
    setSearchParams({}, { replace: true })
  }

  const hasFilters =
    search || (locationFilter && locationFilter !== 'all') || batchFilter || expiryFilter
  const isShelfQuery = locationFilter.startsWith('loc:')
  const isWarehouseQuery = locationFilter.startsWith('wh:')
  const showBatchColumns = isWarehouseQuery || isShelfQuery || expiryFilter
  const isOverviewMode = !isWarehouseQuery && !isShelfQuery && !expiryFilter

  const toggleExpand = (rowKey: string) => {
    setExpandedRows((prev) => {
      const next = new Set(prev)
      if (next.has(rowKey)) {
        next.delete(rowKey)
      } else {
        next.add(rowKey)
      }
      return next
    })
  }

  const colCount = 8 + (isShelfQuery ? 1 : 0) + (showBatchColumns ? 2 : 0)

  return (
    <div className="space-y-6">
      <PageHeader
        title="庫存查詢"
        description="查看各倉庫與貨架的即時庫存現況"
        actions={hasFilters ? (
          <Button variant="outline" size="sm" onClick={clearFilters} className="h-9 rounded-full px-4 border-dashed hover:border-destructive hover:text-destructive transition-colors">
            <X className="h-4 w-4 mr-2" />
            清除所有篩選
          </Button>
        ) : undefined}
      />

      {expiryFilter && (
        <div className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-destructive/10 border border-destructive/20">
          <AlertTriangle className="h-4 w-4 text-destructive shrink-0" />
          <span className="text-sm font-medium text-destructive">效期預警篩選中 — 顯示 60 天內到期的品項</span>
          <button
            onClick={() => { setExpiryFilter(false); setSearchParams({}, { replace: true }) }}
            className="ml-auto p-0.5 rounded hover:bg-destructive/20 text-destructive transition-colors"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

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
                <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">倉庫</SortableTableHead>
                {isShelfQuery && <SortableTableHead sortKey="storage_location_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">貨架</SortableTableHead>}
                <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">品項</SortableTableHead>
                {showBatchColumns && (
                  <>
                    <SortableTableHead sortKey="batch_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">批號</SortableTableHead>
                    <SortableTableHead sortKey="expiry_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">效期</SortableTableHead>
                  </>
                )}
                <SortableTableHead sortKey="qty_on_hand" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold">現有量</SortableTableHead>
                <SortableTableHead sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">單位</SortableTableHead>
                <SortableTableHead sortKey="avg_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold hidden md:table-cell">平均成本</SortableTableHead>
                <TableHead className="text-right font-semibold hidden md:table-cell">庫存價值</TableHead>
                <SortableTableHead sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold hidden md:table-cell">安全庫存</SortableTableHead>
                <SortableTableHead sortKey="last_updated_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold hidden lg:table-cell">最後異動時間</SortableTableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={colCount} className="text-center py-24">
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-8 w-8 animate-spin text-primary" />
                      <p className="text-sm text-muted-foreground animate-pulse">正在調度庫存數據...</p>
                    </div>
                  </TableCell>
                </TableRow>
              ) : sortedData && sortedData.length > 0 ? (
                sortedData.map((item) => {
                  const rowKey = `${item.warehouse_id}-${item.product_id}`
                  const isExpanded = expandedRows.has(rowKey)

                  return (
                    <InventoryRow
                      key={`${item.warehouse_id}-${item.storage_location_id ?? 'wh'}-${item.product_id}-${item.batch_no ?? 'all'}`}
                      item={item}
                      isShelfQuery={isShelfQuery}
                      showBatchColumns={showBatchColumns}
                      isOverviewMode={isOverviewMode}
                      isExpanded={isExpanded}
                      onToggleExpand={() => toggleExpand(rowKey)}
                      colCount={colCount}
                      batchFilter={batchFilter}
                    />
                  )
                })
              ) : (
                <TableRow>
                  <TableCell colSpan={colCount} className="text-center py-20">
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
