import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import type {
  ChangeStatusRequest,
  AssignCoEditorRequest,
} from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { getApiErrorMessage } from '@/lib/validation'
import { toast } from '@/components/ui/use-toast'
import type { ProtocolVersion } from '@/types/aup'

interface UseProtocolMutationsOptions {
  id: string | undefined
  versions: ProtocolVersion[] | undefined
  onStatusChangeSuccess: () => void
}

export function useProtocolMutations({
  id,
  versions,
  onStatusChangeSuccess,
}: UseProtocolMutationsOptions) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const submitMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/submit`),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.submitSuccess') })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.detail(id!) })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.submitFailed')),
        variant: 'destructive',
      })
    },
  })

  const changeStatusMutation = useMutation({
    mutationFn: async (data: ChangeStatusRequest) => api.post(`/protocols/${id}/status`, data),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.statusChangeSuccess') })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.detail(id!) })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.statusHistory(id!) })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.reviewers(id!) })
      onStatusChangeSuccess()
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.statusChangeFailed')),
        variant: 'destructive',
      })
    },
  })

  const assignCoEditorMutation = useMutation({
    mutationFn: async (data: AssignCoEditorRequest) => api.post(`/protocols/${id}/co-editors`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.detail(id!) })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.coEditors(id!) })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.tables.assignCoeditorFailed')),
        variant: 'destructive',
      })
    },
  })

  const addCommentMutation = useMutation({
    mutationFn: async (content: string) => {
      if (!versions || versions.length === 0) throw new Error('No version found')
      return api.post('/reviews/comments', {
        protocol_version_id: versions[0].id,
        content,
      })
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.comment.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.dialogs.comment.failed')),
        variant: 'destructive',
      })
    },
  })

  return {
    submitMutation,
    changeStatusMutation,
    assignCoEditorMutation,
    addCommentMutation,
  }
}
