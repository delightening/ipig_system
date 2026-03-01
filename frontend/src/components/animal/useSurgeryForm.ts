import { useState, useEffect, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { AnimalSurgery } from '@/lib/api'
import { FileInfo } from '@/components/ui/file-upload'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

/** 誘導麻醉藥物項目 */
export interface AnesthesiaDrug {
  name: string
  dose: string
  enabled: boolean
}

/** 生理數值項目 */
export interface VitalSign {
  time: string
  breathing_method: string
  heart_rate: string
  respiration_rate: string
  temperature: string
  spo2: string
}

/** 其他藥物項目 */
export interface MedicationItem {
  name: string
  dose: string
  drug_option_id?: string
  dosage_unit?: string
}

/** 表單狀態 */
export interface SurgeryFormData {
  is_first_experiment: boolean
  surgery_date: string
  surgery_site: string
  induction: {
    atropine: AnesthesiaDrug
    stroless: AnesthesiaDrug
    zoletil50: AnesthesiaDrug
    others: MedicationItem[]
  }
  pre_surgery: {
    medications: MedicationItem[]
    others: MedicationItem[]
  }
  positioning: string
  positioning_others: string[]
  maintenance: {
    o2: AnesthesiaDrug
    n2o: AnesthesiaDrug
    isoflurane: AnesthesiaDrug
    others: MedicationItem[]
  }
  anesthesia_observation: string
  vital_signs: VitalSign[]
  reflex_recovery: string
  respiration_rate_auto: string
  post_ointment: boolean
  post_medications: MedicationItem[]
  remark: string
  no_medication_needed: boolean
  photos: FileInfo[]
  attachments: FileInfo[]
}

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

function toMedicationItems(val: unknown): MedicationItem[] {
  if (!Array.isArray(val)) return []
  return val
    .filter((x): x is MedicationItem => x != null && typeof x === 'object' && 'name' in x && 'dose' in x)
    .map((x) => ({
      name: String((x as { name?: unknown }).name ?? ''),
      dose: String((x as { dose?: unknown }).dose ?? ''),
      drug_option_id: (x as MedicationItem).drug_option_id,
      dosage_unit: (x as MedicationItem).dosage_unit,
    }))
}

export function useSurgeryForm({
  open,
  onOpenChange,
  animalId,
  surgery,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  surgery?: AnimalSurgery
}) {
  const queryClient = useQueryClient()
  const isEdit = !!surgery

  const [formData, setFormData] = useState<SurgeryFormData>(defaultFormData)

  useEffect(() => {
    if (surgery) {
      const induction = (surgery.induction_anesthesia as Record<string, unknown>) || {}
      const maintenance = (surgery.anesthesia_maintenance as Record<string, unknown>) || {}
      const preSurgery = (surgery.pre_surgery_medication as Record<string, unknown>) || {}
      const postSurgery = (surgery.post_surgery_medication as Record<string, unknown>) || {}

      setFormData({
        is_first_experiment: surgery.is_first_experiment,
        surgery_date: surgery.surgery_date.split('T')[0],
        surgery_site: surgery.surgery_site,
        induction: {
          atropine: { name: 'Atropine', dose: String(induction.atropine ?? ''), enabled: !!induction.atropine },
          stroless: { name: 'Stroless', dose: String(induction.stroless ?? ''), enabled: !!induction.stroless },
          zoletil50: { name: 'Zoletil-50', dose: String(induction.zoletil50 ?? ''), enabled: !!induction.zoletil50 },
          others: toMedicationItems(induction.others),
        },
        pre_surgery: {
          medications: toMedicationItems(preSurgery.medications),
          others: toMedicationItems(preSurgery.others),
        },
        positioning: surgery.positioning || '',
        positioning_others: [],
        maintenance: {
          o2: { name: 'O2', dose: String(maintenance.o2 ?? ''), enabled: !!maintenance.o2 },
          n2o: { name: 'N2O', dose: String(maintenance.n2o ?? ''), enabled: !!maintenance.n2o },
          isoflurane: {
            name: 'Isoflurane',
            dose: String(maintenance.isoflurane ?? ''),
            enabled: !!maintenance.isoflurane,
          },
          others: toMedicationItems(maintenance.others),
        },
        anesthesia_observation: surgery.anesthesia_observation || '',
        vital_signs: (surgery.vital_signs || []).map(
          (vs: {
            time?: string
            breathing_method?: string
            heart_rate?: number
            respiration_rate?: number
            temperature?: number
            spo2?: number
          }) => ({
            time: vs.time || '',
            breathing_method: (vs as { breathing_method?: string }).breathing_method || '',
            heart_rate: String(vs.heart_rate || ''),
            respiration_rate: String(vs.respiration_rate || ''),
            temperature: String(vs.temperature || ''),
            spo2: String(vs.spo2 || ''),
          })
        ),
        reflex_recovery: surgery.reflex_recovery || '',
        respiration_rate_auto: surgery.respiration_rate ? String(surgery.respiration_rate) : '',
        post_ointment: !!postSurgery.ointment,
        post_medications: toMedicationItems(postSurgery.others),
        remark: surgery.remark || '',
        no_medication_needed: surgery.no_medication_needed,
        photos: [],
        attachments: [],
      })
    } else {
      setFormData(defaultFormData)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- init only when open/surgery changes
  }, [surgery, open])

  const mutation = useMutation({
    mutationFn: async (data: SurgeryFormData) => {
      const inductionAnesthesia: Record<string, unknown> = {}
      if (data.induction.atropine.enabled) inductionAnesthesia.atropine = data.induction.atropine.dose
      if (data.induction.stroless.enabled) inductionAnesthesia.stroless = data.induction.stroless.dose
      if (data.induction.zoletil50.enabled) inductionAnesthesia.zoletil50 = data.induction.zoletil50.dose
      if (data.induction.others.length > 0) inductionAnesthesia.others = data.induction.others

      const anesthesiaMaintenance: Record<string, unknown> = {}
      if (data.maintenance.o2.enabled) anesthesiaMaintenance.o2 = data.maintenance.o2.dose
      if (data.maintenance.n2o.enabled) anesthesiaMaintenance.n2o = data.maintenance.n2o.dose
      if (data.maintenance.isoflurane.enabled) anesthesiaMaintenance.isoflurane = data.maintenance.isoflurane.dose
      if (data.maintenance.others.length > 0) anesthesiaMaintenance.others = data.maintenance.others

      const preSurgeryMedication: Record<string, unknown> = {}
      if (data.pre_surgery.medications.length > 0) preSurgeryMedication.medications = data.pre_surgery.medications
      if (data.pre_surgery.others.length > 0) preSurgeryMedication.others = data.pre_surgery.others

      const postSurgeryMedication: Record<string, unknown> = {}
      if (data.post_ointment) postSurgeryMedication.ointment = true
      if (data.post_medications.length > 0) postSurgeryMedication.others = data.post_medications

      const payload = {
        is_first_experiment: data.is_first_experiment,
        surgery_date: data.surgery_date,
        surgery_site: data.surgery_site,
        induction_anesthesia:
          Object.keys(inductionAnesthesia).length > 0 ? inductionAnesthesia : null,
        pre_surgery_medication:
          Object.keys(preSurgeryMedication).length > 0 ? preSurgeryMedication : null,
        positioning: data.positioning || null,
        anesthesia_maintenance:
          Object.keys(anesthesiaMaintenance).length > 0 ? anesthesiaMaintenance : null,
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
        post_surgery_medication:
          Object.keys(postSurgeryMedication).length > 0 ? postSurgeryMedication : null,
        remark: data.remark || null,
        no_medication_needed: data.no_medication_needed,
      }

      if (isEdit && surgery) {
        return api.put(`/surgeries/${surgery.id}`, payload)
      }
      return api.post(`/animals/${animalId}/surgeries`, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: '成功', description: isEdit ? '手術紀錄已更新' : '手術紀錄已新增' })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '儲存失敗'),
        variant: 'destructive',
      })
    },
  })

  const jumpToNextEmptyField = useCallback(() => {
    const fields = [
      { id: 'surgery_date', value: formData.surgery_date },
      { id: 'surgery_site', value: formData.surgery_site },
      { id: 'positioning', value: formData.positioning },
      { id: 'anesthesia_observation', value: formData.anesthesia_observation },
      { id: 'reflex_recovery', value: formData.reflex_recovery },
      { id: 'respiration_auto', value: formData.respiration_rate_auto },
      { id: 'remark', value: formData.remark },
    ]

    const nextEmpty = fields.find(
      (f) => !f.value || (typeof f.value === 'string' && f.value.trim() === '')
    )
    if (nextEmpty) {
      const element = document.getElementById(nextEmpty.id)
      if (element) {
        element.focus()
        toast({ title: '已跳轉', description: '跳轉至下一個空白欄位', duration: 2000 })
        return
      }
    }
    toast({ title: '完成', description: '主要欄位皆已填寫', duration: 2000 })
  }, [formData])

  return { formData, setFormData, mutation, jumpToNextEmptyField }
}
