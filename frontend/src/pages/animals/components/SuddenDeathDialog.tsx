import React from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, Zap } from 'lucide-react'

import type { SuddenDeathFormData } from '../hooks/useAnimalDetailMutations'

interface SuddenDeathDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  earTag: string
  form: SuddenDeathFormData
  onFormChange: React.Dispatch<React.SetStateAction<SuddenDeathFormData>>
  isPending: boolean
  onConfirm: () => void
}

export function SuddenDeathDialog({
  open,
  onOpenChange,
  earTag,
  form,
  onFormChange,
  isPending,
  onConfirm,
}: SuddenDeathDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-destructive">
            <Zap className="h-5 w-5" />
            {t('animalPages.suddenDeath.dialogTitle', { earTag })}
          </DialogTitle>
          <DialogDescription>
            {t('animalPages.suddenDeath.description')}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="sd-discovered-at">{t('animalPages.suddenDeath.discoveredAt')} *</Label>
            <Input
              id="sd-discovered-at"
              type="datetime-local"
              value={form.discovered_at}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, discovered_at: e.target.value }))
              }
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-location">{t('animalPages.suddenDeath.location')}</Label>
            <Input
              id="sd-location"
              placeholder={t('animalPages.suddenDeath.locationPlaceholder')}
              value={form.location}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, location: e.target.value }))
              }
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-probable-cause">{t('animalPages.suddenDeath.probableCause')}</Label>
            <Textarea
              id="sd-probable-cause"
              placeholder={t('animalPages.suddenDeath.probableCausePlaceholder')}
              value={form.probable_cause}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, probable_cause: e.target.value }))
              }
              className="min-h-[80px]"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-remark">{t('animalPages.shared.remark')}</Label>
            <Textarea
              id="sd-remark"
              placeholder={t('animalPages.shared.otherRemarks')}
              value={form.remark}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, remark: e.target.value }))
              }
              className="min-h-[60px]"
            />
          </div>
          <div className="flex items-center space-x-2">
            <input
              id="sd-requires-pathology"
              type="checkbox"
              aria-label={t('animalPages.suddenDeath.requiresPathology')}
              checked={form.requires_pathology}
              onChange={(e) =>
                onFormChange((prev) => ({
                  ...prev,
                  requires_pathology: e.target.checked,
                }))
              }
              className="h-4 w-4 rounded border-border text-destructive focus:ring-destructive"
            />
            <Label
              htmlFor="sd-requires-pathology"
              className="text-sm font-normal cursor-pointer"
            >
              {t('animalPages.suddenDeath.requiresPathology')}
            </Label>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            className="bg-destructive hover:bg-destructive/90 text-white"
            disabled={!form.discovered_at || isPending}
            onClick={onConfirm}
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('animalPages.suddenDeath.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
