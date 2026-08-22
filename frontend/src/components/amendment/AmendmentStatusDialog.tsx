import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { amendmentApi } from '@/lib/api'
import { useInvalidateAmendment } from '@/hooks/useInvalidateAmendment'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2 } from 'lucide-react'
import { amendmentStatusColors } from '@/types/amendment'
import type {
  AmendmentStatus,
  ChangeAmendmentStatusRequest,
} from '@/types/amendment'

/**
 * 合法狀態轉移表，鏡射後端 models/amendment.rs::can_transition_to。
 * 排除 EFFECTIVE 目標（後端 change_status 拒絕；生效走 AmendmentsTab 的標記生效按鈕）。
 */
const TRANSITIONS: Partial<Record<AmendmentStatus, AmendmentStatus[]>> = {
  DRAFT: ['SUBMITTED'],
  SUBMITTED: ['CLASSIFIED', 'REJECTED'],
  CLASSIFIED: ['UNDER_REVIEW', 'REJECTED'],
  UNDER_REVIEW: ['REVISION_REQUIRED', 'APPROVED', 'ADMIN_APPROVED', 'REJECTED'],
  REVISION_REQUIRED: ['RESUBMITTED', 'REJECTED'],
  RESUBMITTED: ['UNDER_REVIEW', 'REJECTED'],
}

interface AmendmentStatusDialogProps {
  amendmentId: string
  currentStatus: AmendmentStatus
  open: boolean
  onOpenChange: (open: boolean) => void
}

/**
 * 狀態變更對話框（Phase 3）。POST /amendments/:id/status。
 * 僅供 aup.protocol.change_status；目標限合法轉移且不含 EFFECTIVE。
 */
export function AmendmentStatusDialog({
  amendmentId,
  currentStatus,
  open,
  onOpenChange,
}: AmendmentStatusDialogProps) {
  const { t } = useTranslation()
  const invalidateAmendment = useInvalidateAmendment(amendmentId)
  const [target, setTarget] = useState<AmendmentStatus | ''>('')
  const [remark, setRemark] = useState('')

  const targets = TRANSITIONS[currentStatus] ?? []

  const reset = () => {
    setTarget('')
    setRemark('')
  }

  const mutation = useMutation({
    mutationFn: (data: ChangeAmendmentStatusRequest) => amendmentApi.changeStatus(amendmentId, data),
    onSuccess: () => {
      invalidateAmendment()
      toast({ title: t('common.success'), description: t('amendments.decision.statusDialog.success') })
      reset()
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('amendments.decision.statusDialog.failed')),
        variant: 'destructive',
      })
    },
  })

  // REJECTED 需附理由（對齊計畫狀態變更慣例）
  const requiresRemark = target === 'REJECTED'
  const disabled = !target || mutation.isPending || (requiresRemark && !remark.trim())

  const handleConfirm = () => {
    if (!target) return
    mutation.mutate({ to_status: target, remark: remark.trim() || undefined })
  }

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) reset(); onOpenChange(o) }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('amendments.decision.statusDialog.title')}</DialogTitle>
          <DialogDescription>{t('amendments.decision.statusDialog.desc')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('amendments.decision.statusDialog.current')}</Label>
            <Badge variant={amendmentStatusColors[currentStatus]} className="text-sm">
              {t(`amendments.status.${currentStatus}`)}
            </Badge>
          </div>
          <div className="space-y-2">
            <Label>{t('amendments.decision.statusDialog.target')}</Label>
            <Select value={target} onValueChange={(v) => setTarget(v as AmendmentStatus)}>
              <SelectTrigger>
                <SelectValue placeholder={t('amendments.decision.statusDialog.placeholder')} />
              </SelectTrigger>
              <SelectContent>
                {targets.map((status) => (
                  <SelectItem key={status} value={status}>
                    {t(`amendments.status.${status}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {targets.length === 0 && (
              <p className="text-xs text-muted-foreground">
                {t('amendments.decision.statusDialog.noTransitions')}
              </p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="amendment-status-remark">{t('amendments.decision.statusDialog.remarkLabel')}</Label>
            <Textarea
              id="amendment-status-remark"
              value={remark}
              onChange={(e) => setRemark(e.target.value)}
              placeholder={t('amendments.decision.statusDialog.remarkPlaceholder')}
              rows={3}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => { reset(); onOpenChange(false) }}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleConfirm} disabled={disabled}>
            {mutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('amendments.decision.statusDialog.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
