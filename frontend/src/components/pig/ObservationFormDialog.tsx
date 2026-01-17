import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { PigObservation, RecordType } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import { Repeater } from '@/components/ui/repeater'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { Loader2 } from 'lucide-react'

// 使用儀器選項
const EQUIPMENT_OPTIONS = [
  { value: 'c-arm', label: 'C-arm' },
  { value: 'ultrasound', label: '超音波' },
  { value: 'ct', label: 'CT' },
  { value: 'mri', label: 'MRI' },
  { value: 'xray', label: 'X光' },
  { value: 'other', label: '其他' },
]

// 紀錄性質選項
const RECORD_TYPE_OPTIONS: { value: RecordType; label: string }[] = [
  { value: 'abnormal', label: '異常紀錄' },
  { value: 'experiment', label: '試驗紀錄' },
  { value: 'observation', label: '觀察紀錄' },
]

// 治療藥物項目
interface TreatmentItem {
  drug: string
  dosage: string
  end_date?: string
}

// 表單狀態
interface ObservationFormData {
  event_date: string
  record_type: RecordType
  equipment_used: string[]
  anesthesia_start: string
  anesthesia_end: string
  content: string
  no_medication_needed: boolean
  treatments: TreatmentItem[]
  remark: string
  photos: FileInfo[]
  attachments: FileInfo[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  pigId: number
  earTag: string
  observation?: PigObservation // 編輯時傳入
}

export function ObservationFormDialog({ open, onOpenChange, pigId, earTag, observation }: Props) {
  const queryClient = useQueryClient()
  const isEdit = !!observation

  const defaultFormData: ObservationFormData = {
    event_date: new Date().toISOString().split('T')[0],
    record_type: 'observation',
    equipment_used: [],
    anesthesia_start: '',
    anesthesia_end: '',
    content: '',
    no_medication_needed: false,
    treatments: [],
    remark: '',
    photos: [],
    attachments: [],
  }

  const [formData, setFormData] = useState<ObservationFormData>(defaultFormData)

  // 將 ISO 8601 日期時間轉換為 datetime-local 格式 (YYYY-MM-DDTHH:mm)
  const isoToDateTimeLocal = (isoString: string | undefined): string => {
    if (!isoString) return ''
    try {
      const date = new Date(isoString)
      if (isNaN(date.getTime())) return ''
      // 轉換為本地時間的 YYYY-MM-DDTHH:mm 格式
      const year = date.getFullYear()
      const month = String(date.getMonth() + 1).padStart(2, '0')
      const day = String(date.getDate()).padStart(2, '0')
      const hours = String(date.getHours()).padStart(2, '0')
      const minutes = String(date.getMinutes()).padStart(2, '0')
      return `${year}-${month}-${day}T${hours}:${minutes}`
    } catch {
      return ''
    }
  }

  // 將 datetime-local 格式轉換為 ISO 8601 格式
  const dateTimeLocalToISO = (datetimeLocal: string | null | undefined): string | null => {
    if (!datetimeLocal || datetimeLocal.trim() === '') return null
    try {
      // datetime-local 格式: YYYY-MM-DDTHH:mm
      // 轉換為 ISO 8601 格式: YYYY-MM-DDTHH:mm:ssZ (UTC)
      const date = new Date(datetimeLocal)
      if (isNaN(date.getTime())) return null
      return date.toISOString()
    } catch {
      return null
    }
  }

  // 編輯時填入資料
  useEffect(() => {
    if (observation) {
      setFormData({
        event_date: observation.event_date.split('T')[0],
        record_type: observation.record_type,
        equipment_used: observation.equipment_used || [],
        anesthesia_start: isoToDateTimeLocal(observation.anesthesia_start),
        anesthesia_end: isoToDateTimeLocal(observation.anesthesia_end),
        content: observation.content,
        no_medication_needed: observation.no_medication_needed,
        treatments: observation.treatments || [],
        remark: observation.remark || '',
        photos: [],
        attachments: [],
      })
    } else {
      setFormData(defaultFormData)
    }
  }, [observation, open])

  const handleEquipmentChange = (value: string, checked: boolean) => {
    if (checked) {
      setFormData((prev) => ({
        ...prev,
        equipment_used: [...prev.equipment_used, value],
      }))
    } else {
      setFormData((prev) => ({
        ...prev,
        equipment_used: prev.equipment_used.filter((e) => e !== value),
      }))
    }
  }

  const mutation = useMutation({
    mutationFn: async (data: ObservationFormData) => {
      const payload = {
        event_date: data.event_date,
        record_type: data.record_type,
        equipment_used: data.equipment_used.length > 0 ? data.equipment_used : null,
        anesthesia_start: dateTimeLocalToISO(data.anesthesia_start),
        anesthesia_end: dateTimeLocalToISO(data.anesthesia_end),
        content: data.content,
        no_medication_needed: data.no_medication_needed,
        treatments: data.treatments.length > 0 ? data.treatments : null,
        remark: data.remark || null,
        // TODO: Handle file uploads
      }

      if (isEdit) {
        return api.put(`/observations/${observation.id}`, payload)
      }
      return api.post(`/pigs/${pigId}/observations`, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-observations', pigId] })
      toast({ title: '成功', description: isEdit ? '觀察紀錄已更新' : '觀察紀錄已新增' })
      onOpenChange(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '儲存失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.content.trim()) {
      toast({ title: '錯誤', description: '請填寫內容', variant: 'destructive' })
      return
    }
    mutation.mutate(formData)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEdit ? '編輯觀察試驗紀錄' : '新增觀察試驗紀錄'}</DialogTitle>
          <DialogDescription>耳號：{earTag}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* 基本資訊 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="event_date">事件發生日期 *</Label>
              <Input
                id="event_date"
                type="date"
                value={formData.event_date}
                onChange={(e) => setFormData({ ...formData, event_date: e.target.value })}
                required
              />
            </div>
            <div className="space-y-2">
              <Label>紀錄性質 *</Label>
              <div className="flex gap-4 pt-2">
                {RECORD_TYPE_OPTIONS.map((option) => (
                  <label key={option.value} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="record_type"
                      value={option.value}
                      checked={formData.record_type === option.value}
                      onChange={() => setFormData({ ...formData, record_type: option.value })}
                      className="w-4 h-4 text-purple-600"
                    />
                    <span className="text-sm">{option.label}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>

          {/* 使用儀器 */}
          <div className="space-y-2">
            <Label>使用儀器</Label>
            <div className="flex flex-wrap gap-4">
              {EQUIPMENT_OPTIONS.map((option) => (
                <Checkbox
                  key={option.value}
                  label={option.label}
                  checked={formData.equipment_used.includes(option.value)}
                  onCheckedChange={(checked) => handleEquipmentChange(option.value, checked)}
                />
              ))}
            </div>
          </div>

          {/* 麻醉時間 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="anesthesia_start">麻醉開始時間</Label>
              <Input
                id="anesthesia_start"
                type="datetime-local"
                value={formData.anesthesia_start}
                onChange={(e) => setFormData({ ...formData, anesthesia_start: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="anesthesia_end">麻醉結束時間</Label>
              <Input
                id="anesthesia_end"
                type="datetime-local"
                value={formData.anesthesia_end}
                onChange={(e) => setFormData({ ...formData, anesthesia_end: e.target.value })}
              />
            </div>
          </div>

          {/* 內容 */}
          <div className="space-y-2">
            <Label htmlFor="content">內容 *</Label>
            <Textarea
              id="content"
              value={formData.content}
              onChange={(e) => setFormData({ ...formData, content: e.target.value })}
              placeholder="詳細描述觀察或試驗內容..."
              className="min-h-[120px]"
              required
            />
          </div>

          {/* 停止用藥 */}
          <Checkbox
            label="不需用藥/停止用藥"
            checked={formData.no_medication_needed}
            onCheckedChange={(checked) => setFormData({ ...formData, no_medication_needed: checked })}
          />

          {/* 治療方式 (Repeater) */}
          {!formData.no_medication_needed && (
            <div className="space-y-2">
              <Label>治療方式</Label>
              <Repeater<TreatmentItem>
                value={formData.treatments}
                onChange={(treatments) => setFormData({ ...formData, treatments })}
                defaultItem={() => ({ drug: '', dosage: '', end_date: '' })}
                addLabel="新增用藥"
                renderItem={(item, _index, onChange) => (
                  <div className="grid grid-cols-3 gap-2">
                    <Input
                      placeholder="預計用藥/藥品名稱"
                      value={item.drug}
                      onChange={(e) => onChange({ ...item, drug: e.target.value })}
                    />
                    <Input
                      placeholder="預計劑量"
                      value={item.dosage}
                      onChange={(e) => onChange({ ...item, dosage: e.target.value })}
                    />
                    <Input
                      type="date"
                      placeholder="預計最後用藥日期"
                      value={item.end_date}
                      onChange={(e) => onChange({ ...item, end_date: e.target.value })}
                    />
                  </div>
                )}
              />
            </div>
          )}

          {/* 備註 */}
          <div className="space-y-2">
            <Label htmlFor="remark">備註</Label>
            <Textarea
              id="remark"
              value={formData.remark}
              onChange={(e) => setFormData({ ...formData, remark: e.target.value })}
              placeholder="其他備註..."
            />
          </div>

          {/* 相片上傳 */}
          <div className="space-y-2">
            <Label>相片</Label>
            <FileUpload
              value={formData.photos}
              onChange={(photos) => setFormData({ ...formData, photos })}
              accept="image/*"
              placeholder="拖曳相片到此處，或點擊選擇相片"
              maxSize={10}
              maxFiles={10}
            />
          </div>

          {/* 附件上傳 */}
          <div className="space-y-2">
            <Label>附件</Label>
            <FileUpload
              value={formData.attachments}
              onChange={(attachments) => setFormData({ ...formData, attachments })}
              accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.zip,.rar"
              placeholder="拖曳附件到此處，或點擊選擇檔案"
              maxSize={20}
              maxFiles={10}
              showPreview={false}
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              type="submit"
              disabled={mutation.isPending}
              className="bg-green-600 hover:bg-green-700"
            >
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
