import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

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
      toast({ title: '\u6210\u529F', description: '\u52D5\u7269\u5DF2\u6210\u529F\u5206\u914D\u5230\u8A66\u9A57' })
    },
    onError: (error: unknown) => {
      toast({
        title: '\u932F\u8AA4',
        description: getErrorMessage(error) || '\u5206\u914D\u5931\u6557',
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
        title: '\u5DF2\u767B\u8A18',
        description: '\u731D\u6B7B\u7D00\u9304\u5DF2\u767B\u8A18\uFF0C\u52D5\u7269\u72C0\u614B\u5DF2\u81EA\u52D5\u66F4\u65B0',
      })
      setShowSuddenDeathDialog(false)
      setSuddenDeathForm(INITIAL_SUDDEN_DEATH_FORM)
    },
    onError: (error: unknown) => {
      toast({
        title: '\u932F\u8AA4',
        description: getErrorMessage(error) || '\u731D\u6B7B\u767B\u8A18\u5931\u6557',
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
