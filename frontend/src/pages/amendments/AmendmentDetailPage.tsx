import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { amendmentApi } from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useInvalidateAmendment } from '@/hooks/useInvalidateAmendment'
import { useAuthUser, useAuthHasPermission } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Card, CardContent } from '@/components/ui/card'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { AlertCircle, ArrowLeft, FileText, Gavel, History, Loader2, Users } from 'lucide-react'
import { AmendmentOverview } from '@/components/amendment/AmendmentOverview'
import { AmendmentVoteRoster } from '@/components/amendment/AmendmentVoteRoster'
import { AmendmentDecisionPanel } from '@/components/amendment/AmendmentDecisionPanel'
import { AmendmentClassifyDialog } from '@/components/amendment/AmendmentClassifyDialog'
import { AmendmentStatusDialog } from '@/components/amendment/AmendmentStatusDialog'
import { AmendmentHistoryTimeline } from '@/components/amendment/AmendmentHistoryTimeline'
import { AmendmentStaffActions } from '@/components/amendment/AmendmentStaffActions'

export function AmendmentDetailPage() {
  const { id } = useParams<{ id: string }>()
  const amendmentId = id!
  const navigate = useNavigate()
  const { t } = useTranslation()
  const invalidateAmendment = useInvalidateAmendment(amendmentId)
  const user = useAuthUser()
  const hasPermission = useAuthHasPermission()
  const [classifyOpen, setClassifyOpen] = useState(false)
  const [statusOpen, setStatusOpen] = useState(false)

  const { data: amendment, isLoading } = useQuery({
    queryKey: queryKeys.amendments.detail(amendmentId),
    queryFn: async () => (await amendmentApi.get(amendmentId)).data,
    enabled: !!amendmentId,
  })

  const { data: assignments, isLoading: assignmentsLoading } = useQuery({
    queryKey: queryKeys.amendments.assignments(amendmentId),
    queryFn: async () => (await amendmentApi.getAssignments(amendmentId)).data,
    enabled: !!amendmentId,
  })

  const { data: history, isLoading: historyLoading } = useQuery({
    queryKey: queryKeys.amendments.history(amendmentId),
    queryFn: async () => (await amendmentApi.getHistory(amendmentId)).data,
    enabled: !!amendmentId,
  })

  const startReviewMutation = useMutation({
    mutationFn: () => amendmentApi.startReview(amendmentId),
    onSuccess: () => {
      invalidateAmendment()
      toast({ title: t('common.success'), description: t('amendments.decision.startReviewSuccess') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('amendments.decision.startReviewFailed')),
        variant: 'destructive',
      })
    },
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!amendment) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-muted-foreground mb-4" />
        <p className="text-muted-foreground">{t('amendments.decision.notFound')}</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/my-amendments')}>
          {t('amendments.decision.backToList')}
        </Button>
      </div>
    )
  }

  // 投票面板可見性：具投票權 + 是被指派委員（自己的列）+ 狀態為 UNDER_REVIEW
  // 先確認 user.id 存在，避免 undefined === undefined 誤判（gemini #740）
  const isAssignedReviewer = !!user?.id && !!assignments?.some((a) => a.reviewer_id === user.id)
  const canVote =
    hasPermission('aup.amendment.approve') &&
    isAssignedReviewer &&
    amendment.status === 'UNDER_REVIEW'

  const canClassify =
    hasPermission('aup.amendment.classify') &&
    (amendment.status === 'SUBMITTED' || amendment.status === 'RESUBMITTED')
  const canStartReview =
    hasPermission('aup.protocol.change_status') && amendment.status === 'CLASSIFIED'
  const canChangeStatus = hasPermission('aup.protocol.change_status')

  const tabs = [
    { value: 'overview', label: t('amendments.decision.tabs.overview'), icon: FileText },
    { value: 'decision', label: t('amendments.decision.tabs.decision'), icon: Gavel },
    { value: 'roster', label: t('amendments.decision.tabs.roster'), icon: Users },
    { value: 'history', label: t('amendments.decision.tabs.history'), icon: History },
  ]

  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" className="w-fit" onClick={() => navigate('/my-amendments')}>
        <ArrowLeft className="mr-1 h-4 w-4" />
        {t('amendments.decision.backToList')}
      </Button>

      <PageHeader
        title={amendment.amendment_no}
        description={amendment.title}
        actions={
          <AmendmentStaffActions
            classify={{ enabled: canClassify, onClick: () => setClassifyOpen(true) }}
            startReview={{
              enabled: canStartReview,
              pending: startReviewMutation.isPending,
              onClick: () => startReviewMutation.mutate(),
            }}
            changeStatus={{ enabled: canChangeStatus, onClick: () => setStatusOpen(true) }}
          />
        }
      />

      <PageTabs tabs={tabs} defaultTab="overview">
        <PageTabContent value="overview">
          <AmendmentOverview amendment={amendment} />
        </PageTabContent>
        <PageTabContent value="decision">
          <Card>
            <CardContent className="pt-6 space-y-4">
              {assignmentsLoading ? (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span>{t('common.loading')}</span>
                </div>
              ) : canVote ? (
                <AmendmentDecisionPanel amendmentId={amendmentId} assignments={assignments ?? []} />
              ) : (
                <p className="text-sm text-muted-foreground">
                  {amendment.status === 'UNDER_REVIEW'
                    ? t('amendments.decision.notReviewer')
                    : t('amendments.decision.notVotable')}
                </p>
              )}
              <AmendmentVoteRoster assignments={assignments} isLoading={assignmentsLoading} />
            </CardContent>
          </Card>
        </PageTabContent>
        <PageTabContent value="roster">
          <Card>
            <CardContent className="pt-6">
              <AmendmentVoteRoster assignments={assignments} isLoading={assignmentsLoading} />
            </CardContent>
          </Card>
        </PageTabContent>
        <PageTabContent value="history">
          <Card>
            <CardContent className="pt-6">
              <AmendmentHistoryTimeline history={history} isLoading={historyLoading} />
            </CardContent>
          </Card>
        </PageTabContent>
      </PageTabs>

      {canClassify && (
        <AmendmentClassifyDialog amendmentId={amendmentId} open={classifyOpen} onOpenChange={setClassifyOpen} />
      )}
      {canChangeStatus && (
        <AmendmentStatusDialog
          amendmentId={amendmentId}
          currentStatus={amendment.status}
          open={statusOpen}
          onOpenChange={setStatusOpen}
        />
      )}
    </div>
  )
}
