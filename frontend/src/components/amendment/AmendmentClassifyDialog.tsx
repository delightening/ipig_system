import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { amendmentApi } from '@/lib/api'
import { useInvalidateAmendment } from '@/hooks/useInvalidateAmendment'
import { Button } from '@/components/ui/button'
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
import type { AmendmentType, ClassifyAmendmentRequest } from '@/types/amendment'

type ClassifyType = 'MAJOR' | 'MINOR'

interface AmendmentClassifyDialogProps {
  amendmentId: string
  open: boolean
  onOpenChange: (open: boolean) => void
}

/**
 * 分類對話框（Phase 3）。POST /amendments/:id/classify。
 * MINOR → 立即 ADMIN_APPROVED（終態，一筆行政簽章）；
 * MAJOR → 指派審查委員、進入需全員投票的審查流程。
 */
export function AmendmentClassifyDialog({ amendmentId, open, onOpenChange }: AmendmentClassifyDialogProps) {
  const { t } = useTranslation()
  const invalidateAmendment = useInvalidateAmendment(amendmentId)
  const [type, setType] = useState<ClassifyType>('MAJOR')
  const [remark, setRemark] = useState('')

  const reset = () => {
    setType('MAJOR')
    setRemark('')
  }

  const mutation = useMutation({
    mutationFn: (data: ClassifyAmendmentRequest) => amendmentApi.classify(amendmentId, data),
    onSuccess: () => {
      invalidateAmendment()
      toast({ title: t('common.success'), description: t('amendments.decision.classify.success') })
      reset()
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('amendments.decision.classify.failed')),
        variant: 'destructive',
      })
    },
  })

  const handleConfirm = () => {
    mutation.mutate({
      amendment_type: type as AmendmentType,
      remark: remark.trim() || undefined,
    })
  }

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) reset(); onOpenChange(o) }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('amendments.decision.classify.title')}</DialogTitle>
          <DialogDescription>{t('amendments.decision.classify.desc')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('amendments.decision.classify.typeLabel')}</Label>
            <Select value={type} onValueChange={(v) => setType(v as ClassifyType)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="MAJOR">{t('amendments.types.MAJOR')}</SelectItem>
                <SelectItem value="MINOR">{t('amendments.types.MINOR')}</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {type === 'MINOR'
                ? t('amendments.decision.classify.minorHint')
                : t('amendments.decision.classify.majorHint')}
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="amendment-classify-remark">{t('amendments.decision.classify.remarkLabel')}</Label>
            <Textarea
              id="amendment-classify-remark"
              value={remark}
              onChange={(e) => setRemark(e.target.value)}
              placeholder={t('amendments.decision.classify.remarkPlaceholder')}
              rows={3}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => { reset(); onOpenChange(false) }}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleConfirm} disabled={mutation.isPending}>
            {mutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('amendments.decision.classify.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
