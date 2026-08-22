// R53-5: 廢棄物再利用紀錄 dialog — 對應 R53-4 POST/PATCH endpoints。
//
// requester 二選一：in-system FK (requester_user_id) 或 external 機構名 +
// 聯絡人 (requester_org_name + requester_contact_name，須同時填)。Form 用
// radio 切換。Backend service / migration CHECK 雙層守衛，前端 UI 也擋一次。
import { useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
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
      toast({ title: isEdit ? '已更新採樣紀錄' : '已新增採樣紀錄' })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, isEdit ? '更新失敗' : '新增失敗'),
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!sampledAtLocal) {
      toast({ title: '錯誤', description: '請選擇採樣時間', variant: 'destructive' })
      return
    }
    if (!sampleContent.trim()) {
      toast({ title: '錯誤', description: '請填寫採樣內容', variant: 'destructive' })
      return
    }
    if (
      requesterMode === 'external' &&
      (!requesterOrgName.trim() || !requesterContactName.trim())
    ) {
      toast({
        title: '錯誤',
        description: '外部需求方需填寫機構名與聯絡人',
        variant: 'destructive',
      })
      return
    }
    if (requesterMode === 'internal' && !requesterUserId.trim()) {
      toast({
        title: '錯誤',
        description: '系統內需求方需填寫 user id',
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
            {isEdit ? '編輯' : '新增'}再利用記錄
          </DialogTitle>
          <DialogDescription>耳號：{earTag}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="sampled_at">採樣時間 *</Label>
            <Input
              id="sampled_at"
              type="datetime-local"
              value={sampledAtLocal}
              onChange={(e) => setSampledAtLocal(e.target.value)}
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="sample_content">採樣內容 *</Label>
            <Textarea
              id="sample_content"
              value={sampleContent}
              onChange={(e) => setSampleContent(e.target.value)}
              placeholder="例如：左後肢肌肉組織 5g、心臟血 10mL"
              className="min-h-[80px]"
              required
            />
          </div>

          <div className="space-y-2">
            <Label>需求方類型 *</Label>
            <div className="flex gap-4 text-sm">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="requester_mode"
                  checked={requesterMode === 'external'}
                  onChange={() => setRequesterMode('external')}
                />
                外部（機構 / 聯絡人）
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="requester_mode"
                  checked={requesterMode === 'internal'}
                  onChange={() => setRequesterMode('internal')}
                />
                系統內（user id）
              </label>
            </div>
            {requesterMode === 'external' ? (
              <div className="space-y-2">
                <Input
                  value={requesterOrgName}
                  onChange={(e) => setRequesterOrgName(e.target.value)}
                  placeholder="機構名（例：國防醫學大學）"
                  required
                />
                <Input
                  value={requesterContactName}
                  onChange={(e) => setRequesterContactName(e.target.value)}
                  placeholder="聯絡人（例：王教授）"
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
            <Label htmlFor="notes">備註</Label>
            <Textarea
              id="notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="可空白"
              className="min-h-[60px]"
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {isEdit ? '更新' : '新增'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
