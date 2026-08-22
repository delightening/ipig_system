// 狀態提示 + 送出 / 匯出按鈕列（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { Button } from '@/components/ui/button'
import { Loader2, Send, FileDown } from 'lucide-react'
import type { VetPatrolReportVM } from './useVetPatrolReport'

export function FooterActions({ vm }: { vm: VetPatrolReportVM }) {
    return (
        <div className="flex justify-between items-center mt-4 pt-3 border-t">
            <p className="text-xs text-muted-foreground">
                {vm.isCompleted
                    ? '報告已完成並鎖定（GLP 醫療紀錄不可變）'
                    : vm.isAwaitingAcknowledgement
                        ? (vm.isFollowUpTracker
                            ? '請按「確認收到」進入填寫階段'
                            : '待追蹤者確認收到')
                        : vm.isAwaitingFollowUp
                            ? (vm.isFollowUpTracker
                                ? '請填寫「追蹤改善」欄位後按「確認完成」'
                                : '待追蹤者填寫追蹤改善')
                            : '儲存後，豬隻狀況/病例紀錄會自動同步至對應動物的「獸醫師建議」'}
            </p>
            <div className="flex gap-2">
                <Button variant="outline" onClick={vm.handleClose} disabled={vm.isSubmitting}>
                    關閉
                </Button>
                <Button
                    variant="outline"
                    onClick={vm.handleExportPdf}
                    disabled={vm.isExporting || !vm.savedReportId}
                    title={!vm.savedReportId ? '草稿建立中…請稍候' : '下載 PDF（首次可能需 30 秒）'}
                >
                    {vm.isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileDown className="h-4 w-4 mr-1" />}
                    下載 PDF
                </Button>
                {/* R39+++ 存草稿：draft 階段 + 獸醫 + 待回覆階段 + 追蹤者；只持久化不轉狀態 */}
                {((vm.isDraft && vm.isVet) || (vm.isAwaitingFollowUp && vm.isFollowUpTracker)) && (
                    <Button
                        variant="outline"
                        onClick={() => vm.saveDraftMutation.mutate()}
                        disabled={vm.isSubmitting || vm.saveDraftMutation.isPending || !vm.patrolDate}
                    >
                        {vm.saveDraftMutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : null}
                        存草稿
                    </Button>
                )}
                {/* 階段 1：獸醫 draft → 送出給追蹤者 */}
                {vm.isDraft && vm.isVet && (
                    <Button
                        onClick={() => vm.submitForFollowupMutation.mutate()}
                        disabled={vm.isSubmitting || !vm.patrolDate || !vm.hasContent || !vm.followUpUserId}
                        className="bg-status-success-solid hover:bg-status-success-solid/90"
                        title={!vm.followUpUserId ? '請先選擇陪同人員作為追蹤者' : '送出給追蹤者'}
                    >
                        {vm.isSubmitting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Send className="h-4 w-4 mr-1" />}
                        送出給追蹤者
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
                        確認收到
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
                        確認完成（鎖定）
                    </Button>
                )}
            </div>
        </div>
    )
}
