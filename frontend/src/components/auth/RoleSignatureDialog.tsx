import { useEffect, useRef, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
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
import { Loader2, Smartphone, Monitor } from 'lucide-react'
import {
  HandwrittenSignaturePad,
  type SignatureData,
} from '@/components/ui/handwritten-signature-pad'
import {
  consumeSignatureBridge,
  getSignatureBridgeStatus,
  startSignatureBridge,
  type MutationSignaturePayload,
} from '@/lib/api/system'
import { getErrorMessage } from '@/types/error'

/**
 * R30-27b：role / permission 變更前的雙因子簽章 dialog
 *
 * UX：admin 在同一個 modal 內輸密碼 + 手寫簽名後，由 caller 取得 payload
 * 串入 mutation request body（mutation_signature 欄位）。對應 21 CFR §11.10(d)
 * 存取控制簽章不可否認性。
 *
 * R30-27c-2：桌機滑鼠手寫不友善，加「用手機簽」切換 — 開 bridge session、
 * 顯示 QR、輪詢 status，COMPLETED 後 consume payload 自動套入 mutation。
 */
export interface RoleSignatureDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  /** R30-27c-2：bridge purpose 字串（role.create / role.update / role.delete），audit 用 */
  purpose: string
  /** caller 取得 payload，由 caller 自行 mutation；失敗 throw 即可保持 dialog 開啟 */
  onSubmit: (payload: MutationSignaturePayload) => Promise<void>
}

type Mode = 'desktop' | 'mobile'

interface BridgeState {
  sessionId: string
  mobileToken: string
  expiresAt: string
  status: 'PENDING' | 'COMPLETED' | 'CONSUMED' | 'EXPIRED'
}

const POLL_INTERVAL_MS = 2000

export function RoleSignatureDialog({
  open,
  onOpenChange,
  title,
  description,
  purpose,
  onSubmit,
}: RoleSignatureDialogProps) {
  const [mode, setMode] = useState<Mode>('desktop')
  const [password, setPassword] = useState('')
  const [signature, setSignature] = useState<SignatureData | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Bridge mode 狀態
  const [bridge, setBridge] = useState<BridgeState | null>(null)
  const [bridgeError, setBridgeError] = useState<string | null>(null)
  const [bridgeStarting, setBridgeStarting] = useState(false)
  // 改 setTimeout 遞迴模式（避免 setInterval 與 async tick 重疊執行 → 重複 consume）
  const pollTimeoutRef = useRef<number | null>(null)
  // 取消旗標：dialog 關 / mode 切換 / status 變化時翻 true，正在執行的 tick 看到後不再排下一次
  const pollCancelledRef = useRef(false)

  // 用 ref 追最新的 onSubmit / onOpenChange / purpose，避免 effect 內的閉包過時
  // （父元件 re-render 給新函式時，輪詢路徑仍呼叫舊版）
  const onSubmitRef = useRef(onSubmit)
  const onOpenChangeRef = useRef(onOpenChange)
  const purposeRef = useRef(purpose)
  useEffect(() => {
    onSubmitRef.current = onSubmit
    onOpenChangeRef.current = onOpenChange
    purposeRef.current = purpose
  })

  const stopPolling = () => {
    pollCancelledRef.current = true
    if (pollTimeoutRef.current !== null) {
      window.clearTimeout(pollTimeoutRef.current)
      pollTimeoutRef.current = null
    }
  }

  useEffect(() => {
    if (open) {
      setMode('desktop')
      setPassword('')
      setSignature(null)
      setError(null)
      setBridge(null)
      setBridgeError(null)
      setBridgeStarting(false)
    }
    return () => {
      stopPolling()
    }
  }, [open])

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()
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
      await onSubmit({
        password,
        handwriting_svg: signature.svg,
        stroke_data: signature.strokeData,
      })
      onOpenChange(false)
    } catch (err) {
      setError(getErrorMessage(err) || '簽章驗證失敗，請重試')
    } finally {
      setIsSubmitting(false)
    }
  }

  const startBridge = async () => {
    setBridgeError(null)
    setBridgeStarting(true)
    try {
      const r = await startSignatureBridge(purposeRef.current)
      setBridge({
        sessionId: r.session_id,
        mobileToken: r.mobile_token,
        expiresAt: r.expires_at,
        status: 'PENDING',
      })
    } catch (err) {
      setBridgeError(getErrorMessage(err) || '開啟手機簽名 session 失敗，請重試')
    } finally {
      setBridgeStarting(false)
    }
  }

  // 進入 mobile 模式時自動開 session；離開時清掉
  useEffect(() => {
    if (open && mode === 'mobile' && !bridge && !bridgeStarting) {
      void startBridge()
    }
    if (mode !== 'mobile') {
      stopPolling()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, mode])

  // 啟動輪詢；status COMPLETED 時 consume + 套入 onSubmit
  // 用遞迴 setTimeout：每次 tick 完才排下一次，避免 setInterval + async 重疊
  // 導致重複 consume / 重複呼叫 onSubmit
  useEffect(() => {
    if (!open || mode !== 'mobile' || !bridge || bridge.status !== 'PENDING') {
      return
    }
    const sessionId = bridge.sessionId
    pollCancelledRef.current = false

    const scheduleNext = () => {
      if (pollCancelledRef.current) return
      pollTimeoutRef.current = window.setTimeout(tick, POLL_INTERVAL_MS)
    }

    const tick = async () => {
      if (pollCancelledRef.current) return
      try {
        const r = await getSignatureBridgeStatus(sessionId)
        if (pollCancelledRef.current) return
        if (r.status === 'COMPLETED') {
          stopPolling()
          setBridge((b) => (b ? { ...b, status: 'COMPLETED' } : b))
          try {
            const c = await consumeSignatureBridge(sessionId)
            setIsSubmitting(true)
            try {
              await onSubmitRef.current(c.payload)
              onOpenChangeRef.current(false)
            } catch (err) {
              setBridgeError(getErrorMessage(err) || '簽章驗證失敗，請重試')
            } finally {
              setIsSubmitting(false)
            }
          } catch (err) {
            setBridgeError(getErrorMessage(err) || '取回簽章失敗，請重試')
          }
          return // COMPLETED 終態，不再排下一次
        } else if (r.status === 'EXPIRED' || r.status === 'CONSUMED') {
          stopPolling()
          setBridge((b) => (b ? { ...b, status: r.status as BridgeState['status'] } : b))
          setBridgeError(
            r.status === 'EXPIRED' ? 'QR 已過期，請重新產生' : 'Session 已使用',
          )
          return
        }
      } catch {
        // 暫時性錯誤；下個 tick 繼續
      }
      scheduleNext()
    }

    // 立即排第一次（保留原本「開始輪詢後 2s 才打」的節奏）
    scheduleNext()
    return () => {
      stopPolling()
    }
  }, [open, mode, bridge])

  const qrUrl = bridge
    ? `${window.location.origin}/sign/${bridge.sessionId}?token=${encodeURIComponent(
        bridge.mobileToken,
      )}&purpose=${encodeURIComponent(purpose)}`
    : null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        {/* mode 切換按鈕 */}
        <div className="flex gap-2 pt-2">
          <Button
            type="button"
            size="sm"
            variant={mode === 'desktop' ? 'default' : 'outline'}
            onClick={() => setMode('desktop')}
            disabled={isSubmitting}
          >
            <Monitor className="h-4 w-4 mr-2" />
            桌機簽
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === 'mobile' ? 'default' : 'outline'}
            onClick={() => setMode('mobile')}
            disabled={isSubmitting}
          >
            <Smartphone className="h-4 w-4 mr-2" />
            手機簽
          </Button>
        </div>

        {mode === 'desktop' ? (
          <form onSubmit={handleSubmit} className="space-y-4 py-2">
            <div className="space-y-2">
              <Label htmlFor="role-sig-password">您的登入密碼</Label>
              <Input
                id="role-sig-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="請輸入密碼以確認身份"
                disabled={isSubmitting}
                autoFocus
              />
            </div>
            <div className="space-y-2">
              <Label>手寫簽名</Label>
              <p className="text-xs text-muted-foreground">
                此簽名會記錄到稽核軌跡，日後可用以鑑定操作人身份。桌機建議切「手機簽」用觸控更順手。
              </p>
              <HandwrittenSignaturePad
                onSignatureChange={setSignature}
                height={160}
              />
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isSubmitting}
              >
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                確認簽署並送出
              </Button>
            </DialogFooter>
          </form>
        ) : (
          <div className="space-y-4 py-2">
            <p className="text-sm text-muted-foreground">
              用手機相機掃下方 QR，在手機完成密碼 + 手寫簽名後，桌機會自動接續。QR 5 分鐘有效。
            </p>
            <div className="flex flex-col items-center gap-3">
              {bridgeStarting && (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  正在產生 QR…
                </div>
              )}
              {qrUrl && (
                <>
                  <div className="rounded-md border p-3 bg-white">
                    <QRCodeSVG value={qrUrl} size={220} level="M" />
                  </div>
                  <p className="text-xs text-muted-foreground break-all max-w-full text-center">
                    若手機掃不到，可手動開啟此網址
                  </p>
                  <p className="text-xs font-mono break-all max-w-full text-center text-muted-foreground">
                    {qrUrl}
                  </p>
                  <p className="text-xs text-muted-foreground flex items-center gap-2">
                    <Loader2 className="h-3 w-3 animate-spin" />
                    等待手機完成簽名…
                  </p>
                </>
              )}
              {bridgeError && (
                <div className="space-y-2 text-center">
                  <p className="text-sm text-destructive">{bridgeError}</p>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      setBridge(null)
                      setBridgeError(null)
                      void startBridge()
                    }}
                  >
                    重新產生 QR
                  </Button>
                </div>
              )}
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isSubmitting}
              >
                取消
              </Button>
            </DialogFooter>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
