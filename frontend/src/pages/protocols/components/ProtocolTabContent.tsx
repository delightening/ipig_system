import { lazy, Suspense } from 'react'
import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { FileText } from 'lucide-react'
import type { Protocol, ProtocolStatus } from '@/lib/api'
import type { ProtocolWorkingContent } from '@/types/protocol'
import type { VetReviewAssignment } from '@/types/aup'
import type { TabKey } from '../constants'

// Lazy-loaded tab components
const ProtocolContentView = lazy(() => import('@/components/protocol/ProtocolContentView').then(m => ({ default: m.ProtocolContentView })))
const AmendmentsTab = lazy(() => import('@/components/protocol/AmendmentsTab').then(m => ({ default: m.AmendmentsTab })))
const VersionsTab = lazy(() => import('@/components/protocol/VersionsTab').then(m => ({ default: m.VersionsTab })))
const HistoryTab = lazy(() => import('@/components/protocol/HistoryTab').then(m => ({ default: m.HistoryTab })))
const CommentsTab = lazy(() => import('@/components/protocol/CommentsTab').then(m => ({ default: m.CommentsTab })))
const ReviewersTab = lazy(() => import('@/components/protocol/ReviewersTab').then(m => ({ default: m.ReviewersTab })))
const CoEditorsTab = lazy(() => import('@/components/protocol/CoEditorsTab').then(m => ({ default: m.CoEditorsTab })))
const AttachmentsTab = lazy(() => import('@/components/protocol/AttachmentsTab').then(m => ({ default: m.AttachmentsTab })))

const TabFallback = () => <Skeleton variant="form" fields={4} />

interface ProtocolTabContentProps {
  activeTab: TabKey
  protocolId: string
  protocol: Protocol
  piName?: string
  vetReview?: VetReviewAssignment
  isVetReviewer: boolean
  canAddComment: boolean
  canReply: boolean
  canAssignReviewer: boolean
  canManageAttachments: boolean
  shouldAnonymizeReviewers: boolean
  canShowPanel: boolean
  showCommentPanel: boolean
  cleanedWorkingContent: ProtocolWorkingContent
  sectionOptions: string[]
  isSubmittingComment: boolean
  onToggleCommentPanel: () => void
  onSubmitComment: (content: string) => void
}

export function ProtocolTabContent({
  activeTab,
  protocolId,
  protocol,
  piName,
  vetReview,
  isVetReviewer,
  canAddComment,
  canReply,
  canAssignReviewer,
  canManageAttachments,
  shouldAnonymizeReviewers,
  canShowPanel,
  showCommentPanel,
  cleanedWorkingContent,
  sectionOptions,
  isSubmittingComment,
  onToggleCommentPanel,
  onSubmitComment,
}: ProtocolTabContentProps) {
  const { t } = useTranslation()

  const tabContent = (() => {
    switch (activeTab) {
      case 'content':
        return (
          <Card>
            <CardHeader>
              <CardTitle>{t('protocols.detail.sections.contentTitle')}</CardTitle>
            </CardHeader>
            <CardContent>
              <ProtocolContentView
                workingContent={cleanedWorkingContent}
                protocolTitle={protocol.title}
                protocolId={protocolId}
                startDate={protocol.start_date}
                endDate={protocol.end_date}
                onToggleCommentPanel={onToggleCommentPanel}
                showReviewButton={canShowPanel}
                showCommentPanel={showCommentPanel}
                onSubmitComment={onSubmitComment}
                isSubmittingComment={isSubmittingComment}
                sectionOptions={sectionOptions}
              />
            </CardContent>
          </Card>
        )

      case 'versions':
        return <VersionsTab protocolId={protocolId} protocolTitle={protocol.title} />

      case 'history':
        return <HistoryTab protocolId={protocolId} />

      case 'comments':
        return (
          <CommentsTab
            protocolId={protocolId}
            protocol={protocol}
            piName={piName}
            vetReview={vetReview}
            canAddComment={canAddComment}
            canReply={canReply}
            shouldAnonymizeReviewers={shouldAnonymizeReviewers}
            sectionOptions={sectionOptions}
          />
        )

      case 'reviewers':
        return (
          <ReviewersTab
            protocolId={protocolId}
            protocolStatus={protocol.status}
            vetReview={vetReview}
            isVetReviewer={isVetReviewer}
            canAssignReviewer={canAssignReviewer}
          />
        )

      case 'coeditors':
        return <CoEditorsTab protocolId={protocolId} canAssignReviewer={canAssignReviewer} />

      case 'attachments':
        return <AttachmentsTab protocolId={protocolId} canManageAttachments={canManageAttachments} />

      case 'animals':
        return (
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
        )

      case 'amendments':
        return <AmendmentsTab protocolId={protocolId} protocolStatus={protocol.status as ProtocolStatus} />

      default:
        return null
    }
  })()

  return (
    <Suspense fallback={<TabFallback />}>
      {tabContent}
    </Suspense>
  )
}
