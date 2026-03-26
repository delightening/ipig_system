/**
 * 品項搜尋 Dialog — 支援全品項、庫存模式、PO 待入庫三種模式
 */
import { useQuery } from '@tanstack/react-query'
import api, { Product, DocType, PoReceiptStatus, PoReceiptItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Search } from 'lucide-react'
import { formatNumber, formatUom } from '@/lib/utils'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import type { InventoryOnHand } from '@/types'

/** 庫存查詢結果（/inventory/on-hand 回傳可能包含 category_code） */
interface StockBalanceItem extends InventoryOnHand {
  category_code?: string
}

/** 品項選擇時附帶的額外資料 */
export interface ProductSelectExtraData {
  batch_no?: string
  expiry_date?: string
  storage_location_id?: string
  unit_price?: number
  remaining_qty?: number
  sourceIacuc?: string
}

interface ProductSearchDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  docType: DocType
  /** 目標倉庫（庫存模式用） */
  targetWarehouseId?: string
  /** 是否為 PO 關聯的 GRN */
  isPoLinkedGrn: boolean
  /** PO 入庫狀態資料 */
  poReceiptStatus?: PoReceiptStatus
  /** 搜尋文字 */
  productSearch: string
  setProductSearch: (v: string) => void
  /** 品類篩選 */
  categoryCode?: string
  setCategoryCode: (code: string | undefined) => void
  /** 全品項清單（非庫存模式用） */
  products: Product[] | undefined
  /** 選擇品項回呼 */
  onSelect: (product: Product, extraData?: ProductSelectExtraData) => void
  /** 調整單模式：add=新增品項, modify=修改現有庫存 */
  adjMode?: 'add' | 'modify'
}

export function ProductSearchDialog({
  open,
  onOpenChange,
  docType,
  targetWarehouseId,
  isPoLinkedGrn,
  poReceiptStatus,
  productSearch,
  setProductSearch,
  categoryCode,
  setCategoryCode,
  products,
  onSelect,
  adjMode,
}: ProductSearchDialogProps) {
  const { categories } = useSkuCategories({ enabled: open })
  // ADJ「新增」模式使用產品目錄（手動輸入批號效期），「修改」模式使用庫存清單
  const isStockBasedDoc = docType === 'ADJ'
    ? adjMode === 'modify'
    : ['SO', 'DO', 'PR', 'TR', 'STK'].includes(docType)

  const { data: stockBalances, isLoading: isStockLoading } = useQuery({
    queryKey: ['stock-balances', targetWarehouseId, productSearch],
    queryFn: async () => {
      if (!targetWarehouseId) return []
      const params = new URLSearchParams()
      params.append('warehouse_id', targetWarehouseId)
      if (productSearch) params.append('keyword', productSearch)
      const res = await api.get<StockBalanceItem[]>(`/inventory/on-hand?${params.toString()}`)
      return res.data
    },
    enabled: open && !!targetWarehouseId,
    staleTime: 30000,
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>{isPoLinkedGrn ? '選擇待入庫品項' : '選擇品項'}</DialogTitle>
          <DialogDescription>
            {isPoLinkedGrn ? `採購單 ${poReceiptStatus?.po_no} 的待入庫明細` : '搜尋並選擇要新增的品項'}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜尋品項..."
              value={productSearch}
              onChange={(e) => setProductSearch(e.target.value)}
              className="pl-9"
              autoFocus
            />
          </div>

          {categories && categories.length > 0 && (
            <Tabs
              value={categoryCode || 'all'}
              onValueChange={(v) => setCategoryCode(v === 'all' ? undefined : v)}
            >
              <TabsList className="grid w-full grid-cols-4 md:grid-cols-7 h-auto p-1 flex-wrap">
                <TabsTrigger value="all" className="text-xs py-1">全部</TabsTrigger>
                {categories.map((cat) => (
                  <TabsTrigger key={cat.code} value={cat.code} className="text-xs py-1">
                    {cat.name}
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
          )}

          <div className="max-h-[400px] overflow-y-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  {isPoLinkedGrn ? (
                    <>
                      <TableHead>品項</TableHead>
                      <TableHead>單位</TableHead>
                      <TableHead className="text-right">單價</TableHead>
                      <TableHead className="text-right">數量</TableHead>
                      <TableHead />
                    </>
                  ) : (
                    <>
                      <TableHead>SKU</TableHead>
                      <TableHead>品項名稱</TableHead>
                      <TableHead>規格</TableHead>
                      <TableHead>單位</TableHead>
                      <TableHead />
                    </>
                  )}
                </TableRow>
              </TableHeader>
              <TableBody>
                {isPoLinkedGrn ? (
                  <PoItemRows
                    items={poReceiptStatus?.items ?? []}
                    search={productSearch}
                    onSelect={onSelect}
                  />
                ) : isStockBasedDoc && targetWarehouseId ? (
                  <StockItemRows
                    items={stockBalances ?? []}
                    categoryCode={categoryCode}
                    isLoading={isStockLoading}
                    onSelect={onSelect}
                  />
                ) : (
                  <ProductItemRows products={products} onSelect={onSelect} />
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

// --- Sub row renderers ---

function PoItemRows({
  items,
  search,
  onSelect,
}: {
  items: PoReceiptItem[]
  search: string
  onSelect: (product: Product, extra?: ProductSelectExtraData) => void
}) {
  const filtered = items.filter(
    (item) => item.remaining_qty > 0 && (
      item.product_name.includes(search) || item.product_sku.includes(search) || item.product_id.includes(search)
    )
  )
  return (
    <>
      {filtered.map((item) => (
        <TableRow
          key={item.product_id}
          className="cursor-pointer hover:bg-muted"
          onClick={() =>
            onSelect(
              { id: item.product_id, sku: item.product_sku, name: item.product_name, base_uom: item.base_uom } as Product,
              { unit_price: item.unit_price ?? 0, remaining_qty: item.remaining_qty }
            )
          }
        >
          <TableCell>
            <div className="font-mono text-xs">{item.product_sku}</div>
            <div className="font-medium">{item.product_name}</div>
          </TableCell>
          <TableCell className="text-sm">{formatUom(item.uom || item.base_uom)}</TableCell>
          <TableCell className="text-right text-sm">
            {item.unit_price != null ? `$${formatNumber(item.unit_price, 2)}` : '-'}
          </TableCell>
          <TableCell className="text-right">
            <div className="text-xs">採購: {formatNumber(item.ordered_qty, 0)}</div>
            <div className="text-xs text-muted-foreground">已入庫: {formatNumber(item.received_qty, 0)}</div>
            <div className="text-sm font-bold text-primary">剩餘: {formatNumber(item.remaining_qty, 0)}</div>
          </TableCell>
          <TableCell>
            <Button size="sm" variant="outline">選擇</Button>
          </TableCell>
        </TableRow>
      ))}
    </>
  )
}

function StockItemRows({
  items,
  categoryCode,
  isLoading,
  onSelect,
}: {
  items: StockBalanceItem[]
  categoryCode?: string
  isLoading: boolean
  onSelect: (product: Product, extra?: ProductSelectExtraData) => void
}) {
  const filtered = items.filter((item) => !categoryCode || item.category_code === categoryCode)
  return (
    <>
      {filtered.map((item) => (
        <TableRow
          key={`${item.product_id}-${item.batch_no}-${item.storage_location_id}`}
          className="cursor-pointer hover:bg-muted"
          onClick={() =>
            onSelect(
              { id: item.product_id, sku: item.product_sku, name: item.product_name, base_uom: item.base_uom } as Product,
              { batch_no: item.batch_no, expiry_date: item.expiry_date, storage_location_id: item.storage_location_id }
            )
          }
        >
          <TableCell>
            <div className="font-mono text-xs">{item.product_sku}</div>
            <div className="font-medium">{item.product_name}</div>
          </TableCell>
          <TableCell>
            <div className="text-xs font-semibold">{item.batch_no || '無批號'}</div>
            <div className="text-[10px] text-muted-foreground">{item.expiry_date || '無效期'}</div>
          </TableCell>
          <TableCell className="text-xs text-muted-foreground">
            {item.storage_location_name || '未指定'}
          </TableCell>
          <TableCell className="text-right">
            <div className="font-bold text-primary">{formatNumber(parseFloat(item.qty_on_hand) || 0, 2)}</div>
            <div className="text-[10px]">{formatUom(item.base_uom)}</div>
          </TableCell>
          <TableCell>
            <Button size="sm" variant="outline">選擇</Button>
          </TableCell>
        </TableRow>
      ))}
      {!isLoading && filtered.length === 0 && (
        <TableRow>
          <TableCell colSpan={5} className="text-center py-4 text-muted-foreground text-sm">
            該倉庫目前無可用庫存
          </TableCell>
        </TableRow>
      )}
    </>
  )
}

function ProductItemRows({
  products,
  onSelect,
}: {
  products: Product[] | undefined
  onSelect: (product: Product) => void
}) {
  return (
    <>
      {products?.slice(0, 20).map((product) => (
        <TableRow
          key={product.id}
          className="cursor-pointer hover:bg-muted"
          onClick={() => onSelect(product)}
        >
          <TableCell className="font-mono text-xs">{product.sku}</TableCell>
          <TableCell className="font-medium">{product.name}</TableCell>
          <TableCell>{product.spec || '-'}</TableCell>
          <TableCell>{formatUom(product.base_uom)}</TableCell>
          <TableCell>
            <Button size="sm" variant="outline">選擇</Button>
          </TableCell>
        </TableRow>
      ))}
    </>
  )
}
