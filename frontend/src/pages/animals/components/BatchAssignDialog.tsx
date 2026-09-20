import { useTranslation } from 'react-i18next'
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
import { Loader2 } from 'lucide-react'

interface BatchAssignDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedCount: number
  iacucNo: string
  onIacucNoChange: (value: string) => void
  onSubmit: () => void
  isPending: boolean
}

export function BatchAssignDialog({
  open,
  onOpenChange,
  selectedCount,
  iacucNo,
  onIacucNoChange,
  onSubmit,
  isPending,
}: BatchAssignDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('animalPages.batchAssign.title')}</DialogTitle>
          <DialogDescription>
            {t('animalPages.batchAssign.description', { count: selectedCount })}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="iacuc_no">IACUC No. *</Label>
            <Input
              id="iacuc_no"
              value={iacucNo}
              onChange={(e) => onIacucNoChange(e.target.value)}
              placeholder={t('animalPages.batchAssign.placeholder')}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
          <Button onClick={onSubmit} disabled={isPending || !iacucNo}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('animalPages.batchAssign.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
