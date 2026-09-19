// R53-5: 廢棄物再利用紀錄 dialog — 對應 R53-4 POST/PATCH endpoints。
//
// requester 二選一：in-system FK (requester_user_id) 或 external 機構名 +
// 聯絡人 (requester_org_name + requester_contact_name，須同時填)。Form 用
// radio 切換。Backend service / migration CHECK 雙層守衛，前端 UI 也擋一次。
import { useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2, Recycle } from 'lucide-react'

import {
  byproductSampleApi,
  type ByproductSample,
  type CreateByproductSampleRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** 新建走 animal-path（#445）：byproduct 主要來自計劃內犧牲，不綁安樂死單。 */
  animalId: string
  earTag: string
  /** 編輯模式時帶入既有 row；undefined = 新建 */
  existing?: ByproductSample
}

type RequesterMode = 'internal' | 'external'

export function ByproductSampleDialog({
  open,
  onOpenChange,
  animalId,
  earTag,
  existing,
}: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const isEdit = Boolean(existing)

  const [sampledAtLocal, setSampledAtLocal] = useState('')
  const [sampleContent, setSampleContent] = useState('')
  const [requesterMode, setRequesterMode] = useState<RequesterMode>('external')
  const [requesterOrgName, setRequesterOrgName] = useState('')
  const [requesterContactName, setRequesterContactName] = useState('')
  const [requesterUserId, setRequesterUserId] = useState('')
  const [notes, setNotes] = useState('')

  useEffect(() => {
    if (!open) return
    if (existing) {
      // ISO UTC → local datetime-local
      const localISO = new Date(existing.sampled_at)
      const tzOffsetMin = localISO.getTimezoneOffset()
      const localStr = new Date(localISO.getTime() - tzOffsetMin * 60_000)
        .toISOString()
        .slice(0, 16)
      setSampledAtLocal(localStr)
      setSampleContent(existing.sample_content)
      if (existing.requester_user_id) {
        setRequesterMode('internal')
        setRequesterUserId(existing.requester_user_id)
        setRequesterOrgName('')
        setRequesterContactName('')
      } else {
        setRequesterMode('external')
        setRequesterOrgName(existing.requester_org_name ?? '')
        setRequesterContactName(existing.requester_contact_name ?? '')
        setRequesterUserId('')
      }
      setNotes(existing.notes ?? '')
    } else {
      // 新建：預設今天此刻
      const now = new Date()
      const tzOffsetMin = now.getTimezoneOffset()
      const localStr = new Date(now.getTime() - tzOffsetMin * 60_000)
        .toISOString()
        .slice(0, 16)
      setSampledAtLocal(localStr)
      setSampleContent('')
      setRequesterMode('external')
      setRequesterOrgName('')
      setRequesterContactName('')
      setRequesterUserId('')
      setNotes('')
    }
  }, [open, existing])

  const mutation = useMutation({
    mutationFn: async () => {
      const sampledAtUtc = new Date(sampledAtLocal).toISOString()
      if (isEdit && existing) {
        return byproductSampleApi.update(existing.id, {
          sampled_at: sampledAtUtc,
          sample_content: sampleContent,
          // 整組覆寫：只送當前模式的 requester 欄位，backend 會清掉另一型別殘留。
          requester_user_id:
            requesterMode === 'internal' ? requesterUserId : undefined,
          requester_org_name:
            requesterMode === 'external' ? requesterOrgName : undefined,
          requester_contact_name:
            requesterMode === 'external' ? requesterContactName : undefined,
          notes: notes || undefined,
        })
      }
      // #445：走 animal-path 建立（body 不帶 animal_id / source_protocol_id；
      // backend 從 path animal_id 推導來源計畫並驗動物已犧牲 — IDOR 守衛）。
      const payload: CreateByproductSampleRequest = {
        sampled_at: sampledAtUtc,
        sample_content: sampleContent,
        notes: notes || undefined,
      }
      if (requesterMode === 'internal') {
        payload.requester_user_id = requesterUserId
      } else {
        payload.requester_org_name = requesterOrgName
        payload.requester_contact_name = requesterContactName
      }
      return byproductSampleApi.createForAnimal(animalId, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['byproduct-samples', 'animal', animalId],
      })
      toast({ title: isEdit ? t('animalActions.byproduct.dialog.updatedToast') : t('animalActions.byproduct.dialog.createdToast') })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, isEdit ? t('animalActions.common.updateFailed') : t('animalActions.byproduct.dialog.createFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!sampledAtLocal) {
      toast({ title: t('common.error'), description: t('animalActions.byproduct.dialog.sampledAtRequired'), variant: 'destructive' })
      return
    }
    if (!sampleContent.trim()) {
      toast({ title: t('common.error'), description: t('animalActions.byproduct.dialog.sampleContentRequired'), variant: 'destructive' })
      return
    }
    if (
      requesterMode === 'external' &&
      (!requesterOrgName.trim() || !requesterContactName.trim())
    ) {
      toast({
        title: t('common.error'),
        description: t('animalActions.byproduct.dialog.externalRequesterRequired'),
        variant: 'destructive',
      })
      return
    }
    if (requesterMode === 'internal' && !requesterUserId.trim()) {
      toast({
        title: t('common.error'),
        description: t('animalActions.byproduct.dialog.internalRequesterRequired'),
        variant: 'destructive',
      })
      return
    }
    mutation.mutate()
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Recycle className="h-5 w-5" />
            {isEdit ? t('animalActions.byproduct.dialog.editTitle') : t('animalActions.byproduct.dialog.createTitle')}
          </DialogTitle>
          <DialogDescription>{t('animalActions.common.earTagLabel', { earTag })}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="sampled_at">{t('animalActions.byproduct.dialog.sampledAtLabel')}</Label>
            <Input
              id="sampled_at"
              type="datetime-local"
              value={sampledAtLocal}
              onChange={(e) => setSampledAtLocal(e.target.value)}
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="sample_content">{t('animalActions.byproduct.dialog.sampleContentLabel')}</Label>
            <Textarea
              id="sample_content"
              value={sampleContent}
              onChange={(e) => setSampleContent(e.target.value)}
              placeholder={t('animalActions.byproduct.dialog.sampleContentHint')}
              className="min-h-[80px]"
              required
            />
          </div>

          <div className="space-y-2">
            <Label>{t('animalActions.byproduct.dialog.requesterTypeLabel')}</Label>
            <div className="flex gap-4 text-sm">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="requester_mode"
                  checked={requesterMode === 'external'}
                  onChange={() => setRequesterMode('external')}
                />
                {t('animalActions.byproduct.dialog.requesterExternal')}
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="requester_mode"
                  checked={requesterMode === 'internal'}
                  onChange={() => setRequesterMode('internal')}
                />
                {t('animalActions.byproduct.dialog.requesterInternal')}
              </label>
            </div>
            {requesterMode === 'external' ? (
              <div className="space-y-2">
                <Input
                  value={requesterOrgName}
                  onChange={(e) => setRequesterOrgName(e.target.value)}
                  placeholder={t('animalActions.byproduct.dialog.orgNameHint')}
                  required
                />
                <Input
                  value={requesterContactName}
                  onChange={(e) => setRequesterContactName(e.target.value)}
                  placeholder={t('animalActions.byproduct.dialog.contactNameHint')}
                  required
                />
              </div>
            ) : (
              <Input
                value={requesterUserId}
                onChange={(e) => setRequesterUserId(e.target.value)}
                placeholder="user UUID"
                required
              />
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="notes">{t('animalActions.common.notes')}</Label>
            <Textarea
              id="notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder={t('animalActions.byproduct.dialog.notesHint')}
              className="min-h-[60px]"
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {isEdit ? t('common.update') : t('common.create')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
