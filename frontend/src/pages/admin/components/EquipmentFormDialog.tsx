/**
 * 設備新增／編輯共用 Dialog
 */
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import type { EquipmentForm, CalibrationType, CalibrationCycle } from '../types'
import { CALIBRATION_TYPE_LABELS, CALIBRATION_CYCLE_LABELS } from '../types'

interface PartnerOption {
  id: string
  name: string
}

interface EquipmentFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: EquipmentForm
  onFormChange: (form: EquipmentForm) => void
  onSubmit: () => void
  isPending: boolean
  partnerOptions: PartnerOption[]
  selectedPartnerIds: string[]
  onPartnerIdsChange: (ids: string[]) => void
}

const calTypeOptions: CalibrationType[] = ['calibration', 'validation']
const cycleOptions: CalibrationCycle[] = ['monthly', 'quarterly', 'semi_annual', 'annual']

export function EquipmentFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  isPending,
  partnerOptions,
  selectedPartnerIds,
  onPartnerIdsChange,
}: EquipmentFormDialogProps) {
  const { t } = useTranslation()
  const isCreate = mode === 'create'

  const togglePartner = (partnerId: string) => {
    if (selectedPartnerIds.includes(partnerId)) {
      onPartnerIdsChange(selectedPartnerIds.filter((id) => id !== partnerId))
    } else {
      onPartnerIdsChange([...selectedPartnerIds, partnerId])
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isCreate
              ? t('admin.equipmentFormDialog.createTitle')
              : t('admin.equipmentFormDialog.editTitle')}
          </DialogTitle>
          {isCreate && (
            <DialogDescription>
              {t('admin.equipmentFormDialog.createDescription')}
            </DialogDescription>
          )}
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div>
            <Label>{t('admin.equipmentFormDialog.nameLabel')}</Label>
            <Input
              value={form.name}
              onChange={(e) => onFormChange({ ...form, name: e.target.value })}
              placeholder={
                isCreate ? t('admin.equipmentFormDialog.namePlaceholder') : undefined
              }
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>{t('admin.equipmentFormDialog.modelLabel')}</Label>
              <Input
                value={form.model}
                onChange={(e) => onFormChange({ ...form, model: e.target.value })}
              />
            </div>
            <div>
              <Label>{t('admin.equipmentFormDialog.serialNumberLabel')}</Label>
              <Input
                value={form.serial_number}
                onChange={(e) => onFormChange({ ...form, serial_number: e.target.value })}
              />
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>{t('admin.equipmentFormDialog.locationLabel')}</Label>
              <Input
                value={form.location}
                onChange={(e) => onFormChange({ ...form, location: e.target.value })}
              />
            </div>
            <div>
              <Label>{t('admin.equipmentFormDialog.departmentLabel')}</Label>
              <Input
                value={form.department}
                onChange={(e) => onFormChange({ ...form, department: e.target.value })}
                placeholder={t('admin.equipmentFormDialog.departmentPlaceholder')}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>{t('admin.equipmentFormDialog.purchaseDateLabel')}</Label>
              <Input
                type="date"
                value={form.purchase_date}
                onChange={(e) => onFormChange({ ...form, purchase_date: e.target.value })}
              />
            </div>
            <div>
              <Label>{t('admin.equipmentFormDialog.warrantyExpiryLabel')}</Label>
              <Input
                type="date"
                value={form.warranty_expiry}
                onChange={(e) => onFormChange({ ...form, warranty_expiry: e.target.value })}
              />
            </div>
          </div>

          {/* 廠商選擇 */}
          <div className="border-t pt-4 space-y-2">
            <Label>{t('admin.equipmentFormDialog.vendorLabel')}</Label>
            {partnerOptions.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t('admin.equipmentFormDialog.noVendorData')}
              </p>
            ) : (
              <div
                className="max-h-32 overflow-y-auto rounded-md border p-2 space-y-1"
                role="group"
                aria-label={t('admin.equipmentFormDialog.vendorLabel')}
              >
                {partnerOptions.map((p) => (
                  <label
                    key={p.id}
                    className="flex items-center gap-2 px-1 py-0.5 rounded hover:bg-muted/50 cursor-pointer text-sm"
                  >
                    <input
                      type="checkbox"
                      checked={selectedPartnerIds.includes(p.id)}
                      onChange={() => togglePartner(p.id)}
                      className="rounded border-border"
                    />
                    {p.name}
                  </label>
                ))}
              </div>
            )}
          </div>

          {/* 校正/確效設定 */}
          <div className="border-t pt-4 space-y-4">
            <p className="text-sm font-medium text-muted-foreground">
              {t('admin.equipmentFormDialog.calibrationSectionTitle')}
            </p>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>{t('admin.equipmentFormDialog.calibrationTypeLabel')}</Label>
                <Select
                  value={form.calibration_type || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      calibration_type: v === '_none' ? '' : (v as CalibrationType),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('admin.equipmentFormDialog.notApplicable')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">
                      {t('admin.equipmentFormDialog.notApplicable')}
                    </SelectItem>
                    {calTypeOptions.map((ct) => (
                      <SelectItem key={ct} value={ct}>
                        {t(CALIBRATION_TYPE_LABELS[ct])}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>{t('admin.equipmentFormDialog.calibrationCycleLabel')}</Label>
                <Select
                  value={form.calibration_cycle || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      calibration_cycle: v === '_none' ? '' : (v as CalibrationCycle),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('admin.equipmentFormDialog.notApplicable')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">
                      {t('admin.equipmentFormDialog.notApplicable')}
                    </SelectItem>
                    {cycleOptions.map((c) => (
                      <SelectItem key={c} value={c}>
                        {t(CALIBRATION_CYCLE_LABELS[c])}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>{t('admin.equipmentFormDialog.inspectionCycleLabel')}</Label>
                <Select
                  value={form.inspection_cycle || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      inspection_cycle: v === '_none' ? '' : (v as CalibrationCycle),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('admin.equipmentFormDialog.notApplicable')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">
                      {t('admin.equipmentFormDialog.notApplicable')}
                    </SelectItem>
                    {cycleOptions.map((c) => (
                      <SelectItem key={c} value={c}>
                        {t(CALIBRATION_CYCLE_LABELS[c])}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>

          <div>
            <Label>{t('admin.equipmentFormDialog.notesLabel')}</Label>
            <Input
              value={form.notes}
              onChange={(e) => onFormChange({ ...form, notes: e.target.value })}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {isCreate ? t('common.create') : t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
