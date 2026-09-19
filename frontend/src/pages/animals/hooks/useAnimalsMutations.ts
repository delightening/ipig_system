import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { isAxiosError } from '@/lib/api'
import type { Animal, AnimalListItem, CreateAnimalRequest } from '@/types/animal'
import type { PaginatedResponse } from '@/types/common'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import type { NewAnimalForm, QuickAddForm, DuplicateWarningData } from '../components/AnimalAddDialog'

export type DuplicateWarningPayload = DuplicateWarningData

interface MutationsOptions {
  penZone: string
  penCode: string
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
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const invalidateAll = () => {
    queryClient.invalidateQueries({ queryKey: ['animals'] })
    queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
    queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
  }

  const extractErrorMessage = (error: unknown, fallback: string) =>
    getApiErrorMessage(error, fallback)

  const handleDuplicate409 = (error: unknown, source: 'create' | 'quickAdd') => {
    if (!isAxiosError(error) || error.response?.status !== 409) return false
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
      const { penZone, penCode } = opts
      if (!penZone || !penCode) throw new Error(t('validation.penRequired'))
      // pen_location 直接用欄位自己的 code。先前是「區碼 + 去掉首字的欄位 code」拼回來，
      // 只在 code 恰好是「一字母 + 編號」時才等價；欄位 code 一旦不符那個形狀就會拼錯。
      const penLocation = penCode

      if (!data.ear_tag?.trim()) throw new Error(t('validation.earTagRequired'))
      if (!data.entry_date) throw new Error(t('validation.entryDateRequired'))
      if (!data.birth_date) throw new Error(t('validation.birthDateRequired'))
      if (!data.pre_experiment_code?.trim()) throw new Error(t('validation.preExperimentCodeRequired'))

      let entryWeight: number | undefined
      if (data.entry_weight && data.entry_weight !== '') {
        const weightValue = parseFloat(data.entry_weight)
        if (isNaN(weightValue) || weightValue <= 0) throw new Error(t('animalPages.mutations.entryWeightInvalid'))
        entryWeight = weightValue
      }

      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      if (!dateRegex.test(data.entry_date)) throw new Error(t('animalPages.mutations.entryDateFormat'))
      if (data.birth_date && data.birth_date.trim() !== '' && !dateRegex.test(data.birth_date))
        throw new Error(t('animalPages.mutations.birthDateFormat'))

      let formattedEarTag = data.ear_tag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      // species_id 在後端是 Option<Uuid>，空字串會在反序列化就炸掉、拿不到「請指定物種」
      // 這個可讀錯誤。表單雖然已用 disabled 擋，但不把 UI 當唯一防線。
      const speciesId = data.species_id?.trim()
      if (!speciesId) throw new Error(t('animalPages.mutations.selectBreed'))

      const payload: CreateAnimalRequest & { breed_other?: string } = {
        ear_tag: formattedEarTag,
        // 只送 species_id：breed enum 由後端依物種推導（見 backend SpeciesLink）
        species_id: speciesId,
        gender: data.gender,
        entry_date: data.entry_date,
        birth_date: data.birth_date && data.birth_date.trim() !== '' ? data.birth_date.trim() : undefined,
        entry_weight: entryWeight,
        pen_location: penLocation,
        pre_experiment_code: data.pre_experiment_code?.trim() || undefined,
        remark: data.remark?.trim() || undefined,
        breed_other: data.breed_other?.trim() || undefined,
      }

      if (data.source_id && data.source_id.trim() !== '' && data.source_id.trim() !== 'none') {
        const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
        const trimmedSourceId = data.source_id.trim()
        if (uuidRegex.test(trimmedSourceId)) payload.source_id = trimmedSourceId
        else throw new Error(t('animalPages.mutations.sourceIdInvalid', { id: trimmedSourceId }))
      }

      return api.post('/animals', payload)
    },
    onSuccess: () => {
      invalidateAll()
      toast({ title: t('common.success'), description: t('animalPages.mutations.animalAdded') })
      opts.setShowAddDialog(false)
      opts.resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'create')) return

      let errorMessage = t('animalPages.mutations.addFailedCheckInput')
      if (isAxiosError(error) && error.response?.status === 422) {
        const data = error.response.data as { error?: { message?: string }; message?: string } | undefined
        errorMessage = data?.error?.message || data?.message || t('animalPages.mutations.invalidDataFormat')
      } else {
        errorMessage = extractErrorMessage(error, errorMessage)
      }
      toast({ title: t('common.error'), description: errorMessage, variant: 'destructive' })
    },
  })

  const batchAssignMutation = useMutation({
    mutationFn: () => api.post('/animals/batch/assign', { animal_ids: opts.selectedAnimals, iacuc_no: opts.assignIacucNo }),
    onSuccess: () => {
      invalidateAll()
      toast({ title: t('common.success'), description: t('animalPages.mutations.batchAssigned') })
      opts.setShowBatchAssignDialog(false)
      opts.setSelectedAnimals([])
      opts.setAssignIacucNo('')
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: extractErrorMessage(error, t('animalPages.mutations.batchAssignFailed')), variant: 'destructive' })
    },
  })

  const quickMoveMutation = useMutation({
    mutationFn: async ({ earTag, targetPenLocation }: { earTag: string; targetPenLocation: string }) => {
      let formattedEarTag = earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      const searchRes = await api.get<PaginatedResponse<AnimalListItem>>(`/animals?keyword=${encodeURIComponent(formattedEarTag)}`)
      const matchingAnimals = (searchRes.data.data ?? []).filter(p => p.ear_tag === formattedEarTag && p.pen_location)

      if (matchingAnimals.length === 0) return { notFound: true, formattedEarTag, targetPenLocation }
      if (matchingAnimals.length > 1) throw new Error(t('animalPages.mutations.multipleEarTags', { earTag: formattedEarTag }))

      const animal = matchingAnimals[0]
      if (animal.pen_location === targetPenLocation) throw new Error(t('animalPages.mutations.alreadyInPen', { earTag: formattedEarTag, pen: targetPenLocation }))

      // R30-B: 帶當前 version 防 lost update（從搜尋結果取）
      return { ...await api.put<Animal>(`/animals/${animal.id}`, { pen_location: targetPenLocation, version: animal.version }), notFound: false }
    },
    onSuccess: (data: { notFound?: boolean; formattedEarTag?: string; targetPenLocation?: string }, variables: { earTag: string; targetPenLocation: string }) => {
      if (data.notFound && data.formattedEarTag != null && data.targetPenLocation != null) {
        opts.setQuickAddPending({ earTag: data.formattedEarTag, penLocation: data.targetPenLocation })
        opts.setQuickAddForm({
          species_id: '', breed_other: '', gender: 'male',
          entry_date: new Date().toISOString().split('T')[0], birth_date: '', entry_weight: '',
        })
        opts.setShowQuickAddDialog(true)
        return
      }
      invalidateAll()
      let formattedEarTag = variables.earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')
      toast({
        title: t('common.success'),
        description: t('animalPages.mutations.animalMoved', { earTag: formattedEarTag, pen: variables.targetPenLocation }),
      })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: extractErrorMessage(error, t('animalPages.mutations.moveFailed')), variant: 'destructive' })
    },
  })

  const quickAddMutation = useMutation({
    mutationFn: async () => {
      const { quickAddPending, quickAddForm } = opts
      if (!quickAddPending) throw new Error(t('animalPages.mutations.noPendingAdd'))
      if (!quickAddForm.entry_date) throw new Error(t('validation.entryDateRequired'))
      if (!quickAddForm.birth_date) throw new Error(t('validation.birthDateRequired'))

      const speciesId = quickAddForm.species_id?.trim()
      if (!speciesId) throw new Error(t('animalPages.mutations.selectBreed'))

      return api.post<Animal>('/animals', {
        ear_tag: quickAddPending.earTag,
        species_id: speciesId,
        breed_other: quickAddForm.breed_other?.trim() || undefined,
        gender: quickAddForm.gender,
        entry_date: quickAddForm.entry_date,
        birth_date: quickAddForm.birth_date,
        entry_weight: parseFloat(quickAddForm.entry_weight),
        pen_location: quickAddPending.penLocation,
      })
    },
    onSuccess: () => {
      invalidateAll()
      toast({
        title: t('common.success'),
        description: t('animalPages.mutations.quickAdded', {
          earTag: opts.quickAddPending?.earTag,
          pen: opts.quickAddPending?.penLocation,
        }),
      })
      opts.setShowQuickAddDialog(false)
      opts.setQuickAddPending(null)
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'quickAdd')) return
      toast({ title: t('common.error'), description: extractErrorMessage(error, t('animalPages.shared.addFailed')), variant: 'destructive' })
    },
  })

  const forceCreateMutation = useMutation({
    mutationFn: (payload: CreateAnimalRequest & { breed_other?: string }) => api.post('/animals', { ...payload, force_create: true }),
    onSuccess: () => {
      invalidateAll()
      toast({ title: t('common.success'), description: t('animalPages.mutations.animalAddedDuplicateConfirmed') })
      opts.setShowDuplicateWarning(false)
      opts.setDuplicateWarningData(null)
      opts.setShowAddDialog(false)
      opts.setShowQuickAddDialog(false)
      opts.setQuickAddPending(null)
      opts.resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: extractErrorMessage(error, t('animalPages.shared.addFailed')), variant: 'destructive' })
    },
  })

  return { createAnimalMutation, batchAssignMutation, quickMoveMutation, quickAddMutation, forceCreateMutation }
}
