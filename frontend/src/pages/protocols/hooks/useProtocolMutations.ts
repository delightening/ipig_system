import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import type {
  ChangeStatusRequest,
} from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import type { ProtocolVersion } from '@/types/aup'

interface UseProtocolMutationsOptions {
  id: string | undefined
  versions: ProtocolVersion[] | undefined
  onStatusChangeSuccess: () => void
  onSoftDeleteSuccess: () => void
}

export function useProtocolMutations({
  id,
  versions,
  onStatusChangeSuccess,
  onSoftDeleteSuccess,
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

  // Admin 軟刪除「已否決」計畫（設為 DELETED，從列表隱藏）
  const softDeleteMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/soft-delete`),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.softDeleteSuccess') })
      queryClient.invalidateQueries({ queryKey: queryKeys.protocols.all })
      queryClient.invalidateQueries({ queryKey: queryKeys.myProjects.all })
      onSoftDeleteSuccess()
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.softDeleteFailed')),
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
    softDeleteMutation,
    addCommentMutation,
  }
}
