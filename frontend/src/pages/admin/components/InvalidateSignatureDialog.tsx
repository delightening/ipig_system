import { useState } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import { useMutation } from '@tanstack/react-query'

import api from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

// R30-9b: 撤銷電子簽章對話框（admin only，由 AuditLogsPage 入口開啟）
//
// 設計原則：
// - 撤銷不可逆 — 要求密碼 + 理由雙確認
// - 不在頁面常駐 — 點按鈕才開啟，避免誤點
// - response 不回 signer / record 細節（與後端 handler 對齊；admin 想看細節
//   走 audit log 詳細頁查 SIGNATURE_INVALIDATED 事件）
// - 撤銷成功提醒「不會自動 revert 已使用此簽章的記錄」（避免管理員誤以為
//   一鍵就能 undo 已 APPROVED amendment）

interface InvalidateSignatureDialogProps {
  open: boolean
  onClose: () => void
}

// 對齊 backend `utils::validation::validate_reason_text`：trim 後 1~1000 字
const REASON_MAX = 1000
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function validateInvalidateSignatureForm(
  input: {
    signatureId: string
    reason: string
    password: string
  },
  t: (key: string, options?: Record<string, unknown>) => string,
): { signatureId: string; reason: string; password: string } {
  const signatureId = input.signatureId.trim()
  const reason = input.reason.trim()
  if (!UUID_PATTERN.test(signatureId)) throw new Error(t('admin.invalidateSignatureDialog.errorInvalidUuid'))
  if (reason.length < 1) throw new Error(t('admin.invalidateSignatureDialog.errorReasonRequired'))
  if (reason.length > REASON_MAX)
    throw new Error(t('admin.invalidateSignatureDialog.errorReasonTooLong', { max: REASON_MAX }))
  if (input.password.length < 1) throw new Error(t('admin.invalidateSignatureDialog.errorPasswordRequired'))
  return { signatureId, reason, password: input.password }
}

export function InvalidateSignatureDialog({ open, onClose }: InvalidateSignatureDialogProps) {
  const { t } = useTranslation()
  const [signatureId, setSignatureId] = useState('')
  const [reason, setReason] = useState('')
  const [password, setPassword] = useState('')

  const reset = () => {
    setSignatureId('')
    setReason('')
    setPassword('')
  }

  const handleClose = () => {
    reset()
    onClose()
  }

  const mutation = useMutation({
    mutationFn: async () => {
      // trim + UUID format + 長度驗證，失敗 throw Error 由 onError 經
      // getApiErrorMessage 攤平顯示。規則與 backend validate_reason_text 對齊。
      const parsed = validateInvalidateSignatureForm({ signatureId, reason, password }, t)
      await api.post(`/signatures/${parsed.signatureId}/invalidate`, {
        reason: parsed.reason,
        password: parsed.password,
      })
    },
    onSuccess: () => {
      toast({
        title: t('admin.invalidateSignatureDialog.successTitle'),
        description: t('admin.invalidateSignatureDialog.successDescription'),
      })
      handleClose()
    },
    onError: (err: unknown) => {
      toast({
        title: t('admin.invalidateSignatureDialog.errorTitle'),
        description: getApiErrorMessage(err, t('admin.invalidateSignatureDialog.errorFallback')),
        variant: 'destructive',
      })
    },
  })

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        // 撤銷請求進行中禁止 Escape / 遮罩關閉，避免使用者誤以為操作已取消
        if (!o && !mutation.isPending) handleClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.invalidateSignatureDialog.title')}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <p className="text-sm text-muted-foreground">
            <Trans
              i18nKey="admin.invalidateSignatureDialog.warning"
              components={{ strong: <strong className="text-destructive" /> }}
            />
          </p>

          <div className="space-y-2">
            <Label htmlFor="invalidate-sig-id">{t('admin.invalidateSignatureDialog.signatureIdLabel')}</Label>
            <Input
              id="invalidate-sig-id"
              value={signatureId}
              onChange={(e) => setSignatureId(e.target.value)}
              placeholder="00000000-0000-0000-0000-000000000000"
              disabled={mutation.isPending}
              autoComplete="off"
            />
            <p className="text-xs text-muted-foreground">
              {t('admin.invalidateSignatureDialog.signatureIdHint')}
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="invalidate-reason">{t('admin.invalidateSignatureDialog.reasonLabel')}</Label>
            <Textarea
              id="invalidate-reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder={t('admin.invalidateSignatureDialog.reasonPlaceholder')}
              rows={4}
              maxLength={REASON_MAX}
              disabled={mutation.isPending}
            />
            <p className="text-xs text-muted-foreground">
              {t('admin.invalidateSignatureDialog.reasonCounter', { count: reason.length, max: REASON_MAX })}
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="invalidate-password">{t('admin.invalidateSignatureDialog.passwordLabel')}</Label>
            <Input
              id="invalidate-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('admin.invalidateSignatureDialog.passwordPlaceholder')}
              disabled={mutation.isPending}
              autoComplete="current-password"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={mutation.isPending}>
            {t('common.cancel')}
          </Button>
          <Button
            variant="destructive"
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending || !signatureId.trim() || !reason.trim() || !password}
          >
            {mutation.isPending
              ? t('admin.invalidateSignatureDialog.submitting')
              : t('admin.invalidateSignatureDialog.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
