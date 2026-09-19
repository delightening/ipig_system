import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { CollapsibleSection } from './SurgeryFormComponents'
import type { SurgeryFormData } from './useSurgeryForm'

interface Props {
  formData: SurgeryFormData
  onChange: (data: SurgeryFormData) => void
}

export function SurgeryBasicInfoSection({ formData, onChange }: Props) {
  const { t } = useTranslation()
  return (
    <CollapsibleSection title={t('animalRecords.surgeries.basicInfo')}>
      <div className="grid grid-cols-3 gap-4">
        <div className="space-y-2">
          <Label>{t('animalRecords.surgeries.isFirstExperiment')}</Label>
          <div className="flex gap-4 pt-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="is_first"
                checked={formData.is_first_experiment}
                onChange={() => onChange({ ...formData, is_first_experiment: true })}
                className="w-4 h-4 text-status-purple-text"
              />
              <span className="text-sm">{t('common.yes')}</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="is_first"
                checked={!formData.is_first_experiment}
                onChange={() => onChange({ ...formData, is_first_experiment: false })}
                className="w-4 h-4 text-status-purple-text"
              />
              <span className="text-sm">{t('common.no')}</span>
            </label>
          </div>
        </div>
        <div className="space-y-2">
          <Label htmlFor="surgery_date">{t('animalRecords.surgeries.surgeryDateRequired')}</Label>
          <Input
            id="surgery_date"
            type="date"
            value={formData.surgery_date}
            onChange={(e) => onChange({ ...formData, surgery_date: e.target.value })}
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="surgery_site">{t('animalRecords.surgeries.surgerySiteRequired')}</Label>
          <Input
            id="surgery_site"
            value={formData.surgery_site}
            onChange={(e) => onChange({ ...formData, surgery_site: e.target.value })}
            placeholder={t('animalRecords.surgeries.surgerySitePlaceholder')}
            required
          />
        </div>
      </div>
    </CollapsibleSection>
  )
}
