import { useTranslation } from 'react-i18next'

import { AnimalObservation, RecordType } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload } from '@/components/ui/file-upload'
import { Repeater } from '@/components/ui/repeater'
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, FastForward } from 'lucide-react'
import { DrugCombobox } from '@/components/animal/DrugCombobox'
import { useObservationForm, type TreatmentItem } from './hooks/useObservationForm'
import { ObservationPainSection } from './ObservationPainSection'
import { TREATMENT_CATEGORY_OPTIONS, TREATMENT_ROUTE_OPTIONS } from './treatmentConstants'

const TREATMENT_SELECT_CLASS =
  'h-9 rounded-md border border-border bg-white px-2 text-sm focus:border-status-info-solid focus:outline-hidden'

// 模組級選項只存 i18n 鍵（渲染時才 t()）；C-arm / CT / MRI 為專有縮寫，label 直接顯示原文
const EQUIPMENT_OPTIONS: { value: string; label?: string; labelKey?: string }[] = [
  { value: 'c-arm', label: 'C-arm' },
  { value: 'ultrasound', labelKey: 'animalRecords.observations.equipment.ultrasound' },
  { value: 'ct', label: 'CT' },
  { value: 'mri', label: 'MRI' },
  { value: 'xray', labelKey: 'animalRecords.observations.equipment.xray' },
  { value: 'other', labelKey: 'animalRecords.observations.equipment.other' },
]

const RECORD_TYPE_OPTIONS: { value: RecordType; labelKey: string }[] = [
  { value: 'abnormal', labelKey: 'animalRecords.observations.recordType.abnormal' },
  { value: 'experiment', labelKey: 'animalRecords.observations.recordType.experiment' },
  { value: 'observation', labelKey: 'animalRecords.observations.recordType.observation' },
]

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  observation?: AnimalObservation
}

export function ObservationFormDialog({ open, onOpenChange, animalId, earTag, observation }: Props) {
  const { t } = useTranslation()
  const {
    formData, setFormData, isEdit, mutation,
    handleEquipmentChange, handlePhotoUpload, handleSubmit, jumpToNextEmptyField,
  } = useObservationForm({ open, animalId, observation, onOpenChange })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div>
              <DialogTitle>{isEdit ? t('animalRecords.observations.editTitle') : t('animalRecords.observations.createTitle')}</DialogTitle>
              <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
            </div>
            <Button type="button" variant="outline" size="sm" onClick={jumpToNextEmptyField}
              className="flex items-center gap-2 border-status-purple-border text-status-purple-text hover:bg-status-purple-bg" title={t('animalRecords.shared.shortcutAltN')}>
              <FastForward className="h-4 w-4" />
              {t('animalRecords.shared.jumpNextEmpty')}
            </Button>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="event_date">{t('animalRecords.observations.eventDateRequired')}</Label>
              <Input id="event_date" type="date" value={formData.event_date}
                onChange={(e) => setFormData({ ...formData, event_date: e.target.value })} required />
            </div>
            <div className="space-y-2">
              <Label>{t('animalRecords.observations.recordNatureRequired')}</Label>
              <div className="flex gap-4 pt-2">
                {RECORD_TYPE_OPTIONS.map((option) => (
                  <label key={option.value} className="flex items-center gap-2 cursor-pointer">
                    <input type="radio" name="record_type" value={option.value}
                      checked={formData.record_type === option.value}
                      onChange={() => setFormData({ ...formData, record_type: option.value })}
                      className="w-4 h-4 text-status-purple-text" />
                    <span className="text-sm">{t(option.labelKey)}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.observations.equipmentUsed')}</Label>
            <div className="flex flex-wrap gap-4">
              {EQUIPMENT_OPTIONS.map((option) => (
                <Checkbox key={option.value} label={option.labelKey ? t(option.labelKey) : (option.label ?? option.value)}
                  checked={formData.equipment_used.includes(option.value)}
                  onCheckedChange={(checked) => handleEquipmentChange(option.value, checked)} />
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="anesthesia_start">{t('animalRecords.observations.anesthesiaStart')}</Label>
              <Input id="anesthesia_start" type="datetime-local" value={formData.anesthesia_start}
                onChange={(e) => setFormData({ ...formData, anesthesia_start: e.target.value })} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="anesthesia_end">{t('animalRecords.observations.anesthesiaEnd')}</Label>
              <Input id="anesthesia_end" type="datetime-local" value={formData.anesthesia_end}
                onChange={(e) => setFormData({ ...formData, anesthesia_end: e.target.value })} />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="content">{t('animalRecords.observations.contentRequired')}</Label>
            <Textarea id="content" value={formData.content}
              onChange={(e) => setFormData({ ...formData, content: e.target.value })}
              placeholder={t('animalRecords.observations.contentPlaceholder')} className="min-h-[120px]" required />
          </div>

          <Checkbox label={t('animalRecords.shared.noMedicationNeeded')} checked={formData.no_medication_needed}
            onCheckedChange={(checked) => setFormData({ ...formData, no_medication_needed: checked })} />

          {!formData.no_medication_needed && (
            <div className="space-y-2">
              <Label>{t('animalRecords.observations.treatments')}</Label>
              <Repeater<TreatmentItem>
                value={formData.treatments}
                onChange={(treatments) => setFormData({ ...formData, treatments })}
                defaultItem={() => ({ drug: '', dosage: '', end_date: '', drug_option_id: undefined, dosage_unit: '', category: '', route: '' })}
                addLabel={t('animalRecords.observations.addTreatment')}
                renderItem={(item, _index, onChange) => (
                  <div className="space-y-2">
                    <div className="grid grid-cols-2 gap-2">
                      <select aria-label={t('animalRecords.treatment.categoryLabel')} value={item.category || ''} className={TREATMENT_SELECT_CLASS}
                        onChange={(e) => onChange({
                          ...item, category: e.target.value,
                          // 換類別時清掉已選藥物，避免殘留上一個類別選的藥物與新類別不一致
                          drug: '', drug_option_id: undefined, dosage: '', dosage_unit: '',
                        })}>
                        <option value="">{t('animalRecords.treatment.categoryLabel')}</option>
                        {TREATMENT_CATEGORY_OPTIONS.map((o) => (<option key={o.value} value={o.value}>{t(o.labelKey)}</option>))}
                      </select>
                      <select aria-label={t('animalRecords.treatment.routeLabel')} value={item.route || ''} className={TREATMENT_SELECT_CLASS}
                        onChange={(e) => onChange({ ...item, route: e.target.value })}>
                        <option value="">{t('animalRecords.treatment.routeLabel')}</option>
                        {TREATMENT_ROUTE_OPTIONS.map((o) => (<option key={o.value} value={o.value}>{t(o.labelKey)}</option>))}
                      </select>
                    </div>
                    <DrugCombobox
                      categoryFilter={item.category || undefined}
                      value={{ drug_option_id: item.drug_option_id, drug_name: item.drug, dosage_value: item.dosage, dosage_unit: item.dosage_unit || '' }}
                      onChange={(sel) => onChange({ ...item, drug: sel.drug_name, dosage: sel.dosage_value, drug_option_id: sel.drug_option_id, dosage_unit: sel.dosage_unit })}
                    />
                    <Input type="date" placeholder={t('animalRecords.observations.endDatePlaceholder')} value={item.end_date}
                      onChange={(e) => onChange({ ...item, end_date: e.target.value })} />
                  </div>
                )}
              />
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="remark">{t('animalRecords.shared.remark')}</Label>
            <Textarea id="remark" value={formData.remark}
              onChange={(e) => setFormData({ ...formData, remark: e.target.value })} placeholder={t('animalRecords.shared.otherRemarkPlaceholder')} />
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.shared.photos')}</Label>
            <FileUpload value={formData.photos} onChange={(photos) => setFormData({ ...formData, photos })}
              onUpload={handlePhotoUpload} accept="image/*,.pdf,.doc,.docx"
              placeholder={t('animalRecords.shared.photoPlaceholder')} maxSize={20} maxFiles={10} />
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.shared.attachments')}</Label>
            <FileUpload value={formData.attachments} onChange={(attachments) => setFormData({ ...formData, attachments })}
              onUpload={handlePhotoUpload} accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx"
              placeholder={t('animalRecords.shared.attachmentPlaceholder')} maxSize={20} maxFiles={10} showPreview={false} />
          </div>

          {/* 疼痛評估（可收合） */}
          <ObservationPainSection
            observationId={observation?.id != null ? String(observation.id) : undefined}
            entries={formData.painAssessments}
            onChange={(painAssessments) => setFormData({ ...formData, painAssessments })}
          />

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
            <Button type="submit" disabled={mutation.isPending} className="bg-status-success-solid hover:bg-status-success-solid/90">
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
