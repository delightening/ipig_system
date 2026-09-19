import { useTranslation } from 'react-i18next'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload } from '@/components/ui/file-upload'
import { Label } from '@/components/ui/label'
import { CollapsibleSection } from './SurgeryFormComponents'
import { SurgeryPainSection } from './SurgeryPainSection'
import type { SurgeryFormData } from './useSurgeryForm'

interface Props {
  formData: SurgeryFormData
  onChange: (data: SurgeryFormData) => void
  surgeryId?: string
}

export function SurgeryProcedureSection({ formData, onChange, surgeryId }: Props) {
  const { t } = useTranslation()
  return (
    <>
      {/* 疼痛評估區塊 */}
      <CollapsibleSection title={t('animalDetail.tabs.painAssessment')}>
        <div className="space-y-4">
          <SurgeryPainSection
            surgeryId={surgeryId}
            entries={formData.painAssessments}
            onChange={(painAssessments) => onChange({ ...formData, painAssessments })}
          />

          <div className="pt-2">
            <Label className="text-sm text-muted-foreground block mb-1">{t('animalRecords.shared.noMedicationNeeded')}</Label>
            <Checkbox
              label={t('animalRecords.shared.noMedicationNeeded')}
              checked={formData.no_medication_needed}
              onCheckedChange={(checked) =>
                onChange({ ...formData, no_medication_needed: checked === true })
              }
            />
          </div>
        </div>
      </CollapsibleSection>

      {/* 檔案上傳 */}
      <CollapsibleSection title={t('animalRecords.surgeries.fileUpload')} defaultOpen={false}>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t('animalRecords.shared.photos')}</Label>
            <FileUpload
              value={formData.photos}
              onChange={(photos) => onChange({ ...formData, photos })}
              accept="image/*"
              placeholder={t('animalRecords.shared.photoPlaceholder')}
              maxSize={10}
              maxFiles={10}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.shared.attachments')}</Label>
            <FileUpload
              value={formData.attachments}
              onChange={(attachments) => onChange({ ...formData, attachments })}
              accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.zip,.rar"
              placeholder={t('animalRecords.shared.attachmentPlaceholder')}
              maxSize={20}
              maxFiles={10}
              showPreview={false}
            />
          </div>
        </div>
      </CollapsibleSection>
    </>
  )
}
