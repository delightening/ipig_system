import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Loader2, Boxes, ChevronRight, PackagePlus } from 'lucide-react'
import api from '@/lib/api'
import type { InventoryOnHand, UnassignedSourceDoc } from '@/types/erp'
import { EmptyState } from '@/components/ui/empty-state'
import { Button } from '@/components/ui/button'
import { useAuthHasPermission } from '@/stores/auth'
import { AssignToShelfDialog } from '@/pages/inventory/components/AssignToShelfDialog'

// 庫存量顯示：保留必要小數（如 1.5 kg）並自動去尾零（370 顯示 370，非 370.00）。
// 不用 formatNumber(x, 0)（會截斷小數）；toLocaleString 預設 max 3 位小數亦順帶清浮點誤差。
const fmtQty = (n: number) => n.toLocaleString('zh-TW')

interface ProductInventorySnapshotProps {
  productId: string
  uomLabel: string
}

interface LocationGroup {
  name: string
  qty: number
  unassigned: boolean
}

interface WarehouseGroup {
  name: string
  total: number
  locations: Map<string, LocationGroup>
}

/**
 * 未分配列：可展開追溯來源 GRN（erp.stock.view，已能看本頁者皆有），倉管（erp.stock.adjust）
 * 另有「分配」鈕開既有 AssignToShelfDialog。分配走自核准 TR，成功後 ['inventory'] 前綴 query
 * 一併失效 → 本頁快照與此處來源自動刷新。
 */
interface UnassignedLocRowProps {
  warehouseId: string
  warehouseName: string
  productId: string
  productName: string
  productSku: string
  baseUom: string
  qty: number
  uomLabel: string
  canAssign: boolean
}

function UnassignedLocRow({
  warehouseId,
  warehouseName,
  productId,
  productName,
  productSku,
  baseUom,
  qty,
  uomLabel,
  canAssign,
}: UnassignedLocRowProps) {
  const [expanded, setExpanded] = useState(false)
  const [assignOpen, setAssignOpen] = useState(false)

  // 展開才 lazy 抓來源，不影響本頁初始載入。
  const { data: sources, isLoading } = useQuery({
    queryKey: ['inventory', 'unassigned', 'sources', warehouseId, productId],
    queryFn: async () => {
      const res = await api.get<UnassignedSourceDoc[]>('/inventory/unassigned/sources', {
        params: { warehouse_id: warehouseId, product_id: productId },
      })
      return res.data
    },
    enabled: expanded,
  })

  return (
    <li>
      <div className="flex items-center justify-between px-3 py-1.5 text-sm">
        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          className="flex items-center gap-1.5 text-amber-600 hover:underline dark:text-amber-500"
        >
          <ChevronRight
            className={`h-3.5 w-3.5 transition-transform ${expanded ? 'rotate-90' : ''}`}
          />
          未分配
        </button>
        <div className="flex items-center gap-2">
          <span className="text-amber-600 dark:text-amber-500">
            {fmtQty(qty)} {uomLabel}
          </span>
          {canAssign && (
            <Button
              size="sm"
              variant="outline"
              className="h-7 gap-1.5"
              onClick={() => setAssignOpen(true)}
            >
              <PackagePlus className="h-3.5 w-3.5" />
              分配
            </Button>
          )}
        </div>
      </div>

      {expanded && (
        <div className="px-3 pb-2 pl-8 text-xs">
          {isLoading ? (
            <div className="flex items-center gap-2 py-1 text-muted-foreground">
              <Loader2 className="h-3.5 w-3.5 animate-spin" /> 載入來源單據…
            </div>
          ) : !sources || sources.length === 0 ? (
            <p className="py-1 text-muted-foreground">
              無可追溯的採購入庫來源（可能為調整單 / 期初匯入）。
            </p>
          ) : (
            <ul className="space-y-1.5">
              {sources.map((s, i) => (
                <li
                  key={`${s.document_id}-${i}`}
                  className="rounded border-l-2 border-amber-500/70 bg-muted/30 px-2 py-1"
                >
                  <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
                    <span className="font-mono font-medium">{s.doc_no}</span>
                    <span className="text-muted-foreground">{s.doc_date}</span>
                    {s.partner_name && (
                      <span className="text-muted-foreground">· {s.partner_name}</span>
                    )}
                    <span className="ml-auto font-medium text-amber-600 dark:text-amber-500">
                      {fmtQty(parseFloat(s.remaining_unshelved) || 0)} {uomLabel}
                    </span>
                  </div>
                  {(s.batch_no || s.expiry_date) && (
                    <div className="text-muted-foreground/80">
                      {s.batch_no ? `批號 ${s.batch_no}` : '無批號'}
                      {s.expiry_date ? ` · 效期 ${s.expiry_date}` : ''}
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      {canAssign && (
        <AssignToShelfDialog
          open={assignOpen}
          onOpenChange={setAssignOpen}
          warehouseId={warehouseId}
          warehouseName={warehouseName}
          productId={productId}
          productName={productName}
          productSku={productSku}
          unassignedQty={qty}
          baseUom={baseUom}
        />
      )}
    </li>
  )
}

/**
 * 產品詳情頁「各倉庫庫存快照」：呼叫 GET /inventory/on-hand?product_id= 取本產品現況，
 * 依倉庫 → 儲位分組顯示「存貨放在哪裡」。批號可能把同儲位拆成多列，此處按儲位加總。
 * query key ['inventory', 'on-hand', 'product', id] 被 bare ['inventory'] 前綴涵蓋，
 * 因此入庫 / 分配 / 調撥等 mutation 後會自動刷新（見 AssignToShelfDialog / queryInvalidation.ts）。
 */
export function ProductInventorySnapshot({ productId, uomLabel }: ProductInventorySnapshotProps) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['inventory', 'on-hand', 'product', productId],
    queryFn: async () => {
      const res = await api.get<InventoryOnHand[]>(`/inventory/on-hand?product_id=${productId}`)
      return res.data
    },
    enabled: !!productId,
  })
  const hasPermission = useAuthHasPermission()
  const canAssign = hasPermission('erp.stock.adjust')

  if (isLoading) {
    return (
      <div className="flex justify-center py-8">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }
  if (isError) {
    return <div className="py-8 text-center text-sm text-destructive">載入庫存分佈失敗，請稍後再試。</div>
  }
  if (!data || data.length === 0) {
    return <EmptyState icon={Boxes} title="目前無庫存" />
  }

  // 品項層級欄位對本 productId 為定值，取首列供分配對話框帶入。
  const productName = data[0].product_name
  const productSku = data[0].product_sku
  const baseUom = data[0].base_uom

  const byWarehouse = new Map<string, WarehouseGroup>()
  for (const row of data) {
    const wh = byWarehouse.get(row.warehouse_id) ?? {
      name: row.warehouse_name,
      total: 0,
      locations: new Map<string, LocationGroup>(),
    }
    const qty = parseFloat(row.qty_on_hand) || 0
    wh.total += qty
    const isUnassigned = !row.storage_location_id
    const locKey = row.storage_location_id ?? '__unassigned__'
    const locName = isUnassigned
      ? '未分配'
      : row.storage_location_name || row.storage_location_code || '—'
    const loc = wh.locations.get(locKey) ?? { name: locName, qty: 0, unassigned: isUnassigned }
    loc.qty += qty
    wh.locations.set(locKey, loc)
    byWarehouse.set(row.warehouse_id, wh)
  }

  return (
    <div className="space-y-4">
      {[...byWarehouse.entries()].map(([whId, wh]) => (
        <div key={whId} className="rounded-lg border">
          <div className="flex items-center justify-between border-b bg-muted/40 px-3 py-2">
            <span className="font-medium">{wh.name}</span>
            <span className="text-sm text-muted-foreground">
              合計 {fmtQty(wh.total)} {uomLabel}
            </span>
          </div>
          <ul className="divide-y">
            {[...wh.locations.entries()].map(([locKey, loc]) =>
              loc.unassigned ? (
                <UnassignedLocRow
                  key={locKey}
                  warehouseId={whId}
                  warehouseName={wh.name}
                  productId={productId}
                  productName={productName}
                  productSku={productSku}
                  baseUom={baseUom}
                  qty={loc.qty}
                  uomLabel={uomLabel}
                  canAssign={canAssign}
                />
              ) : (
                <li
                  key={locKey}
                  className="flex items-center justify-between px-3 py-1.5 text-sm"
                >
                  <span>{loc.name}</span>
                  <span>
                    {fmtQty(loc.qty)} {uomLabel}
                  </span>
                </li>
              )
            )}
          </ul>
        </div>
      ))}
    </div>
  )
}
