import { useMemo } from 'react'
import { Link } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import { PageTabs } from '@/components/ui/page-tabs'
import { Loader2, AlertTriangle, FileText, ClipboardList, History, Clock, MessageSquare, Users, UserPlus, Paperclip, FileEdit } from 'lucide-react'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useProtocolDetail } from './hooks/useProtocolDetail'
import { useAuthStore } from '@/stores/auth'
import { ProtocolDetailHeader } from './components/ProtocolDetailHeader'
import { ProtocolInfoCards } from './components/ProtocolInfoCards'
import { ProtocolTabContent } from './components/ProtocolTabContent'
import { StatusChangeDialog } from './components/StatusChangeDialog'
import { StaffReviewAssistPanel } from '@/components/protocol/StaffReviewAssistPanel'

const STAFF_REVIEW_STATUSES = [
  'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
  'VET_REVIEW', 'VET_REVISION_REQUIRED',
  'UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED',
] as const

export function ProtocolDetailPage() {
  const { user: authUser } = useAuthStore()
  const isStaffOrChair = authUser?.roles?.some(
    r => ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN'].includes(r)
  ) ?? false
  const {
    id,
    t,
    protocol,
    pi_name,
    pi_email,
    pi_organization,
    vet_review,
    isLoading,
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
    canAddComment,
    canReply,
    canEditProtocol,
    canAssignReviewer,
    isRevisionStatus,
    canManageAttachments,
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
  } = useProtocolDetail()

  const tabs = useMemo(() => [
    { value: 'content', label: t('protocols.detail.tabs.content'), icon: FileText },
    { value: 'animals', label: t('protocols.detail.tabs.animals'), icon: ClipboardList },
    { value: 'versions', label: t('protocols.detail.tabs.versions'), icon: History },
    { value: 'history', label: t('protocols.detail.tabs.history'), icon: Clock },
    { value: 'comments', label: t('protocols.detail.tabs.comments'), icon: MessageSquare },
    { value: 'reviewers', label: t('protocols.detail.tabs.reviewers'), icon: Users, hidden: shouldAnonymizeReviewers },
    { value: 'coeditors', label: t('protocols.detail.tabs.coeditors'), icon: UserPlus },
    { value: 'attachments', label: t('protocols.detail.tabs.attachments'), icon: Paperclip },
    { value: 'amendments', label: t('protocols.detail.tabs.amendments'), icon: FileEdit },
  ], [t, shouldAnonymizeReviewers])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!protocol || !id) {
    return (
      <div className="text-center py-12">
        <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-status-warning-text" />
        <h2 className="text-xl font-semibold mb-2">{t('protocols.detail.notFound')}</h2>
        <p className="text-muted-foreground mb-4">{t('protocols.detail.notFoundDesc')}</p>
        <Button asChild>
          <Link to="/protocols">{t('protocols.detail.backToList')}</Link>
        </Button>
      </div>
    )
  }

  return (
    <>
      <div className="space-y-6">
        <ProtocolDetailHeader
          protocol={protocol}
          protocolId={id}
          isRevisionStatus={!!isRevisionStatus}
          canEditProtocol={canEditProtocol}
          canAssignReviewer={canAssignReviewer}
          isVet={isVet}
          availableTransitions={availableTransitions}
          submitIsPending={submitMutation.isPending}
          onSubmit={handleSubmit}
          onOpenStatusDialog={() => setShowStatusDialog(true)}
        />

        <ProtocolInfoCards
          protocol={protocol}
          piName={pi_name}
          piEmail={pi_email}
          piOrganization={pi_organization}
        />

        {/* R20-7: 執行秘書 AI 標註面板 */}
        {isStaffOrChair && protocol && id && STAFF_REVIEW_STATUSES.includes(protocol.status as typeof STAFF_REVIEW_STATUSES[number]) && (
          <StaffReviewAssistPanel protocolId={id} />
        )}

        <PageTabs tabs={tabs} defaultTab="content">
          <ProtocolTabContent
            protocolId={id}
            protocol={protocol}
            piName={pi_name}
            vetReview={vet_review}
            isVetReviewer={isVetReviewer}
            canAddComment={canAddComment}
            canReply={canReply}
            canAssignReviewer={canAssignReviewer}
            canManageAttachments={canManageAttachments}
            shouldAnonymizeReviewers={shouldAnonymizeReviewers}
            canShowPanel={canShowPanel}
            showCommentPanel={showCommentPanel}
            cleanedWorkingContent={cleanedWorkingContent}
            sectionOptions={sectionOptions}
            isSubmittingComment={addCommentMutation.isPending}
            onToggleCommentPanel={handleToggleCommentPanel}
            onSubmitComment={(content) => addCommentMutation.mutate(content)}
          />
        </PageTabs>

        <StatusChangeDialog
          open={showStatusDialog}
          onOpenChange={setShowStatusDialog}
          currentStatus={protocol.status}
          newStatus={newStatus}
          onNewStatusChange={setNewStatus}
          statusRemark={statusRemark}
          onStatusRemarkChange={setStatusRemark}
          availableTransitions={availableTransitions}
          availableReviewers={availableReviewers}
          availableExperimentStaff={availableExperimentStaff}
          selectedReviewerIds={selectedReviewerIds}
          selectedCoEditorId={selectedCoEditorId}
          onCoEditorChange={setSelectedCoEditorId}
          onReviewerToggle={handleReviewerToggle}
          onConfirm={handleChangeStatus}
          isChanging={changeStatusMutation.isPending}
          isAssigning={assignCoEditorMutation.isPending}
        />

        <ConfirmDialog state={dialogState} />
      </div>
    </>
  )
}
