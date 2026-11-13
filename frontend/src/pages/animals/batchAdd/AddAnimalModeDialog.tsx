/**
 * 「新增動物」的入口選擇：單隻，或批次（批量進豬）。
 *
 * 使用者 2026-11-12 裁定：點「新增動物」時先跳出這個對話框選擇，而不是另外加一顆按鈕。
 */
import { useTranslation } from 'react-i18next'
import { Layers, PawPrint } from 'lucide-react'

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelectSingle: () => void
  onSelectBatch: () => void
}

export function AddAnimalModeDialog({ open, onOpenChange, onSelectSingle, onSelectBatch }: Props) {
  const { t } = useTranslation()

  const option = (
    icon: React.ReactNode,
    title: string,
    description: string,
    onClick: () => void,
  ) => (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-start gap-3 rounded-lg border bg-card p-4 text-left transition-colors hover:bg-muted focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
    >
      <span className="mt-0.5 text-primary">{icon}</span>
      <span>
        <span className="block font-semibold">{title}</span>
        <span className="block text-sm text-muted-foreground">{description}</span>
      </span>
    </button>
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t('animalPages.addMode.title')}</DialogTitle>
          <DialogDescription>{t('animalPages.addMode.description')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-2 py-2">
          {option(
            <PawPrint className="h-5 w-5" />,
            t('animalPages.addMode.single'),
            t('animalPages.addMode.singleDesc'),
            onSelectSingle,
          )}
          {option(
            <Layers className="h-5 w-5" />,
            t('animalPages.addMode.batch'),
            t('animalPages.addMode.batchDesc'),
            onSelectBatch,
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
