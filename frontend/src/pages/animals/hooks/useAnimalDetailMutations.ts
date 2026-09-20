import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { Animal } from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { toast } from '@/components/ui/use-toast'

interface SuddenDeathFormData {
  discovered_at: string
  probable_cause: string
  location: string
  remark: string
  requires_pathology: boolean
}

const INITIAL_SUDDEN_DEATH_FORM: SuddenDeathFormData = {
  discovered_at: new Date().toISOString().slice(0, 16),
  probable_cause: '',
  location: '',
  remark: '',
  requires_pathology: false,
}

export function useAnimalDetailMutations(animalId: string) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const [showSuddenDeathDialog, setShowSuddenDeathDialog] = useState(false)
  const [suddenDeathForm, setSuddenDeathForm] = useState<SuddenDeathFormData>(
    INITIAL_SUDDEN_DEATH_FORM,
  )

  const assignTrialMutation = useMutation({
    mutationFn: async (iacucNo: string) => {
      // R30-B: 帶當前 version 防 lost update（從 query cache 取，避免 stale form state）
      const animal = queryClient.getQueryData<Animal>(['animal', animalId])
      return api.put(`/animals/${animalId}`, {
        iacuc_no: iacucNo,
        status: 'in_experiment',
        version: animal?.version,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: t('common.success'), description: t('animalPages.mutations.assignedToStudy') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getErrorMessage(error) || t('animalPages.mutations.assignFailed'),
        variant: 'destructive',
      })
    },
  })

  const createSuddenDeathMutation = useMutation({
    mutationFn: async (data: SuddenDeathFormData) => {
      return api.post(`/animals/${animalId}/sudden-death`, {
        discovered_at: new Date(data.discovered_at).toISOString(),
        probable_cause: data.probable_cause || undefined,
        location: data.location || undefined,
        remark: data.remark || undefined,
        requires_pathology: data.requires_pathology,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animal-sudden-death', animalId] })
      toast({
        title: t('animalPages.mutations.suddenDeathRegisteredTitle'),
        description: t('animalPages.mutations.suddenDeathRegistered'),
      })
      setShowSuddenDeathDialog(false)
      setSuddenDeathForm(INITIAL_SUDDEN_DEATH_FORM)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getErrorMessage(error) || t('animalPages.mutations.suddenDeathFailed'),
        variant: 'destructive',
      })
    },
  })

  return {
    showSuddenDeathDialog,
    setShowSuddenDeathDialog,
    suddenDeathForm,
    setSuddenDeathForm,
    assignTrialMutation,
    createSuddenDeathMutation,
  }
}

export type { SuddenDeathFormData }
