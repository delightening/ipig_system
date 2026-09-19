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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import { PanelIcon } from '@/components/ui/panel-icon'
import type { BloodTestTemplate, BloodTestPanel } from '@/lib/api'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type BloodTestTemplateFormData = {
  code: string
  name: string
  default_unit: string
  reference_range: string
  default_price: number
  sort_order: number
  panel_id?: string
}

interface BloodTestTemplateFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingTemplate: BloodTestTemplate | null
  form: UseFormReturn<BloodTestTemplateFormData>
  panels: BloodTestPanel[] | undefined
  isCreatePending: boolean
  isUpdatePending: boolean
  onSubmit: (e?: React.BaseSyntheticEvent) => Promise<void>
}

export function BloodTestTemplateFormDialog({
  open,
  onOpenChange,
  editingTemplate,
  form,
  panels,
  isCreatePending,
  isUpdatePending,
  onSubmit,
}: BloodTestTemplateFormDialogProps) {
  const { t } = useTranslation()
  const { register, setValue, watch, formState: { errors } } = form
  const panelIdValue = watch('panel_id')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>
            {editingTemplate ? t('erpMaster.bloodTest.templateForm.editTitle') : t('erpMaster.bloodTest.templateForm.addTitle')}
          </DialogTitle>
          <DialogDescription>
            {editingTemplate
              ? t('erpMaster.bloodTest.templateForm.editDescription', { code: editingTemplate.code })
              : t('erpMaster.bloodTest.templateForm.addDescription')}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="code" className="text-right">
                {t('erpMaster.common.code')} <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="code"
                  {...register('code', {
                    required: !editingTemplate ? 'erpMaster.bloodTest.validation.codeRequired' : false,
                    onChange: (e) => {
                      e.target.value = e.target.value.toUpperCase()
                    },
                  })}
                  className="font-mono"
                  placeholder={t('erpMaster.bloodTest.templateForm.codePlaceholder')}
                  disabled={!!editingTemplate}
                  maxLength={20}
                />
                {errors.code && (
                  <p className="text-sm text-destructive">{t(errors.code.message ?? 'validation.required')}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">
                {t('erpMaster.common.name')} <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="name"
                  {...register('name', { required: 'erpMaster.bloodTest.validation.nameRequired' })}
                  placeholder={t('erpMaster.bloodTest.templateForm.namePlaceholder')}
                  maxLength={200}
                />
                {errors.name && (
                  <p className="text-sm text-destructive">{t(errors.name.message ?? 'validation.required')}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="default_unit" className="text-right">
                {t('erpMaster.bloodTest.templateForm.defaultUnit')}
              </Label>
              <Input
                id="default_unit"
                {...register('default_unit')}
                className="col-span-3"
                placeholder={t('erpMaster.bloodTest.templateForm.unitPlaceholder')}
                maxLength={50}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="reference_range" className="text-right">
                {t('erpMaster.bloodTest.referenceRange')}
              </Label>
              <Input
                id="reference_range"
                {...register('reference_range')}
                className="col-span-3"
                placeholder={t('erpMaster.bloodTest.templateForm.rangePlaceholder')}
                maxLength={100}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="default_price" className="text-right">
                {t('erpMaster.bloodTest.templateForm.defaultPrice')}
              </Label>
              <Input
                id="default_price"
                type="number"
                min="0"
                step="1"
                {...register('default_price', { valueAsNumber: true })}
                className="col-span-3"
                placeholder="0"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_id" className="text-right">
                {t('erpMaster.bloodTest.templateForm.panel')}
              </Label>
              <Select
                value={panelIdValue || 'none'}
                onValueChange={(val) =>
                  setValue('panel_id', val === 'none' ? undefined : val)
                }
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder={t('erpMaster.bloodTest.templateForm.selectPanel')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t('erpMaster.bloodTest.uncategorized')}</SelectItem>
                  {panels?.map((panel) => (
                    <SelectItem key={panel.id} value={panel.id}>
                      <PanelIcon icon={panel.icon} /> {panel.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              type="submit"
              disabled={isCreatePending || isUpdatePending}
            >
              {(isCreatePending || isUpdatePending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {editingTemplate ? t('common.update') : t('erpMaster.common.createSubmit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
