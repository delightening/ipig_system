import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, Package } from 'lucide-react'
import { ProductImportDialog } from '@/components/product/ProductImportDialog'
import { EditCategoriesDialog } from '@/components/product/EditCategoriesDialog'

import type { ExtendedProduct, StatusAction } from './productTypes'

interface StatusDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  product: ExtendedProduct | null
  action: StatusAction
  isPending: boolean
  onConfirm: () => void
  onClose: () => void
}

/** 單筆狀態變更對話框 */
export function StatusChangeDialog({
  open,
  onOpenChange,
  product,
  action,
  isPending,
  onConfirm,
  onClose,
}: StatusDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {action === 'activate' && '啟用產品'}
            {action === 'deactivate' && '停用產品'}
            {action === 'discontinue' && '標記停產'}
          </DialogTitle>
          <DialogDescription>
            {action === 'activate' && '確定要啟用此產品嗎？啟用後可在採購、銷貨等模組中使用。'}
            {action === 'deactivate' && '確定要停用此產品嗎？停用後將無法在新單據中選擇此產品。'}
            {action === 'discontinue' && '確定要將此產品標記為停產嗎？停產後僅供歷史查詢，無法恢復為啟用狀態。'}
          </DialogDescription>
        </DialogHeader>
        {product && (
          <div className="py-4">
            <div className="p-3 bg-muted rounded-lg">
              <div className="flex items-center gap-3">
                <Package className="h-8 w-8 text-muted-foreground" />
                <div>
                  <p className="font-medium">{product.name}</p>
                  <p className="text-sm text-muted-foreground font-mono">{product.sku}</p>
                </div>
              </div>
            </div>
          </div>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            取消
          </Button>
          <Button
            variant={action === 'discontinue' ? 'destructive' : 'default'}
            onClick={onConfirm}
            disabled={isPending}
          >
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            確認
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface BatchStatusDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectionSize: number
  isPending: boolean
  onConfirm: () => void
  onClose: () => void
}

/** 批次狀態變更對話框 */
export function BatchStatusDialog({
  open,
  onOpenChange,
  selectionSize,
  isPending,
  onConfirm,
  onClose,
}: BatchStatusDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>批次停用產品</DialogTitle>
          <DialogDescription>
            確定要停用選中的 {selectionSize} 個產品嗎？停用後將無法在新單據中選擇這些產品。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            取消
          </Button>
          <Button onClick={onConfirm} disabled={isPending}>
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            確認停用
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface HardDeleteDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  product: ExtendedProduct | null
  isPending: boolean
  onConfirm: () => void
  onClose: () => void
}

/** 硬刪除確認對話框 */
export function HardDeleteDialog({
  open,
  onOpenChange,
  product,
  isPending,
  onConfirm,
  onClose,
}: HardDeleteDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="text-destructive">硬刪除產品</DialogTitle>
          <DialogDescription>
            此操作將永久刪除產品資料，無法復原。若產品已有單據、庫存或藥物選單關聯則無法執行。確定要硬刪除此產品嗎？
          </DialogDescription>
        </DialogHeader>
        {product && (
          <div className="py-4">
            <div className="p-3 bg-destructive/10 border border-destructive/30 rounded-lg">
              <div className="flex items-center gap-3">
                <Package className="h-8 w-8 text-destructive" />
                <div>
                  <p className="font-medium">{product.name}</p>
                  <p className="text-sm text-muted-foreground font-mono">{product.sku}</p>
                </div>
              </div>
            </div>
          </div>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            取消
          </Button>
          <Button variant="destructive" onClick={onConfirm} disabled={isPending}>
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            確認硬刪除
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface ImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/** 匯入對話框（委託至共用元件） */
export function ImportDialog({ open, onOpenChange }: ImportDialogProps) {
  return <ProductImportDialog open={open} onOpenChange={onOpenChange} />
}

interface EditCategoriesDialogWrapperProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/** 編輯分類對話框（委託至共用元件） */
export function EditCategoriesDialogWrapper({ open, onOpenChange }: EditCategoriesDialogWrapperProps) {
  return <EditCategoriesDialog open={open} onOpenChange={onOpenChange} />
}
