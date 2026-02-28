import { useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import type { TwoFactorSetupResponse } from '@/types/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import {
  Dialog, DialogContent, DialogDescription, DialogFooter,
  DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { ShieldCheck, ShieldOff, Loader2, Copy, Check } from 'lucide-react'

interface Props {
  totpEnabled: boolean
  onStatusChange: () => void
}

export function TwoFactorSetup({ totpEnabled, onStatusChange }: Props) {
  const [setupData, setSetupData] = useState<TwoFactorSetupResponse | null>(null)
  const [showSetupDialog, setShowSetupDialog] = useState(false)
  const [showDisableDialog, setShowDisableDialog] = useState(false)
  const [verifyCode, setVerifyCode] = useState('')
  const [disablePassword, setDisablePassword] = useState('')
  const [disableCode, setDisableCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [copied, setCopied] = useState(false)
  const [step, setStep] = useState<'qr' | 'backup'>('qr')

  const startSetup = async () => {
    setLoading(true)
    try {
      const res = await api.post<TwoFactorSetupResponse>('/auth/2fa/setup')
      setSetupData(res.data)
      setShowSetupDialog(true)
      setStep('qr')
      setVerifyCode('')
    } catch (error: unknown) {
      toast({
        title: '啟用失敗',
        description: getErrorMessage(error) || '無法產生 2FA 設定',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const confirmSetup = async () => {
    if (verifyCode.length < 6) return
    setLoading(true)
    try {
      await api.post('/auth/2fa/confirm', { code: verifyCode })
      toast({ title: '2FA 已啟用', description: '您的帳號已受兩步驟驗證保護' })
      setStep('backup')
    } catch (error: unknown) {
      toast({
        title: '驗證失敗',
        description: getErrorMessage(error) || '驗證碼錯誤，請重試',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const finishSetup = () => {
    setShowSetupDialog(false)
    setSetupData(null)
    onStatusChange()
  }

  const disableTwoFactor = async () => {
    if (!disablePassword || disableCode.length < 6) return
    setLoading(true)
    try {
      await api.post('/auth/2fa/disable', { password: disablePassword, code: disableCode })
      toast({ title: '2FA 已停用' })
      setShowDisableDialog(false)
      setDisablePassword('')
      setDisableCode('')
      onStatusChange()
    } catch (error: unknown) {
      toast({
        title: '停用失敗',
        description: getErrorMessage(error) || '密碼或驗證碼錯誤',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const copyBackupCodes = () => {
    if (!setupData) return
    navigator.clipboard.writeText(setupData.backup_codes.join('\n'))
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5" />
            兩步驟驗證 (2FA)
          </CardTitle>
          <CardDescription>
            使用 Google Authenticator 或其他 TOTP 驗證器應用程式增加帳號安全性
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`h-3 w-3 rounded-full ${totpEnabled ? 'bg-green-500' : 'bg-gray-300'}`} />
              <span className="text-sm font-medium">
                {totpEnabled ? '已啟用' : '未啟用'}
              </span>
            </div>
            {totpEnabled ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowDisableDialog(true)}
              >
                <ShieldOff className="mr-2 h-4 w-4" />停用 2FA
              </Button>
            ) : (
              <Button size="sm" onClick={startSetup} disabled={loading}>
                {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <ShieldCheck className="mr-2 h-4 w-4" />}
                啟用 2FA
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Setup Dialog */}
      <Dialog open={showSetupDialog} onOpenChange={(open) => { if (!open && step === 'backup') finishSetup(); else if (!open) setShowSetupDialog(false) }}>
        <DialogContent className="max-w-md">
          {step === 'qr' && setupData && (
            <>
              <DialogHeader>
                <DialogTitle>設定兩步驟驗證</DialogTitle>
                <DialogDescription>
                  使用驗證器 App 掃描下方 QR Code，然後輸入顯示的 6 位數驗證碼
                </DialogDescription>
              </DialogHeader>
              <div className="flex flex-col items-center gap-4 py-4">
                <div className="rounded-lg border bg-white p-4">
                  <QRCodeSVG value={setupData.otpauth_uri} size={200} level="M" />
                </div>
                <p className="text-xs text-muted-foreground text-center">
                  支援 Google Authenticator、Microsoft Authenticator、Authy 等
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="verify-code">驗證碼</Label>
                <Input
                  id="verify-code"
                  type="text"
                  inputMode="numeric"
                  placeholder="000000"
                  maxLength={6}
                  value={verifyCode}
                  onChange={(e) => setVerifyCode(e.target.value.replace(/\D/g, ''))}
                  onKeyDown={(e) => { if (e.key === 'Enter') confirmSetup() }}
                  className="text-center text-xl tracking-[0.5em] font-mono"
                />
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setShowSetupDialog(false)}>取消</Button>
                <Button onClick={confirmSetup} disabled={loading || verifyCode.length < 6}>
                  {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  確認啟用
                </Button>
              </DialogFooter>
            </>
          )}
          {step === 'backup' && setupData && (
            <>
              <DialogHeader>
                <DialogTitle>備用碼</DialogTitle>
                <DialogDescription>
                  請妥善保存以下備用碼。當您無法使用驗證器 App 時，可使用備用碼登入（每組僅能使用一次）。
                </DialogDescription>
              </DialogHeader>
              <div className="rounded-lg border bg-muted/50 p-4">
                <div className="grid grid-cols-2 gap-2 font-mono text-sm">
                  {setupData.backup_codes.map((code, i) => (
                    <div key={i} className="rounded bg-background px-3 py-1.5 text-center">
                      {code}
                    </div>
                  ))}
                </div>
              </div>
              <Button variant="outline" className="w-full" onClick={copyBackupCodes}>
                {copied ? <Check className="mr-2 h-4 w-4 text-green-500" /> : <Copy className="mr-2 h-4 w-4" />}
                {copied ? '已複製' : '複製備用碼'}
              </Button>
              <DialogFooter>
                <Button onClick={finishSetup}>完成</Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      {/* Disable Dialog */}
      <Dialog open={showDisableDialog} onOpenChange={setShowDisableDialog}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>停用兩步驟驗證</DialogTitle>
            <DialogDescription>
              停用後帳號將不再需要驗證碼登入。請輸入密碼和目前的驗證碼確認。
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="disable-password">密碼</Label>
              <Input
                id="disable-password"
                type="password"
                value={disablePassword}
                onChange={(e) => setDisablePassword(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="disable-code">驗證碼</Label>
              <Input
                id="disable-code"
                type="text"
                inputMode="numeric"
                placeholder="000000"
                maxLength={8}
                value={disableCode}
                onChange={(e) => setDisableCode(e.target.value.replace(/\D/g, ''))}
                className="text-center font-mono tracking-widest"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDisableDialog(false)}>取消</Button>
            <Button
              variant="destructive"
              onClick={disableTwoFactor}
              disabled={loading || !disablePassword || disableCode.length < 6}
            >
              {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              確認停用
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
