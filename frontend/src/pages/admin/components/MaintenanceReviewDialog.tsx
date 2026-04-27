/**
 * 維修/保養驗收簽核 Dialog
 * — 顯示紀錄摘要 + 驗收備註 + 電子簽章（手寫）
 */
import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'
import { toast } from '@/components/ui/use-toast'
import api from '@/lib/api'
import { signatureApi, type SignRecordRequest } from '@/lib/api/signature'
import { getApiErrorMessage } from '@/lib/validation'

import type { MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS } from '../types'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  record: MaintenanceRecordWithDetails | null
  mode: 'approve' | 'reject'
}

export function MaintenanceReviewDialog({ open, onOpenChange, record, mode }: Props) {
  const queryClient = useQueryClient()
  const [reviewNotes, setReviewNotes] = useState('')
  const [signatureData, setSignatureData] = useState<SignatureData | null>(null)
  const [password, setPassword] = useState('')

  const isApprove = mode === 'approve'

  const reviewMutation = useMutation({
    mutationFn: async () => {
      if (!record) return

      // 驗收通過需要簽章 + 密碼（GLP 21 CFR 11 雙因子）
      if (isApprove && signatureData) {
        const sigReq: SignRecordRequest = {
          handwriting_svg: signatureData.svg,
          stroke_data: signatureData.strokeData,
          password,
        }
        await signatureApi.signMaintenanceReviewer(record.id, sigReq)
      }

      await api.post(`/equipment-maintenance/${record.id}/review`, {
        approved: isApprove,
        review_notes: reviewNotes || null,
      })
    },
    onSuccess: () => {
      toast({
        title: isApprove ? '驗收通過' : '已退回',
        description: isApprove ? '設備已恢復啟用' : '紀錄已退回維修人員',
      })
      queryClient.invalidateQueries({ queryKey: ['equipment-maintenance'] })
      queryClient.invalidateQueries({ queryKey: ['recent-maintenance'] })
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      handleClose()
    },
    onError: (error) => {
      toast({
        title: '操作失敗',
        description: getApiErrorMessage(error, '請稍後再試'),
        variant: 'destructive',
      })
    },
  })

  const handleClose = () => {
    setReviewNotes('')
    setSignatureData(null)
    setPassword('')
    onOpenChange(false)
  }

  if (!record) return null

  const description = record.maintenance_type === 'repair'
    ? record.problem_description
    : record.maintenance_items

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{isApprove ? '驗收簽核' : '退回維修'}</DialogTitle>
          <DialogDescription>
            {isApprove ? '確認設備恢復正常並簽章' : '將紀錄退回維修人員重新處理'}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="p-3 bg-muted rounded-lg text-sm space-y-1">
            <p><span className="text-muted-foreground">設備：</span>{record.equipment_name}</p>
            <p><span className="text-muted-foreground">類型：</span>{MAINTENANCE_TYPE_LABELS[record.maintenance_type]}</p>
            {description && (
              <p><span className="text-muted-foreground">描述：</span>{description}</p>
            )}
            {record.repair_content && (
              <p><span className="text-muted-foreground">維修內容：</span>{record.repair_content}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label>{isApprove ? '驗收備註' : '退回原因'}</Label>
            <Textarea
              value={reviewNotes}
              onChange={(e) => setReviewNotes(e.target.value)}
              placeholder={isApprove ? '選填，驗收備註...' : '請說明退回原因...'}
              rows={2}
            />
          </div>

          {isApprove && (
            <>
              <div className="space-y-2">
                <Label>電子簽章</Label>
                <HandwrittenSignaturePad
                  onSignatureChange={setSignatureData}
                  height={140}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="maintenance-review-password">登入密碼</Label>
                <Input
                  id="maintenance-review-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="請輸入您的登入密碼以完成簽章"
                  autoComplete="current-password"
                />
                <p className="text-xs text-muted-foreground">
                  GLP 合規：手寫簽章須再驗證密碼以確認身分
                </p>
              </div>
            </>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onClick={handleClose}>
              取消
            </Button>
            <Button
              variant={isApprove ? 'default' : 'destructive'}
              onClick={() => reviewMutation.mutate()}
              disabled={
                reviewMutation.isPending ||
                (isApprove && (!signatureData || password.length === 0))
              }
            >
              {reviewMutation.isPending
                ? '處理中...'
                : isApprove ? '驗收通過' : '確認退回'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
