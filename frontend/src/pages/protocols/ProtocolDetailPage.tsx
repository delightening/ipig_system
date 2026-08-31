import { useMemo } from 'react'
import { Link } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import { PageTabs } from '@/components/ui/page-tabs'
import { Loader2, AlertTriangle, FileText, ClipboardList, History, Clock, MessageSquare, Users, Paperclip, FileEdit } from 'lucide-react'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useProtocolDetail } from './hooks/useProtocolDetail'
import { useAuthUser } from '@/stores/auth'
import { ProtocolDetailHeader } from './components/ProtocolDetailHeader'
import { ProtocolInfoCards } from './components/ProtocolInfoCards'
import { ProtocolTabContent } from './components/ProtocolTabContent'
import { StatusChangeDialog } from './components/StatusChangeDialog'
import { NoticeAcknowledgementCard } from './components/application-notices/NoticeAcknowledgementCard'
import { PiDelegateCard } from './components/PiDelegateCard'
import { StaffReviewAssistPanel } from '@/components/protocol/StaffReviewAssistPanel'

const STAFF_REVIEW_STATUSES = [
  'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
  'VET_REVIEW', 'VET_REVISION_REQUIRED',
  'UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED',
] as const

export function ProtocolDetailPage() {
  const authUser = useAuthUser()
  const isStaffOrChair = authUser?.roles?.some(
    r => ['IACUC_STAFF', 'IACUC_CHAIR', 'SYSTEM_ADMIN'].includes(r)
  ) ?? false
  const {
    id,
    t,
    protocol,
    pi_name,
    pi_organization,
    sd_name,
    created_by_name,
    vet_review,
    isLoading,
    showStatusDialog,
    setShowStatusDialog,
    newStatus,
    setNewStatus,
    statusRemark,
    setStatusRemark,
    selectedReviewerIds,
    showCommentPanel,
    cleanedWorkingContent,
    availableTransitions,
    availableReviewers,
    hasProtocolSignature,
    isVetReviewer,
    isVet,
    canAddComment,
    canReply,
    canEditProtocol,
    canWriteAmendment,
    isPiDelegate,
    canAssignReviewer,
    canChangeStatus,
    isRevisionStatus,
    canManageAttachments,
    shouldAnonymizeReviewers,
    canShowPanel,
    sectionOptions,
    submitMutation,
    changeStatusMutation,
    softDeleteMutation,
    addCommentMutation,
    canSoftDelete,
    handleSubmit,
    handleSoftDelete,
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
          canChangeStatus={canChangeStatus}
          isVet={isVet}
          availableTransitions={availableTransitions}
          submitIsPending={submitMutation.isPending}
          canSoftDelete={canSoftDelete}
          softDeleteIsPending={softDeleteMutation.isPending}
          onSubmit={handleSubmit}
          onOpenStatusDialog={() => setShowStatusDialog(true)}
          onSoftDelete={handleSoftDelete}
        />

        {protocol.import_pending && (
          <div className="flex items-center justify-between gap-3 rounded-lg border border-status-warning-border bg-status-warning-bg px-4 py-3">
            <p className="text-sm text-status-warning-text">
              此計劃為「補登中」：可編輯內容並補登歷史審查文件，完成補登後將鎖定。
            </p>
            <Button asChild size="sm" className="shrink-0">
              <Link to={`/protocols/${id}/import-review`}>前往補登作業</Link>
            </Button>
          </div>
        )}

        {protocol.status === 'DRAFT' && <NoticeAcknowledgementCard protocolId={id} />}

        <PiDelegateCard protocolId={id} />

        <ProtocolInfoCards
          protocol={protocol}
          piName={pi_name}
          piOrganization={pi_organization}
          sdName={sd_name}
          createdByName={created_by_name}
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
            canWriteAmendment={canWriteAmendment}
            isPiDelegate={isPiDelegate}
            canAssignReviewer={canAssignReviewer}
            canManageAttachments={canManageAttachments}
            shouldAnonymizeReviewers={shouldAnonymizeReviewers}
            canShowPanel={canShowPanel}
            showCommentPanel={showCommentPanel}
            cleanedWorkingContent={cleanedWorkingContent}
            sectionOptions={sectionOptions}
            isSubmittingComment={addCommentMutation.isPending}
            onToggleCommentPanel={handleToggleCommentPanel}
            onSubmitComment={(content, commentType) =>
              addCommentMutation.mutateAsync({ content, commentType })
            }
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
          hasProtocolSignature={hasProtocolSignature}
          selectedReviewerIds={selectedReviewerIds}
          onReviewerToggle={handleReviewerToggle}
          onConfirm={handleChangeStatus}
          isChanging={changeStatusMutation.isPending}
        />

        <ConfirmDialog state={dialogState} />
      </div>
    </>
  )
}
