// 狀態提示 + 送出 / 匯出按鈕列（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Loader2, Send, FileDown } from 'lucide-react'
import type { VetPatrolReportVM } from './useVetPatrolReport'

export function FooterActions({ vm }: { vm: VetPatrolReportVM }) {
    const { t } = useTranslation()
    return (
        <div className="flex justify-between items-center mt-4 pt-3 border-t">
            <p className="text-xs text-muted-foreground">
                {vm.isCompleted
                    ? t('animalActions.vetPatrol.footer.completedLocked')
                    : vm.isAwaitingAcknowledgement
                        ? (vm.isFollowUpTracker
                            ? t('animalActions.vetPatrol.footer.pressAcknowledge')
                            : t('animalActions.vetPatrol.footer.waitingAcknowledge'))
                        : vm.isAwaitingFollowUp
                            ? (vm.isFollowUpTracker
                                ? t('animalActions.vetPatrol.footer.fillFollowUp')
                                : t('animalActions.vetPatrol.footer.waitingFollowUp'))
                            : t('animalActions.vetPatrol.footer.autoSyncHint')}
            </p>
            <div className="flex gap-2">
                <Button variant="outline" onClick={vm.handleClose} disabled={vm.isSubmitting}>
                    {t('common.closeDialog')}
                </Button>
                <Button
                    variant="outline"
                    onClick={vm.handleExportPdf}
                    disabled={vm.isExporting || !vm.savedReportId}
                    title={!vm.savedReportId ? t('animalActions.vetPatrol.footer.draftCreatingWait') : t('animalActions.vetPatrol.footer.downloadPdfHint')}
                >
                    {vm.isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileDown className="h-4 w-4 mr-1" />}
                    {t('common.pdfExport.downloadPdf')}
                </Button>
                {/* R39+++ 存草稿：draft 階段 + 獸醫 + 待回覆階段 + 追蹤者；只持久化不轉狀態 */}
                {((vm.isDraft && vm.isVet) || (vm.isAwaitingFollowUp && vm.isFollowUpTracker)) && (
                    <Button
                        variant="outline"
                        onClick={() => vm.saveDraftMutation.mutate()}
                        disabled={vm.isSubmitting || vm.saveDraftMutation.isPending || !vm.patrolDate}
                    >
                        {vm.saveDraftMutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : null}
                        {t('animalActions.vetPatrol.footer.saveDraft')}
                    </Button>
                )}
                {/* 階段 1：獸醫 draft → 送出給追蹤者 */}
                {vm.isDraft && vm.isVet && (
                    <Button
                        onClick={() => vm.submitForFollowupMutation.mutate()}
                        disabled={vm.isSubmitting || !vm.patrolDate || !vm.hasContent || !vm.followUpUserId}
                        className="bg-status-success-solid hover:bg-status-success-solid/90"
                        title={!vm.followUpUserId ? t('animalActions.vetPatrol.footer.selectTrackerFirst') : t('animalActions.vetPatrol.footer.submitToTracker')}
                    >
                        {vm.isSubmitting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Send className="h-4 w-4 mr-1" />}
                        {t('animalActions.vetPatrol.footer.submitToTracker')}
                    </Button>
                )}
                {/* 階段 2：追蹤者 awaiting_acknowledgement → 確認收到（已改自動觸發；此按鈕為 auto-ack 失敗時的 fallback） */}
                {vm.isAwaitingAcknowledgement && vm.isFollowUpTracker && (
                    <Button
                        onClick={() => vm.acknowledgeMutation.mutate()}
                        disabled={vm.isSubmitting}
                        className="bg-status-success-solid hover:bg-status-success-solid/90"
                    >
                        {vm.isSubmitting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Send className="h-4 w-4 mr-1" />}
                        {t('animalActions.vetPatrol.footer.acknowledge')}
                    </Button>
                )}
                {/* 階段 3：追蹤者 awaiting_follow_up → 確認完成 */}
                {vm.isAwaitingFollowUp && vm.isFollowUpTracker && (
                    <Button
                        onClick={() => vm.completeFollowupMutation.mutate()}
                        disabled={vm.isSubmitting}
                        className="bg-status-success-solid hover:bg-status-success-solid/90"
                    >
                        {vm.isSubmitting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Send className="h-4 w-4 mr-1" />}
                        {t('animalActions.vetPatrol.footer.confirmComplete')}
                    </Button>
                )}
            </div>
        </div>
    )
}
