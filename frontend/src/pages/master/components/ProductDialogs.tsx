import { useTranslation } from 'react-i18next'

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
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {action === 'activate' && t('erpMaster.products.statusDialog.activateTitle')}
            {action === 'deactivate' && t('erpMaster.products.statusDialog.deactivateTitle')}
            {action === 'discontinue' && t('erpMaster.products.statusDialog.discontinueTitle')}
          </DialogTitle>
          <DialogDescription>
            {action === 'activate' && t('erpMaster.products.statusDialog.activateDescription')}
            {action === 'deactivate' && t('erpMaster.products.statusDialog.deactivateDescription')}
            {action === 'discontinue' && t('erpMaster.products.statusDialog.discontinueDescription')}
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
            {t('common.cancel')}
          </Button>
          <Button
            variant={action === 'discontinue' ? 'destructive' : 'default'}
            onClick={onConfirm}
            disabled={isPending}
          >
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('common.confirm')}
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
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('erpMaster.products.batchDialog.title')}</DialogTitle>
          <DialogDescription>
            {t('erpMaster.products.batchDialog.description', { count: selectionSize })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onConfirm} disabled={isPending}>
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('erpMaster.products.batchDialog.confirm')}
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
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="text-destructive">{t('erpMaster.products.hardDeleteDialog.title')}</DialogTitle>
          <DialogDescription>
            {t('erpMaster.products.hardDeleteDialog.description')}
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
            {t('common.cancel')}
          </Button>
          <Button variant="destructive" onClick={onConfirm} disabled={isPending}>
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('erpMaster.products.hardDeleteDialog.confirm')}
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
