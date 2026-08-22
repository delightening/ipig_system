import { useMemo, useState } from 'react'
import { useParams, useSearchParams } from 'react-router-dom'
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
    if (!purpose) return '電子簽章'
    if (purpose.startsWith('role.')) {
      const op = purpose.split('.')[1]
      return `角色變更：${
        op === 'create' ? '建立' : op === 'update' ? '更新' : op === 'delete' ? '刪除' : op
      }`
    }
    return purpose
  }, [purpose])

  const missing = !id || !token

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()
    if (!id || !token) {
      setError('簽名連結無效，請從桌機重新產生 QR')
      return
    }
    if (!password.trim()) {
      setError('請輸入密碼')
      return
    }
    if (!signature?.svg) {
      setError('請完成手寫簽名')
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
      setError(getErrorMessage(err) || '簽章送出失敗，請確認密碼正確或 QR 是否已過期')
    } finally {
      setIsSubmitting(false)
    }
  }

  if (done) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-background">
        <div className="w-full max-w-md space-y-4 text-center">
          <h1 className="text-2xl font-semibold">簽署完成</h1>
          <p className="text-muted-foreground">
            請回到桌機，操作會在數秒內自動繼續。本頁可關閉。
          </p>
        </div>
      </div>
    )
  }

  if (missing) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-background">
        <div className="w-full max-w-md space-y-4 text-center">
          <h1 className="text-2xl font-semibold text-destructive">連結無效</h1>
          <p className="text-muted-foreground">
            缺少 session 或 token；請從桌機簽章視窗重新產生 QR。
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-start justify-center p-4 bg-background">
      <div className="w-full max-w-md space-y-4 py-6">
        <div className="space-y-1">
          <h1 className="text-xl font-semibold">手機簽名</h1>
          <p className="text-sm text-muted-foreground">
            操作項目：{purposeLabel}
          </p>
          <p className="text-xs text-muted-foreground">
            此 QR 5 分鐘內單次有效；簽名與密碼會記錄到稽核軌跡。
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="mobile-sign-password">您的登入密碼</Label>
            <Input
              id="mobile-sign-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="請輸入密碼以確認身份"
              disabled={isSubmitting}
              autoComplete="current-password"
              autoFocus
            />
          </div>

          <div className="space-y-2">
            <Label>手寫簽名</Label>
            <HandwrittenSignaturePad
              onSignatureChange={setSignature}
              height={200}
            />
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}

          <Button type="submit" className="w-full" disabled={isSubmitting}>
            {isSubmitting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            送出簽章
          </Button>
        </form>
      </div>
    </div>
  )
}

export default MobileSignPage
