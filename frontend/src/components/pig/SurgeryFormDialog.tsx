import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { PigSurgery } from '@/lib/api'
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
import { Loader2, ChevronDown, ChevronUp } from 'lucide-react'

// 誘導麻醉藥物項目
interface AnesthesiaDrug {
  name: string
  dose: string
  enabled: boolean
}

// 生理數值項目
interface VitalSign {
  time: string
  breathing_method: string
  heart_rate: string
  respiration_rate: string
  temperature: string
  spo2: string
}

// 其他藥物項目
interface MedicationItem {
  name: string
  dose: string
}

// 表單狀態
interface SurgeryFormData {
  is_first_experiment: boolean
  surgery_date: string
  surgery_site: string
  // 誘導麻醉
  induction: {
    atropine: AnesthesiaDrug
    stroless: AnesthesiaDrug
    zoletil50: AnesthesiaDrug
    others: MedicationItem[]
  }
  // 術前給藥
  pre_surgery: {
    medications: MedicationItem[]
    others: MedicationItem[]
  }
  // 固定姿勢
  positioning: string
  positioning_others: string[]
  // 麻醉維持
  maintenance: {
    o2: AnesthesiaDrug
    n2o: AnesthesiaDrug
    isoflurane: AnesthesiaDrug
    others: MedicationItem[]
  }
  // 監測與恢復
  anesthesia_observation: string
  vital_signs: VitalSign[]
  reflex_recovery: string
  respiration_rate_auto: string
  // 術後
  post_ointment: boolean
  post_medications: MedicationItem[]
  remark: string
  no_medication_needed: boolean
  photos: FileInfo[]
  attachments: FileInfo[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  pigId: number
  earTag: string
  surgery?: PigSurgery
}

// 可折疊區塊組件
function CollapsibleSection({
  title,
  children,
  defaultOpen = true,
}: {
  title: string
  children: React.ReactNode
  defaultOpen?: boolean
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen)

  return (
    <div className="border rounded-lg">
      <button
        type="button"
        className="flex items-center justify-between w-full px-4 py-3 text-left font-medium bg-slate-50 hover:bg-slate-100 rounded-t-lg"
        onClick={() => setIsOpen(!isOpen)}
      >
        {title}
        {isOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
      </button>
      {isOpen && <div className="p-4 space-y-4">{children}</div>}
    </div>
  )
}

// 藥物勾選輸入組件
function DrugCheckInput({
  label,
  drug,
  onChange,
}: {
  label: string
  drug: AnesthesiaDrug
  onChange: (drug: AnesthesiaDrug) => void
}) {
  return (
    <div className="flex items-center gap-3">
      <Checkbox
        label={label}
        checked={drug.enabled}
        onChange={(e) => onChange({ ...drug, enabled: e.target.checked })}
      />
      {drug.enabled && (
        <Input
          className="w-32"
          placeholder="劑量"
          value={drug.dose}
          onChange={(e) => onChange({ ...drug, dose: e.target.value })}
        />
      )}
    </div>
  )
}

export function SurgeryFormDialog({ open, onOpenChange, pigId, earTag, surgery }: Props) {
  const queryClient = useQueryClient()
  const isEdit = !!surgery

  const defaultFormData: SurgeryFormData = {
    is_first_experiment: true,
    surgery_date: new Date().toISOString().split('T')[0],
    surgery_site: '',
    induction: {
      atropine: { name: 'Atropine', dose: '', enabled: false },
      stroless: { name: 'Stroless', dose: '', enabled: false },
      zoletil50: { name: 'Zoletil-50', dose: '', enabled: false },
      others: [],
    },
    pre_surgery: {
      medications: [],
      others: [],
    },
    positioning: '',
    positioning_others: [],
    maintenance: {
      o2: { name: 'O2', dose: '', enabled: false },
      n2o: { name: 'N2O', dose: '', enabled: false },
      isoflurane: { name: 'Isoflurane', dose: '', enabled: false },
      others: [],
    },
    anesthesia_observation: '',
    vital_signs: [],
    reflex_recovery: '',
    respiration_rate_auto: '',
    post_ointment: false,
    post_medications: [],
    remark: '',
    no_medication_needed: false,
    photos: [],
    attachments: [],
  }

  const [formData, setFormData] = useState<SurgeryFormData>(defaultFormData)

  // 編輯時填入資料
  useEffect(() => {
    if (surgery) {
      const induction = surgery.induction_anesthesia as any || {}
      const maintenance = surgery.anesthesia_maintenance as any || {}
      const preSurgery = surgery.pre_surgery_medication as any || {}
      const postSurgery = surgery.post_surgery_medication as any || {}

      setFormData({
        is_first_experiment: surgery.is_first_experiment,
        surgery_date: surgery.surgery_date.split('T')[0],
        surgery_site: surgery.surgery_site,
        induction: {
          atropine: { name: 'Atropine', dose: induction.atropine || '', enabled: !!induction.atropine },
          stroless: { name: 'Stroless', dose: induction.stroless || '', enabled: !!induction.stroless },
          zoletil50: { name: 'Zoletil-50', dose: induction.zoletil50 || '', enabled: !!induction.zoletil50 },
          others: induction.others || [],
        },
        pre_surgery: {
          medications: preSurgery.medications || [],
          others: preSurgery.others || [],
        },
        positioning: surgery.positioning || '',
        positioning_others: [],
        maintenance: {
          o2: { name: 'O2', dose: maintenance.o2 || '', enabled: !!maintenance.o2 },
          n2o: { name: 'N2O', dose: maintenance.n2o || '', enabled: !!maintenance.n2o },
          isoflurane: { name: 'Isoflurane', dose: maintenance.isoflurane || '', enabled: !!maintenance.isoflurane },
          others: maintenance.others || [],
        },
        anesthesia_observation: surgery.anesthesia_observation || '',
        vital_signs: (surgery.vital_signs || []).map((vs: any) => ({
          time: vs.time || '',
          breathing_method: vs.breathing_method || '',
          heart_rate: String(vs.heart_rate || ''),
          respiration_rate: String(vs.respiration_rate || ''),
          temperature: String(vs.temperature || ''),
          spo2: String(vs.spo2 || ''),
        })),
        reflex_recovery: surgery.reflex_recovery || '',
        respiration_rate_auto: surgery.respiration_rate ? String(surgery.respiration_rate) : '',
        post_ointment: postSurgery.ointment || false,
        post_medications: postSurgery.others || [],
        remark: surgery.remark || '',
        no_medication_needed: surgery.no_medication_needed,
        photos: [],
        attachments: [],
      })
    } else {
      setFormData(defaultFormData)
    }
  }, [surgery, open])

  const mutation = useMutation({
    mutationFn: async (data: SurgeryFormData) => {
      // 組裝 payload
      const inductionAnesthesia: Record<string, any> = {}
      if (data.induction.atropine.enabled) inductionAnesthesia.atropine = data.induction.atropine.dose
      if (data.induction.stroless.enabled) inductionAnesthesia.stroless = data.induction.stroless.dose
      if (data.induction.zoletil50.enabled) inductionAnesthesia.zoletil50 = data.induction.zoletil50.dose
      if (data.induction.others.length > 0) inductionAnesthesia.others = data.induction.others

      const anesthesiaMaintenance: Record<string, any> = {}
      if (data.maintenance.o2.enabled) anesthesiaMaintenance.o2 = data.maintenance.o2.dose
      if (data.maintenance.n2o.enabled) anesthesiaMaintenance.n2o = data.maintenance.n2o.dose
      if (data.maintenance.isoflurane.enabled) anesthesiaMaintenance.isoflurane = data.maintenance.isoflurane.dose
      if (data.maintenance.others.length > 0) anesthesiaMaintenance.others = data.maintenance.others

      const preSurgeryMedication: Record<string, any> = {}
      if (data.pre_surgery.medications.length > 0) preSurgeryMedication.medications = data.pre_surgery.medications
      if (data.pre_surgery.others.length > 0) preSurgeryMedication.others = data.pre_surgery.others

      const postSurgeryMedication: Record<string, any> = {}
      if (data.post_ointment) postSurgeryMedication.ointment = true
      if (data.post_medications.length > 0) postSurgeryMedication.others = data.post_medications

      const payload = {
        is_first_experiment: data.is_first_experiment,
        surgery_date: data.surgery_date,
        surgery_site: data.surgery_site,
        induction_anesthesia: Object.keys(inductionAnesthesia).length > 0 ? inductionAnesthesia : null,
        pre_surgery_medication: Object.keys(preSurgeryMedication).length > 0 ? preSurgeryMedication : null,
        positioning: data.positioning || null,
        anesthesia_maintenance: Object.keys(anesthesiaMaintenance).length > 0 ? anesthesiaMaintenance : null,
        anesthesia_observation: data.anesthesia_observation || null,
        vital_signs:
          data.vital_signs.length > 0
            ? data.vital_signs.map((vs) => ({
                time: vs.time,
                breathing_method: vs.breathing_method,
                heart_rate: parseFloat(vs.heart_rate) || 0,
                respiration_rate: parseFloat(vs.respiration_rate) || 0,
                temperature: parseFloat(vs.temperature) || 0,
                spo2: parseFloat(vs.spo2) || 0,
              }))
            : null,
        reflex_recovery: data.reflex_recovery || null,
        respiration_rate: data.respiration_rate_auto ? parseFloat(data.respiration_rate_auto) : null,
        post_surgery_medication: Object.keys(postSurgeryMedication).length > 0 ? postSurgeryMedication : null,
        remark: data.remark || null,
        no_medication_needed: data.no_medication_needed,
      }

      if (isEdit) {
        return api.put(`/surgeries/${surgery.id}`, payload)
      }
      return api.post(`/pigs/${pigId}/surgeries`, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-surgeries', pigId] })
      toast({ title: '成功', description: isEdit ? '手術紀錄已更新' : '手術紀錄已新增' })
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
    if (!formData.surgery_site.trim()) {
      toast({ title: '錯誤', description: '請填寫手術部位', variant: 'destructive' })
      return
    }
    mutation.mutate(formData)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEdit ? '編輯手術紀錄' : '新增手術紀錄'}</DialogTitle>
          <DialogDescription>耳號：{earTag}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* 基本資訊 */}
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
                      onChange={() => setFormData({ ...formData, is_first_experiment: true })}
                      className="w-4 h-4 text-purple-600"
                    />
                    <span className="text-sm">是</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="is_first"
                      checked={!formData.is_first_experiment}
                      onChange={() => setFormData({ ...formData, is_first_experiment: false })}
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
                  onChange={(e) => setFormData({ ...formData, surgery_date: e.target.value })}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="surgery_site">手術部位 *</Label>
                <Input
                  id="surgery_site"
                  value={formData.surgery_site}
                  onChange={(e) => setFormData({ ...formData, surgery_site: e.target.value })}
                  placeholder="如：雙眼眼底鏡觀察及ERG"
                  required
                />
              </div>
            </div>
          </CollapsibleSection>

          {/* 誘導麻醉 */}
          <CollapsibleSection title="誘導麻醉">
            <div className="space-y-3">
              <DrugCheckInput
                label="Atropine"
                drug={formData.induction.atropine}
                onChange={(drug) =>
                  setFormData({ ...formData, induction: { ...formData.induction, atropine: drug } })
                }
              />
              <DrugCheckInput
                label="Stroless"
                drug={formData.induction.stroless}
                onChange={(drug) =>
                  setFormData({ ...formData, induction: { ...formData.induction, stroless: drug } })
                }
              />
              <DrugCheckInput
                label="Zoletil-50"
                drug={formData.induction.zoletil50}
                onChange={(drug) =>
                  setFormData({ ...formData, induction: { ...formData.induction, zoletil50: drug } })
                }
              />
              <div className="pt-2">
                <Label className="text-sm text-slate-600">其他藥劑</Label>
                <Repeater<MedicationItem>
                  value={formData.induction.others}
                  onChange={(others) =>
                    setFormData({ ...formData, induction: { ...formData.induction, others } })
                  }
                  defaultItem={() => ({ name: '', dose: '' })}
                  addLabel="新增藥劑"
                  renderItem={(item, _index, onChange) => (
                    <div className="grid grid-cols-2 gap-2">
                      <Input
                        placeholder="藥劑名稱"
                        value={item.name}
                        onChange={(e) => onChange({ ...item, name: e.target.value })}
                      />
                      <Input
                        placeholder="劑量"
                        value={item.dose}
                        onChange={(e) => onChange({ ...item, dose: e.target.value })}
                      />
                    </div>
                  )}
                />
              </div>
            </div>
          </CollapsibleSection>

          {/* 術前給藥 */}
          <CollapsibleSection title="術前給藥" defaultOpen={false}>
            <Repeater<MedicationItem>
              value={formData.pre_surgery.medications}
              onChange={(medications) =>
                setFormData({ ...formData, pre_surgery: { ...formData.pre_surgery, medications } })
              }
              defaultItem={() => ({ name: '', dose: '' })}
              addLabel="新增術前藥品"
              renderItem={(item, _index, onChange) => (
                <div className="grid grid-cols-2 gap-2">
                  <Input
                    placeholder="藥品名稱"
                    value={item.name}
                    onChange={(e) => onChange({ ...item, name: e.target.value })}
                  />
                  <Input
                    placeholder="劑量"
                    value={item.dose}
                    onChange={(e) => onChange({ ...item, dose: e.target.value })}
                  />
                </div>
              )}
            />
          </CollapsibleSection>

          {/* 固定姿勢 */}
          <CollapsibleSection title="固定姿勢" defaultOpen={false}>
            <div className="space-y-2">
              <Label htmlFor="positioning">固定姿勢</Label>
              <Input
                id="positioning"
                value={formData.positioning}
                onChange={(e) => setFormData({ ...formData, positioning: e.target.value })}
                placeholder="描述固定姿勢..."
              />
            </div>
          </CollapsibleSection>

          {/* 麻醉維持 */}
          <CollapsibleSection title="麻醉維持">
            <div className="space-y-3">
              <DrugCheckInput
                label="O2"
                drug={formData.maintenance.o2}
                onChange={(drug) =>
                  setFormData({ ...formData, maintenance: { ...formData.maintenance, o2: drug } })
                }
              />
              <DrugCheckInput
                label="N2O"
                drug={formData.maintenance.n2o}
                onChange={(drug) =>
                  setFormData({ ...formData, maintenance: { ...formData.maintenance, n2o: drug } })
                }
              />
              <DrugCheckInput
                label="Isoflurane"
                drug={formData.maintenance.isoflurane}
                onChange={(drug) =>
                  setFormData({ ...formData, maintenance: { ...formData.maintenance, isoflurane: drug } })
                }
              />
              <div className="pt-2">
                <Label className="text-sm text-slate-600">其他藥劑</Label>
                <Repeater<MedicationItem>
                  value={formData.maintenance.others}
                  onChange={(others) =>
                    setFormData({ ...formData, maintenance: { ...formData.maintenance, others } })
                  }
                  defaultItem={() => ({ name: '', dose: '' })}
                  addLabel="新增藥劑"
                  renderItem={(item, _index, onChange) => (
                    <div className="grid grid-cols-2 gap-2">
                      <Input
                        placeholder="藥劑名稱"
                        value={item.name}
                        onChange={(e) => onChange({ ...item, name: e.target.value })}
                      />
                      <Input
                        placeholder="劑量"
                        value={item.dose}
                        onChange={(e) => onChange({ ...item, dose: e.target.value })}
                      />
                    </div>
                  )}
                />
              </div>
            </div>
          </CollapsibleSection>

          {/* 監測與恢復 */}
          <CollapsibleSection title="監測與恢復">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="anesthesia_observation">麻醉觀察過程</Label>
                <Textarea
                  id="anesthesia_observation"
                  value={formData.anesthesia_observation}
                  onChange={(e) => setFormData({ ...formData, anesthesia_observation: e.target.value })}
                  placeholder="描述麻醉觀察過程..."
                />
              </div>

              <div className="space-y-2">
                <Label>生理數值</Label>
                <Repeater<VitalSign>
                  value={formData.vital_signs}
                  onChange={(vital_signs) => setFormData({ ...formData, vital_signs })}
                  defaultItem={() => ({
                    time: '',
                    breathing_method: '',
                    heart_rate: '',
                    respiration_rate: '',
                    temperature: '',
                    spo2: '',
                  })}
                  addLabel="新增測量紀錄"
                  renderItem={(item, _index, onChange) => (
                    <div className="grid grid-cols-6 gap-2 p-2 bg-slate-50 rounded">
                      <Input
                        type="time"
                        placeholder="時間"
                        value={item.time}
                        onChange={(e) => onChange({ ...item, time: e.target.value })}
                      />
                      <Input
                        placeholder="呼吸方式"
                        value={item.breathing_method}
                        onChange={(e) => onChange({ ...item, breathing_method: e.target.value })}
                      />
                      <Input
                        type="number"
                        placeholder="心跳/分"
                        value={item.heart_rate}
                        onChange={(e) => onChange({ ...item, heart_rate: e.target.value })}
                      />
                      <Input
                        type="number"
                        placeholder="呼吸/分"
                        value={item.respiration_rate}
                        onChange={(e) => onChange({ ...item, respiration_rate: e.target.value })}
                      />
                      <Input
                        type="number"
                        step="0.1"
                        placeholder="體溫°C"
                        value={item.temperature}
                        onChange={(e) => onChange({ ...item, temperature: e.target.value })}
                      />
                      <Input
                        type="number"
                        placeholder="SPO2%"
                        value={item.spo2}
                        onChange={(e) => onChange({ ...item, spo2: e.target.value })}
                      />
                    </div>
                  )}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="reflex_recovery">反射恢復觀察</Label>
                <Textarea
                  id="reflex_recovery"
                  value={formData.reflex_recovery}
                  onChange={(e) => setFormData({ ...formData, reflex_recovery: e.target.value })}
                  placeholder="描述反射恢復狀況..."
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="respiration_auto">自主呼吸：呼吸次數/分鐘</Label>
                <Input
                  id="respiration_auto"
                  type="number"
                  value={formData.respiration_rate_auto}
                  onChange={(e) => setFormData({ ...formData, respiration_rate_auto: e.target.value })}
                  className="w-32"
                />
              </div>
            </div>
          </CollapsibleSection>

          {/* 術後區塊 */}
          <CollapsibleSection title="術後">
            <div className="space-y-4">
              <Checkbox
                label="術後給藥-優點軟膏"
                checked={formData.post_ointment}
                onChange={(e) => setFormData({ ...formData, post_ointment: e.target.checked })}
              />

              <div className="space-y-2">
                <Label>術後給藥-其他</Label>
                <Repeater<MedicationItem>
                  value={formData.post_medications}
                  onChange={(post_medications) => setFormData({ ...formData, post_medications })}
                  defaultItem={() => ({ name: '', dose: '' })}
                  addLabel="新增術後藥品"
                  renderItem={(item, _index, onChange) => (
                    <div className="grid grid-cols-2 gap-2">
                      <Input
                        placeholder="藥品名稱"
                        value={item.name}
                        onChange={(e) => onChange({ ...item, name: e.target.value })}
                      />
                      <Input
                        placeholder="劑量"
                        value={item.dose}
                        onChange={(e) => onChange({ ...item, dose: e.target.value })}
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
                  onChange={(e) => setFormData({ ...formData, remark: e.target.value })}
                  placeholder="其他備註..."
                />
              </div>

              <Checkbox
                label="不需用藥/停止用藥"
                checked={formData.no_medication_needed}
                onChange={(e) => setFormData({ ...formData, no_medication_needed: e.target.checked })}
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
                  onChange={(photos) => setFormData({ ...formData, photos })}
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
                  onChange={(attachments) => setFormData({ ...formData, attachments })}
                  accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.zip,.rar"
                  placeholder="拖曳附件到此處，或點擊選擇檔案"
                  maxSize={20}
                  maxFiles={10}
                  showPreview={false}
                />
              </div>
            </div>
          </CollapsibleSection>

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
