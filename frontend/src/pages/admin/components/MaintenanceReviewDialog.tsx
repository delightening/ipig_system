/**
 * 維修/保養驗收簽核 Dialog
 * — 顯示紀錄摘要 + 驗收備註 + 電子簽章（手寫）
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
import { getApiErrorMessage } from '@/lib/apiError'

import type { MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS } from '../types'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  record: MaintenanceRecordWithDetails | null
  mode: 'approve' | 'reject'
}

export function MaintenanceReviewDialog({ open, onOpenChange, record, mode }: Props) {
  const { t } = useTranslation()
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
        title: isApprove ? t('admin.maintenanceReviewDialog.approveSuccessTitle') : t('admin.maintenanceReviewDialog.rejectSuccessTitle'),
        description: isApprove ? t('admin.maintenanceReviewDialog.approveSuccessDescription') : t('admin.maintenanceReviewDialog.rejectSuccessDescription'),
      })
      queryClient.invalidateQueries({ queryKey: ['equipment-maintenance'] })
      queryClient.invalidateQueries({ queryKey: ['recent-maintenance'] })
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      handleClose()
    },
    onError: (error) => {
      toast({
        title: t('admin.maintenanceReviewDialog.operationFailed'),
        description: getApiErrorMessage(error, t('admin.maintenanceReviewDialog.tryAgainLater')),
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
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isApprove ? t('admin.maintenanceReviewDialog.approveTitle') : t('admin.maintenanceReviewDialog.rejectTitle')}</DialogTitle>
          <DialogDescription>
            {isApprove ? t('admin.maintenanceReviewDialog.approveDescription') : t('admin.maintenanceReviewDialog.rejectDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="p-3 bg-muted rounded-lg text-sm space-y-1">
            <p><span className="text-muted-foreground">{t('admin.maintenanceReviewDialog.equipmentLabel')}</span>{record.equipment_name}</p>
            <p><span className="text-muted-foreground">{t('admin.maintenanceReviewDialog.typeLabel')}</span>{t(MAINTENANCE_TYPE_LABELS[record.maintenance_type] ?? record.maintenance_type)}</p>
            {description && (
              <p><span className="text-muted-foreground">{t('admin.maintenanceReviewDialog.descriptionLabel')}</span>{description}</p>
            )}
            {record.repair_content && (
              <p><span className="text-muted-foreground">{t('admin.maintenanceReviewDialog.repairContentLabel')}</span>{record.repair_content}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label>{isApprove ? t('admin.maintenanceReviewDialog.reviewNotesLabel') : t('admin.maintenanceReviewDialog.rejectReasonLabel')}</Label>
            <Textarea
              value={reviewNotes}
              onChange={(e) => setReviewNotes(e.target.value)}
              placeholder={isApprove ? t('admin.maintenanceReviewDialog.reviewNotesPlaceholder') : t('admin.maintenanceReviewDialog.rejectReasonPlaceholder')}
              rows={2}
            />
          </div>

          {isApprove && (
            <>
              <div className="space-y-2">
                <Label>{t('admin.maintenanceReviewDialog.signatureLabel')}</Label>
                <HandwrittenSignaturePad
                  onSignatureChange={setSignatureData}
                  height={140}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="maintenance-review-password">{t('admin.maintenanceReviewDialog.passwordLabel')}</Label>
                <Input
                  id="maintenance-review-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder={t('admin.maintenanceReviewDialog.passwordPlaceholder')}
                  autoComplete="current-password"
                />
                <p className="text-xs text-muted-foreground">
                  {t('admin.maintenanceReviewDialog.glpComplianceHint')}
                </p>
              </div>
            </>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onClick={handleClose}>
              {t('common.cancel')}
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
                ? t('common.processed')
                : isApprove ? t('admin.maintenanceReviewDialog.approveButton') : t('admin.maintenanceReviewDialog.rejectButton')}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
