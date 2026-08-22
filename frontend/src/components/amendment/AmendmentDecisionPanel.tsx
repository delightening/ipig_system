import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { amendmentApi } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { useInvalidateAmendment } from '@/hooks/useInvalidateAmendment'
import { getApiErrorMessage } from '@/lib/apiError'
import { CheckCircle2, Loader2, RotateCcw, XCircle } from 'lucide-react'
import type {
  AmendmentReviewAssignment,
  RecordAmendmentDecisionRequest,
} from '@/types/amendment'

type Decision = RecordAmendmentDecisionRequest['decision']

interface AmendmentDecisionPanelProps {
  amendmentId: string
  assignments: AmendmentReviewAssignment[]
}

/**
 * 委員投票面板（Phase 2）。三鍵 APPROVE / REJECT / REVISION → POST /amendments/:id/decision。
 * 結算規則（後端 check_all_decisions_tx）：全員投完才結算，precedence REVISION > REJECT > APPROVE；
 * 單一否決不會立即否決，故 UI 文案明確呈現「n / N 投票，全部投完後結算」。
 */
export function AmendmentDecisionPanel({ amendmentId, assignments }: AmendmentDecisionPanelProps) {
  const { t } = useTranslation()
  const invalidateAmendment = useInvalidateAmendment(amendmentId)
  const { dialogState, confirm } = useConfirmDialog()
  const [comment, setComment] = useState('')

  const total = assignments.length
  const decided = assignments.filter((a) => !!a.decision).length

  const mutation = useMutation({
    mutationFn: (data: RecordAmendmentDecisionRequest) =>
      amendmentApi.recordDecision(amendmentId, data),
    onSuccess: () => {
      invalidateAmendment()
      toast({ title: t('common.success'), description: t('amendments.decision.submitSuccess') })
      setComment('')
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('amendments.decision.submitFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleVote = async (decision: Decision) => {
    if (!comment.trim()) {
      toast({
        title: t('common.error'),
        description: t('amendments.decision.commentRequired'),
        variant: 'destructive',
      })
      return
    }
    const ok = await confirm({
      title: t('amendments.decision.confirmTitle'),
      description: t(`amendments.decision.confirm.${decision}`),
      variant: decision === 'REJECT' ? 'destructive' : 'default',
      confirmLabel: t(`amendments.decision.vote.${decision}`),
    })
    if (ok) mutation.mutate({ decision, comment: comment.trim() })
  }

  return (
    <div className="space-y-4 rounded-lg border bg-card p-4">
      <div className="space-y-1">
        <p className="text-sm font-medium text-foreground">{t('amendments.decision.panelTitle')}</p>
        <p className="text-sm text-muted-foreground">
          {t('amendments.decision.progress', { decided, total })}
        </p>
        <p className="text-xs text-muted-foreground">{t('amendments.decision.precedence')}</p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="amendment-decision-comment">{t('amendments.decision.commentLabel')}</Label>
        <Textarea
          id="amendment-decision-comment"
          value={comment}
          onChange={(e) => setComment(e.target.value)}
          placeholder={t('amendments.decision.commentPlaceholder')}
          rows={3}
        />
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          variant="default"
          onClick={() => handleVote('APPROVE')}
          disabled={mutation.isPending}
        >
          {mutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <CheckCircle2 className="mr-2 h-4 w-4" />}
          {t('amendments.decision.vote.APPROVE')}
        </Button>
        <Button
          variant="outline"
          onClick={() => handleVote('REVISION')}
          disabled={mutation.isPending}
        >
          <RotateCcw className="mr-2 h-4 w-4" />
          {t('amendments.decision.vote.REVISION')}
        </Button>
        <Button
          variant="destructive"
          onClick={() => handleVote('REJECT')}
          disabled={mutation.isPending}
        >
          <XCircle className="mr-2 h-4 w-4" />
          {t('amendments.decision.vote.REJECT')}
        </Button>
      </div>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
