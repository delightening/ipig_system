import { useState, useEffect, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { InventoryOnHand, LowStockTotal } from '@/lib/api'
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
import { LowStockTotalTable } from './components/LowStockTotalTable'

export function InventoryPage() {
  const { t } = useTranslation()
  const [searchParams, setSearchParams] = useSearchParams()
  const [search, setSearch] = useState('')
  const [locationFilter, setLocationFilter] = useState<string>('all')
  const [batchFilter, setBatchFilter] = useState('')
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set())
  const [expiryFilter, setExpiryFilter] = useState(
    () => searchParams.get('filter') === 'expiry_warning',
  )
  const [lowStockFilter, setLowStockFilter] = useState(
    () => searchParams.get('filter') === 'low_stock',
  )

  // R15-3: filter 變更時清除展開狀態
  useEffect(() => {
    setExpandedRows(new Set())
  }, [locationFilter, search, batchFilter, expiryFilter, lowStockFilter])

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
    enabled: !lowStockFilter,
  })

  // 低庫存改全公司總量（一品項一筆 + 各倉分布），與逐倉庫概覽分流
  const { data: lowStockData, isLoading: lowStockLoading } = useQuery({
    queryKey: ['inventory', 'low-stock-totals'],
    queryFn: async () => (await api.get<LowStockTotal[]>('/inventory/low-stock')).data,
    enabled: lowStockFilter,
  })

  const filteredLowStock = useMemo(() => {
    if (!lowStockData) return undefined
    const kw = search.trim().toLowerCase()
    if (!kw) return lowStockData
    return lowStockData.filter(
      (d) =>
        d.product_name.toLowerCase().includes(kw) ||
        d.product_sku.toLowerCase().includes(kw),
    )
  }, [lowStockData, search])

  const { sortedData, sort, toggleSort } = useTableSort(inventory)

  const clearFilters = () => {
    setSearch('')
    setLocationFilter('all')
    setBatchFilter('')
    setExpiryFilter(false)
    setLowStockFilter(false)
    setExpandedRows(new Set())
    setSearchParams({}, { replace: true })
  }

  const hasFilters =
    search || (locationFilter && locationFilter !== 'all') || batchFilter || expiryFilter || lowStockFilter
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
        title={t('nav.erpInventory')}
        description={t('erpDocs.inventory.page.description')}
        actions={hasFilters ? (
          <Button variant="outline" size="sm" onClick={clearFilters} className="h-9 rounded-full px-4 border-dashed hover:border-destructive hover:text-destructive transition-colors">
            <X className="h-4 w-4 mr-2" />
            {t('erpDocs.inventory.page.clearAllFilters')}
          </Button>
        ) : undefined}
      />

      {expiryFilter && (
        <div className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-destructive/10 border border-destructive/20">
          <AlertTriangle className="h-4 w-4 text-destructive shrink-0" />
          <span className="text-sm font-medium text-destructive">{t('erpDocs.inventory.page.expiryBanner')}</span>
          <button
            onClick={() => { setExpiryFilter(false); setSearchParams({}, { replace: true }) }}
            className="ml-auto p-0.5 rounded hover:bg-destructive/20 text-destructive transition-colors"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

      {lowStockFilter && (
        <div className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-status-warning-bg border border-status-warning-border">
          <AlertTriangle className="h-4 w-4 text-status-warning-text shrink-0" />
          <span className="text-sm font-medium text-status-warning-text">{t('erpDocs.inventory.page.lowStockBanner')}</span>
          <button
            onClick={() => { setLowStockFilter(false); setSearchParams({}, { replace: true }) }}
            className="ml-auto p-0.5 rounded hover:bg-status-warning-text/20 text-status-warning-text transition-colors"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

      <div className="grid gap-4 md:flex md:items-center bg-card p-4 rounded-xl border shadow-xs items-stretch">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60" />
          <Input
            placeholder={t('erpDocs.inventory.page.searchPlaceholder')}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9 h-10 border-muted-foreground/20 focus-visible:ring-primary/30"
          />
        </div>
        {!lowStockFilter && (
          <>
            <WarehouseShelfTreeSelect
              value={locationFilter}
              onValueChange={(v) => setLocationFilter(v)}
              className="h-10"
            />
            <div className="relative w-full md:w-56">
              <Input
                placeholder={t('erpDocs.inventory.page.batchSearchPlaceholder')}
                value={batchFilter}
                onChange={(e) => setBatchFilter(e.target.value)}
                className="h-10 border-muted-foreground/20 focus-visible:ring-primary/30"
              />
            </div>
          </>
        )}
      </div>

      {lowStockFilter ? (
        <LowStockTotalTable data={filteredLowStock} isLoading={lowStockLoading} />
      ) : (
      <div className="rounded-xl border bg-card shadow-xs overflow-hidden">
        <div className="overflow-x-auto">
          <Table>
            <TableHeader className="bg-muted/30">
              <TableRow>
                <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.shared.warehouse')}</SortableTableHead>
                {isShelfQuery && <SortableTableHead sortKey="storage_location_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.warehouse.locationType.shelf')}</SortableTableHead>}
                <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.shared.item')}</SortableTableHead>
                {showBatchColumns && (
                  <>
                    <SortableTableHead sortKey="batch_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.shared.batchNo')}</SortableTableHead>
                    <SortableTableHead sortKey="expiry_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.shared.expiryDate')}</SortableTableHead>
                  </>
                )}
                <SortableTableHead sortKey="qty_on_hand" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold">{t('erpDocs.inventory.page.onHand')}</SortableTableHead>
                <SortableTableHead sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold">{t('erpDocs.shared.unit')}</SortableTableHead>
                <SortableTableHead sortKey="avg_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold hidden md:table-cell">{t('erpDocs.inventory.page.avgCost')}</SortableTableHead>
                <TableHead className="text-right font-semibold hidden md:table-cell">{t('erpDocs.shared.inventoryValue')}</TableHead>
                <SortableTableHead sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right font-semibold hidden md:table-cell">{t('erpDocs.inventory.safetyStock')}</SortableTableHead>
                <SortableTableHead sortKey="last_updated_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="font-semibold hidden lg:table-cell">{t('erpDocs.inventory.page.lastUpdated')}</SortableTableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={colCount} className="text-center py-24">
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-8 w-8 animate-spin text-primary" />
                      <p className="text-sm text-muted-foreground animate-pulse">{t('erpDocs.inventory.loadingData')}</p>
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
                        {hasFilters ? t('erpDocs.inventory.page.notFound') : t('erpDocs.inventory.page.noData')}
                      </h3>
                      <p className="text-sm text-muted-foreground text-center">
                        {hasFilters
                          ? t('erpDocs.inventory.page.hintFiltered')
                          : t('erpDocs.inventory.page.hintEmpty')}
                      </p>
                      {hasFilters && (
                        <Button variant="link" onClick={clearFilters} className="mt-2 text-primary">
                          {t('erpDocs.inventory.page.resetFilters')}
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
      )}
    </div>
  )
}
