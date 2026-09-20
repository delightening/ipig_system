import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
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
import { formatNumber, cn, formatUom } from '@/lib/utils'
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

function getStatusBadge(product: ExtendedProduct, t: TFunction) {
  const status = product.status || (product.is_active ? 'active' : 'inactive')
  switch (status) {
    case 'active':
      return <Badge variant="success">{t('erpMaster.common.active')}</Badge>
    case 'inactive':
      return <Badge variant="warning">{t('erpMaster.common.inactive')}</Badge>
    case 'discontinued':
      return <Badge variant="destructive">{t('erpMaster.products.status.discontinued')}</Badge>
    default:
      return <Badge variant="secondary">{t('erpMaster.common.unknown')}</Badge>
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
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { sortedData, sort, toggleSort } = useTableSort(products)

  const handleCopySku = async (sku: string) => {
    await navigator.clipboard.writeText(sku)
    toast({ title: t('erpMaster.common.copied'), description: `SKU: ${sku}` })
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
                  aria-label={t('erpMaster.products.table.selectAll')}
                />
              </TableHead>
              <SortableTableHead sortKey="sku" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                SKU
              </SortableTableHead>
              <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                {t('erpMaster.common.name')}
              </SortableTableHead>
              <TableHead className="hidden @[900px]:table-cell">{t('erpMaster.common.spec')}</TableHead>
              <SortableTableHead className="hidden @[750px]:table-cell" sortKey="base_uom" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                {t('erpMaster.common.unit')}
              </SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell text-right" sortKey="safety_stock" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                {t('erpMaster.products.safetyStock')}
              </SortableTableHead>
              <SortableTableHead className="hidden @[1050px]:table-cell text-center" sortKey="track_batch" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                {t('erpMaster.products.table.batchNo')}
              </SortableTableHead>
              <SortableTableHead className="hidden @[1050px]:table-cell text-center" sortKey="track_expiry" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                {t('erpMaster.products.table.expiry')}
              </SortableTableHead>
              <TableHead className="text-right">{t('common.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={9} className="p-0">
                  <TableSkeleton rows={8} cols={9} />
                </TableCell>
              </TableRow>
            ) : isEmpty ? (
              <TableEmptyRow
                colSpan={9}
                icon={Package}
                title={hasFilters ? t('erpMaster.products.table.noMatch') : t('erpMaster.products.table.empty')}
                action={
                  hasFilters
                    ? undefined
                    : { label: t('erpMaster.products.table.createFirst'), onClick: () => navigate('/products/new'), icon: Plus }
                }
              />
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
  const { t } = useTranslation()

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
          aria-label={t('erpMaster.products.table.selectAll')}
        />
        <span>
          {selectionSize > 0
            ? t('erpMaster.products.table.selectedOf', { selected: selectionSize, total: products.length })
            : t('erpMaster.products.table.selectAllCount', { count: products.length })}
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
  const { t } = useTranslation()
  const status = product.status || (product.is_active ? 'active' : 'inactive')
  const statusRowClass =
    status === 'discontinued' ? 'bg-destructive/5 text-muted-foreground'
    : status === 'inactive' ? 'bg-muted/40'
    : ''
  const statusTitle =
    status === 'discontinued' ? t('erpMaster.products.table.discontinuedHint')
    : status === 'inactive' ? t('erpMaster.products.table.inactiveHint')
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
          aria-label={t('erpMaster.products.table.selectProduct', { sku: product.sku })}
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
          {formatUom(product.base_uom)}
        </span>
      </TableCell>
      <TableCell className="hidden @[900px]:table-cell text-right tabular-nums">
        {product.safety_stock ? (
          <span>
            {formatNumber(product.safety_stock, 0)}
            <span className="text-muted-foreground text-xs ml-1">
              {formatUom(product.base_uom)}
            </span>
          </span>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </TableCell>
      <TableCell className="hidden @[1050px]:table-cell text-center">
        <BoolIcon value={!!product.track_batch} label={t('erpMaster.products.table.batchNo')} />
      </TableCell>
      <TableCell className="hidden @[1050px]:table-cell text-center">
        <BoolIcon value={!!product.track_expiry} label={t('erpMaster.products.table.expiry')} />
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
  const { t } = useTranslation()

  return (
    <code
      className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded cursor-pointer hover:bg-muted-foreground/20 transition-colors line-clamp-3 leading-tight break-words"
      title={t('erpMaster.products.table.clickToCopy', { sku })}
      onClick={() => onCopy(sku)}
    >
      {sku}
    </code>
  )
}

function BoolIcon({ value, label }: { value: boolean; label: string }) {
  const { t } = useTranslation()

  return value ? (
    <span
      className="mx-auto inline-block h-2.5 w-2.5 rounded-full bg-status-success-text"
      aria-label={t('erpMaster.products.table.boolOn', { label })}
      title={t('erpMaster.products.table.boolOn', { label })}
    />
  ) : (
    <X className="mx-auto h-4 w-4 text-destructive" aria-label={t('erpMaster.products.table.boolOff', { label })} />
  )
}

function LoadingCard() {
  const { t } = useTranslation()

  return (
    <div className="rounded-lg border bg-card py-12 text-center">
      <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
      <p className="mt-2 text-sm text-muted-foreground">{t('common.loading')}</p>
    </div>
  )
}

function EmptyCard({ hasFilters }: { hasFilters: boolean }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  return (
    <div className="rounded-lg border bg-card py-12 text-center">
      <Package className="h-12 w-12 mx-auto mb-3 text-muted-foreground/50" />
      <p className="text-muted-foreground">
        {hasFilters ? t('erpMaster.products.table.noMatch') : t('erpMaster.products.table.empty')}
      </p>
      {!hasFilters && (
        <Button variant="outline" className="mt-4" onClick={() => navigate('/products/new')}>
          <Plus className="mr-2 h-4 w-4" />
          {t('erpMaster.products.table.createFirst')}
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
  const { t } = useTranslation()
  const uom = formatUom(product.base_uom)
  return (
    <div className={cn('rounded-lg border bg-card p-3 space-y-2', isSelected && 'bg-primary/5 border-primary/30')}>
      <div className="flex items-start gap-2">
        <input
          type="checkbox"
          checked={isSelected}
          onChange={() => onSelect(product.id)}
          className="mt-1 h-4 w-4 rounded border-input shrink-0"
          aria-label={t('erpMaster.products.table.selectProduct', { sku: product.sku })}
        />
        <div className="flex-1 min-w-0 space-y-1">
          <div className="flex items-center justify-between gap-2">
            <SkuCell sku={product.sku} onCopy={onCopySku} />
            {getStatusBadge(product, t)}
          </div>
          <button
            className="font-medium text-left hover:text-primary hover:underline transition-colors block w-full break-words"
            title={product.name}
            onClick={() => navigate(`/products/${product.id}`)}
          >
            {product.name}
          </button>
          {product.spec && (
            <div className="text-xs text-muted-foreground break-words">{t('erpMaster.products.table.specLabel', { spec: product.spec })}</div>
          )}
        </div>
      </div>

      <div className="flex flex-wrap gap-x-3 gap-y-1 pl-6 text-xs text-muted-foreground">
        <span>{t('erpMaster.products.table.unitLabel')}<span className="text-foreground">{uom}</span></span>
        {product.safety_stock ? (
          <span>
            {t('erpMaster.products.table.safetyStockLabel')}
            <span className="text-foreground tabular-nums">{formatNumber(product.safety_stock, 0)} {uom}</span>
          </span>
        ) : null}
        {product.track_batch && <Badge variant="secondary" className="text-[10px] px-1.5">{t('erpMaster.products.table.batchNo')}</Badge>}
        {product.track_expiry && <Badge variant="secondary" className="text-[10px] px-1.5">{t('erpMaster.products.table.expiry')}</Badge>}
      </div>

      <div className="flex items-center justify-end gap-0.5 pt-1 border-t">
        <Button variant="ghost" size="icon" className="h-8 w-8" title={t('common.view')} aria-label={t('common.view')} onClick={() => navigate(`/products/${product.id}`)}>
          <Eye className="h-4 w-4" />
        </Button>
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('common.edit')} aria-label={t('common.edit')} onClick={() => navigate(`/products/${product.id}/edit`)}>
            <Pencil className="h-4 w-4" />
          </Button>
        </Can>
        <Can permission={PERMISSIONS.ERP_PRODUCT_CREATE}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.common.copy')} aria-label={t('erpMaster.common.copy')} onClick={() => navigate(`/products/new?copy=${product.id}`)}>
            <Copy className="h-4 w-4" />
          </Button>
        </Can>
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          {product.is_active ? (
            <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.actions.deactivate')} aria-label={t('erpMaster.products.actions.deactivate')} onClick={() => onStatusChange(product, 'deactivate')}>
              <PowerOff className="h-4 w-4 text-destructive" />
            </Button>
          ) : (
            <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.actions.activate')} aria-label={t('erpMaster.products.actions.activate')} onClick={() => onStatusChange(product, 'activate')}>
              <Power className="h-4 w-4 text-status-success-text" />
            </Button>
          )}
        </Can>
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.statusDialog.discontinueTitle')} aria-label={t('erpMaster.products.statusDialog.discontinueTitle')} onClick={() => onStatusChange(product, 'discontinue')}>
            <Ban className="h-4 w-4 text-muted-foreground" />
          </Button>
        </Can>
        {isAdmin && (
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-destructive hover:text-destructive"
            title={t('erpMaster.products.actions.hardDelete')}
            aria-label={t('erpMaster.products.actions.hardDelete')}
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
  const { t } = useTranslation()

  return (
    <div className="flex flex-col items-end gap-0.5" aria-label={t('erpMaster.products.table.rowActions', { sku: product.sku })}>
      <div className="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" className="h-8 w-8" title={t('common.view')} aria-label={t('common.view')} onClick={() => navigate(`/products/${product.id}`)}>
          <Eye className="h-4 w-4" />
        </Button>
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('common.edit')} aria-label={t('common.edit')} onClick={() => navigate(`/products/${product.id}/edit`)}>
            <Pencil className="h-4 w-4" />
          </Button>
        </Can>
        <Can permission={PERMISSIONS.ERP_PRODUCT_CREATE}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.common.copy')} aria-label={t('erpMaster.common.copy')} onClick={() => navigate(`/products/new?copy=${product.id}`)}>
            <Copy className="h-4 w-4" />
          </Button>
        </Can>
      </div>
      <div className="flex items-center gap-0.5">
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          {product.is_active ? (
            <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.actions.deactivate')} aria-label={t('erpMaster.products.actions.deactivate')} onClick={() => onStatusChange(product, 'deactivate')}>
              <PowerOff className="h-4 w-4 text-destructive" />
            </Button>
          ) : (
            <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.actions.activate')} aria-label={t('erpMaster.products.actions.activate')} onClick={() => onStatusChange(product, 'activate')}>
              <Power className="h-4 w-4 text-status-success-text" />
            </Button>
          )}
        </Can>
        <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
          <Button variant="ghost" size="icon" className="h-8 w-8" title={t('erpMaster.products.statusDialog.discontinueTitle')} aria-label={t('erpMaster.products.statusDialog.discontinueTitle')} onClick={() => onStatusChange(product, 'discontinue')}>
            <Ban className="h-4 w-4 text-muted-foreground" />
          </Button>
        </Can>
        {isAdmin && (
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-destructive hover:text-destructive"
            title={t('erpMaster.products.actions.hardDelete')}
            aria-label={t('erpMaster.products.actions.hardDelete')}
            onClick={() => onHardDelete(product)}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  )
}
