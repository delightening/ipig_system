import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import type { UnassignedSourceDoc } from '@/types/erp'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import {
  WarehouseShelfTreeSelect,
  type WarehouseShelfValue,
} from '@/components/inventory/WarehouseShelfTreeSelect'

interface AssignToShelfDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  warehouseId: string
  warehouseName: string
  productId: string
  productName: string
  productSku: string
  unassignedQty: number
  baseUom: string
}

/** 全部（依 FIFO 自動分批）的選項 key */
const ALL_BATCHES = '__ALL__'

interface BatchOption {
  key: string
  batchNo: string | null
  expiryDate: string | null
  remaining: number
  label: string
}

export function AssignToShelfDialog({
  open,
  onOpenChange,
  warehouseId,
  warehouseName,
  productId,
  productName,
  productSku,
  unassignedQty,
  baseUom,
}: AssignToShelfDialogProps) {
  const { t } = useTranslation()
  const [storageLocationId, setStorageLocationId] = useState('')
  const [batchKey, setBatchKey] = useState(ALL_BATCHES)
  const [qty, setQty] = useState(String(unassignedQty))
  const queryClient = useQueryClient()

  // 來源批號：只有對話框開啟時查，供「依批號分配」用（批號忠實上架）。
  const { data: sources } = useQuery({
    queryKey: ['inventory', 'unassigned', 'sources', warehouseId, productId],
    queryFn: async () => {
      const res = await api.get<UnassignedSourceDoc[]>('/inventory/unassigned/sources', {
        params: { warehouse_id: warehouseId, product_id: productId },
      })
      return res.data
    },
    enabled: open,
  })

  // 依 (批號, 效期) 聚合來源剩餘量；只保留有批號或效期者（無批號品項不顯示選擇器）。
  const batchOptions = useMemo<BatchOption[]>(() => {
    if (!sources) return []
    const map = new Map<string, BatchOption>()
    for (const s of sources) {
      if (!s.batch_no && !s.expiry_date) continue
      const key = `${s.batch_no ?? ''}__${s.expiry_date ?? ''}`
      const remaining = parseFloat(s.remaining_unshelved)
      const existing = map.get(key)
      if (existing) {
        existing.remaining += remaining
      } else {
        const parts = [
          s.batch_no ? t('erpDocs.shared.batchLabel', { batchNo: s.batch_no }) : t('erpDocs.shared.noBatch'),
        ]
        if (s.expiry_date) parts.push(t('erpDocs.shared.expiryLabel', { date: s.expiry_date }))
        map.set(key, {
          key,
          batchNo: s.batch_no,
          expiryDate: s.expiry_date,
          remaining,
          label: parts.join(' · '),
        })
      }
    }
    return Array.from(map.values())
  }, [sources, t])

  const selectedBatch = batchOptions.find((b) => b.key === batchKey)
  const maxQty = batchKey === ALL_BATCHES ? unassignedQty : (selectedBatch?.remaining ?? 0)

  const handleBatchChange = (key: string) => {
    setBatchKey(key)
    const next = key === ALL_BATCHES ? unassignedQty : (batchOptions.find((b) => b.key === key)?.remaining ?? 0)
    setQty(String(next))
  }

  const mutation = useMutation({
    mutationFn: async () => {
      const parsedQty = parseFloat(qty)
      if (isNaN(parsedQty) || parsedQty <= 0) throw new Error(t('erpDocs.inventory.assign.errQtyPositive'))
      if (parsedQty > maxQty) {
        throw new Error(t('erpDocs.inventory.assign.errQtyExceeds', { max: maxQty }))
      }
      if (!storageLocationId) throw new Error(t('erpDocs.inventory.assign.errSelectLocation'))
      await api.post('/inventory/unassigned/assign', {
        warehouse_id: warehouseId,
        product_id: productId,
        storage_location_id: storageLocationId,
        qty: parsedQty,
        // 指定批號時帶入，後端只從相符來源攤扣並以該批號上架；未指定則 FIFO 自動分批。
        batch_no: batchKey === ALL_BATCHES ? null : (selectedBatch?.batchNo ?? null),
        expiry_date: batchKey === ALL_BATCHES ? null : (selectedBatch?.expiryDate ?? null),
      })
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('erpDocs.inventory.assign.assigned') })
      // 庫存查詢頁（InventoryPage / WarehouseDetailTabs）：bare ['inventory'] 前綴涵蓋主清單
      // ['inventory', locationFilter…] / ['inventory','unassigned'…] / ['inventory','batch-detail'…]
      //（同 queryInvalidation.ts 慣例，避免逐一列 key 又漏掉；原 ['inventory','on-hand'] 為死 key）。
      queryClient.invalidateQueries({ queryKey: ['inventory'] })
      queryClient.invalidateQueries({ queryKey: ['storage-location-inventory'] })
      // 倉庫佈局頁（WarehouseLayoutPage）：未分配清單 + 佈局圖各貨架數量。
      // 缺這兩個 → 從佈局頁分配後未分配清單與貨架數字不會即時刷新（需手動重整）。
      queryClient.invalidateQueries({ queryKey: ['unassigned-inventory'] })
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] })
      onOpenChange(false)
      setStorageLocationId('')
      setBatchKey(ALL_BATCHES)
      setQty(String(unassignedQty))
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.inventory.assign.assignFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleShelfSelect = (value: WarehouseShelfValue) => {
    if (value.startsWith('loc:')) {
      setStorageLocationId(value.slice(4))
    } else {
      setStorageLocationId('')
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t('erpDocs.inventory.assign.title')}</DialogTitle>
          <DialogDescription>
            {t('erpDocs.inventory.assign.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="rounded-md border bg-muted/30 p-3 space-y-1 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.shared.warehouse')}</span>
              <span className="font-medium">{warehouseName}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.shared.item')}</span>
              <span className="font-medium text-right">
                {productName}
                <span className="block text-xs font-mono text-muted-foreground/70">{productSku}</span>
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.inventory.assign.assignableQty')}</span>
              <span className="font-bold text-status-warning-text">
                {maxQty} {baseUom}
              </span>
            </div>
          </div>

          {batchOptions.length > 0 && (
            <div className="space-y-2">
              <Label>{t('erpDocs.shared.batchExpiry')}</Label>
              <Select value={batchKey} onValueChange={handleBatchChange}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={ALL_BATCHES}>{t('erpDocs.inventory.assign.allBatchesFifo')}</SelectItem>
                  {batchOptions.map((b) => (
                    <SelectItem key={b.key} value={b.key}>
                      {t('erpDocs.inventory.assign.batchOption', { label: b.label, qty: b.remaining, uom: baseUom })}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                {t('erpDocs.inventory.assign.batchHint')}
              </p>
            </div>
          )}

          <div className="space-y-2">
            <Label>{t('erpDocs.inventory.assign.targetLocationRequired')}</Label>
            <WarehouseShelfTreeSelect
              value={storageLocationId ? `loc:${storageLocationId}` : ''}
              onValueChange={handleShelfSelect}
              selectLevel="shelf"
              allowAll={false}
              parentId={warehouseId}
              placeholder={t('erpDocs.shared.selectTargetLocation')}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('erpDocs.inventory.assign.qtyRequired')}</Label>
            <Input
              type="number"
              value={qty}
              onChange={(e) => setQty(e.target.value)}
              min="0"
              max={maxQty}
              step="any"
            />
            <p className="text-xs text-muted-foreground">
              {t('erpDocs.inventory.assign.max', { max: maxQty, uom: baseUom })}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={mutation.isPending}>
            {t('common.cancel')}
          </Button>
          <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>
            {mutation.isPending ? t('erpDocs.inventory.assign.assigning') : t('erpDocs.inventory.assign.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
