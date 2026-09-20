/**
 * R30-16: 修正血檢單筆 item Dialog
 *
 * 行為：append-only supersede。原 row 標記 superseded，新 row 為 current。
 * `correction_reason` 必填 ≥10 字。
 */
import { useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { bloodTestApi } from '@/lib/api'
import type { AnimalBloodTestItem, CorrectBloodTestItemRequest } from '@/types'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function validateCorrection(draft: CorrectBloodTestItemRequest, t: TFunction): string | null {
  const name = draft.item_name.trim()
  if (name.length === 0) return t('validation.required')
  if (name.length > 200) return t('animalRecords.bloodTest.validation.nameTooLong')
  if (draft.template_id !== undefined && !UUID_PATTERN.test(draft.template_id)) {
    return t('animalRecords.bloodTest.validation.templateIdInvalid')
  }
  const reason = draft.correction_reason.trim()
  if (reason.length < 10) return t('animalRecords.bloodTest.validation.reasonTooShort')
  if (reason.length > 500) return t('animalRecords.bloodTest.validation.reasonTooLong')
  return null
}

interface Props {
  open: boolean
  testId: string
  item: AnimalBloodTestItem | null
  onOpenChange: (open: boolean) => void
  /** R30-16: 修正成功後通知 parent 重新載入 detail（避免 stale editingItems 對已 superseded row 再開 dialog） */
  onCorrected?: () => void
}

export function CorrectBloodTestItemDialog({
  open,
  testId,
  item,
  onOpenChange,
  onCorrected,
}: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [draft, setDraft] = useState<CorrectBloodTestItemRequest | null>(null)

  // R30-16: 切換到不同 item 時重設 draft（用 item.id 而非 item_name 比對，避免同名 item 不更新）
  useEffect(() => {
    if (!item) {
      setDraft(null)
      return
    }
    setDraft({
      item_name: item.item_name,
      // 後端 Option<Uuid> 序列化成 null；Zod uuid() 不收 null，正規化為 undefined
      template_id: item.template_id ?? undefined,
      result_value: item.result_value || '',
      result_unit: item.result_unit || '',
      reference_range: item.reference_range || '',
      is_abnormal: item.is_abnormal,
      remark: item.remark || '',
      sort_order: item.sort_order,
      correction_reason: '',
    })
  }, [item])

  const correctMutation = useMutation({
    mutationFn: (payload: CorrectBloodTestItemRequest) =>
      bloodTestApi.correctItem(testId, item!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-detail'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-item-history', testId] })
      // 列表頁的 abnormal_count / item_count 受 is_abnormal 變動影響
      queryClient.invalidateQueries({ queryKey: ['animal-blood-tests'] })
      onCorrected?.()
      toast({ title: t('animalRecords.bloodTest.correctedTitle'), description: t('animalRecords.bloodTest.correctedDescription') })
      onOpenChange(false)
      setDraft(null)
    },
    onError: () => {
      toast({ title: t('common.error'), description: t('animalRecords.bloodTest.correctFailed'), variant: 'destructive' })
    },
  })

  const handleSubmit = () => {
    if (!draft) return
    const error = validateCorrection(draft, t)
    if (error) {
      toast({ title: t('common.error'), description: error, variant: 'destructive' })
      return
    }
    // trim string fields 對齊原 schema .trim() 行為
    correctMutation.mutate({
      ...draft,
      item_name: draft.item_name.trim(),
      correction_reason: draft.correction_reason.trim(),
    })
  }

  if (!item || !draft) return null

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) { onOpenChange(false); setDraft(null) } }}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{t('animalRecords.bloodTest.correctTitle', { name: item.item_name })}</DialogTitle>
          <DialogDescription>
            {t('animalRecords.bloodTest.correctDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* 原值對照 */}
          <div className="bg-muted/50 rounded-lg p-3 space-y-1 text-sm">
            <div className="font-medium text-muted-foreground">{t('animalRecords.bloodTest.originalValue')}</div>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1">
              <div><span className="text-muted-foreground">{t('animalRecords.bloodTest.resultColon')}</span>{item.result_value || '—'} {item.result_unit || ''}</div>
              <div><span className="text-muted-foreground">{t('animalRecords.bloodTest.referenceColon')}</span>{item.reference_range || '—'}</div>
              <div><span className="text-muted-foreground">{t('animalRecords.bloodTest.abnormalColon')}</span>{item.is_abnormal ? t('common.yes') : t('common.no')}</div>
              <div><span className="text-muted-foreground">{t('animalRecords.bloodTest.remarkColon')}</span>{item.remark || '—'}</div>
            </div>
          </div>

          {/* 修正後新值 */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <Label>{t('animalRecords.bloodTest.itemNameRequired')}</Label>
              <Input
                value={draft.item_name}
                onChange={(e) => setDraft({ ...draft, item_name: e.target.value })}
                readOnly={!!item.template_id}
              />
            </div>
            <div className="space-y-1">
              <Label>{t('animalRecords.bloodTest.resultValue')}</Label>
              <Input
                value={draft.result_value || ''}
                onChange={(e) => setDraft({ ...draft, result_value: e.target.value })}
              />
            </div>
            <div className="space-y-1">
              <Label>{t('animalRecords.shared.unit')}</Label>
              <Input
                value={draft.result_unit || ''}
                onChange={(e) => setDraft({ ...draft, result_unit: e.target.value })}
              />
            </div>
            <div className="space-y-1">
              <Label>{t('animalRecords.bloodTest.referenceRange')}</Label>
              <Input
                value={draft.reference_range || ''}
                onChange={(e) => setDraft({ ...draft, reference_range: e.target.value })}
              />
            </div>
            <div className="space-y-1">
              <Label>{t('animalRecords.shared.remark')}</Label>
              <Input
                value={draft.remark || ''}
                onChange={(e) => setDraft({ ...draft, remark: e.target.value })}
              />
            </div>
            <div className="flex items-center gap-2 pt-6">
              <input
                id="correct-is-abnormal"
                type="checkbox"
                checked={draft.is_abnormal}
                onChange={(e) => setDraft({ ...draft, is_abnormal: e.target.checked })}
                className="h-4 w-4 rounded border-border text-status-error-text focus:ring-destructive"
              />
              <Label htmlFor="correct-is-abnormal">{t('animalRecords.bloodTest.markAbnormal')}</Label>
            </div>
          </div>

          {/* 修正原因（必填 ≥10 字） */}
          <div className="space-y-1">
            <Label>
              {t('animalRecords.bloodTest.correctionReasonRequired')} <span className="text-xs text-muted-foreground">{t('animalRecords.bloodTest.correctionReasonHint')}</span>
            </Label>
            <Textarea
              value={draft.correction_reason}
              onChange={(e) => setDraft({ ...draft, correction_reason: e.target.value })}
              placeholder={t('animalRecords.bloodTest.correctionReasonPlaceholder')}
              rows={3}
            />
            <div className="text-xs text-muted-foreground text-right">
              {draft.correction_reason.length} / 500
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => { onOpenChange(false); setDraft(null) }}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSubmit} disabled={correctMutation.isPending}>
            {correctMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('animalRecords.bloodTest.submitCorrection')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
