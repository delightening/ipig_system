import { useState, useMemo, useCallback, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import type {
  ProtocolResponse,
  ProtocolStatus,
  ChangeStatusRequest,
  AssignCoEditorRequest,
  User,
} from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { getApiErrorMessage } from '@/lib/validation'
import { toast } from '@/components/ui/use-toast'
import { useAuthStore } from '@/stores/auth'
import { useSidebarStore } from '@/stores/sidebar'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { allowedTransitions, REVIEWABLE_STATUSES } from '../constants'
import type { TabKey } from '../constants'
import type { ProtocolVersion } from '@/types/aup'

interface ReviewerOption {
  id: string
  email: string
  display_name: string
}

export function useProtocolDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { user } = useAuthStore()
  const { dialogState, confirm } = useConfirmDialog()

  const [activeTab, setActiveTab] = useState<TabKey>('content')
  const [showStatusDialog, setShowStatusDialog] = useState(false)
  const [newStatus, setNewStatus] = useState<ProtocolStatus | ''>('')
  const [statusRemark, setStatusRemark] = useState('')
  const [selectedReviewerIds, setSelectedReviewerIds] = useState<string[]>([])
  const [selectedCoEditorId, setSelectedCoEditorId] = useState('')
  const [showCommentPanel, setShowCommentPanel] = useState(false)
  const { sidebarOpen, setSidebarOpen } = useSidebarStore()
  const sidebarWasOpenRef = useRef(true)

  // --- Queries ---

  const { data: protocolResponse, isLoading } = useQuery({
    queryKey: queryKeys.protocols.detail(id!),
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const { protocol, pi_name, pi_email, pi_organization, vet_review } = protocolResponse || {}

  const { data: allUsers } = useQuery({
    queryKey: queryKeys.users.all,
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
    enabled: showStatusDialog,
  })

  // --- Computed values ---

  const isVetReviewer = useMemo(() => {
    if (!user || !vet_review) return false
    return vet_review.vet_id === user.id
  }, [user, vet_review])

  const availableReviewers = useMemo<ReviewerOption[] | undefined>(() =>
    allUsers?.filter(u => u.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
      .map(u => ({ id: u.id, email: u.email, display_name: u.display_name || u.email })),
    [allUsers]
  )

  const availableExperimentStaff = useMemo<ReviewerOption[] | undefined>(() =>
    allUsers?.filter(u => u.roles?.includes('EXPERIMENT_STAFF'))
      .map(u => ({ id: u.id, email: u.email, display_name: u.display_name || u.email })),
    [allUsers]
  )

  const availableTransitions = useMemo(() => {
    if (!protocol) return []
    return allowedTransitions[protocol.status] || []
  }, [protocol])

  const cleanedWorkingContent = useMemo(() => {
    if (!protocol?.working_content) return null
    const cleanedContent = JSON.parse(JSON.stringify(protocol.working_content))
    if (cleanedContent.basic && cleanedContent.basic.apply_study_number !== undefined) {
      delete cleanedContent.basic.apply_study_number
    }
    return cleanedContent
  }, [protocol?.working_content])

  const isVet = user?.roles?.includes('VET')
  const isReviewer = user?.roles?.some(r => ['REVIEWER', 'VET'].includes(r))
  const isIACUCOrAdmin = user?.roles?.some(r =>
    ['IACUC_CHAIR', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )
  const canAddComment = isIACUCOrAdmin || (
    isReviewer && protocol?.status && REVIEWABLE_STATUSES.includes(protocol.status)
  )
  const canReply = user?.roles?.some(r =>
    ['PI', 'EXPERIMENT_STAFF', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )
  const canEditProtocol = user?.roles?.some(r =>
    ['PI', 'EXPERIMENT_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )
  const canAssignReviewer = user?.roles?.some(r =>
    ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )
  const isRevisionStatus = protocol?.status === 'REVISION_REQUIRED'
    || protocol?.status === 'PRE_REVIEW_REVISION_REQUIRED'
    || protocol?.status === 'VET_REVISION_REQUIRED'
  const canManageAttachments = protocol?.status === 'DRAFT' || isRevisionStatus
  const shouldAnonymizeReviewers = !user?.roles?.some(r =>
    ['IACUC_STAFF', 'IACUC_CHAIR', 'REVIEWER', 'VET', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )

  // --- Comment panel queries ---

  const canShowPanel = !!canAddComment && protocol?.status !== 'DRAFT'

  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', id],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${id}/versions`)
      return response.data
    },
    enabled: !!id && canShowPanel,
  })

  // --- Mutations ---

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
      setShowStatusDialog(false)
      setNewStatus('')
      setStatusRemark('')
      setSelectedReviewerIds([])
      setSelectedCoEditorId('')
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

  // --- Handlers ---

  const handleSubmit = useCallback(async () => {
    const ok = await confirm({
      title: '送出計畫書',
      description: t('protocols.detail.submitConfirm'),
      confirmLabel: '確認送出',
    })
    if (ok) submitMutation.mutate()
  }, [confirm, t, submitMutation])

  const handleChangeStatus = useCallback(async () => {
    if (!newStatus) return

    if (newStatus === 'UNDER_REVIEW') {
      if (selectedReviewerIds.length < 2 || selectedReviewerIds.length > 3) {
        toast({
          title: t('common.error'),
          description: t('protocols.detail.dialogs.status.selected', { count: selectedReviewerIds.length }),
          variant: 'destructive',
        })
        return
      }
    }

    if (newStatus === 'PRE_REVIEW' && !selectedCoEditorId) {
      toast({
        title: t('common.error'),
        description: t('protocols.detail.tables.assignCoeditorFailed'),
        variant: 'destructive',
      })
      return
    }

    try {
      if (newStatus === 'PRE_REVIEW' && selectedCoEditorId) {
        await assignCoEditorMutation.mutateAsync({
          protocol_id: id!,
          user_id: selectedCoEditorId,
        })
      }
      await changeStatusMutation.mutateAsync({
        to_status: newStatus,
        remark: statusRemark || undefined,
        reviewer_ids: newStatus === 'UNDER_REVIEW' ? selectedReviewerIds : undefined,
      })
    } catch {
      // Errors handled in mutation callbacks
    }
  }, [newStatus, selectedReviewerIds, selectedCoEditorId, id, assignCoEditorMutation, changeStatusMutation, statusRemark, t])

  const handleReviewerToggle = useCallback((reviewerId: string, checked: boolean) => {
    setSelectedReviewerIds(prev =>
      checked ? [...prev, reviewerId] : prev.filter(r => r !== reviewerId)
    )
  }, [])

  const handleToggleCommentPanel = useCallback(() => {
    setShowCommentPanel(prev => {
      const willOpen = !prev
      if (willOpen) {
        sidebarWasOpenRef.current = sidebarOpen
        setSidebarOpen(false)
      } else {
        if (sidebarWasOpenRef.current) {
          setSidebarOpen(true)
        }
      }
      return willOpen
    })
  }, [sidebarOpen, setSidebarOpen])

  const sectionOptions = useMemo(() => [
    t('protocols.content.sections.researchInfo'),
    t('protocols.content.sections.purpose'),
    t('protocols.content.sections.items'),
    t('protocols.content.sections.design'),
    t('protocols.content.sections.guidelines'),
    t('protocols.content.sections.surgery'),
    t('protocols.content.sections.animals'),
    t('protocols.content.sections.personnel'),
    t('protocols.content.sections.attachments'),
    t('protocols.content.sections.signatures'),
  ], [t])

  return {
    id,
    navigate,
    t,
    protocol,
    pi_name,
    pi_email,
    pi_organization,
    vet_review,
    isLoading,
    activeTab,
    setActiveTab,
    showStatusDialog,
    setShowStatusDialog,
    newStatus,
    setNewStatus,
    statusRemark,
    setStatusRemark,
    selectedReviewerIds,
    selectedCoEditorId,
    setSelectedCoEditorId,
    showCommentPanel,
    cleanedWorkingContent,
    availableTransitions,
    availableReviewers,
    availableExperimentStaff,
    isVetReviewer,
    isVet,
    canAddComment: !!canAddComment,
    canReply: !!canReply,
    canEditProtocol: !!canEditProtocol,
    canAssignReviewer: !!canAssignReviewer,
    isRevisionStatus,
    canManageAttachments: !!canManageAttachments,
    shouldAnonymizeReviewers,
    canShowPanel,
    sectionOptions,
    submitMutation,
    changeStatusMutation,
    assignCoEditorMutation,
    addCommentMutation,
    handleSubmit,
    handleChangeStatus,
    handleReviewerToggle,
    handleToggleCommentPanel,
    dialogState,
  }
}
