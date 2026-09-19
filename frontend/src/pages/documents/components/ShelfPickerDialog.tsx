/**
 * SO 跨倉儲位挑選器（2026-07-21 /system_table_chats 定案：清單｜平面圖並存兩 tab）。
 *
 * - 兩 tab 皆顯示「該品項在各儲位的現有存量」防呆：無存量儲位淡化不可選，
 *   選前就知道有沒有貨（避免核准才爆庫存不足）。
 * - 平面圖沿用倉庫佈局頁的座標/顏色資料（12 欄 grid），空間直覺對應實體找貨；
 *   未排座標或牆/門/窗以清單 tab 保底。
 * - localStorage 記住上次使用的 tab。
 * - 點選回傳儲位 + 所屬倉（行倉庫＝儲位倉，#1004 一段式多倉）。
 * - 未選品項時無從判斷存量：全部可選並提示先選品項。
 */
import { useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
  InventoryOnHand,
  StorageLocationType,
  StorageLocationWithWarehouse,
  WarehouseTreeNode,
} from '@/lib/api'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Check, LayoutGrid, List, Loader2, Map as MapIcon } from 'lucide-react'
import { cn } from '@/lib/utils'
import { warehouseColor } from '../warehouseColors'

export interface ShelfPickerSelection {
  locationId: string
  warehouseId: string
  /** 「倉庫 - 儲位」顯示字串 */
  label: string
}

interface ShelfPickerDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** 該行品項（用來查各儲位存量；空字串 = 尚未選品項，不做存量防呆） */
  productId: string
  productName?: string
  /** 表頭「預設倉庫」：平面圖 tab 的初始倉 */
  defaultWarehouseId?: string
  /** 目前已選儲位 id */
  value?: string
  onSelect: (sel: ShelfPickerSelection) => void
}

const MODE_KEY = 'so-shelf-picker-mode'
/** 可收納物的儲位類型（牆/門/窗為結構元素不可選）；顏色對齊 StorageLocationEditor DEFAULT_COLORS */
const PICKABLE_TYPES: StorageLocationType[] = ['shelf', 'rack', 'zone', 'bin']
const TYPE_COLORS: Record<StorageLocationType, string> = {
  shelf: '#3b82f6',
  rack: '#10b981',
  zone: '#f59e0b',
  bin: '#6366f1',
  wall: '#475569',
  door: '#94a3b8',
  window: '#bae6fd',
}

const fmtQty = (n: number) => n.toLocaleString('zh-TW')

export function ShelfPickerDialog({
  open,
  onOpenChange,
  productId,
  productName,
  defaultWarehouseId,
  value,
  onSelect,
}: ShelfPickerDialogProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<string>(() => localStorage.getItem(MODE_KEY) || 'list')
  const [floorWarehouseId, setFloorWarehouseId] = useState<string>('')

  // 對話框常駐掛載（SO 下 isSo 恆真）：每次開啟重設平面圖倉別，
  // 避免上一行切過的倉殘留到下一行（gemini #1013 review）。
  useEffect(() => {
    if (open) setFloorWarehouseId('')
  }, [open])

  const { data: tree, isLoading: treeLoading } = useQuery({
    queryKey: ['warehouses-with-shelves'],
    queryFn: async () => {
      const res = await api.get<WarehouseTreeNode[]>('/warehouses/with-shelves')
      return res.data
    },
    staleTime: 5 * 60 * 1000,
    enabled: open,
  })

  // 該品項各儲位存量（與 ProductInventorySnapshot 同 key，共用快取、mutation 後自動刷新）
  const { data: onHand } = useQuery({
    queryKey: ['inventory', 'on-hand', 'product', productId],
    queryFn: async () => {
      const res = await api.get<InventoryOnHand[]>(`/inventory/on-hand?product_id=${productId}`)
      return res.data
    },
    enabled: open && !!productId,
  })

  const qtyByLoc = useMemo(() => {
    const m = new Map<string, number>()
    onHand?.forEach((row) => {
      if (!row.storage_location_id) return
      m.set(
        row.storage_location_id,
        (m.get(row.storage_location_id) ?? 0) + (parseFloat(row.qty_on_hand) || 0),
      )
    })
    return m
  }, [onHand])

  const qtyByWh = useMemo(() => {
    const m = new Map<string, number>()
    onHand?.forEach((row) => {
      m.set(row.warehouse_id, (m.get(row.warehouse_id) ?? 0) + (parseFloat(row.qty_on_hand) || 0))
    })
    return m
  }, [onHand])

  // 存量防呆只在「已選品項」時生效
  const hasStockInfo = !!productId

  // 平面圖目前倉：使用者選過 > 預設倉 > 第一個有存量的倉 > 第一個倉
  const effectiveFloorWh =
    floorWarehouseId ||
    defaultWarehouseId ||
    tree?.find((w) => (qtyByWh.get(w.id) ?? 0) > 0)?.id ||
    tree?.[0]?.id ||
    ''

  const { data: floorLocations, isLoading: floorLoading } = useQuery({
    queryKey: ['storage-locations', effectiveFloorWh],
    queryFn: async () => {
      const res = await api.get<StorageLocationWithWarehouse[]>(
        `/storage-locations?warehouse_id=${effectiveFloorWh}`,
      )
      return res.data
    },
    enabled: open && mode === 'floor' && !!effectiveFloorWh,
  })

  const changeMode = (m: string) => {
    setMode(m)
    localStorage.setItem(MODE_KEY, m)
  }

  const pick = (locationId: string, warehouseId: string, whName: string, shelfLabel: string) => {
    onSelect({ locationId, warehouseId, label: `${whName} - ${shelfLabel}` })
    onOpenChange(false)
  }

  // 只顯示「有該品項存量」的倉（2026-07-22 使用者現場回饋：無關位置整個隱藏，不只淡化）；
  // 未選品項時無從過濾，維持全列。清單排序：預設倉優先，其餘依樹序。
  const orderedTree = useMemo(() => {
    if (!tree) return []
    const base = hasStockInfo ? tree.filter((w) => (qtyByWh.get(w.id) ?? 0) > 0) : tree
    if (!defaultWarehouseId) return base
    return [...base].sort((a, b) =>
      a.id === defaultWarehouseId ? -1 : b.id === defaultWarehouseId ? 1 : 0,
    )
  }, [tree, defaultWarehouseId, hasStockInfo, qtyByWh])

  const whIndex = useMemo(() => {
    const m = new Map<string, number>()
    tree?.forEach((w, i) => m.set(w.id, i))
    return m
  }, [tree])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>
            {productName
              ? t('erpDocs.documents.shelfPicker.titleWithProduct', { product: productName })
              : t('erpDocs.shared.selectStorageLocation')}
          </DialogTitle>
          <DialogDescription>
            {hasStockInfo
              ? t('erpDocs.documents.shelfPicker.descWithStock')
              : t('erpDocs.documents.shelfPicker.descNoStock')}
          </DialogDescription>
        </DialogHeader>

        <Tabs value={mode} onValueChange={changeMode}>
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="list" className="gap-1.5">
              <List className="h-4 w-4" /> {t('erpDocs.documents.shelfPicker.tabList')}
            </TabsTrigger>
            <TabsTrigger value="floor" className="gap-1.5">
              <MapIcon className="h-4 w-4" /> {t('erpDocs.documents.shelfPicker.tabFloor')}
            </TabsTrigger>
          </TabsList>

          {/* ── 清單 tab ── */}
          <TabsContent value="list" className="mt-3">
            {treeLoading ? (
              <div className="flex justify-center py-10">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : hasStockInfo && orderedTree.length === 0 ? (
              <div className="rounded-lg border bg-muted/30 py-10 text-center text-sm text-muted-foreground">
                {t('erpDocs.documents.shelfPicker.noStockAtAll')}
              </div>
            ) : (
              <div className="max-h-[420px] space-y-3 overflow-y-auto pr-1">
                {orderedTree.map((wh) => {
                  const whQty = qtyByWh.get(wh.id) ?? 0
                  const idx = whIndex.get(wh.id) ?? 0
                  // 只列有存量的儲位（未選品項時全列）
                  const shelves = hasStockInfo
                    ? wh.shelves.filter((s) => (qtyByLoc.get(s.id) ?? 0) > 0)
                    : wh.shelves
                  return (
                    <div key={wh.id} className="rounded-lg border">
                      <div className="flex items-center justify-between border-b bg-muted/40 px-3 py-2">
                        <span className="flex items-center gap-2 font-medium">
                          <span
                            className="h-2.5 w-2.5 rounded-sm"
                            style={{ backgroundColor: warehouseColor(idx) }}
                          />
                          {wh.name}
                          {wh.id === defaultWarehouseId && (
                            <span className="text-xs font-normal text-muted-foreground">{t('erpDocs.documents.shelfPicker.defaultWarehouse')}</span>
                          )}
                        </span>
                        {hasStockInfo && (
                          <span className="text-xs text-muted-foreground">{t('erpDocs.documents.shelfPicker.total', { qty: fmtQty(whQty) })}</span>
                        )}
                      </div>
                      {shelves.length === 0 ? (
                        <div className="px-3 py-2 text-xs text-muted-foreground">
                          {hasStockInfo
                            ? t('erpDocs.documents.shelfPicker.allUnassigned', { qty: fmtQty(whQty) })
                            : t('erpDocs.documents.shelfPicker.noShelves')}
                        </div>
                      ) : (
                        <ul className="divide-y">
                          {shelves.map((shelf) => {
                            const qty = qtyByLoc.get(shelf.id) ?? 0
                            const selected = value === shelf.id
                            return (
                              <li key={shelf.id}>
                                <button
                                  type="button"
                                  onClick={() =>
                                    pick(shelf.id, wh.id, wh.name, shelf.name || shelf.code)
                                  }
                                  className={cn(
                                    'flex w-full items-center justify-between px-3 py-2 text-sm transition-colors hover:bg-primary/5 hover:text-primary',
                                    selected && 'bg-primary/10 font-semibold text-primary',
                                  )}
                                >
                                  <span className="flex items-center gap-2">
                                    <LayoutGrid className="h-3.5 w-3.5 opacity-50" />
                                    {shelf.name || shelf.code}
                                    {selected && <Check className="h-3.5 w-3.5" />}
                                  </span>
                                  {hasStockInfo && (
                                    <span className="text-xs font-medium">{t('erpDocs.documents.shelfPicker.qtyLabel', { qty: fmtQty(qty) })}</span>
                                  )}
                                </button>
                              </li>
                            )
                          })}
                        </ul>
                      )}
                    </div>
                  )
                })}
              </div>
            )}
          </TabsContent>

          {/* ── 平面圖 tab ── */}
          <TabsContent value="floor" className="mt-3 space-y-3">
            <div className="flex items-center gap-3">
              <Select value={effectiveFloorWh} onValueChange={setFloorWarehouseId}>
                <SelectTrigger className="w-56">
                  <SelectValue placeholder={t('erpDocs.shared.selectWarehouse')} />
                </SelectTrigger>
                <SelectContent>
                  {(hasStockInfo
                    ? tree?.filter((wh) => (qtyByWh.get(wh.id) ?? 0) > 0)
                    : tree
                  )?.map((wh) => (
                    <SelectItem key={wh.id} value={wh.id}>
                      {hasStockInfo
                        ? t('erpDocs.documents.shelfPicker.nameWithQty', { name: wh.name, qty: fmtQty(qtyByWh.get(wh.id) ?? 0) })
                        : wh.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <span className="text-xs text-muted-foreground">
                {hasStockInfo
                  ? t('erpDocs.documents.shelfPicker.floorHintStock')
                  : t('erpDocs.documents.shelfPicker.floorHintNoStock')}
              </span>
            </div>

            {floorLoading ? (
              <div className="flex justify-center py-10">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : !floorLocations || floorLocations.length === 0 ? (
              <div className="rounded-lg border bg-muted/30 py-10 text-center text-sm text-muted-foreground">
                {t('erpDocs.documents.shelfPicker.noLayout')}
              </div>
            ) : (
              (() => {
                // 只畫「有存量的儲位」＋牆/門/窗（定位參考）；無關儲位整個隱藏
                // （2026-07-22 使用者現場回饋）。未選品項時維持全畫。
                const visible = hasStockInfo
                  ? floorLocations.filter(
                      (loc) =>
                        !PICKABLE_TYPES.includes(loc.location_type) ||
                        (qtyByLoc.get(loc.id) ?? 0) > 0,
                    )
                  : floorLocations
                const stockedCells = visible.filter((loc) =>
                  PICKABLE_TYPES.includes(loc.location_type),
                )
                if (hasStockInfo && stockedCells.length === 0) {
                  return (
                    <div className="rounded-lg border bg-muted/30 py-10 text-center text-sm text-muted-foreground">
                      {t('erpDocs.documents.shelfPicker.floorAllUnassigned')}
                    </div>
                  )
                }
                return (
                  <div
                    className="grid max-h-[380px] gap-1.5 overflow-y-auto rounded-lg border bg-muted/20 p-3"
                    style={{ gridTemplateColumns: 'repeat(12, minmax(0, 1fr))', gridAutoRows: '34px' }}
                  >
                    {visible.map((loc) => {
                      const pickable = PICKABLE_TYPES.includes(loc.location_type) && loc.is_active
                      const qty = qtyByLoc.get(loc.id) ?? 0
                      const disabled = !pickable || (hasStockInfo && qty <= 0)
                      const selected = value === loc.id
                      const whName =
                        tree?.find((w) => w.id === effectiveFloorWh)?.name || loc.warehouse_name
                      return (
                        <button
                          key={loc.id}
                          type="button"
                          disabled={disabled}
                          title={
                            hasStockInfo
                              ? t('erpDocs.documents.shelfPicker.nameWithQty', { name: loc.name || loc.code, qty: fmtQty(qty) })
                              : loc.name || loc.code
                          }
                          onClick={() => pick(loc.id, loc.warehouse_id, whName, loc.name || loc.code)}
                          className={cn(
                            'flex flex-col items-start justify-center overflow-hidden rounded-md px-1.5 text-left text-[11px] leading-tight text-white transition-transform',
                            disabled ? 'cursor-not-allowed opacity-60' : 'hover:scale-[1.03]',
                            selected && 'ring-2 ring-foreground ring-offset-1',
                          )}
                          style={{
                            gridColumn: `${(loc.col_index ?? 0) + 1} / span ${Math.max(loc.width || 2, 1)}`,
                            gridRow: `${(loc.row_index ?? 0) + 1} / span ${Math.max(loc.height || 2, 1)}`,
                            backgroundColor: loc.color || TYPE_COLORS[loc.location_type] || '#64748b',
                          }}
                        >
                          <span className="w-full truncate font-semibold">{loc.name || loc.code}</span>
                          {pickable && hasStockInfo && (
                            <span className="w-full truncate opacity-90">{t('erpDocs.documents.shelfPicker.qtyLabel', { qty: fmtQty(qty) })}</span>
                          )}
                        </button>
                      )
                    })}
                  </div>
                )
              })()
            )}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}
