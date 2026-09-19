import { type UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2 } from 'lucide-react'
import { PanelIcon } from '@/components/ui/panel-icon'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type BloodTestPanelFormData = {
  key: string
  name: string
  icon: string
  sort_order: number
}

interface BloodTestPanelFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  form: UseFormReturn<BloodTestPanelFormData>
  isPending: boolean
  onSubmit: (e?: React.BaseSyntheticEvent) => Promise<void>
}

export function BloodTestPanelFormDialog({
  open,
  onOpenChange,
  form,
  isPending,
  onSubmit,
}: BloodTestPanelFormDialogProps) {
  const { t } = useTranslation()
  const { register, watch, formState: { errors } } = form
  const iconValue = watch('icon')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t('erpMaster.bloodTest.panelForm.title')}</DialogTitle>
          <DialogDescription>
            {t('erpMaster.bloodTest.panelForm.description')}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_key" className="text-right">
                {t('erpMaster.common.code')} <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="panel_key"
                  {...register('key', {
                    required: 'erpMaster.bloodTest.validation.codeRequired',
                    onChange: (e) => {
                      e.target.value = e.target.value.toUpperCase()
                    },
                  })}
                  className="font-mono"
                  placeholder={t('erpMaster.bloodTest.panelForm.codePlaceholder')}
                  maxLength={20}
                />
                {errors.key && (
                  <p className="text-sm text-destructive">{t(errors.key.message ?? 'validation.required')}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_name" className="text-right">
                {t('erpMaster.common.name')} <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="panel_name"
                  {...register('name', { required: 'erpMaster.bloodTest.validation.nameRequired' })}
                  placeholder={t('erpMaster.bloodTest.panelForm.namePlaceholder')}
                  maxLength={100}
                />
                {errors.name && (
                  <p className="text-sm text-destructive">{t(errors.name.message ?? 'validation.required')}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_icon" className="text-right">
                {t('erpMaster.common.icon')}
              </Label>
              <div className="col-span-3 flex items-center gap-2">
                <Input
                  id="panel_icon"
                  {...register('icon')}
                  placeholder={t('erpMaster.bloodTest.panelForm.iconPlaceholder')}
                  maxLength={200}
                />
                {iconValue && (
                  <PanelIcon icon={iconValue} size={24} />
                )}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('erpMaster.common.createSubmit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
