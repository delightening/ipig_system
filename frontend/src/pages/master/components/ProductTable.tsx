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
  Trash2,
  X,
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
  const isEmpty = !sortedData || sortedData.length === 0

  return (
    <div className="@container">
      {/* ≥ 600px：表格視圖；欄位依容器寬度漸進顯露 */}
      <div className="hidden @[600px]:block rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead className="w-10">
                <input
                  type="checkbox"
                  checked={products.length > 0 && selectionSize === products.length}
                  onChange={onSelectAll}
                  className="h-4 w-4 rounded border-input"
                  aria-label="全選產品"
                />
              </TableHead>
              <SortableTableHead sortKey="sku" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                SKU
              </SortableTableHead>
              <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                名稱
              </SortableTableHead>
              <TableHead className="hidden @[900px]:table-cell">規格</TableHead>
              <SortableTableHead className="hidden @[750px]:table-cell" sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                單位
              </SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell text-right" sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                安全庫存
              </SortableTableHead>
              <SortableTableHead className="hidden @[1050px]:table-cell text-center" sortKey="track_batch" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                批號
              </SortableTableHead>
              <SortableTableHead className="hidden @[1050px]:table-cell text-center" sortKey="track_expiry" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                效期
              </SortableTableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <LoadingRow />
            ) : isEmpty ? (
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

      {/* < 600px：卡片視圖 */}
      <div className="@[600px]:hidden">
        <ProductCardList
          products={products}
          sortedData={sortedData ?? []}
          isLoading={isLoading}
          isEmpty={isEmpty}
          hasFilters={hasFilters}
          selectionHas={selectionHas}
          selectionSize={selectionSize}
          onSelectAll={onSelectAll}
          onSelect={onSelect}
          onCopySku={handleCopySku}
          onStatusChange={onStatusChange}
          onHardDelete={onHardDelete}
          isAdmin={isAdmin}
          navigate={navigate}
        />
      </div>
    </div>
  )
}

function ProductCardList({
  products,
  sortedData,
  isLoading,
  isEmpty,
  hasFilters,
  selectionHas,
  selectionSize,
  onSelectAll,
  onSelect,
  onCopySku,
  onStatusChange,
  onHardDelete,
  isAdmin,
  navigate,
}: {
  products: ExtendedProduct[]
  sortedData: ExtendedProduct[]
  isLoading: boolean
  isEmpty: boolean
  hasFilters: boolean
  selectionHas: (id: string) => boolean
  selectionSize: number
  onSelectAll: () => void
  onSelect: (id: string) => void
  onCopySku: (sku: string) => void
  onStatusChange: (product: ExtendedProduct, action: StatusAction) => void
  onHardDelete: (product: ExtendedProduct) => void
  isAdmin: boolean
  navigate: ReturnType<typeof useNavigate>
}) {
  if (isLoading) return <LoadingCard />
  if (isEmpty) return <EmptyCard hasFilters={hasFilters} />
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 px-1 text-sm text-muted-foreground">
        <input
          type="checkbox"
          checked={products.length > 0 && selectionSize === products.length}
          onChange={onSelectAll}
          className="h-4 w-4 rounded border-input"
          aria-label="全選產品"
        />
        <span>
          {selectionSize > 0 ? `已選 ${selectionSize} / ${products.length}` : `全選（${products.length} 筆）`}
        </span>
      </div>
      {sortedData.map((product) => (
        <ProductCard
          key={product.id}
          product={product}
          isSelected={selectionHas(product.id)}
          onSelect={onSelect}
          onCopySku={onCopySku}
          onStatusChange={onStatusChange}
          onHardDelete={onHardDelete}
          isAdmin={isAdmin}
          navigate={navigate}
        />
      ))}
    </div>
  )
}

function LoadingRow() {
  return (
    <TableRow>
      <TableCell colSpan={9} className="text-center py-12">
        <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
        <p className="mt-2 text-sm text-muted-foreground">載入中...</p>
      </TableCell>
    </TableRow>
  )
}

function EmptyRow({ hasFilters }: { hasFilters: boolean }) {
  const navigate = useNavigate()
  return (
    <TableRow>
      <TableCell colSpan={9} className="text-center py-12">
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
  const status = product.status || (product.is_active ? 'active' : 'inactive')
  const statusRowClass =
    status === 'discontinued' ? 'bg-destructive/5 text-muted-foreground'
    : status === 'inactive' ? 'bg-muted/40'
    : ''
  const statusTitle =
    status === 'discontinued' ? '此產品已停產'
    : status === 'inactive' ? '此產品已停用'
    : undefined

  return (
    <TableRow
      className={cn('group', isSelected ? 'bg-primary/5' : statusRowClass)}
      title={statusTitle}
      aria-label={statusTitle}
    >
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
      <TableCell className="align-middle">
        <button
          className={cn(
            'font-medium text-left hover:text-primary hover:underline transition-colors w-full line-clamp-2 leading-tight break-words',
            status === 'discontinued' && 'line-through decoration-destructive/40'
          )}
          title={product.name}
          onClick={() => navigate(`/products/${product.id}`)}
        >
          {product.name}
        </button>
      </TableCell>
      <TableCell className="hidden @[900px]:table-cell text-muted-foreground align-middle">
        <span className="line-clamp-2 leading-tight break-words">{product.spec || '-'}</span>
      </TableCell>
      <TableCell className="hidden @[750px]:table-cell">
        <span className="text-xs px-1.5 py-0.5 bg-muted rounded">
          {UOM_MAP[product.base_uom] || product.base_uom}
        </span>
      </TableCell>
      <TableCell className="hidden @[900px]:table-cell text-right tabular-nums">
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
      <TableCell className="hidden @[1050px]:table-cell text-center">
        <BoolIcon value={!!product.track_batch} label="批號" />
      </TableCell>
      <TableCell className="hidden @[1050px]:table-cell text-center">
        <BoolIcon value={!!product.track_expiry} label="效期" />
      </TableCell>
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

function SkuCell({ sku, onCopy }: { sku: string; onCopy: (sku: string) => void }) {
  return (
    <code
      className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded cursor-pointer hover:bg-muted-foreground/20 transition-colors line-clamp-3 leading-tight break-words"
      title={`點擊複製 ${sku}`}
      onClick={() => onCopy(sku)}
    >
      {sku}
    </code>
  )
}

function BoolIcon({ value, label }: { value: boolean; label: string }) {
  return value ? (
    <span
      className="mx-auto inline-block h-2.5 w-2.5 rounded-full bg-status-success-text"
      aria-label={`${label}：啟用`}
      title={`${label}：啟用`}
    />
  ) : (
    <X className="mx-auto h-4 w-4 text-destructive" aria-label={`${label}：未啟用`} />
  )
}

function LoadingCard() {
  return (
    <div className="rounded-lg border bg-card py-12 text-center">
      <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
      <p className="mt-2 text-sm text-muted-foreground">載入中...</p>
    </div>
  )
}

function EmptyCard({ hasFilters }: { hasFilters: boolean }) {
  const navigate = useNavigate()
  return (
    <div className="rounded-lg border bg-card py-12 text-center">
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
    </div>
  )
}

function ProductCard({
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
  const uom = UOM_MAP[product.base_uom] || product.base_uom
  return (
    <div className={cn('rounded-lg border bg-card p-3 space-y-2', isSelected && 'bg-primary/5 border-primary/30')}>
      <div className="flex items-start gap-2">
        <input
          type="checkbox"
          checked={isSelected}
          onChange={() => onSelect(product.id)}
          className="mt-1 h-4 w-4 rounded border-input flex-shrink-0"
          aria-label={`選擇產品 ${product.sku}`}
        />
        <div className="flex-1 min-w-0 space-y-1">
          <div className="flex items-center justify-between gap-2">
            <SkuCell sku={product.sku} onCopy={onCopySku} />
            {getStatusBadge(product)}
          </div>
          <button
            className="font-medium text-left hover:text-primary hover:underline transition-colors block w-full break-words"
            title={product.name}
            onClick={() => navigate(`/products/${product.id}`)}
          >
            {product.name}
          </button>
          {product.spec && (
            <div className="text-xs text-muted-foreground break-words">規格：{product.spec}</div>
          )}
        </div>
      </div>

      <div className="flex flex-wrap gap-x-3 gap-y-1 pl-6 text-xs text-muted-foreground">
        <span>單位：<span className="text-foreground">{uom}</span></span>
        {product.safety_stock ? (
          <span>
            安全庫存：
            <span className="text-foreground tabular-nums">{formatNumber(product.safety_stock, 0)} {uom}</span>
          </span>
        ) : null}
        {product.track_batch && <Badge variant="secondary" className="text-[10px] px-1.5">批號</Badge>}
        {product.track_expiry && <Badge variant="secondary" className="text-[10px] px-1.5">效期</Badge>}
      </div>

      <div className="flex items-center justify-end gap-0.5 pt-1 border-t">
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
            <Power className="h-4 w-4 text-status-success-text" />
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
    </div>
  )
}

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
    <div className="flex flex-col items-end gap-0.5" aria-label={`產品 ${product.sku} 操作`}>
      <div className="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" className="h-8 w-8" title="檢視" aria-label="檢視" onClick={() => navigate(`/products/${product.id}`)}>
          <Eye className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" className="h-8 w-8" title="編輯" aria-label="編輯" onClick={() => navigate(`/products/${product.id}/edit`)}>
          <Pencil className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" className="h-8 w-8" title="複製" aria-label="複製" onClick={() => navigate(`/products/new?copy=${product.id}`)}>
          <Copy className="h-4 w-4" />
        </Button>
      </div>
      <div className="flex items-center gap-0.5">
        {product.is_active ? (
          <Button variant="ghost" size="icon" className="h-8 w-8" title="停用" aria-label="停用" onClick={() => onStatusChange(product, 'deactivate')}>
            <PowerOff className="h-4 w-4 text-destructive" />
          </Button>
        ) : (
          <Button variant="ghost" size="icon" className="h-8 w-8" title="啟用" aria-label="啟用" onClick={() => onStatusChange(product, 'activate')}>
            <Power className="h-4 w-4 text-status-success-text" />
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
    </div>
  )
}
