import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
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
  const [storageLocationId, setStorageLocationId] = useState('')
  const [qty, setQty] = useState(String(unassignedQty))
  const queryClient = useQueryClient()

  const mutation = useMutation({
    mutationFn: async () => {
      const parsedQty = parseFloat(qty)
      if (isNaN(parsedQty) || parsedQty <= 0) throw new Error('分配數量必須大於 0')
      if (parsedQty > unassignedQty) {
        throw new Error(`分配數量不可超過未分配量 ${unassignedQty}`)
      }
      if (!storageLocationId) throw new Error('請選擇目標儲位')
      await api.post('/inventory/unassigned/assign', {
        warehouse_id: warehouseId,
        product_id: productId,
        storage_location_id: storageLocationId,
        qty: parsedQty,
      })
    },
    onSuccess: () => {
      toast({ title: '成功', description: '已分配至儲位' })
      queryClient.invalidateQueries({ queryKey: ['inventory', 'unassigned'] })
      queryClient.invalidateQueries({ queryKey: ['inventory', 'on-hand'] })
      queryClient.invalidateQueries({ queryKey: ['inventory', 'batch-detail'] })
      queryClient.invalidateQueries({ queryKey: ['storage-location-inventory'] })
      onOpenChange(false)
      setStorageLocationId('')
      setQty(String(unassignedQty))
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '分配失敗'),
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
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>分配未分配庫存至儲位</DialogTitle>
          <DialogDescription>
            將倉庫層級的未分配庫存指派到具體貨架
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="rounded-md border bg-muted/30 p-3 space-y-1 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">倉庫</span>
              <span className="font-medium">{warehouseName}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">品項</span>
              <span className="font-medium text-right">
                {productName}
                <span className="block text-xs font-mono text-muted-foreground/70">{productSku}</span>
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">可分配量</span>
              <span className="font-bold text-status-warning-text">
                {unassignedQty} {baseUom}
              </span>
            </div>
          </div>

          <div className="space-y-2">
            <Label>目標儲位 *</Label>
            <WarehouseShelfTreeSelect
              value={storageLocationId ? `loc:${storageLocationId}` : ''}
              onValueChange={handleShelfSelect}
              selectLevel="shelf"
              allowAll={false}
              parentId={warehouseId}
              placeholder="選擇目標儲位"
            />
          </div>

          <div className="space-y-2">
            <Label>分配數量 *</Label>
            <Input
              type="number"
              value={qty}
              onChange={(e) => setQty(e.target.value)}
              min="0"
              max={unassignedQty}
              step="any"
            />
            <p className="text-xs text-muted-foreground">
              最大 {unassignedQty} {baseUom}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={mutation.isPending}>
            取消
          </Button>
          <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>
            {mutation.isPending ? '分配中...' : '確認分配'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
