import { useLayoutEffect, useRef, useState } from 'react'
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

/** 欄寬計算 — 依優先順序分配，雙向觸發 300 名稱臨界值 */
type WidthMap = {
  select: number
  sku: number
  name: number
  spec: number
  base_uom: number
  safety_stock: number
  track_batch: number
  track_expiry: number
  actions: number
}

const NAME_SWITCH_THRESHOLD = 300
export const MIN_TABLE_WIDTH = 720

function computeWidths(W: number): WidthMap {
  // 9 欄全顯示
  // 最大非 name 總和 = 40 + 120 + 100*4 + 120 + 200 = 880
  // 名稱=300 時 W = 880 + 300 = 1180（策略 A/B 切換點）
  const sumNonNameMax = 880
  if (W >= sumNonNameMax + NAME_SWITCH_THRESHOLD) {
    // 策略 A：所有非 name 吃到 max，name 吸收剩下
    return {
      select: 40, sku: 120, name: Math.max(NAME_SWITCH_THRESHOLD, W - sumNonNameMax),
      spec: 100, base_uom: 100, safety_stock: 120,
      track_batch: 100, track_expiry: 100, actions: 200,
    }
  }
  // 策略 B：name 最多 300；依序 actions → group → SKU → safety → name 分配 excess
  // 最小總和 = 40 + 80 + 140 + 65*4 + 80 + 120 = 720
  const w: WidthMap = {
    select: 40, sku: 80, name: 140, spec: 65, base_uom: 65,
    safety_stock: 80, track_batch: 65, track_expiry: 65, actions: 120,
  }
  let excess = W - MIN_TABLE_WIDTH
  if (excess <= 0) return w
  // 1. actions 120 → 200 (+80)
  const toActions = Math.min(excess, 200 - w.actions)
  w.actions += toActions; excess -= toActions
  if (excess <= 0) return w
  // 2. group（4 欄）同步 +Δ，每欄 65 → 100 (+35)
  const toGroup = Math.min(excess, 4 * 35)
  const per = toGroup / 4
  w.spec += per; w.base_uom += per; w.track_batch += per; w.track_expiry += per
  excess -= toGroup
  if (excess <= 0) return w
  // 3. SKU 二選一（80 or 120），有 40 的空間才 snap
  if (excess >= 40) { w.sku = 120; excess -= 40 }
  if (excess <= 0) return w
  // 4. safety 二選一
  if (excess >= 40) { w.safety_stock = 120; excess -= 40 }
  if (excess <= 0) return w
  // 5. 剩下給 name（一般到這裡 excess 幾乎為 0）
  w.name += excess
  return w
}

const DEFAULT_WIDTHS: WidthMap = {
  select: 40, sku: 80, name: 240, spec: 65, base_uom: 65,
  safety_stock: 80, track_batch: 65, track_expiry: 65, actions: 120,
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

  const containerRef = useRef<HTMLDivElement>(null)
  const [containerWidth, setContainerWidth] = useState<number | null>(null)

  useLayoutEffect(() => {
    const el = containerRef.current
    if (!el) return
    const update = () => {
      const W = el.clientWidth
      if (W === 0) return
      setContainerWidth(W)
    }
    update()
    const ro = new ResizeObserver(update)
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  // 不可裁剪保險：容器寬度不足以容納表格最小總和時，改 render 卡片
  const canRenderTable = containerWidth === null || containerWidth >= MIN_TABLE_WIDTH
  const widths = containerWidth !== null && canRenderTable
    ? computeWidths(containerWidth)
    : DEFAULT_WIDTHS

  const handleCopySku = async (sku: string) => {
    await navigator.clipboard.writeText(sku)
    toast({ title: '已複製', description: `SKU: ${sku}` })
  }

  const hasFilters = !!listState.filters.search || listState.activeFilterCount > 0

  const isEmpty = !sortedData || sortedData.length === 0

  const cardListProps = {
    products,
    sortedData: sortedData ?? [],
    isLoading,
    isEmpty,
    hasFilters,
    selectionHas,
    selectionSize,
    onSelectAll,
    onSelect,
    onCopySku: handleCopySku,
    onStatusChange,
    onHardDelete,
    isAdmin,
    navigate,
  }

  return (
    <>
      {/* Desktop / Tablet 容器：依實際寬度切換表格 / 卡片（避免 overflow 裁切） */}
      <div
        ref={containerRef}
        className={cn(
          "hidden md:block",
          canRenderTable && "rounded-lg border bg-card overflow-hidden [&>div]:overflow-x-hidden",
        )}
      >
        {canRenderTable ? (
          <Table className="table-fixed">
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead style={{ width: widths.select }}>
                  <input
                    type="checkbox"
                    checked={products.length > 0 && selectionSize === products.length}
                    onChange={onSelectAll}
                    className="h-4 w-4 rounded border-input"
                    aria-label="全選產品"
                  />
                </TableHead>
                <SortableTableHead sortKey="sku" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ width: widths.sku }}>
                  SKU
                </SortableTableHead>
                <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ width: widths.name }}>
                  名稱
                </SortableTableHead>
                <TableHead style={{ width: widths.spec }}>
                  <span className="line-clamp-2 leading-tight">規格</span>
                </TableHead>
                <SortableTableHead sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ width: widths.base_uom }}>
                  單位
                </SortableTableHead>
                <SortableTableHead sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right" style={{ width: widths.safety_stock }}>
                  安全庫存
                </SortableTableHead>
                <SortableTableHead sortKey="track_batch" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-center" style={{ width: widths.track_batch }}>
                  批號
                </SortableTableHead>
                <SortableTableHead sortKey="track_expiry" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-center" style={{ width: widths.track_expiry }}>
                  效期
                </SortableTableHead>
                <TableHead className="text-right" style={{ width: widths.actions }}>
                  <span className="line-clamp-2 leading-tight">操作</span>
                </TableHead>
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
        ) : (
          <ProductCardList {...cardListProps} />
        )}
      </div>

      {/* Mobile：卡片（< md） */}
      <div className="md:hidden">
        <ProductCardList {...cardListProps} />
      </div>
    </>
  )
}

/** 共用卡片列表（mobile 與 desktop 容器過窄時 fallback） */
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

/** 載入中列 */
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

/** 空資料列 */
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
      className={cn("group", isSelected ? "bg-primary/5" : statusRowClass)}
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
      <TableCell className="max-w-0 align-middle">
        <button
          className={cn(
            "font-medium text-left hover:text-primary hover:underline transition-colors w-full line-clamp-2 leading-tight break-words",
            status === 'discontinued' && "line-through decoration-destructive/40"
          )}
          title={product.name}
          onClick={() => navigate(`/products/${product.id}`)}
        >
          {product.name}
        </button>
      </TableCell>
      <TableCell className="text-muted-foreground align-middle">
        <span className="line-clamp-2 leading-tight break-words">{product.spec || '-'}</span>
      </TableCell>
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
        <BoolIcon value={!!product.track_batch} label="批號" />
      </TableCell>
      <TableCell className="text-center">
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

/** SKU 欄位（點擊即複製） */
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

/** Bool 圖示：true = 綠圓、false = 紅叉 */
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

/** 載入中卡片（手機） */
function LoadingCard() {
  return (
    <div className="rounded-lg border bg-card py-12 text-center">
      <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
      <p className="mt-2 text-sm text-muted-foreground">載入中...</p>
    </div>
  )
}

/** 空資料卡片（手機） */
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

/** 單筆產品卡片（手機） */
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
    <div className={cn("rounded-lg border bg-card p-3 space-y-2", isSelected && "bg-primary/5 border-primary/30")}>
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
            className="font-medium text-left hover:text-primary hover:underline transition-colors block w-full truncate"
            title={product.name}
            onClick={() => navigate(`/products/${product.id}`)}
          >
            {product.name}
          </button>
          {product.spec && (
            <div className="text-xs text-muted-foreground truncate">規格：{product.spec}</div>
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
