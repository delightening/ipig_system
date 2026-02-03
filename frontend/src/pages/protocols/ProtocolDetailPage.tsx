import React, { useState, useRef, useMemo } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolVersion,
  ProtocolStatusHistory,
  ReviewCommentResponse,
  ReviewAssignmentResponse,
  ProtocolAttachment,
  ProtocolStatus,
  protocolStatusNames,
  ChangeStatusRequest,
  CreateCommentRequest,
  ReplyCommentRequest,
  AssignReviewerRequest,
  AssignCoEditorRequest,
  CoEditorAssignmentResponse,
  UserSimple,
  User,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Edit,
  Send,
  CheckCircle,
  XCircle,
  Clock,
  History,
  MessageSquare,
  FileText,
  User as UserIcon,
  Building,
  Calendar,
  Loader2,
  AlertTriangle,
  UserPlus,
  Paperclip,
  Upload,
  Download,
  Trash2,
  Eye,
  CheckCircle2,
  Users,
  ClipboardList,
  Reply,
  FileEdit,
} from 'lucide-react'
import { formatDate, formatDateTime, formatFileSize } from '@/lib/utils'
import { useAuthStore } from '@/stores/auth'
import { ProtocolContentView } from '@/components/protocol/ProtocolContentView'
import { AmendmentsTab } from '@/components/protocol/AmendmentsTab'
import { ProtocolComparisonDialog } from '@/components/protocols/ProtocolComparisonDialog'

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
  DELETED: 'destructive',
}

// 狀態轉換規則
const allowedTransitions: Record<ProtocolStatus, ProtocolStatus[]> = {
  DRAFT: ['SUBMITTED'],
  SUBMITTED: ['PRE_REVIEW'],
  PRE_REVIEW: ['UNDER_REVIEW'],
  UNDER_REVIEW: ['REVISION_REQUIRED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'REJECTED', 'DEFERRED'],
  REVISION_REQUIRED: ['RESUBMITTED'],
  RESUBMITTED: ['PRE_REVIEW', 'UNDER_REVIEW'],
  APPROVED: ['SUSPENDED', 'CLOSED'],
  APPROVED_WITH_CONDITIONS: ['SUSPENDED', 'CLOSED'],
  DEFERRED: ['UNDER_REVIEW', 'CLOSED'],
  REJECTED: ['CLOSED'],
  SUSPENDED: ['UNDER_REVIEW', 'CLOSED'],
  CLOSED: [],
  DELETED: [],
}

export function ProtocolDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { user } = useAuthStore()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [activeTab, setActiveTab] = useState<'content' | 'versions' | 'history' | 'comments' | 'reviewers' | 'coeditors' | 'attachments' | 'pigs' | 'amendments'>('content')
  const [showStatusDialog, setShowStatusDialog] = useState(false)
  const [showCommentDialog, setShowCommentDialog] = useState(false)
  const [showReplyDialog, setShowReplyDialog] = useState(false)
  const [showAssignDialog, setShowAssignDialog] = useState(false)
  const [showVersionDialog, setShowVersionDialog] = useState(false)
  const [showCoEditorDialog, setShowCoEditorDialog] = useState(false)
  const [selectedVersion, setSelectedVersion] = useState<ProtocolVersion | null>(null)
  const [selectedCommentForReply, setSelectedCommentForReply] = useState<ReviewCommentResponse | null>(null)
  const [newStatus, setNewStatus] = useState<ProtocolStatus | ''>('')
  const [statusRemark, setStatusRemark] = useState('')
  const [commentContent, setCommentContent] = useState('')
  const [replyContent, setReplyContent] = useState('')
  const [selectedReviewerId, setSelectedReviewerId] = useState('')
  const [selectedReviewerIds, setSelectedReviewerIds] = useState<string[]>([])
  const [selectedCoEditorId, setSelectedCoEditorId] = useState('')

  // Comparison state
  const [comparisonOpen, setComparisonOpen] = useState(false)
  const [versionA, setVersionA] = useState<ProtocolVersion | null>(null)
  const [versionB, setVersionB] = useState<ProtocolVersion | null>(null)
  const [compareMode, setCompareMode] = useState(false)
  const [selectedCompareIds, setSelectedCompareIds] = useState<string[]>([])

  // Fetch protocol details
  const { data: protocol, isLoading } = useQuery({
    queryKey: ['protocol', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  // Fetch version list
  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', id],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${id}/versions`)
      return response.data
    },
    enabled: !!id && (activeTab === 'versions' || activeTab === 'comments'),
  })

  // Fetch status history
  const { data: statusHistory } = useQuery({
    queryKey: ['protocol-status-history', id],
    queryFn: async () => {
      const response = await api.get<ProtocolStatusHistory[]>(`/protocols/${id}/status-history`)
      return response.data
    },
    enabled: !!id && activeTab === 'history',
  })

  // Fetch review comments
  const { data: comments, isLoading: commentsLoading } = useQuery({
    queryKey: ['protocol-comments', id, versions?.[0]?.id],
    queryFn: async () => {
      if (!versions || versions.length === 0) return []
      const latestVersionId = versions[0].id
      const response = await api.get<ReviewCommentResponse[]>(`/reviews/comments`, {
        params: { protocol_version_id: latestVersionId }
      })
      return response.data
    },
    enabled: !!id && !!versions && versions.length > 0 && activeTab === 'comments',
  })

  // Fetch review assignments
  const { data: reviewers, isLoading: reviewersLoading } = useQuery({
    queryKey: ['protocol-reviewers', id],
    queryFn: async () => {
      const response = await api.get<ReviewAssignmentResponse[]>(`/reviews/assignments`, {
        params: { protocol_id: id }
      })
      return response.data
    },
    enabled: !!id && activeTab === 'reviewers',
  })

  // Fetch attachments
  const { data: attachments, isLoading: attachmentsLoading } = useQuery({
    queryKey: ['protocol-attachments', id],
    queryFn: async () => {
      const response = await api.get<ProtocolAttachment[]>(`/attachments`, {
        params: { protocol_id: id }
      })
      return response.data
    },
    enabled: !!id && activeTab === 'attachments',
  })

  // Fetch co-editor list
  const { data: coEditors, isLoading: coEditorsLoading } = useQuery({
    queryKey: ['protocol-co-editors', id],
    queryFn: async () => {
      const response = await api.get<CoEditorAssignmentResponse[]>(`/protocols/${id}/co-editors`)
      return response.data
    },
    enabled: !!id && activeTab === 'coeditors',
  })

  // Fetch assignable reviewers
  const { data: availableReviewers } = useQuery({
    queryKey: ['available-reviewers'],
    queryFn: async () => {
      // 获取所有用户，然后在前端过滤出 REVIEWER 和 VET 角色
      const response = await api.get<User[]>('/users')
      // 过滤出具有 REVIEWER 或 VET 角色的用户
      return response.data
        .filter(user => user.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: showAssignDialog || (showStatusDialog && newStatus === 'UNDER_REVIEW'),
  })

  // Fetch assignable experiment staff (co-editor)
  const { data: availableExperimentStaff } = useQuery({
    queryKey: ['available-experiment-staff'],
    queryFn: async () => {
      // 获取所有用户，然后在前端过滤出 EXPERIMENT_STAFF 角色
      const response = await api.get<User[]>('/users')
      // 过滤出具有 EXPERIMENT_STAFF 角色的用户
      return response.data
        .filter(user => user.roles?.includes('EXPERIMENT_STAFF'))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: (showStatusDialog && newStatus === 'PRE_REVIEW') || showCoEditorDialog,
  })

  // Submit protocol mutation
  const submitMutation = useMutation({
    mutationFn: async () => {
      return api.post(`/protocols/${id}/submit`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.submitSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.submitFailed'),
        variant: 'destructive',
      })
    },
  })

  // Change status mutation
  const changeStatusMutation = useMutation({
    mutationFn: async (data: ChangeStatusRequest) => {
      return api.post(`/protocols/${id}/status`, data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.statusChangeSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocol-status-history', id] })
      queryClient.invalidateQueries({ queryKey: ['protocol-reviewers', id] })
      setShowStatusDialog(false)
      setNewStatus('')
      setStatusRemark('')
      setSelectedReviewerIds([])
      setSelectedCoEditorId('')
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.statusChangeFailed'),
        variant: 'destructive',
      })
    },
  })

  // Add review comment mutation
  const addCommentMutation = useMutation({
    mutationFn: async (data: CreateCommentRequest) => {
      return api.post('/reviews/comments', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.comment.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
      setShowCommentDialog(false)
      setCommentContent('')
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.dialogs.comment.failed'),
        variant: 'destructive',
      })
    },
  })

  // Reply to review comment mutation
  const replyCommentMutation = useMutation({
    mutationFn: async (data: ReplyCommentRequest) => {
      return api.post('/reviews/comments/reply', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.reply.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
      setShowReplyDialog(false)
      setReplyContent('')
      setSelectedCommentForReply(null)
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.dialogs.reply.failed'),
        variant: 'destructive',
      })
    },
  })

  // Resolve review comment mutation
  const resolveCommentMutation = useMutation({
    mutationFn: async (commentId: string) => {
      return api.post(`/reviews/comments/${commentId}/resolve`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.resolved') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('common.error'),
        variant: 'destructive',
      })
    },
  })

  // Assign reviewer mutation
  const assignReviewerMutation = useMutation({
    mutationFn: async (data: AssignReviewerRequest) => {
      return api.post('/reviews/assignments', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.assignSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-reviewers', id] })
      setShowAssignDialog(false)
      setSelectedReviewerId('')
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.tables.assignFailed'),
        variant: 'destructive',
      })
    },
  })

  // Assign co-editor mutation
  const assignCoEditorMutation = useMutation({
    mutationFn: async (data: AssignCoEditorRequest) => {
      return api.post(`/protocols/${id}/co-editors`, data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.assignCoeditorSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.tables.assignCoeditorFailed'),
        variant: 'destructive',
      })
    },
  })

  // Remove co-editor mutation
  const removeCoEditorMutation = useMutation({
    mutationFn: async (userId: string) => {
      return api.delete(`/protocols/${id}/co-editors/${userId}`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.removeCoeditorSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.actions.removeCoeditorFailed'),
        variant: 'destructive',
      })
    },
  })

  // Upload attachment mutation
  const uploadAttachmentMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      return api.post(`/attachments?protocol_id=${id}`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.uploadSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.tables.uploadFailed'),
        variant: 'destructive',
      })
    },
  })

  // Delete attachment mutation
  const deleteAttachmentMutation = useMutation({
    mutationFn: async (attachmentId: string) => {
      return api.delete(`/attachments/${attachmentId}`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.deleteSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', id] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.actions.deleteFailed'),
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = () => {
    if (confirm(t('protocols.detail.submitConfirm'))) {
      submitMutation.mutate()
    }
  }

  const handleChangeStatus = async () => {
    if (!newStatus) return

    // Validate UNDER_REVIEW must select 2-3 reviewers
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

    // Change status first
    try {
      await changeStatusMutation.mutateAsync({
        to_status: newStatus,
        remark: statusRemark || undefined,
        reviewer_ids: newStatus === 'UNDER_REVIEW' ? selectedReviewerIds : undefined,
      })

      // If target status is PRE_REVIEW and co-editor is selected, assign co-editor
      if (newStatus === 'PRE_REVIEW' && selectedCoEditorId && id) {
        try {
          await assignCoEditorMutation.mutateAsync({
            protocol_id: id,
            user_id: selectedCoEditorId,
          })
        } catch (error) {
          // co-editor 指派失敗不影響狀態變更，只顯示警告
          toast({
            title: t('common.warning'),
            description: t('protocols.detail.tables.assignCoeditorFailedWarning'),
            variant: 'destructive',
          })
        }
      }
    } catch (error) {
      // 錯誤已在 mutation 中處理
    }
  }

  const handleAddComment = () => {
    if (!commentContent.trim() || !versions || versions.length === 0) return
    addCommentMutation.mutate({
      protocol_version_id: versions[0].id,
      content: commentContent.trim(),
    })
  }

  const handleAssignReviewer = () => {
    if (!selectedReviewerId || !id) return
    assignReviewerMutation.mutate({
      protocol_id: id,
      reviewer_id: selectedReviewerId,
    })
  }

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      uploadAttachmentMutation.mutate(file)
    }
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const handleDownloadAttachment = async (attachment: ProtocolAttachment) => {
    try {
      const response = await api.get(`/attachments/${attachment.id}/download`, {
        responseType: 'blob'
      })
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', attachment.file_name)
      document.body.appendChild(link)
      link.click()
      link.remove()
    } catch (error) {
      toast({ title: t('common.error'), description: t('common.downloadFailed'), variant: 'destructive' })
    }
  }

  const getAvailableTransitions = () => {
    if (!protocol) return []
    return allowedTransitions[protocol.status] || []
  }

  // Reviewers can only add comments in UNDER_REVIEW status
  const isReviewer = user?.roles?.some(r => ['REVIEWER', 'VET'].includes(r))
  const isIACUCOrAdmin = user?.roles?.some(r => ['IACUC_CHAIR', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const canAddComment = isIACUCOrAdmin || (isReviewer && protocol?.status === 'UNDER_REVIEW')

  // PI, co-editor, and IACUC_STAFF can reply to review comments
  const canReply = user?.roles?.some(r => ['PI', 'EXPERIMENT_STAFF', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))

  // Only PI and co-editor can edit/revise the protocol
  const canEditProtocol = user?.roles?.some(r => ['PI', 'EXPERIMENT_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))

  const canAssignReviewer = user?.roles?.some(r => ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const canManageAttachments = protocol?.status === 'DRAFT' || protocol?.status === 'REVISION_REQUIRED'

  // Reviewer anonymization logic
  // PI/Client can only see "Reviewer A/B/C", IACUC staff and other reviewers see real names
  const shouldAnonymizeReviewers = !user?.roles?.some(r =>
    ['IACUC_STAFF', 'IACUC_CHAIR', 'REVIEWER', 'VET', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )

  // Create anonymous mapping: Shuffling reviewer order randomly
  const reviewerAnonymousMap = React.useMemo(() => {
    if (!comments) return new Map<string, string>()

    const uniqueReviewerIds = Array.from(
      new Set(comments.map(c => c.reviewer_id).filter(Boolean))
    )

    // 使用穩定的隨機排序（基於 protocol ID 作為種子）
    const seed = id?.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) || 0
    const shuffled = [...uniqueReviewerIds].sort((a, b) => {
      const hashA = ((seed * a.charCodeAt(0)) % 26)
      const hashB = ((seed * b.charCodeAt(0)) % 26)
      return hashA - hashB
    })

    const map = new Map<string, string>()
    shuffled.forEach((reviewerId, index) => {
      const letter = String.fromCharCode(65 + index) // A, B, C, ...
      map.set(reviewerId, t(`protocols.detail.actions.reviewer${letter}`))
    })

    return map
  }, [comments, id])

  // Get anonymized reviewer display name
  const getReviewerDisplayName = (comment: ReviewCommentResponse) => {
    if (!shouldAnonymizeReviewers) {
      return comment.reviewer_name || comment.reviewer_email
    }
    return reviewerAnonymousMap.get(comment.reviewer_id) || t('protocols.detail.actions.reviewer')
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!protocol) {
    return (
      <div className="text-center py-12">
        <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-yellow-500" />
        <h2 className="text-xl font-semibold mb-2">{t('protocols.detail.notFound')}</h2>
        <p className="text-muted-foreground mb-4">{t('protocols.detail.notFoundDesc')}</p>
        <Button asChild>
          <Link to="/protocols">{t('protocols.detail.backToList')}</Link>
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate(-1)}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <div className="flex items-center gap-3">
              <h1 className="text-2xl font-bold">{protocol.title}</h1>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          {protocol.status === 'DRAFT' && (
            <>
              <Button variant="outline" asChild>
                <Link to={`/protocols/${id}/edit`}>
                  <Edit className="mr-2 h-4 w-4" />
                  {t('protocols.detail.edit')}
                </Link>
              </Button>
              <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
                {submitMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Send className="mr-2 h-4 w-4" />
                )}
                {t('protocols.detail.submit')}
              </Button>
            </>
          )}
          {protocol.status === 'REVISION_REQUIRED' && canEditProtocol && (
            <Button variant="outline" asChild>
              <Link to={`/protocols/${id}/edit`}>
                <Edit className="mr-2 h-4 w-4" />
                {t('protocols.detail.revise')}
              </Link>
            </Button>
          )}
          {/* 只有 IACUC_STAFF、CHAIR、SYSTEM_ADMIN 或 admin 可以變更狀態 */}
          {getAvailableTransitions().length > 0 && protocol.status !== 'DRAFT' &&
            user?.roles?.some(r => ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN', 'admin'].includes(r)) && (
              <Button variant="outline" onClick={() => setShowStatusDialog(true)}>
                {t('protocols.detail.changeStatus')}
              </Button>
            )}
        </div>
      </div>

      {/* Info Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <FileText className="h-4 w-4 text-blue-500" />
              {protocol.iacuc_no?.startsWith('APIG-') ? t('protocols.detail.info.apigNo') : t('protocols.detail.info.iacucNo')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold text-orange-600">
              {protocol.iacuc_no || t('protocols.detail.info.notIssued')}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <UserIcon className="h-4 w-4 text-green-500" />
              {t('protocols.detail.info.pi')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">{protocol.pi_name || '-'}</p>
            <p className="text-sm text-muted-foreground">{protocol.pi_email}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Building className="h-4 w-4 text-purple-500" />
              {t('protocols.detail.info.organization')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">{protocol.pi_organization || '-'}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Calendar className="h-4 w-4 text-yellow-500" />
              {t('protocols.detail.info.period')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">
              {protocol.start_date && protocol.end_date
                ? `${formatDate(protocol.start_date)} ~ ${formatDate(protocol.end_date)}`
                : t('protocols.detail.info.notSet')}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <div className="border-b">
        <nav className="flex gap-4 overflow-x-auto">
          {[
            { key: 'content', label: t('protocols.detail.tabs.content'), icon: FileText },
            { key: 'pigs', label: t('protocols.detail.tabs.pigs'), icon: ClipboardList },
            { key: 'versions', label: t('protocols.detail.tabs.versions'), icon: History },
            { key: 'history', label: t('protocols.detail.tabs.history'), icon: Clock },
            { key: 'comments', label: t('protocols.detail.tabs.comments'), icon: MessageSquare },
            { key: 'reviewers', label: t('protocols.detail.tabs.reviewers'), icon: Users },
            { key: 'coeditors', label: t('protocols.detail.tabs.coeditors'), icon: UserPlus },
            { key: 'attachments', label: t('protocols.detail.tabs.attachments'), icon: Paperclip },
            { key: 'amendments', label: t('protocols.detail.tabs.amendments'), icon: FileEdit },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key as typeof activeTab)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors whitespace-nowrap ${activeTab === tab.key
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-muted-foreground hover:text-foreground'
                }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      {activeTab === 'content' && (
        <Card>
          <CardHeader>
            <CardTitle>{t('protocols.detail.sections.contentTitle')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.contentDesc')}</CardDescription>
          </CardHeader>
          <CardContent>
            <ProtocolContentView
              workingContent={(() => {
                if (!protocol.working_content) return null
                // 創建一個副本，去除 apply_study_number 字段
                const cleanedContent = JSON.parse(JSON.stringify(protocol.working_content))
                if (cleanedContent.basic && cleanedContent.basic.apply_study_number !== undefined) {
                  delete cleanedContent.basic.apply_study_number
                }
                return cleanedContent
              })()}
              protocolTitle={protocol.title}
              protocolId={id}
              startDate={protocol.start_date}
              endDate={protocol.end_date}
            />
          </CardContent>
        </Card>
      )}

      {activeTab === 'versions' && (
        <Card>
          <CardHeader>
            <div className="flex justify-between items-center mb-4">
              <CardDescription>{t('protocols.detail.sections.versionsDesc')}</CardDescription>
              {versions && versions.length >= 2 && (
                <Button
                  variant={compareMode ? "destructive" : "outline"}
                  size="sm"
                  onClick={() => {
                    setCompareMode(!compareMode)
                    setSelectedCompareIds([])
                  }}
                >
                  {compareMode ? t('common.cancel') : '比較版本'}
                </Button>
              )}
            </div>
          </CardHeader>
          <CardContent>
            {versions && versions.length > 0 ? (
              <div className="space-y-4">
                {compareMode && selectedCompareIds.length === 2 && (
                  <div className="bg-blue-50 border border-blue-100 p-3 rounded-md flex justify-between items-center">
                    <span className="text-sm font-medium text-blue-700">
                      已選擇兩個版本進行比較
                    </span>
                    <Button
                      size="sm"
                      onClick={() => {
                        const vA = versions.find(v => v.id === selectedCompareIds[0])
                        const vB = versions.find(v => v.id === selectedCompareIds[1])
                        if (vA && vB) {
                          // Ensure A is the older version based on version_no
                          if (vA.version_no > vB.version_no) {
                            setVersionA(vB)
                            setVersionB(vA)
                          } else {
                            setVersionA(vA)
                            setVersionB(vB)
                          }
                          setComparisonOpen(true)
                        }
                      }}
                    >
                      開始比較
                    </Button>
                  </div>
                )}
                <Table>
                  <TableHeader>
                    <TableRow>
                      {compareMode && <TableHead className="w-[50px]"></TableHead>}
                      <TableHead>{t('protocols.detail.tables.versionNo')}</TableHead>
                      <TableHead>{t('protocols.detail.tables.submitTime')}</TableHead>
                      <TableHead>{t('protocols.detail.tables.actions')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {versions.map((version) => (
                      <TableRow key={version.id}>
                        {compareMode && (
                          <TableCell>
                            <Checkbox
                              checked={selectedCompareIds.includes(version.id)}
                              onCheckedChange={(checked: boolean) => {
                                if (checked) {
                                  if (selectedCompareIds.length < 2) {
                                    setSelectedCompareIds([...selectedCompareIds, version.id])
                                  }
                                } else {
                                  setSelectedCompareIds(selectedCompareIds.filter(id => id !== version.id))
                                }
                              }}
                              disabled={!selectedCompareIds.includes(version.id) && selectedCompareIds.length >= 2}
                            />
                          </TableCell>
                        )}
                        <TableCell className="font-medium">v{version.version_no}</TableCell>
                        <TableCell>{formatDateTime(version.submitted_at)}</TableCell>
                        <TableCell>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setSelectedVersion(version)
                              setShowVersionDialog(true)
                            }}
                          >
                            <Eye className="mr-1 h-4 w-4" />
                            {t('protocols.detail.tables.viewContent')}
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <History className="h-12 w-12 mx-auto mb-2" />
                <p>{t('protocols.detail.tables.noVersions')}</p>
              </div>
            )}

          </CardContent>
        </Card>
      )}

      {activeTab === 'history' && (
        <Card>
          <CardHeader>
            <CardTitle>{t('protocols.detail.sections.historyTitle')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.historyDesc')}</CardDescription>
          </CardHeader>
          <CardContent>
            {statusHistory && statusHistory.length > 0 ? (
              <div className="relative">
                <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-slate-200" />
                <ul className="space-y-4">
                  {statusHistory.map((history, index) => (
                    <li key={history.id} className="relative pl-10">
                      <div className="absolute left-2 w-4 h-4 rounded-full bg-blue-600 border-2 border-white" />
                      <div className="bg-slate-50 p-4 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          {history.from_status && (
                            <>
                              <Badge variant={statusColors[history.from_status]}>
                                {protocolStatusNames[history.from_status]}
                              </Badge>
                              <span className="text-muted-foreground">→</span>
                            </>
                          )}
                          <Badge variant={statusColors[history.to_status]}>
                            {protocolStatusNames[history.to_status]}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          {formatDateTime(history.created_at)}
                        </p>
                        {history.remark && (
                          <p className="text-sm mt-2 text-slate-600">{history.remark}</p>
                        )}
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Clock className="h-12 w-12 mx-auto mb-2" />
                <p>{t('protocols.detail.tables.noHistory')}</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'comments' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>{t('protocols.detail.sections.commentsTitle')}</CardTitle>
              <CardDescription>{t('protocols.detail.sections.commentsDesc')}</CardDescription>
            </div>
            {canAddComment && protocol.status !== 'DRAFT' && (
              <Button onClick={() => setShowCommentDialog(true)}>
                <MessageSquare className="mr-2 h-4 w-4" />
                {t('protocols.detail.dialogs.comment.title')}
              </Button>
            )}
          </CardHeader>
          <CardContent>
            {commentsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : comments && comments.length > 0 ? (
              <div className="space-y-4">
                {comments
                  .filter(c => !c.parent_comment_id) // 只顯示主評論
                  .map((comment) => (
                    <div key={comment.id} className="space-y-2">
                      <div
                        className={`p-4 rounded-lg border ${comment.is_resolved ? 'bg-green-50 border-green-200' : 'bg-white border-slate-200'}`}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex items-center gap-2">
                            <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center">
                              <UserIcon className="h-4 w-4 text-blue-600" />
                            </div>
                            <div>
                              <p className="font-medium">{getReviewerDisplayName(comment)}</p>
                              <p className="text-xs text-muted-foreground">{formatDateTime(comment.created_at)}</p>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {canReply && !comment.is_resolved && (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                  setSelectedCommentForReply(comment)
                                  setShowReplyDialog(true)
                                }}
                              >
                                <Reply className="mr-1 h-4 w-4" />
                                {t('protocols.detail.actions.reply')}
                              </Button>
                            )}
                            {comment.is_resolved ? (
                              <Badge variant="success" className="flex items-center gap-1">
                                <CheckCircle2 className="h-3 w-3" />
                                {t('protocols.detail.actions.resolved')}
                              </Badge>
                            ) : (
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => resolveCommentMutation.mutate(comment.id)}
                                disabled={resolveCommentMutation.isPending}
                              >
                                {resolveCommentMutation.isPending ? (
                                  <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                  <>
                                    <CheckCircle className="mr-1 h-4 w-4" />
                                    {t('protocols.detail.actions.markResolved')}
                                  </>
                                )}
                              </Button>
                            )}
                          </div>
                        </div>
                        <p className="mt-3 text-sm text-slate-700 whitespace-pre-wrap">{comment.content}</p>
                      </div>

                      {/* 顯示回覆 */}
                      {comments
                        .filter(reply => reply.parent_comment_id === comment.id)
                        .map(reply => (
                          <div key={reply.id} className="ml-8 p-4 rounded-lg border bg-slate-50 border-slate-200">
                            <div className="flex items-center gap-2">
                              <div className="h-6 w-6 rounded-full bg-green-100 flex items-center justify-center">
                                <Reply className="h-3 w-3 text-green-600" />
                              </div>
                              <div>
                                <p className="font-medium text-sm">{reply.replied_by_name || reply.reviewer_name || reply.reviewer_email}</p>
                                <p className="text-xs text-muted-foreground">{formatDateTime(reply.created_at)}</p>
                              </div>
                            </div>
                            <p className="mt-2 text-sm text-slate-700 whitespace-pre-wrap">{reply.content}</p>
                          </div>
                        ))}
                    </div>
                  ))}
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <MessageSquare className="h-12 w-12 mx-auto mb-2" />
                <p>{t('protocols.detail.tables.noComments')}</p>
                {canAddComment && protocol.status !== 'DRAFT' && (
                  <Button variant="link" onClick={() => setShowCommentDialog(true)} className="mt-2">
                    {t('protocols.detail.dialogs.comment.firstComment')}
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'reviewers' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>{t('protocols.detail.sections.reviewersTitle')}</CardTitle>
              <CardDescription>{t('protocols.detail.sections.reviewersDesc')}</CardDescription>
            </div>
            {canAssignReviewer && protocol.status !== 'DRAFT' && protocol.status !== 'CLOSED' && (
              <Button onClick={() => setShowAssignDialog(true)}>
                <UserPlus className="mr-2 h-4 w-4" />
                {t('protocols.detail.dialogs.assign.title')}
              </Button>
            )}
          </CardHeader>
          <CardContent>
            {reviewersLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : reviewers && reviewers.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t('protocols.detail.tables.reviewer')}</TableHead>
                    <TableHead>{t('protocols.detail.tables.assignedTime')}</TableHead>
                    <TableHead>{t('protocols.detail.tables.assignedBy')}</TableHead>
                    <TableHead>{t('protocols.detail.tables.completedTime')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {reviewers.map((reviewer) => (
                    <TableRow key={reviewer.id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">{reviewer.reviewer_name || '-'}</p>
                          <p className="text-sm text-muted-foreground">{reviewer.reviewer_email}</p>
                        </div>
                      </TableCell>
                      <TableCell>{formatDateTime(reviewer.assigned_at)}</TableCell>
                      <TableCell>{reviewer.assigned_by_name || '-'}</TableCell>
                      <TableCell>
                        {reviewer.completed_at ? (
                          <Badge variant="success">{formatDateTime(reviewer.completed_at)}</Badge>
                        ) : (
                          <Badge variant="secondary">{t('protocols.detail.tables.reviewing')}</Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Users className="h-12 w-12 mx-auto mb-2" />
                <p>{t('protocols.detail.tables.noReviewers')}</p>
                {canAssignReviewer && protocol.status !== 'DRAFT' && (
                  <Button variant="link" onClick={() => setShowAssignDialog(true)} className="mt-2">
                    {t('protocols.detail.tables.assignFirst')}
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'coeditors' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>{t('protocols.detail.tabs.coeditors')}</CardTitle>
              <CardDescription>{t('protocols.detail.sections.coeditorsDesc')}</CardDescription>
            </div>
            {canAssignReviewer && (
              <Button onClick={() => setShowCoEditorDialog(true)}>
                <UserPlus className="mr-2 h-4 w-4" />
                + Co-Editor
              </Button>
            )}
          </CardHeader>
          <CardContent>
            {coEditorsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : coEditors && coEditors.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Co-Editor</TableHead>
                    <TableHead>指派時間</TableHead>
                    <TableHead>指派者</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {coEditors.map((coEditor) => (
                    <TableRow key={coEditor.user_id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">{coEditor.user_name}</p>
                          <p className="text-sm text-muted-foreground">{coEditor.user_email}</p>
                        </div>
                      </TableCell>
                      <TableCell>{formatDateTime(coEditor.granted_at)}</TableCell>
                      <TableCell>{coEditor.granted_by_name || '-'}</TableCell>
                      <TableCell>
                        {canAssignReviewer && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              if (confirm('確定要移除此 co-editor 嗎？')) {
                                removeCoEditorMutation.mutate(coEditor.user_id)
                              }
                            }}
                            disabled={removeCoEditorMutation.isPending}
                          >
                            {removeCoEditorMutation.isPending ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <>
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </>
                            )}
                          </Button>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <UserPlus className="h-12 w-12 mx-auto mb-2" />
                <p>尚未指派 co-editor</p>
                <p className="text-sm mt-2">可在狀態變更為「行政預審」時指派 co-editor</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'attachments' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>附件</CardTitle>
              <CardDescription>計畫書相關文件與檔案</CardDescription>
            </div>
            {canManageAttachments && (
              <>
                <input
                  type="file"
                  ref={fileInputRef}
                  onChange={handleFileUpload}
                  className="hidden"
                />
                <Button onClick={() => fileInputRef.current?.click()} disabled={uploadAttachmentMutation.isPending}>
                  {uploadAttachmentMutation.isPending ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Upload className="mr-2 h-4 w-4" />
                  )}
                  上傳附件
                </Button>
              </>
            )}
          </CardHeader>
          <CardContent>
            {attachmentsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : attachments && attachments.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>檔案名稱</TableHead>
                    <TableHead>大小</TableHead>
                    <TableHead>上傳者</TableHead>
                    <TableHead>上傳時間</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {attachments.map((attachment) => (
                    <TableRow key={attachment.id}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Paperclip className="h-4 w-4 text-muted-foreground" />
                          <span className="font-medium">{attachment.file_name}</span>
                        </div>
                      </TableCell>
                      <TableCell>{formatFileSize(attachment.file_size)}</TableCell>
                      <TableCell>{attachment.uploaded_by_name || '-'}</TableCell>
                      <TableCell>{formatDateTime(attachment.created_at)}</TableCell>
                      <TableCell>
                        <div className="flex gap-2">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDownloadAttachment(attachment)}
                          >
                            <Download className="h-4 w-4" />
                          </Button>
                          {canManageAttachments && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                if (confirm('確定要刪除此附件嗎？')) {
                                  deleteAttachmentMutation.mutate(attachment.id)
                                }
                              }}
                              disabled={deleteAttachmentMutation.isPending}
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Paperclip className="h-12 w-12 mx-auto mb-2" />
                <p>尚無附件</p>
                {canManageAttachments && (
                  <Button variant="link" onClick={() => fileInputRef.current?.click()} className="mt-2">
                    上傳第一個附件
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'pigs' && (
        <Card>
          <CardHeader>
            <CardTitle>{t('protocols.detail.tabs.pigs')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.pigsDesc')}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12">
              <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-semibold mb-2">{t('protocols.detail.tables.noPigs')}</h3>
              <p className="text-muted-foreground">
                {t('protocols.detail.tables.noPigsDesc')}
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      {activeTab === 'amendments' && id && (
        <AmendmentsTab protocolId={id} protocolStatus={protocol.status} />
      )}

      {/* Status change dialog */}
      <Dialog open={showStatusDialog} onOpenChange={setShowStatusDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.status.title')}</DialogTitle>
            <DialogDescription>
              {t('protocols.detail.dialogs.status.desc')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.status.current')}</Label>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.status.target')}</Label>
              <Select value={newStatus} onValueChange={(v) => setNewStatus(v as ProtocolStatus)}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.status.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {getAvailableTransitions().map((status) => (
                    <SelectItem key={status} value={status}>
                      {protocolStatusNames[status]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {newStatus === 'PRE_REVIEW' && (
              <div className="space-y-2">
                <Label>{t('protocols.detail.dialogs.status.coeditor')}</Label>
                <Select value={selectedCoEditorId} onValueChange={setSelectedCoEditorId}>
                  <SelectTrigger>
                    <SelectValue placeholder={t('protocols.detail.dialogs.status.coeditorPlaceholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    {availableExperimentStaff?.map((staff) => (
                      <SelectItem key={staff.id} value={staff.id}>
                        {staff.display_name || staff.email}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {t('protocols.detail.dialogs.status.coeditorHint')}
                </p>
              </div>
            )}
            {newStatus === 'UNDER_REVIEW' && (
              <div className="space-y-2">
                <Label>{t('protocols.detail.dialogs.status.reviewers')}</Label>
                <div className="space-y-2 max-h-48 overflow-y-auto border rounded p-2">
                  {availableReviewers?.map((reviewer) => (
                    <label key={reviewer.id} className="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input
                        type="checkbox"
                        checked={selectedReviewerIds.includes(reviewer.id)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedReviewerIds([...selectedReviewerIds, reviewer.id])
                          } else {
                            setSelectedReviewerIds(selectedReviewerIds.filter(id => id !== reviewer.id))
                          }
                        }}
                        className="h-4 w-4"
                      />
                      <span>{reviewer.display_name || reviewer.email}</span>
                    </label>
                  ))}
                  {(!availableReviewers || availableReviewers.length === 0) && (
                    <p className="text-sm text-muted-foreground py-2 text-center">{t('protocols.detail.dialogs.status.noReviewers')}</p>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {t('protocols.detail.dialogs.status.selected', { count: selectedReviewerIds.length })}
                </p>
              </div>
            )}
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.status.remark')}</Label>
              <Textarea
                value={statusRemark}
                onChange={(e) => setStatusRemark(e.target.value)}
                placeholder={t('protocols.detail.dialogs.status.remarkPlaceholder')}
                rows={3}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowStatusDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleChangeStatus}
              disabled={
                !newStatus ||
                changeStatusMutation.isPending ||
                assignCoEditorMutation.isPending ||
                (newStatus === 'UNDER_REVIEW' && (selectedReviewerIds.length < 2 || selectedReviewerIds.length > 3))
              }
            >
              {(changeStatusMutation.isPending || assignCoEditorMutation.isPending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('protocols.detail.dialogs.status.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add review comment dialog */}
      <Dialog open={showCommentDialog} onOpenChange={setShowCommentDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.comment.title')}</DialogTitle>
            <DialogDescription>
              {t('protocols.detail.dialogs.comment.desc')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.comment.placeholder')}</Label>
              <Textarea
                value={commentContent}
                onChange={(e) => setCommentContent(e.target.value)}
                placeholder={t('protocols.detail.dialogs.comment.placeholder')}
                rows={5}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCommentDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAddComment}
              disabled={!commentContent.trim() || addCommentMutation.isPending}
            >
              {addCommentMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('protocols.detail.dialogs.comment.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Reply to review comment dialog */}
      <Dialog open={showReplyDialog} onOpenChange={(open) => {
        setShowReplyDialog(open)
        if (!open) {
          setReplyContent('')
          setSelectedCommentForReply(null)
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.reply.title')}</DialogTitle>
            <DialogDescription>
              {t('protocols.detail.dialogs.reply.desc')}
            </DialogDescription>
          </DialogHeader>
          {selectedCommentForReply && (
            <div className="bg-slate-50 p-3 rounded-lg border mb-4">
              <p className="text-sm font-medium text-slate-600">{t('protocols.detail.dialogs.reply.original')}</p>
              <p className="text-sm text-slate-700 mt-1">{selectedCommentForReply.content}</p>
              <p className="text-xs text-muted-foreground mt-2">
                — {selectedCommentForReply.reviewer_name || selectedCommentForReply.reviewer_email}
              </p>
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.reply.placeholder')}</Label>
              <Textarea
                value={replyContent}
                onChange={(e) => setReplyContent(e.target.value)}
                placeholder={t('protocols.detail.dialogs.reply.placeholder')}
                rows={4}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowReplyDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => {
                if (selectedCommentForReply && replyContent.trim()) {
                  replyCommentMutation.mutate({
                    parent_comment_id: selectedCommentForReply.id,
                    content: replyContent.trim(),
                  })
                }
              }}
              disabled={!replyContent.trim() || replyCommentMutation.isPending}
            >
              {replyCommentMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('protocols.detail.dialogs.reply.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Assign reviewer dialog */}
      <Dialog open={showAssignDialog} onOpenChange={setShowAssignDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.assign.title')}</DialogTitle>
            <DialogDescription>
              {t('protocols.detail.dialogs.assign.desc')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.assign.placeholder')}</Label>
              <Select value={selectedReviewerId} onValueChange={setSelectedReviewerId}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.assign.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {availableReviewers?.map((reviewer) => (
                    <SelectItem key={reviewer.id} value={reviewer.id}>
                      {reviewer.display_name || reviewer.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAssignDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAssignReviewer}
              disabled={!selectedReviewerId || assignReviewerMutation.isPending}
            >
              {assignReviewerMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('protocols.detail.dialogs.assign.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Assign Co-Editor dialog */}
      <Dialog open={showCoEditorDialog} onOpenChange={setShowCoEditorDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.coeditor.title')}</DialogTitle>
            <DialogDescription>
              {t('protocols.detail.dialogs.coeditor.desc')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.coeditor.placeholder')}</Label>
              <Select value={selectedCoEditorId} onValueChange={setSelectedCoEditorId}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.coeditor.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {availableExperimentStaff?.map((staff) => (
                    <SelectItem key={staff.id} value={staff.id}>
                      {staff.display_name || staff.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setShowCoEditorDialog(false)
              setSelectedCoEditorId('')
            }}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => {
                if (selectedCoEditorId && id) {
                  assignCoEditorMutation.mutate({
                    protocol_id: id,
                    user_id: selectedCoEditorId,
                  }, {
                    onSuccess: () => {
                      setShowCoEditorDialog(false)
                      setSelectedCoEditorId('')
                    }
                  })
                }
              }}
              disabled={!selectedCoEditorId || assignCoEditorMutation.isPending}
            >
              {assignCoEditorMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('protocols.detail.dialogs.assign.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Version content view dialog */}
      <Dialog open={showVersionDialog} onOpenChange={setShowVersionDialog}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.version.title', { version: selectedVersion?.version_no })}</DialogTitle>
            <DialogDescription>
              {selectedVersion && t('protocols.detail.dialogs.version.submitted', { time: formatDateTime(selectedVersion.submitted_at) })}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-auto max-h-[60vh]">
            {selectedVersion?.content_snapshot ? (
              <pre className="bg-slate-50 p-4 rounded-lg text-sm">
                {JSON.stringify(selectedVersion.content_snapshot, null, 2)}
              </pre>
            ) : (
              <p className="text-center text-muted-foreground py-8">{t('protocols.detail.dialogs.version.noContent')}</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowVersionDialog(false)}>
              {t('common.close')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Version Comparison Dialog */}
      {versionA && versionB && (
        <ProtocolComparisonDialog
          open={comparisonOpen}
          onOpenChange={setComparisonOpen}
          versionA={{
            version_no: versionA.version_no,
            content: versionA.content_snapshot as any
          }}
          versionB={{
            version_no: versionB.version_no,
            content: versionB.content_snapshot as any
          }}
          protocolTitle={protocol.title}
        />
      )}
    </div>
  )
}
