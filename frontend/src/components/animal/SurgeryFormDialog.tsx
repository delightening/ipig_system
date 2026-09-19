import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, FastForward } from 'lucide-react'
import { useSurgeryForm } from './useSurgeryForm'
import { SurgeryBasicInfoSection } from './SurgeryBasicInfoSection'
import { SurgeryAnesthesiaSection } from './SurgeryAnesthesiaSection'
import { SurgeryProcedureSection } from './SurgeryProcedureSection'
import type { AnimalSurgery } from '@/lib/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  surgery?: AnimalSurgery
}

export function SurgeryFormDialog({
  open,
  onOpenChange,
  animalId,
  earTag,
  surgery,
}: Props) {
  const { t } = useTranslation()
  const isEdit = !!surgery
  const { formData, setFormData, mutation, jumpToNextEmptyField } = useSurgeryForm({
    open,
    onOpenChange,
    animalId,
    surgery,
  })

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key === 'n') {
        e.preventDefault()
        jumpToNextEmptyField()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [jumpToNextEmptyField])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.surgery_site.trim()) {
      toast({ title: t('common.error'), description: t('animalRecords.surgeries.surgerySiteRequiredError'), variant: 'destructive' })
      return
    }
    mutation.mutate(formData)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="2xl">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div>
              <DialogTitle>
                {isEdit ? t('animalRecords.surgeries.editTitle') : t('animalRecords.surgeries.createTitle')}
              </DialogTitle>
              <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
            </div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={jumpToNextEmptyField}
              className="flex items-center gap-2 border-status-purple-border text-status-purple-text hover:bg-status-purple-bg mr-4"
              title={t('animalRecords.shared.shortcutAltN')}
            >
              <FastForward className="h-4 w-4" />
              {t('animalRecords.shared.jumpNextEmpty')}
            </Button>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <SurgeryBasicInfoSection formData={formData} onChange={setFormData} />
          <SurgeryAnesthesiaSection formData={formData} onChange={setFormData} />
          <SurgeryProcedureSection formData={formData} onChange={setFormData} surgeryId={surgery?.id != null ? String(surgery.id) : undefined} />

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              type="submit"
              disabled={mutation.isPending}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
