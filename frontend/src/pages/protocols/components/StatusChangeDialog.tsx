import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
import { AlertTriangle, Loader2 } from 'lucide-react'
import type { ProtocolStatus } from '@/lib/api'
import { statusColors } from '../constants'

interface ReviewerOption {
  id: string
  email: string
  display_name: string
}

interface StatusChangeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentStatus: ProtocolStatus
  newStatus: ProtocolStatus | ''
  onNewStatusChange: (status: ProtocolStatus | '') => void
  statusRemark: string
  onStatusRemarkChange: (remark: string) => void
  availableTransitions: ProtocolStatus[]
  availableReviewers?: ReviewerOption[]
  // 三態：true=已簽、false=確認未簽、undefined=狀態未知（載入中／查詢失敗）
  hasProtocolSignature: boolean | undefined
  selectedReviewerIds: string[]
  onReviewerToggle: (reviewerId: string, checked: boolean) => void
  onConfirm: () => void
  isChanging: boolean
}

export function StatusChangeDialog({
  open,
  onOpenChange,
  currentStatus,
  newStatus,
  onNewStatusChange,
  statusRemark,
  onStatusRemarkChange,
  availableTransitions,
  availableReviewers,
  hasProtocolSignature,
  selectedReviewerIds,
  onReviewerToggle,
  onConfirm,
  isChanging,
}: StatusChangeDialogProps) {
  const { t } = useTranslation()

  // R71-7：核准 / 附條件核准前須已完成計畫電子簽章（先簽再核准；後端亦守衛）。
  // 僅在「確認未簽（=== false）」時擋下並警示；狀態未知（undefined，載入中／查詢失敗）
  // 不顯示誤導警示，交由後端守衛裁定。
  const needsSignature =
    (newStatus === 'APPROVED' || newStatus === 'APPROVED_WITH_CONDITIONS')
    && hasProtocolSignature === false

  const isDisabled = !newStatus
    || isChanging
    || (newStatus === 'UNDER_REVIEW' && (selectedReviewerIds.length < 2 || selectedReviewerIds.length > 3))
    || (newStatus === 'REJECTED' && !statusRemark.trim())
    || needsSignature

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('protocols.detail.dialogs.status.title')}</DialogTitle>
          <DialogDescription>{t('protocols.detail.dialogs.status.desc')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('protocols.detail.dialogs.status.current')}</Label>
            <Badge variant={statusColors[currentStatus]} className="text-sm">
              {t(`protocols.status.${currentStatus}`)}
            </Badge>
          </div>
          <div className="space-y-2">
            <Label>{t('protocols.detail.dialogs.status.target')}</Label>
            <Select value={newStatus} onValueChange={(v) => onNewStatusChange(v as ProtocolStatus)}>
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
          {needsSignature && (
            <div className="flex items-start gap-2 rounded-md border border-status-warning-border bg-status-warning-bg px-3 py-2">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-status-warning-text" />
              <p className="text-sm text-status-warning-text">
                {t('protocols.detail.dialogs.status.signatureRequired')}
              </p>
            </div>
          )}
          {newStatus === 'UNDER_REVIEW' && (
            <ReviewerSelection
              availableReviewers={availableReviewers}
              selectedReviewerIds={selectedReviewerIds}
              onReviewerToggle={onReviewerToggle}
            />
          )}
          <div className="space-y-2">
            <Label>{t('protocols.detail.dialogs.status.remark')}</Label>
            <Textarea
              value={statusRemark}
              onChange={(e) => onStatusRemarkChange(e.target.value)}
              placeholder={t('protocols.detail.dialogs.status.remarkPlaceholder')}
              rows={3}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onConfirm} disabled={isDisabled}>
            {isChanging && (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            )}
            {t('protocols.detail.dialogs.status.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function ReviewerSelection({
  availableReviewers,
  selectedReviewerIds,
  onReviewerToggle,
}: {
  availableReviewers?: ReviewerOption[]
  selectedReviewerIds: string[]
  onReviewerToggle: (reviewerId: string, checked: boolean) => void
}) {
  const { t } = useTranslation()

  return (
    <div className="space-y-2">
      <Label>{t('protocols.detail.dialogs.status.reviewers')}</Label>
      <div className="space-y-2 max-h-48 overflow-y-auto border rounded p-2">
        {availableReviewers?.map((reviewer) => (
          <label key={reviewer.id} className="flex items-center gap-2 cursor-pointer hover:bg-muted p-1 rounded">
            <input
              type="checkbox"
              checked={selectedReviewerIds.includes(reviewer.id)}
              onChange={(e) => onReviewerToggle(reviewer.id, e.target.checked)}
              className="h-4 w-4"
            />
            <span>{reviewer.display_name || reviewer.email}</span>
          </label>
        ))}
        {(!availableReviewers || availableReviewers.length === 0) && (
          <p className="text-sm text-muted-foreground py-2 text-center">
            {t('protocols.detail.dialogs.status.noReviewers')}
          </p>
        )}
      </div>
      <p className="text-xs text-muted-foreground">
        {t('protocols.detail.dialogs.status.selected', { count: selectedReviewerIds.length })}
      </p>
    </div>
  )
}

