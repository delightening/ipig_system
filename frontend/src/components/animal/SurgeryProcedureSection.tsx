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
  return (
    <>
      {/* 疼痛評估區塊 */}
      <CollapsibleSection title="疼痛評估">
        <div className="space-y-4">
          <SurgeryPainSection
            surgeryId={surgeryId}
            entries={formData.painAssessments}
            onChange={(painAssessments) => onChange({ ...formData, painAssessments })}
          />

          <div className="pt-2">
            <Label className="text-sm text-muted-foreground block mb-1">不需用藥/停止用藥</Label>
            <Checkbox
              label="不需用藥/停止用藥"
              checked={formData.no_medication_needed}
              onCheckedChange={(checked) =>
                onChange({ ...formData, no_medication_needed: checked === true })
              }
            />
          </div>
        </div>
      </CollapsibleSection>

      {/* 檔案上傳 */}
      <CollapsibleSection title="檔案上傳" defaultOpen={false}>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>相片</Label>
            <FileUpload
              value={formData.photos}
              onChange={(photos) => onChange({ ...formData, photos })}
              accept="image/*"
              placeholder="拖曳相片到此處，或點擊選擇相片"
              maxSize={10}
              maxFiles={10}
            />
          </div>

          <div className="space-y-2">
            <Label>附件</Label>
            <FileUpload
              value={formData.attachments}
              onChange={(attachments) => onChange({ ...formData, attachments })}
              accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.zip,.rar"
              placeholder="拖曳附件到此處，或點擊選擇檔案"
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
