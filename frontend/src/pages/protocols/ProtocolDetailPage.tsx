import React, { useState, useMemo, useCallback } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolStatus,
  ChangeStatusRequest,
  AssignCoEditorRequest,
  User,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
  Users,
  ClipboardList,
  FileEdit,
} from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { queryKeys } from '@/lib/queryKeys'
import { getApiErrorMessage } from '@/lib/validation'
import { useAuthStore } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { ProtocolContentView } from '@/components/protocol/ProtocolContentView'
import { AmendmentsTab } from '@/components/protocol/AmendmentsTab'
import { VersionsTab } from '@/components/protocol/VersionsTab'
import { HistoryTab } from '@/components/protocol/HistoryTab'
import { CommentsTab } from '@/components/protocol/CommentsTab'
import { ReviewersTab } from '@/components/protocol/ReviewersTab'
import { CoEditorsTab } from '@/components/protocol/CoEditorsTab'
import { AttachmentsTab } from '@/components/protocol/AttachmentsTab'

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  PRE_REVIEW_REVISION_REQUIRED: 'destructive',
  VET_REVIEW: 'warning',
  VET_REVISION_REQUIRED: 'destructive',
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

const allowedTransitions: Record<ProtocolStatus, ProtocolStatus[]> = {
  DRAFT: ['SUBMITTED'],
  SUBMITTED: ['PRE_REVIEW', 'VET_REVIEW'],
  PRE_REVIEW: ['VET_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED'],
  PRE_REVIEW_REVISION_REQUIRED: ['PRE_REVIEW'],
  VET_REVIEW: ['UNDER_REVIEW', 'VET_REVISION_REQUIRED'],
  VET_REVISION_REQUIRED: ['VET_REVIEW'],
  UNDER_REVIEW: ['REVISION_REQUIRED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'REJECTED', 'DEFERRED'],
  REVISION_REQUIRED: ['RESUBMITTED'],
  RESUBMITTED: ['UNDER_REVIEW'],
  APPROVED: ['SUSPENDED', 'CLOSED'],
  APPROVED_WITH_CONDITIONS: ['SUSPENDED', 'CLOSED'],
  DEFERRED: ['UNDER_REVIEW', 'CLOSED'],
  REJECTED: ['CLOSED'],
  SUSPENDED: ['UNDER_REVIEW', 'CLOSED'],
  CLOSED: [],
  DELETED: [],
}

type TabKey = 'content' | 'versions' | 'history' | 'comments' | 'reviewers' | 'coeditors' | 'attachments' | 'animals' | 'amendments'

export function ProtocolDetailPage() {
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

  const { data: protocolResponse, isLoading } = useQuery({
    queryKey: queryKeys.protocols.detail(id!),
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const {
    protocol,
    pi_name,
    pi_email,
    pi_organization,
    vet_review,
  } = protocolResponse || {}

  const isVetReviewer = useMemo(() => {
    if (!user || !vet_review) return false
    return vet_review.vet_id === user.id
  }, [user, vet_review])

  const { data: allUsers } = useQuery({
    queryKey: queryKeys.users.all,
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
    enabled: showStatusDialog,
  })

  const availableReviewers = useMemo(() =>
    allUsers?.filter(u => u.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
      .map(u => ({ id: u.id, email: u.email, display_name: u.display_name || u.email })),
    [allUsers]
  )

  const availableExperimentStaff = useMemo(() =>
    allUsers?.filter(u => u.roles?.includes('EXPERIMENT_STAFF'))
      .map(u => ({ id: u.id, email: u.email, display_name: u.display_name || u.email })),
    [allUsers]
  )

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

  const handleSubmit = useCallback(async () => {
    const ok = await confirm({ title: '送出計畫書', description: t('protocols.detail.submitConfirm'), confirmLabel: '確認送出' })
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

  const handleReviewerToggle = useCallback((reviewerId: string, checked: boolean) => {
    setSelectedReviewerIds(prev =>
      checked ? [...prev, reviewerId] : prev.filter(r => r !== reviewerId)
    )
  }, [])

  const isVet = user?.roles?.includes('VET')
  const isReviewer = user?.roles?.some(r => ['REVIEWER', 'VET'].includes(r))
  const isIACUCOrAdmin = user?.roles?.some(r => ['IACUC_CHAIR', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const reviewableStatuses: ProtocolStatus[] = ['SUBMITTED', 'PRE_REVIEW', 'VET_REVIEW', 'UNDER_REVIEW', 'APPROVED', 'APPROVED_WITH_CONDITIONS']
  const canAddComment = isIACUCOrAdmin || (isReviewer && reviewableStatuses.includes(protocol?.status || ''))
  const canReply = user?.roles?.some(r => ['PI', 'EXPERIMENT_STAFF', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const canEditProtocol = user?.roles?.some(r => ['PI', 'EXPERIMENT_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const canAssignReviewer = user?.roles?.some(r => ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const isRevisionStatus = protocol?.status === 'REVISION_REQUIRED'
    || protocol?.status === 'PRE_REVIEW_REVISION_REQUIRED'
    || protocol?.status === 'VET_REVISION_REQUIRED'
  const canManageAttachments = protocol?.status === 'DRAFT' || isRevisionStatus
  const shouldAnonymizeReviewers = !user?.roles?.some(r =>
    ['IACUC_STAFF', 'IACUC_CHAIR', 'REVIEWER', 'VET', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )

  const tabItems = useMemo<{ key: TabKey; label: string; icon: typeof FileText }[]>(() => [
    { key: 'content', label: t('protocols.detail.tabs.content'), icon: FileText },
    { key: 'animals', label: t('protocols.detail.tabs.animals'), icon: ClipboardList },
    { key: 'versions', label: t('protocols.detail.tabs.versions'), icon: History },
    { key: 'history', label: t('protocols.detail.tabs.history'), icon: Clock },
    { key: 'comments', label: t('protocols.detail.tabs.comments'), icon: MessageSquare },
    { key: 'reviewers', label: t('protocols.detail.tabs.reviewers'), icon: Users },
    { key: 'coeditors', label: t('protocols.detail.tabs.coeditors'), icon: UserPlus },
    { key: 'attachments', label: t('protocols.detail.tabs.attachments'), icon: Paperclip },
    { key: 'amendments', label: t('protocols.detail.tabs.amendments'), icon: FileEdit },
  ], [t])

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
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center gap-3 md:gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate(-1)} aria-label="返回">
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <div className="flex flex-wrap items-center gap-2 md:gap-3">
              <h1 className="text-xl md:text-2xl font-bold">{protocol.title}</h1>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {t(`protocols.status.${protocol.status}`)}
              </Badge>
            </div>
          </div>
        </div>
        <div className="flex flex-wrap gap-2 pl-11 md:pl-0">
          {(protocol.status === 'DRAFT' || (isRevisionStatus && canEditProtocol)) && (
            <Button variant="outline" asChild>
              <Link to={`/protocols/${id}/edit`}>
                <Edit className="mr-2 h-4 w-4" />
                {isRevisionStatus ? t('protocols.detail.revise') : t('protocols.detail.edit')}
              </Link>
            </Button>
          )}
          {(protocol.status === 'DRAFT' || (isRevisionStatus && canEditProtocol)) && (
            <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              {t('protocols.detail.submit')}
            </Button>
          )}
          {availableTransitions.length > 0 && protocol.status !== 'DRAFT' &&
            (canAssignReviewer || (isVet && protocol.status === 'VET_REVIEW')) && (
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
            <p className="text-lg font-semibold">{pi_name || '-'}</p>
            <p className="text-sm text-muted-foreground">{pi_email}</p>
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
            <p className="text-lg font-semibold">{pi_organization || '-'}</p>
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
        <nav className="flex flex-wrap gap-4">
          {tabItems
            .filter(tab => {
              if (tab.key === 'reviewers') return !shouldAnonymizeReviewers
              return true
            })
            .map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${activeTab === tab.key
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
          </CardHeader>
          <CardContent>
            <ProtocolContentView
              workingContent={cleanedWorkingContent}
              protocolTitle={protocol.title}
              protocolId={id}
              startDate={protocol.start_date}
              endDate={protocol.end_date}
            />
          </CardContent>
        </Card>
      )}

      {activeTab === 'versions' && id && (
        <VersionsTab protocolId={id} protocolTitle={protocol.title} />
      )}

      {activeTab === 'history' && id && (
        <HistoryTab protocolId={id} />
      )}

      {activeTab === 'comments' && id && (
        <CommentsTab
          protocolId={id}
          protocol={protocol}
          piName={pi_name}
          vetReview={vet_review}
          canAddComment={!!canAddComment}
          canReply={!!canReply}
          shouldAnonymizeReviewers={shouldAnonymizeReviewers}
        />
      )}

      {activeTab === 'reviewers' && id && (
        <ReviewersTab
          protocolId={id}
          protocolStatus={protocol.status}
          vetReview={vet_review}
          isVetReviewer={isVetReviewer}
          canAssignReviewer={!!canAssignReviewer}
        />
      )}

      {activeTab === 'coeditors' && id && (
        <CoEditorsTab protocolId={id} canAssignReviewer={!!canAssignReviewer} />
      )}

      {activeTab === 'attachments' && id && (
        <AttachmentsTab protocolId={id} canManageAttachments={!!canManageAttachments} />
      )}

      {activeTab === 'animals' && (
        <Card>
          <CardHeader>
            <CardTitle>{t('protocols.detail.tabs.animals')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12">
              <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-semibold mb-2">{t('protocols.detail.tables.noAnimals')}</h3>
              <p className="text-muted-foreground">{t('protocols.detail.tables.noAnimalsDesc')}</p>
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
            <DialogDescription>{t('protocols.detail.dialogs.status.desc')}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.status.current')}</Label>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {t(`protocols.status.${protocol.status}`)}
              </Badge>
            </div>
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.status.target')}</Label>
              <Select value={newStatus} onValueChange={(v) => setNewStatus(v as ProtocolStatus)}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.status.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {availableTransitions.map((status) => (
                    <SelectItem key={status} value={status}>
                      {t(`protocols.status.${status}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {newStatus === 'UNDER_REVIEW' && (
              <div className="space-y-2">
                <Label>{t('protocols.detail.dialogs.status.reviewers')}</Label>
                <div className="space-y-2 max-h-48 overflow-y-auto border rounded p-2">
                  {availableReviewers?.map((reviewer) => (
                    <label key={reviewer.id} className="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input
                        type="checkbox"
                        checked={selectedReviewerIds.includes(reviewer.id)}
                        onChange={(e) => handleReviewerToggle(reviewer.id, e.target.checked)}
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
            {newStatus === 'PRE_REVIEW' && (
              <div className="space-y-2">
                <Label>{t('protocols.detail.tabs.coeditors')}</Label>
                <Select value={selectedCoEditorId} onValueChange={setSelectedCoEditorId}>
                  <SelectTrigger>
                    <SelectValue placeholder={t('protocols.detail.tables.coeditorHint')} />
                  </SelectTrigger>
                  <SelectContent>
                    {availableExperimentStaff?.map((staff) => (
                      <SelectItem key={staff.id} value={staff.id}>
                        {staff.display_name || staff.email}
                      </SelectItem>
                    ))}
                    {(!availableExperimentStaff || availableExperimentStaff.length === 0) && (
                      <div className="text-center py-2 text-sm text-muted-foreground">
                        {t('protocols.detail.tables.noCoeditors')}
                      </div>
                    )}
                  </SelectContent>
                </Select>
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
                (newStatus === 'UNDER_REVIEW' && (selectedReviewerIds.length < 2 || selectedReviewerIds.length > 3)) ||
                (newStatus === 'PRE_REVIEW' && !selectedCoEditorId)
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

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
