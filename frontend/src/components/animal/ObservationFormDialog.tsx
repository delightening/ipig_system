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

const EQUIPMENT_OPTIONS = [
  { value: 'c-arm', label: 'C-arm' },
  { value: 'ultrasound', label: '超音波' },
  { value: 'ct', label: 'CT' },
  { value: 'mri', label: 'MRI' },
  { value: 'xray', label: 'X光' },
  { value: 'other', label: '其他' },
]

const RECORD_TYPE_OPTIONS: { value: RecordType; label: string }[] = [
  { value: 'abnormal', label: '異常紀錄' },
  { value: 'experiment', label: '試驗紀錄' },
  { value: 'observation', label: '觀察紀錄' },
]

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  observation?: AnimalObservation
}

export function ObservationFormDialog({ open, onOpenChange, animalId, earTag, observation }: Props) {
  const {
    formData, setFormData, isEdit, mutation,
    handleEquipmentChange, handlePhotoUpload, handleSubmit, jumpToNextEmptyField,
  } = useObservationForm({ open, animalId, observation, onOpenChange })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div>
              <DialogTitle>{isEdit ? '編輯觀察試驗紀錄' : '新增觀察試驗紀錄'}</DialogTitle>
              <DialogDescription>耳號：{earTag}</DialogDescription>
            </div>
            <Button type="button" variant="outline" size="sm" onClick={jumpToNextEmptyField}
              className="flex items-center gap-2 border-status-purple-border text-status-purple-text hover:bg-status-purple-bg" title="快捷鍵: Alt + N">
              <FastForward className="h-4 w-4" />
              下一個空白欄位
            </Button>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="event_date">事件發生日期 *</Label>
              <Input id="event_date" type="date" value={formData.event_date}
                onChange={(e) => setFormData({ ...formData, event_date: e.target.value })} required />
            </div>
            <div className="space-y-2">
              <Label>紀錄性質 *</Label>
              <div className="flex gap-4 pt-2">
                {RECORD_TYPE_OPTIONS.map((option) => (
                  <label key={option.value} className="flex items-center gap-2 cursor-pointer">
                    <input type="radio" name="record_type" value={option.value}
                      checked={formData.record_type === option.value}
                      onChange={() => setFormData({ ...formData, record_type: option.value })}
                      className="w-4 h-4 text-status-purple-text" />
                    <span className="text-sm">{option.label}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <Label>使用儀器</Label>
            <div className="flex flex-wrap gap-4">
              {EQUIPMENT_OPTIONS.map((option) => (
                <Checkbox key={option.value} label={option.label}
                  checked={formData.equipment_used.includes(option.value)}
                  onCheckedChange={(checked) => handleEquipmentChange(option.value, checked)} />
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="anesthesia_start">麻醉開始時間</Label>
              <Input id="anesthesia_start" type="datetime-local" value={formData.anesthesia_start}
                onChange={(e) => setFormData({ ...formData, anesthesia_start: e.target.value })} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="anesthesia_end">麻醉結束時間</Label>
              <Input id="anesthesia_end" type="datetime-local" value={formData.anesthesia_end}
                onChange={(e) => setFormData({ ...formData, anesthesia_end: e.target.value })} />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="content">內容 *</Label>
            <Textarea id="content" value={formData.content}
              onChange={(e) => setFormData({ ...formData, content: e.target.value })}
              placeholder="詳細描述觀察或試驗內容..." className="min-h-[120px]" required />
          </div>

          <Checkbox label="不需用藥/停止用藥" checked={formData.no_medication_needed}
            onCheckedChange={(checked) => setFormData({ ...formData, no_medication_needed: checked })} />

          {!formData.no_medication_needed && (
            <div className="space-y-2">
              <Label>治療方式</Label>
              <Repeater<TreatmentItem>
                value={formData.treatments}
                onChange={(treatments) => setFormData({ ...formData, treatments })}
                defaultItem={() => ({ drug: '', dosage: '', end_date: '', drug_option_id: undefined, dosage_unit: '' })}
                addLabel="新增用藥"
                renderItem={(item, _index, onChange) => (
                  <div className="space-y-2">
                    <DrugCombobox
                      value={{ drug_option_id: item.drug_option_id, drug_name: item.drug, dosage_value: item.dosage, dosage_unit: item.dosage_unit || '' }}
                      onChange={(sel) => onChange({ ...item, drug: sel.drug_name, dosage: sel.dosage_value, drug_option_id: sel.drug_option_id, dosage_unit: sel.dosage_unit })}
                    />
                    <Input type="date" placeholder="預計最後用藥日期" value={item.end_date}
                      onChange={(e) => onChange({ ...item, end_date: e.target.value })} />
                  </div>
                )}
              />
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="remark">備註</Label>
            <Textarea id="remark" value={formData.remark}
              onChange={(e) => setFormData({ ...formData, remark: e.target.value })} placeholder="其他備註..." />
          </div>

          <div className="space-y-2">
            <Label>相片</Label>
            <FileUpload value={formData.photos} onChange={(photos) => setFormData({ ...formData, photos })}
              onUpload={handlePhotoUpload} accept="image/*,.pdf,.doc,.docx"
              placeholder="拖曳相片到此處，或點擊選擇相片" maxSize={20} maxFiles={10} />
          </div>

          <div className="space-y-2">
            <Label>附件</Label>
            <FileUpload value={formData.attachments} onChange={(attachments) => setFormData({ ...formData, attachments })}
              onUpload={handlePhotoUpload} accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx"
              placeholder="拖曳附件到此處，或點擊選擇檔案" maxSize={20} maxFiles={10} showPreview={false} />
          </div>

          {/* 疼痛評估（可收合） */}
          <ObservationPainSection
            observationId={observation?.id != null ? String(observation.id) : undefined}
            entries={formData.painAssessments}
            onChange={(painAssessments) => setFormData({ ...formData, painAssessments })}
          />

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
            <Button type="submit" disabled={mutation.isPending} className="bg-status-success-solid hover:bg-status-success-solid/90">
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
