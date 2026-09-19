import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useDialogSet } from '@/hooks/useDialogSet'
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
  const { t } = useTranslation()
  const [setupData, setSetupData] = useState<TwoFactorSetupResponse | null>(null)
  const dialogs = useDialogSet(['setup', 'disable'] as const)
  const [verifyCode, setVerifyCode] = useState('')
  const [disablePassword, setDisablePassword] = useState('')
  const [disableCode, setDisableCode] = useState('')
  const [copied, setCopied] = useState(false)
  const [step, setStep] = useState<'qr' | 'backup'>('qr')

  const startSetupMutation = useMutation({
    mutationFn: () => api.post<TwoFactorSetupResponse>('/auth/2fa/setup'),
    onSuccess: (res) => {
      setSetupData(res.data)
      dialogs.open('setup')
      setStep('qr')
      setVerifyCode('')
    },
    onError: (error: unknown) => {
      toast({
        title: t('auth.twoFactor.enableFailed'),
        description: getErrorMessage(error) || t('auth.twoFactor.setupDataFailed'),
        variant: 'destructive',
      })
    },
  })

  const confirmSetupMutation = useMutation({
    mutationFn: () => api.post('/auth/2fa/confirm', { code: verifyCode }),
    onSuccess: () => {
      toast({ title: t('auth.twoFactor.enabledToastTitle'), description: t('auth.twoFactor.enabledToastDescription') })
      setStep('backup')
    },
    onError: (error: unknown) => {
      toast({
        title: t('auth.twoFactor.verifyFailed'),
        description: getErrorMessage(error) || t('auth.twoFactor.codeInvalidRetry'),
        variant: 'destructive',
      })
    },
  })

  const confirmSetup = () => {
    if (verifyCode.length < 6) return
    confirmSetupMutation.mutate()
  }

  const finishSetup = () => {
    dialogs.close('setup')
    setSetupData(null)
    onStatusChange()
  }

  const disableMutation = useMutation({
    mutationFn: () => api.post('/auth/2fa/disable', { password: disablePassword, code: disableCode }),
    onSuccess: () => {
      toast({ title: t('auth.twoFactor.disabledToastTitle') })
      dialogs.close('disable')
      setDisablePassword('')
      setDisableCode('')
      onStatusChange()
    },
    onError: (error: unknown) => {
      toast({
        title: t('auth.twoFactor.disableFailed'),
        description: getErrorMessage(error) || t('auth.twoFactor.passwordOrCodeInvalid'),
        variant: 'destructive',
      })
    },
  })

  const disableTwoFactor = () => {
    if (!disablePassword || disableCode.length < 6) return
    disableMutation.mutate()
  }

  const loading = startSetupMutation.isPending || confirmSetupMutation.isPending || disableMutation.isPending

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
            {t('auth.twoFactor.cardTitle')}
          </CardTitle>
          <CardDescription>
            {t('auth.twoFactor.cardDescription')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`h-3 w-3 rounded-full ${totpEnabled ? 'bg-status-success-bg0' : 'bg-muted'}`} />
              <span className="text-sm font-medium">
                {totpEnabled ? t('auth.twoFactor.enabled') : t('auth.twoFactor.notEnabled')}
              </span>
            </div>
            {totpEnabled ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => dialogs.open('disable')}
              >
                <ShieldOff className="mr-2 h-4 w-4" />{t('auth.twoFactor.disableButton')}
              </Button>
            ) : (
              <Button size="sm" onClick={() => startSetupMutation.mutate()} disabled={loading}>
                {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <ShieldCheck className="mr-2 h-4 w-4" />}
                {t('auth.twoFactor.enableButton')}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Setup Dialog */}
      <Dialog open={dialogs.isOpen('setup')} onOpenChange={(open) => { if (!open && step === 'backup') finishSetup(); else if (!open) dialogs.close('setup') }}>
        <DialogContent size="sm">
          {step === 'qr' && setupData && (
            <>
              <DialogHeader>
                <DialogTitle>{t('auth.twoFactor.setupDialogTitle')}</DialogTitle>
                <DialogDescription>
                  {t('auth.twoFactor.setupDialogDescription')}
                </DialogDescription>
              </DialogHeader>
              <div className="flex flex-col items-center gap-4 py-4">
                <div className="rounded-lg border bg-white p-4">
                  <QRCodeSVG value={setupData.otpauth_uri} size={200} level="M" />
                </div>
                <p className="text-xs text-muted-foreground text-center">
                  {t('auth.twoFactor.supportedApps')}
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="verify-code">{t('auth.twoFactor.codeLabel')}</Label>
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
                <Button variant="outline" onClick={() => dialogs.close('setup')}>{t('common.cancel')}</Button>
                <Button onClick={confirmSetup} disabled={loading || verifyCode.length < 6}>
                  {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  {t('auth.twoFactor.confirmEnable')}
                </Button>
              </DialogFooter>
            </>
          )}
          {step === 'backup' && setupData && (
            <>
              <DialogHeader>
                <DialogTitle>{t('auth.twoFactor.backupCodesTitle')}</DialogTitle>
                <DialogDescription>
                  {t('auth.twoFactor.backupCodesDescription')}
                </DialogDescription>
              </DialogHeader>
              <div className="rounded-lg border bg-muted/50 p-4">
                <div className="grid grid-cols-2 gap-2 font-mono text-sm">
                  {setupData.backup_codes.map((code) => (
                    <div key={code} className="rounded bg-background px-3 py-1.5 text-center">
                      {code}
                    </div>
                  ))}
                </div>
              </div>
              <Button variant="outline" className="w-full" onClick={copyBackupCodes}>
                {copied ? <Check className="mr-2 h-4 w-4 text-status-success-solid" /> : <Copy className="mr-2 h-4 w-4" />}
                {copied ? t('auth.twoFactor.copied') : t('auth.twoFactor.copyBackupCodes')}
              </Button>
              <DialogFooter>
                <Button onClick={finishSetup}>{t('auth.twoFactor.done')}</Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      {/* Disable Dialog */}
      <Dialog open={dialogs.isOpen('disable')} onOpenChange={dialogs.setOpen('disable')}>
        <DialogContent size="sm">
          <DialogHeader>
            <DialogTitle>{t('auth.twoFactor.disableDialogTitle')}</DialogTitle>
            <DialogDescription>
              {t('auth.twoFactor.disableDialogDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="disable-password">{t('auth.fields.password')}</Label>
              <Input
                id="disable-password"
                type="password"
                value={disablePassword}
                onChange={(e) => setDisablePassword(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="disable-code">{t('auth.twoFactor.codeLabel')}</Label>
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
            <Button variant="outline" onClick={() => dialogs.close('disable')}>{t('common.cancel')}</Button>
            <Button
              variant="destructive"
              onClick={disableTwoFactor}
              disabled={loading || !disablePassword || disableCode.length < 6}
            >
              {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              {t('auth.twoFactor.confirmDisable')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
