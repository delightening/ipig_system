import { useMutation, useQueryClient } from '@tanstack/react-query'
import axios from 'axios'

import api from '@/lib/api'
import type { Animal, AnimalListItem, CreateAnimalRequest } from '@/types/animal'
import type { PaginatedResponse } from '@/types/common'
import { getApiErrorMessage } from '@/lib/validation'
import { toast } from '@/components/ui/use-toast'
import type { NewAnimalForm, QuickAddForm, DuplicateWarningData } from '../components/AnimalAddDialog'

export type DuplicateWarningPayload = DuplicateWarningData

interface MutationsOptions {
  penZone: string
  penNumber: string
  selectedAnimals: string[]
  assignIacucNo: string
  newAnimal: NewAnimalForm
  quickAddPending: { earTag: string; penLocation: string } | null
  quickAddForm: QuickAddForm
  setQuickAddForm: (form: QuickAddForm) => void
  setShowAddDialog: (open: boolean) => void
  setShowBatchAssignDialog: (open: boolean) => void
  setShowQuickAddDialog: (open: boolean) => void
  setShowDuplicateWarning: (open: boolean) => void
  setSelectedAnimals: (ids: string[]) => void
  setAssignIacucNo: (v: string) => void
  setQuickAddPending: (v: { earTag: string; penLocation: string } | null) => void
  setDuplicateWarningData: (v: DuplicateWarningPayload | null) => void
  setQuickEditAnimalId: (id: string | null) => void
  resetNewAnimalForm: () => void
}

export function useAnimalsMutations(opts: MutationsOptions) {
  const queryClient = useQueryClient()

  const invalidateAll = () => {
    queryClient.invalidateQueries({ queryKey: ['animals'] })
    queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
    queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
  }

  const extractErrorMessage = (error: unknown, fallback: string) =>
    getApiErrorMessage(error, fallback)

  const handleDuplicate409 = (error: unknown, source: 'create' | 'quickAdd') => {
    if (!axios.isAxiosError(error) || error.response?.status !== 409) return false
    const errData = error.response.data?.error
    if (errData?.warning_type !== 'duplicate_ear_tag' || errData?.blocking !== false) return false
    let payload: Record<string, unknown> = {}
    try { payload = JSON.parse(error.config?.data || '{}') } catch { /* ignore */ }
    opts.setDuplicateWarningData({
      earTag: (payload.ear_tag as string) || opts.quickAddPending?.earTag || '',
      existingAnimals: errData.existing_animals || [],
      source,
      pendingPayload: payload as unknown as CreateAnimalRequest & { breed_other?: string },
    })
    opts.setShowDuplicateWarning(true)
    return true
  }

  const createAnimalMutation = useMutation({
    mutationFn: async (data: NewAnimalForm) => {
      const { penZone, penNumber } = opts
      if (!penZone || !penNumber) throw new Error('欄位為必填，請選擇欄位區和欄位編號')
      const penLocation = `${penZone}${penNumber}`

      if (!data.ear_tag?.trim()) throw new Error('耳號為必填')
      if (!data.entry_date) throw new Error('進場日期為必填')
      if (!data.birth_date) throw new Error('出生日期為必填')
      if (!data.pre_experiment_code?.trim()) throw new Error('實驗前代號為必填')

      let entryWeight: number | undefined
      if (data.entry_weight && data.entry_weight !== '') {
        const weightValue = parseFloat(data.entry_weight)
        if (isNaN(weightValue) || weightValue <= 0) throw new Error('進場體重必須是大於 0 的數字')
        entryWeight = weightValue
      }

      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      if (!dateRegex.test(data.entry_date)) throw new Error('進場日期格式不正確，必須是 YYYY-MM-DD 格式')
      if (data.birth_date && data.birth_date.trim() !== '' && !dateRegex.test(data.birth_date))
        throw new Error('出生日期格式不正確，必須是 YYYY-MM-DD 格式')

      let formattedEarTag = data.ear_tag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      const payload: CreateAnimalRequest & { breed_other?: string } = {
        ear_tag: formattedEarTag,
        breed: data.breed,
        gender: data.gender,
        entry_date: data.entry_date,
        birth_date: data.birth_date && data.birth_date.trim() !== '' ? data.birth_date.trim() : undefined,
        entry_weight: entryWeight,
        pen_location: penLocation,
        pre_experiment_code: data.pre_experiment_code?.trim() || undefined,
        remark: data.remark?.trim() || undefined,
        breed_other: data.breed === 'other' ? data.breed_other : undefined,
      }

      if (data.source_id && data.source_id.trim() !== '' && data.source_id.trim() !== 'none') {
        const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
        const trimmedSourceId = data.source_id.trim()
        if (uuidRegex.test(trimmedSourceId)) payload.source_id = trimmedSourceId
        else throw new Error(`來源 ID 格式不正確: ${trimmedSourceId}`)
      }

      return api.post('/animals', payload)
    },
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: '動物已新增' })
      opts.setShowAddDialog(false)
      opts.resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'create')) return

      let errorMessage = '新增失敗，請檢查輸入資料'
      if (axios.isAxiosError(error) && error.response?.status === 422) {
        const data = error.response.data as { error?: { message?: string }; message?: string } | undefined
        errorMessage = data?.error?.message || data?.message || '資料格式錯誤：請檢查所有欄位的格式是否正確'
      } else {
        errorMessage = extractErrorMessage(error, errorMessage)
      }
      toast({ title: '錯誤', description: errorMessage, variant: 'destructive' })
    },
  })

  const batchAssignMutation = useMutation({
    mutationFn: () => api.post('/animals/batch/assign', { animal_ids: opts.selectedAnimals, iacuc_no: opts.assignIacucNo }),
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: '動物已分配至計劃並進入實驗中' })
      opts.setShowBatchAssignDialog(false)
      opts.setSelectedAnimals([])
      opts.setAssignIacucNo('')
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '批次分配失敗'), variant: 'destructive' })
    },
  })

  const quickMoveMutation = useMutation({
    mutationFn: async ({ earTag, targetPenLocation }: { earTag: string; targetPenLocation: string }) => {
      let formattedEarTag = earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      const searchRes = await api.get<PaginatedResponse<AnimalListItem>>(`/animals?keyword=${encodeURIComponent(formattedEarTag)}`)
      const matchingAnimals = (searchRes.data.data ?? []).filter(p => p.ear_tag === formattedEarTag && p.pen_location)

      if (matchingAnimals.length === 0) return { notFound: true, formattedEarTag, targetPenLocation }
      if (matchingAnimals.length > 1) throw new Error(`找到多隻耳號為 "${formattedEarTag}" 的動物，請使用編輯功能手動移動`)

      const animal = matchingAnimals[0]
      if (animal.pen_location === targetPenLocation) throw new Error(`動物 ${formattedEarTag} 已經在 ${targetPenLocation} 欄位`)

      return { ...await api.put<Animal>(`/animals/${animal.id}`, { pen_location: targetPenLocation }), notFound: false }
    },
    onSuccess: (data: { notFound?: boolean; formattedEarTag?: string; targetPenLocation?: string }, variables: { earTag: string; targetPenLocation: string }) => {
      if (data.notFound && data.formattedEarTag != null && data.targetPenLocation != null) {
        opts.setQuickAddPending({ earTag: data.formattedEarTag, penLocation: data.targetPenLocation })
        opts.setQuickAddForm({
          breed: 'minipig', breed_other: '', gender: 'male',
          entry_date: new Date().toISOString().split('T')[0], birth_date: '', entry_weight: '',
        })
        opts.setShowQuickAddDialog(true)
        return
      }
      invalidateAll()
      let formattedEarTag = variables.earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')
      toast({ title: '成功', description: `動物 ${formattedEarTag} 已移動到 ${variables.targetPenLocation}` })
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '移動失敗'), variant: 'destructive' })
    },
  })

  const quickAddMutation = useMutation({
    mutationFn: async () => {
      const { quickAddPending, quickAddForm } = opts
      if (!quickAddPending) throw new Error('無待處理的新增請求')
      if (!quickAddForm.entry_date) throw new Error('進場日期為必填')
      if (!quickAddForm.birth_date) throw new Error('出生日期為必填')

      return api.post<Animal>('/animals', {
        ear_tag: quickAddPending.earTag,
        breed: quickAddForm.breed,
        breed_other: quickAddForm.breed === 'other' ? quickAddForm.breed_other : undefined,
        gender: quickAddForm.gender,
        entry_date: quickAddForm.entry_date,
        birth_date: quickAddForm.birth_date,
        entry_weight: parseFloat(quickAddForm.entry_weight),
        pen_location: quickAddPending.penLocation,
      })
    },
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: `已新增動物 ${opts.quickAddPending?.earTag} 至 ${opts.quickAddPending?.penLocation}` })
      opts.setShowQuickAddDialog(false)
      opts.setQuickAddPending(null)
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'quickAdd')) return
      toast({ title: '錯誤', description: extractErrorMessage(error, '新增失敗'), variant: 'destructive' })
    },
  })

  const forceCreateMutation = useMutation({
    mutationFn: (payload: CreateAnimalRequest & { breed_other?: string }) => api.post('/animals', { ...payload, force_create: true }),
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: '動物已新增（已確認耳號重複）' })
      opts.setShowDuplicateWarning(false)
      opts.setDuplicateWarningData(null)
      opts.setShowAddDialog(false)
      opts.setShowQuickAddDialog(false)
      opts.setQuickAddPending(null)
      opts.resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '新增失敗'), variant: 'destructive' })
    },
  })

  return { createAnimalMutation, batchAssignMutation, quickMoveMutation, quickAddMutation, forceCreateMutation }
}
