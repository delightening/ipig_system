import { useNavigate } from 'react-router-dom'
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
import { toast } from '@/components/ui/use-toast'
import {
  Plus,
  Loader2,
  Package,
  Eye,
  Pencil,
  Copy,
  Power,
  PowerOff,
  Ban,
  ClipboardCopy,
  Trash2,
} from 'lucide-react'
import { formatNumber, cn, UOM_MAP } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'

import type { ExtendedProduct, StatusAction, ProductListState } from './productTypes'

interface ProductTableProps {
  products: ExtendedProduct[]
  isLoading: boolean
  listState: ProductListState
  selectionHas: (id: string) => boolean
  selectionSize: number
  onSelectAll: () => void
  onSelect: (id: string) => void
  onStatusChange: (product: ExtendedProduct, action: StatusAction) => void
  onHardDelete: (product: ExtendedProduct) => void
  isAdmin: boolean
}

/** 取得狀態 Badge */
function getStatusBadge(product: ExtendedProduct) {
  const status = product.status || (product.is_active ? 'active' : 'inactive')
  switch (status) {
    case 'active':
      return <Badge variant="success">啟用</Badge>
    case 'inactive':
      return <Badge variant="warning">停用</Badge>
    case 'discontinued':
      return <Badge variant="destructive">停產</Badge>
    default:
      return <Badge variant="secondary">未知</Badge>
  }
}

export function ProductTable({
  products,
  isLoading,
  listState,
  selectionHas,
  selectionSize,
  onSelectAll,
  onSelect,
  onStatusChange,
  onHardDelete,
  isAdmin,
}: ProductTableProps) {
  const navigate = useNavigate()
  const { sortedData, sort, toggleSort } = useTableSort(products)

  const handleCopySku = async (sku: string) => {
    await navigator.clipboard.writeText(sku)
    toast({ title: '已複製', description: `SKU: ${sku}` })
  }

  const hasFilters = !!listState.filters.search || listState.activeFilterCount > 0

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            <TableHead className="w-[40px]">
              <input
                type="checkbox"
                checked={products.length > 0 && selectionSize === products.length}
                onChange={onSelectAll}
                className="h-4 w-4 rounded border-input"
                aria-label="全選產品"
              />
            </TableHead>
            <SortableTableHead sortKey="sku" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[180px]">
              SKU
            </SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
              名稱
            </SortableTableHead>
            <TableHead className="w-[150px]">規格</TableHead>
            <SortableTableHead sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[60px]">
              單位
            </SortableTableHead>
            <SortableTableHead sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[100px] text-right">
              安全庫存
            </SortableTableHead>
            <SortableTableHead sortKey="track_batch" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[60px] text-center">
              批號
            </SortableTableHead>
            <SortableTableHead sortKey="track_expiry" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[60px] text-center">
              效期
            </SortableTableHead>
            <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[80px]">
              狀態
            </SortableTableHead>
            <TableHead className="w-[200px] text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <LoadingRow />
          ) : !sortedData || sortedData.length === 0 ? (
            <EmptyRow hasFilters={hasFilters} />
          ) : (
            sortedData.map((product) => (
              <ProductRow
                key={product.id}
                product={product}
                isSelected={selectionHas(product.id)}
                onSelect={onSelect}
                onCopySku={handleCopySku}
                onStatusChange={onStatusChange}
                onHardDelete={onHardDelete}
                isAdmin={isAdmin}
                navigate={navigate}
              />
            ))
          )}
        </TableBody>
      </Table>
    </div>
  )
}

/** 載入中列 */
function LoadingRow() {
  return (
    <TableRow>
      <TableCell colSpan={10} className="text-center py-12">
        <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
        <p className="mt-2 text-sm text-muted-foreground">載入中...</p>
      </TableCell>
    </TableRow>
  )
}

/** 空資料列 */
function EmptyRow({ hasFilters }: { hasFilters: boolean }) {
  const navigate = useNavigate()
  return (
    <TableRow>
      <TableCell colSpan={10} className="text-center py-12">
        <Package className="h-12 w-12 mx-auto mb-3 text-muted-foreground/50" />
        <p className="text-muted-foreground">
          {hasFilters ? '找不到符合條件的產品' : '尚無產品資料'}
        </p>
        {!hasFilters && (
          <Button variant="outline" className="mt-4" onClick={() => navigate('/products/new')}>
            <Plus className="mr-2 h-4 w-4" />
            建立第一個產品
          </Button>
        )}
      </TableCell>
    </TableRow>
  )
}

/** 單筆產品列 */
function ProductRow({
  product,
  isSelected,
  onSelect,
  onCopySku,
  onStatusChange,
  onHardDelete,
  isAdmin,
  navigate,
}: {
  product: ExtendedProduct
  isSelected: boolean
  onSelect: (id: string) => void
  onCopySku: (sku: string) => void
  onStatusChange: (product: ExtendedProduct, action: StatusAction) => void
  onHardDelete: (product: ExtendedProduct) => void
  isAdmin: boolean
  navigate: ReturnType<typeof useNavigate>
}) {
  return (
    <TableRow className={cn("group", isSelected && "bg-primary/5")}>
      <TableCell>
        <input
          type="checkbox"
          checked={isSelected}
          onChange={() => onSelect(product.id)}
          className="h-4 w-4 rounded border-input"
          aria-label={`選擇產品 ${product.sku}`}
        />
      </TableCell>
      <TableCell>
        <SkuCell sku={product.sku} onCopy={onCopySku} />
      </TableCell>
      <TableCell>
        <button
          className="font-medium text-left hover:text-primary hover:underline transition-colors"
          onClick={() => navigate(`/products/${product.id}`)}
        >
          {product.name}
        </button>
      </TableCell>
      <TableCell className="text-muted-foreground">{product.spec || '-'}</TableCell>
      <TableCell>
        <span className="text-xs px-1.5 py-0.5 bg-muted rounded">
          {UOM_MAP[product.base_uom] || product.base_uom}
        </span>
      </TableCell>
      <TableCell className="text-right tabular-nums">
        {product.safety_stock ? (
          <span>
            {formatNumber(product.safety_stock, 0)}
            <span className="text-muted-foreground text-xs ml-1">
              {UOM_MAP[product.base_uom] || product.base_uom}
            </span>
          </span>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </TableCell>
      <TableCell className="text-center">
        {product.track_batch
          ? <Badge variant="secondary" className="text-[10px] px-1.5">啟用</Badge>
          : <span className="text-muted-foreground">-</span>}
      </TableCell>
      <TableCell className="text-center">
        {product.track_expiry
          ? <Badge variant="secondary" className="text-[10px] px-1.5">啟用</Badge>
          : <span className="text-muted-foreground">-</span>}
      </TableCell>
      <TableCell>{getStatusBadge(product)}</TableCell>
      <TableCell className="text-right">
        <ProductActions
          product={product}
          onStatusChange={onStatusChange}
          onHardDelete={onHardDelete}
          isAdmin={isAdmin}
          navigate={navigate}
        />
      </TableCell>
    </TableRow>
  )
}

/** SKU 欄位（含複製） */
function SkuCell({ sku, onCopy }: { sku: string; onCopy: (sku: string) => void }) {
  return (
    <div className="flex items-center gap-1.5 group/sku">
      <code
        className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded cursor-pointer hover:bg-muted-foreground/20 transition-colors max-w-[160px] truncate"
        title={sku}
        onClick={() => onCopy(sku)}
      >
        {sku}
      </code>
      <button
        onClick={() => onCopy(sku)}
        className="opacity-0 group-hover/sku:opacity-100 transition-opacity"
        title="複製 SKU"
        aria-label="複製 SKU"
      >
        <ClipboardCopy className="h-3 w-3 text-muted-foreground hover:text-foreground" />
      </button>
    </div>
  )
}

/** 操作按鈕群組 */
function ProductActions({
  product,
  onStatusChange,
  onHardDelete,
  isAdmin,
  navigate,
}: {
  product: ExtendedProduct
  onStatusChange: (product: ExtendedProduct, action: StatusAction) => void
  onHardDelete: (product: ExtendedProduct) => void
  isAdmin: boolean
  navigate: ReturnType<typeof useNavigate>
}) {
  return (
    <div className="flex items-center justify-end gap-0.5" aria-label={`產品 ${product.sku} 操作`}>
      <Button variant="ghost" size="icon" className="h-8 w-8" title="檢視" aria-label="檢視" onClick={() => navigate(`/products/${product.id}`)}>
        <Eye className="h-4 w-4" />
      </Button>
      <Button variant="ghost" size="icon" className="h-8 w-8" title="編輯" aria-label="編輯" onClick={() => navigate(`/products/${product.id}/edit`)}>
        <Pencil className="h-4 w-4" />
      </Button>
      <Button variant="ghost" size="icon" className="h-8 w-8" title="複製" aria-label="複製" onClick={() => navigate(`/products/new?copy=${product.id}`)}>
        <Copy className="h-4 w-4" />
      </Button>
      {product.is_active ? (
        <Button variant="ghost" size="icon" className="h-8 w-8" title="停用" aria-label="停用" onClick={() => onStatusChange(product, 'deactivate')}>
          <PowerOff className="h-4 w-4 text-destructive" />
        </Button>
      ) : (
        <Button variant="ghost" size="icon" className="h-8 w-8" title="啟用" aria-label="啟用" onClick={() => onStatusChange(product, 'activate')}>
          <Power className="h-4 w-4 text-green-600" />
        </Button>
      )}
      <Button variant="ghost" size="icon" className="h-8 w-8" title="標記停產" aria-label="標記停產" onClick={() => onStatusChange(product, 'discontinue')}>
        <Ban className="h-4 w-4 text-muted-foreground" />
      </Button>
      {isAdmin && (
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-destructive hover:text-destructive"
          title="硬刪除（僅管理員）"
          aria-label="硬刪除（僅管理員）"
          onClick={() => onHardDelete(product)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      )}
    </div>
  )
}
