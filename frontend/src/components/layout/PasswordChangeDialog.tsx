import { useState } from 'react'
import { useToggle } from '@/hooks/useToggle'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import api from '@/lib/api'
import type { ChangeOwnPasswordRequest } from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { getPasswordError, checkPasswordComplexity, getStrengthColor, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
import { Key, Loader2, Eye, EyeOff } from 'lucide-react'

interface PasswordChangeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PasswordChangeDialog({ open, onOpenChange }: PasswordChangeDialogProps) {
  const { t } = useTranslation()
  const { user } = useAuthStore()

  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showCurrentPassword, toggleCurrentPassword] = useToggle()
  const [showNewPassword, toggleNewPassword] = useToggle()
  const [showConfirmPassword, toggleConfirmPassword] = useToggle()

  const resetForm = () => {
    setCurrentPassword('')
    setNewPassword('')
    setConfirmPassword('')
  }

  const changePasswordMutation = useMutation({
    mutationFn: async (data: ChangeOwnPasswordRequest) => {
      return api.put('/me/password', data)
    },
    onSuccess: async () => {
      toast({ title: t('common.success'), description: t('password.success') })
      onOpenChange(false)
      resetForm()
      const { checkAuth } = useAuthStore.getState()
      await checkAuth()
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getErrorMessage(error) || t('password.failed'),
        variant: 'destructive',
      })
    },
  })

  const passwordChecks = checkPasswordComplexity(newPassword)
  const passwordStrength = Object.values(passwordChecks).filter(Boolean).length

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!currentPassword || !newPassword || !confirmPassword) {
      toast({ title: t('common.error'), description: t('password.fillAllFields'), variant: 'destructive' })
      return
    }
    const pwError = getPasswordError(newPassword)
    if (pwError) {
      toast({ title: t('common.error'), description: pwError, variant: 'destructive' })
      return
    }
    if (newPassword !== confirmPassword) {
      toast({ title: t('common.error'), description: t('password.mismatch'), variant: 'destructive' })
      return
    }
    changePasswordMutation.mutate({
      current_password: currentPassword,
      new_password: newPassword,
    })
  }

  return (
    <Dialog open={open} onOpenChange={(isOpen) => {
      onOpenChange(isOpen)
      if (!isOpen) resetForm()
    }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            {t('password.title')}
          </DialogTitle>
          <DialogDescription>
            {t('password.description')}
          </DialogDescription>
        </DialogHeader>
        <form id="password-change-form" onSubmit={handleSubmit} className="space-y-4 py-4">
          {/* 無障礙：密碼表單應包含 username 欄位 */}
          <input
            type="text"
            name="username"
            autoComplete="username"
            value={user?.email ?? ''}
            readOnly
            tabIndex={-1}
            className="absolute opacity-0 pointer-events-none h-0 w-0"
            aria-hidden
          />
          <div className="space-y-2">
            <Label htmlFor="current-password">{t('password.currentPassword')}</Label>
            <div className="relative">
              <Input
                id="current-password"
                type={showCurrentPassword ? 'text' : 'password'}
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder={t('password.currentPassword')}
                className="pr-10"
                autoComplete="current-password"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleCurrentPassword}
                aria-label={showCurrentPassword ? t('password.hidePassword') : t('password.showPassword')}
              >
                {showCurrentPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="new-password">{t('password.newPassword')}</Label>
            <div className="relative">
              <Input
                id="new-password"
                type={showNewPassword ? 'text' : 'password'}
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder={`${PASSWORD_MIN_LENGTH} 字元以上，含大小寫與數字`}
                className="pr-10"
                autoComplete="new-password"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleNewPassword}
                aria-label={showNewPassword ? t('password.hidePassword') : t('password.showPassword')}
              >
                {showNewPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
            {newPassword && (
              <div className="space-y-2 mt-2">
                <div className="flex gap-1">
                  {[1, 2, 3, 4, 5].map((level) => (
                    <div
                      key={level}
                      className={`h-1 flex-1 rounded-full transition-colors ${
                        level <= passwordStrength ? getStrengthColor(passwordStrength) : 'bg-muted'
                      }`}
                    />
                  ))}
                </div>
                <div className="text-xs space-y-0.5 text-muted-foreground">
                  <p className={passwordChecks.length ? 'text-green-600' : ''}>{passwordChecks.length ? '\u2713' : '\u25CB'} {`\u81F3\u5C11 ${PASSWORD_MIN_LENGTH} \u500B\u5B57\u5143`}</p>
                  <p className={passwordChecks.uppercase ? 'text-green-600' : ''}>{passwordChecks.uppercase ? '\u2713' : '\u25CB'} {'\u5305\u542B\u5927\u5BEB\u5B57\u6BCD'}</p>
                  <p className={passwordChecks.lowercase ? 'text-green-600' : ''}>{passwordChecks.lowercase ? '\u2713' : '\u25CB'} {'\u5305\u542B\u5C0F\u5BEB\u5B57\u6BCD'}</p>
                  <p className={passwordChecks.number ? 'text-green-600' : ''}>{passwordChecks.number ? '\u2713' : '\u25CB'} {'\u5305\u542B\u6578\u5B57'}</p>
                  <p className={passwordChecks.notCommon ? 'text-green-600' : ''}>{passwordChecks.notCommon ? '\u2713' : '\u25CB'} {'\u975E\u5E38\u898B\u5F31\u5BC6\u78BC'}</p>
                </div>
              </div>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="confirm-password">{t('password.confirmPassword')}</Label>
            <div className="relative">
              <Input
                id="confirm-password"
                type={showConfirmPassword ? 'text' : 'password'}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder={t('password.confirmPassword')}
                className="pr-10"
                autoComplete="new-password"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleConfirmPassword}
                aria-label={showConfirmPassword ? t('password.hidePassword') : t('password.showPassword')}
              >
                {showConfirmPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
          </div>
        </form>
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => {
              onOpenChange(false)
              resetForm()
            }}
            disabled={changePasswordMutation.isPending}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            form="password-change-form"
            disabled={changePasswordMutation.isPending}
          >
            {changePasswordMutation.isPending && (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            )}
            {t('password.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
