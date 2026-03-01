import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload } from '@/components/ui/file-upload'
import { Repeater } from '@/components/ui/repeater'
import { DrugCombobox } from '@/components/animal/DrugCombobox'
import { CollapsibleSection } from './SurgeryFormComponents'
import type { SurgeryFormData, MedicationItem } from './useSurgeryForm'

interface Props {
  formData: SurgeryFormData
  onChange: (data: SurgeryFormData) => void
}

export function SurgeryProcedureSection({ formData, onChange }: Props) {
  return (
    <>
      {/* 術後區塊 */}
      <CollapsibleSection title="術後">
        <div className="space-y-4">
          <Checkbox
            label="術後給藥-優點軟膏"
            checked={formData.post_ointment}
            onCheckedChange={(checked) =>
              onChange({ ...formData, post_ointment: checked === true })
            }
          />

          <div className="space-y-2">
            <Label>術後給藥-其他</Label>
            <Repeater<MedicationItem>
              value={formData.post_medications}
              onChange={(post_medications) => onChange({ ...formData, post_medications })}
              defaultItem={() => ({ name: '', dose: '', drug_option_id: undefined, dosage_unit: '' })}
              addLabel="新增術後藥品"
              renderItem={(item, _index, onItemChange) => (
                <div className="grid grid-cols-2 gap-2">
                  <DrugCombobox
                    value={{
                      drug_option_id: item.drug_option_id,
                      drug_name: item.name,
                      dosage_value: item.dose,
                      dosage_unit: item.dosage_unit || '',
                    }}
                    onChange={(sel) =>
                      onItemChange({
                        ...item,
                        name: sel.drug_name,
                        dose: sel.dosage_value,
                        drug_option_id: sel.drug_option_id,
                        dosage_unit: sel.dosage_unit,
                      })
                    }
                  />
                </div>
              )}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="remark">備註</Label>
            <Textarea
              id="remark"
              value={formData.remark}
              onChange={(e) => onChange({ ...formData, remark: e.target.value })}
              placeholder="其他備註..."
            />
          </div>

          <Checkbox
            label="不需用藥/停止用藥"
            checked={formData.no_medication_needed}
            onCheckedChange={(checked) =>
              onChange({ ...formData, no_medication_needed: checked === true })
            }
          />
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
