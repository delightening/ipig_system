import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { CollapsibleSection } from './SurgeryFormComponents'
import type { SurgeryFormData } from './useSurgeryForm'

interface Props {
  formData: SurgeryFormData
  onChange: (data: SurgeryFormData) => void
}

export function SurgeryBasicInfoSection({ formData, onChange }: Props) {
  return (
    <CollapsibleSection title="基本資訊">
      <div className="grid grid-cols-3 gap-4">
        <div className="space-y-2">
          <Label>是否為第一次實驗 *</Label>
          <div className="flex gap-4 pt-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="is_first"
                checked={formData.is_first_experiment}
                onChange={() => onChange({ ...formData, is_first_experiment: true })}
                className="w-4 h-4 text-purple-600"
              />
              <span className="text-sm">是</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="is_first"
                checked={!formData.is_first_experiment}
                onChange={() => onChange({ ...formData, is_first_experiment: false })}
                className="w-4 h-4 text-purple-600"
              />
              <span className="text-sm">否</span>
            </label>
          </div>
        </div>
        <div className="space-y-2">
          <Label htmlFor="surgery_date">手術日期 *</Label>
          <Input
            id="surgery_date"
            type="date"
            value={formData.surgery_date}
            onChange={(e) => onChange({ ...formData, surgery_date: e.target.value })}
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="surgery_site">手術部位 *</Label>
          <Input
            id="surgery_site"
            value={formData.surgery_site}
            onChange={(e) => onChange({ ...formData, surgery_site: e.target.value })}
            placeholder="如：雙眼眼底鏡觀察及ERG"
            required
          />
        </div>
      </div>
    </CollapsibleSection>
  )
}
