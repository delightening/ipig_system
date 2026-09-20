import { useMemo, useState } from 'react'
import { useParams, useSearchParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  HandwrittenSignaturePad,
  type SignatureData,
} from '@/components/ui/handwritten-signature-pad'
import { submitSignatureBridgePublic } from '@/lib/api/system'
import { getErrorMessage } from '@/types/error'

/**
 * R30-27c-2：手機從 QR 開的公開簽名頁。
 *
 * 流程：桌機 → POST /signing-bridge/start → 拿 session_id + mobile_token →
 * 編進 QR `https://host/sign/:id?token=...` → 手機掃描開本頁 → 輸入密碼 +
 * 手寫簽名 → POST /public/signing-bridge/:id/submit（mobile_token bearer）→
 * 桌機輪詢 status COMPLETED → consume payload → 自動套入 mutation。
 *
 * 安全：本頁不需 JWT；mobile_token 為 64-char crypto-random，5min TTL，
 * 單次使用，從 query string 取（QR 編碼）。token 洩漏即拿到 session 寫入權，
 * 但每個 session 對應特定 admin 的特定操作（purpose），且短命單次使用，
 * 攻擊面有限。
 */
export function MobileSignPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token') ?? ''
  const purpose = searchParams.get('purpose') ?? ''

  const [password, setPassword] = useState('')
  const [signature, setSignature] = useState<SignatureData | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [done, setDone] = useState(false)

  const purposeLabel = useMemo(() => {
    if (!purpose) return t('mobileSign.purposeDefault')
    if (purpose.startsWith('role.')) {
      const op = purpose.split('.')[1]
      return t('mobileSign.purposeRole', {
        op:
          op === 'create'
            ? t('mobileSign.roleOps.create')
            : op === 'update'
              ? t('mobileSign.roleOps.update')
              : op === 'delete'
                ? t('mobileSign.roleOps.delete')
                : op,
      })
    }
    return purpose
  }, [purpose, t])

  const missing = !id || !token

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()
    if (!id || !token) {
      setError(t('mobileSign.invalidLinkRegenerate'))
      return
    }
    if (!password.trim()) {
      setError(t('auth.validation.passwordRequired'))
      return
    }
    if (!signature?.svg) {
      setError(t('auth.validation.signatureRequired'))
      return
    }
    setError(null)
    setIsSubmitting(true)
    try {
      await submitSignatureBridgePublic(id, token, {
        password,
        handwriting_svg: signature.svg,
        stroke_data: signature.strokeData,
      })
      setDone(true)
    } catch (err) {
      setError(getErrorMessage(err) || t('mobileSign.submitFailed'))
    } finally {
      setIsSubmitting(false)
    }
  }

  if (done) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-background">
        <div className="w-full max-w-md space-y-4 text-center">
          <h1 className="text-2xl font-semibold">{t('mobileSign.doneTitle')}</h1>
          <p className="text-muted-foreground">
            {t('mobileSign.doneDescription')}
          </p>
        </div>
      </div>
    )
  }

  if (missing) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-background">
        <div className="w-full max-w-md space-y-4 text-center">
          <h1 className="text-2xl font-semibold text-destructive">{t('mobileSign.invalidTitle')}</h1>
          <p className="text-muted-foreground">
            {t('mobileSign.invalidDescription')}
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-start justify-center p-4 bg-background">
      <div className="w-full max-w-md space-y-4 py-6">
        <div className="space-y-1">
          <h1 className="text-xl font-semibold">{t('mobileSign.title')}</h1>
          <p className="text-sm text-muted-foreground">
            {t('mobileSign.operationItem', { purpose: purposeLabel })}
          </p>
          <p className="text-xs text-muted-foreground">
            {t('mobileSign.qrNotice')}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="mobile-sign-password">{t('auth.confirmPassword.passwordLabel')}</Label>
            <Input
              id="mobile-sign-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('auth.confirmPassword.passwordPlaceholder')}
              disabled={isSubmitting}
              autoComplete="current-password"
              autoFocus
            />
          </div>

          <div className="space-y-2">
            <Label>{t('signature.handwriting')}</Label>
            <HandwrittenSignaturePad
              onSignatureChange={setSignature}
              height={200}
            />
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}

          <Button type="submit" className="w-full" disabled={isSubmitting}>
            {isSubmitting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('mobileSign.submit')}
          </Button>
        </form>
      </div>
    </div>
  )
}

export default MobileSignPage
