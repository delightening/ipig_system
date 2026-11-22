import { useId, useState, useSyncExternalStore } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { toast } from '@/components/ui/use-toast'
import { animalFieldCorrectionApi } from '@/lib/api/animalFieldCorrection'
import { getApiErrorMessage } from '@/lib/apiError'
import { isIsoDateAfter, taipeiTodayIso } from '@/lib/utils'
import type { AnimalEvent } from '@/types/animal'

/** 補登原因的長度上限，與後端一致（超過會被拒）。 */
const REASON_MAX_CHARS = 500

function subscribeOnline(onChange: () => void) {
  window.addEventListener('online', onChange)
  window.addEventListener('offline', onChange)
  return () => {
    window.removeEventListener('online', onChange)
    window.removeEventListener('offline', onChange)
  }
}

interface BackfillIacucEventDateDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  /** 要補登的 IACUC 變更事件；沒有選到事件時不顯示對話框內容。 */
  event: AnimalEvent | null
}

/**
 * 為 IACUC 變更事件補登實際變更日期。
 *
 * 日期是否合理（不早於進場日、不晚於登錄日與下一事件、第一次分配不晚於第一筆需計畫的試驗紀錄）
 * 由後端依該動物的全部事件計算並驗證；超出範圍時後端的錯誤訊息會帶出允許的起訖日，
 * 直接顯示在對話框內讓使用者修正，不在前端重複一份規則。
 */
export function BackfillIacucEventDateDialog({
  open,
  onOpenChange,
  animalId,
  event,
}: BackfillIacucEventDateDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dateId = useId()
  const reasonId = useId()
  const [date, setDate] = useState('')
  const [reason, setReason] = useState('')
  // 離線時 TanStack Query 會把寫入暫停排隊，送出中的對話框又不可關閉，使用者會被鎖住；
  // 所以離線就直接停用「送出」，不讓請求進入暫停狀態
  const online = useSyncExternalStore(subscribeOnline, () => navigator.onLine, () => true)

  const todayIso = taipeiTodayIso()
  const dateInFuture = isIsoDateAfter(date, todayIso)

  const mutation = useMutation({
    mutationFn: (req: { eventId: string; actual_date: string; reason: string }) =>
      animalFieldCorrectionApi.backfillIacucEventDate(animalId, req.eventId, {
        actual_date: req.actual_date,
        reason: req.reason,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-iacuc-events', animalId] })
      toast({ title: t('animalActions.timeline.backfillSuccess') })
      close()
    },
  })

  const close = () => {
    setDate('')
    setReason('')
    mutation.reset()
    onOpenChange(false)
  }

  const none = t('animalActions.timeline.none')
  const change = event
    ? `${event.before_data?.iacuc_no || none} → ${event.after_data?.iacuc_no || none}`
    : ''
  const canSubmit =
    event !== null && online && date !== '' && !dateInFuture && reason.trim() !== ''

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (event === null || !canSubmit || mutation.isPending) return
    mutation.mutate({ eventId: event.id, actual_date: date, reason: reason.trim() })
  }

  return (
    // 送出中不可關閉：關掉對話框並不會取消已送出的寫入，使用者會誤以為沒補登
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (mutation.isPending) return
        if (!next) close()
        else onOpenChange(true)
      }}
    >
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t('animalActions.timeline.backfillTitle')}</DialogTitle>
          <DialogDescription>
            {t('animalActions.timeline.backfillDescription', { change })}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor={dateId}>{t('animalActions.timeline.backfillDateLabel')}</Label>
              <Input
                id={dateId}
                type="date"
                value={date}
                max={todayIso}
                onChange={(e) => setDate(e.target.value)}
                aria-required
                aria-invalid={dateInFuture}
              />
              {dateInFuture ? (
                <p className="text-sm text-destructive" role="alert">
                  {t('animalActions.timeline.backfillDateFuture')}
                </p>
              ) : (
                <p className="text-xs text-muted-foreground">
                  {t('animalActions.timeline.backfillRangeHint')}
                </p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor={reasonId}>{t('animalActions.timeline.backfillReasonLabel')}</Label>
              <Textarea
                id={reasonId}
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder={t('animalActions.timeline.backfillReasonPlaceholder')}
                maxLength={REASON_MAX_CHARS}
                rows={3}
                aria-required
              />
            </div>
            {!online && (
              <p className="text-sm text-destructive" role="alert">
                {t('animalActions.timeline.backfillOffline')}
              </p>
            )}
            {mutation.isError && (
              <p className="text-sm text-destructive" role="alert">
                {getApiErrorMessage(mutation.error)}
              </p>
            )}
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={close}
              disabled={mutation.isPending}
            >
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={!canSubmit || mutation.isPending}>
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('animalActions.timeline.backfillSubmit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
