import { useForm } from 'react-hook-form'
import { useToggle } from '@/hooks/useToggle'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { useAuthStore } from '@/stores/auth'
import { checkPasswordComplexity, getStrengthColor, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Lock, Eye, EyeOff, ShieldAlert } from 'lucide-react'

type ChangePasswordFormData = {
  current_password: string
  new_password: string
  confirm_password: string
}

export function ForceChangePasswordPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { user, checkAuth } = useAuthStore()

  const {
    register,
    handleSubmit,
    watch,
    setError,
    formState: { errors },
  } = useForm<ChangePasswordFormData>({
    defaultValues: {
      current_password: '',
      new_password: '',
      confirm_password: '',
    },
  })

  const [showCurrentPassword, toggleCurrentPassword] = useToggle()
  const [showNewPassword, toggleNewPassword] = useToggle()
  const [showConfirmPassword, toggleConfirmPassword] = useToggle()

  const newPassword = watch('new_password')
  const confirmPassword = watch('confirm_password')

  // 密碼強度檢查（使用共用模組）
  const passwordChecks = checkPasswordComplexity(newPassword)
  const passwordStrength = Object.values(passwordChecks).filter(Boolean).length

  const changePasswordMutation = useMutation({
    mutationFn: async (data: ChangePasswordFormData) => {
      return api.put('/me/password', {
        current_password: data.current_password,
        new_password: data.new_password,
        // C3：後端要求新密碼二次確認；frontend 已有 confirm_password 欄位
        new_password_confirmation: data.confirm_password,
      })
    },
    onSuccess: async () => {
      toast({
        title: t('auth.forceChange.successToastTitle'),
        description: t('auth.forceChange.successToastDescription'),
      })
      // 重新載入用戶資訊
      await checkAuth()
      navigate('/dashboard')
    },
    onError: (error: unknown) => {
      const message = getErrorMessage(error) || t('auth.forceChange.failedFallback')
      toast({
        title: t('common.error'),
        description: message,
        variant: 'destructive',
      })
    },
  })

  const onValid = (data: ChangePasswordFormData) => {
    if (data.current_password === data.new_password) {
      setError('new_password', { message: t('auth.validation.newPasswordSameAsCurrent') })
      return
    }
    changePasswordMutation.mutate(data)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

      <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
        <CardHeader className="space-y-1 pb-6">
          <div className="mx-auto w-14 h-14 bg-status-warning-bg rounded-xl flex items-center justify-center mb-4">
            <ShieldAlert className="h-7 w-7 text-status-warning-text" />
          </div>
          <CardTitle className="text-2xl font-bold text-center text-foreground">
            {t('auth.forceChange.title')}
          </CardTitle>
          <CardDescription className="text-center text-muted-foreground">
            {t('auth.forceChange.description')}
          </CardDescription>
          {user && (
            <p className="text-center text-sm text-muted-foreground pt-2">
              {t('auth.forceChange.loggedInAs', { email: user.email })}
            </p>
          )}
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onValid)} className="space-y-5">
            {/* 無障礙：密碼表單應包含 username 欄位，供輔助技術識別 */}
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
            <FormField label={t('password.currentPassword')} htmlFor="currentPassword" error={errors.current_password?.message}>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="currentPassword"
                  type={showCurrentPassword ? 'text' : 'password'}
                  placeholder={t('auth.fields.currentPasswordPlaceholder')}
                  {...register('current_password', { required: t('auth.validation.currentPasswordRequired') })}
                  className="pl-9 pr-10 bg-input border-border text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-ring"
                  autoComplete="current-password"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={toggleCurrentPassword}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showCurrentPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </FormField>

            <div className="space-y-2">
              <FormField label={t('password.newPassword')} htmlFor="newPassword" error={errors.new_password?.message}>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="newPassword"
                    type={showNewPassword ? 'text' : 'password'}
                    placeholder={t('auth.fields.newPasswordPlaceholder')}
                    {...register('new_password', {
                      required: t('auth.validation.newPasswordRequired'),
                      minLength: { value: PASSWORD_MIN_LENGTH, message: t('auth.passwordRules.minLengthShort', { min: PASSWORD_MIN_LENGTH }) },
                    })}
                    className="pl-9 pr-10 bg-input border-border text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-ring"
                    autoComplete="new-password"
                  />
                  <button
                    type="button"
                    onClick={toggleNewPassword}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  >
                    {showNewPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </FormField>

              {/* Password Strength Indicator */}
              {newPassword && (
                <div className="space-y-2 mt-3">
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
                  <div className="text-xs space-y-1 text-muted-foreground">
                    <p className={passwordChecks.length ? 'text-status-success-text' : ''}>
                      {passwordChecks.length ? '\u2713' : '\u25CB'} {t('auth.passwordChecks.minLength', { min: PASSWORD_MIN_LENGTH })}
                    </p>
                    <p className={passwordChecks.uppercase ? 'text-status-success-text' : ''}>
                      {passwordChecks.uppercase ? '\u2713' : '\u25CB'} {t('auth.passwordChecks.uppercase')}
                    </p>
                    <p className={passwordChecks.lowercase ? 'text-status-success-text' : ''}>
                      {passwordChecks.lowercase ? '\u2713' : '\u25CB'} {t('auth.passwordChecks.lowercase')}
                    </p>
                    <p className={passwordChecks.number ? 'text-status-success-text' : ''}>
                      {passwordChecks.number ? '\u2713' : '\u25CB'} {t('auth.passwordChecks.number')}
                    </p>
                    <p className={passwordChecks.notCommon ? 'text-status-success-text' : ''}>
                      {passwordChecks.notCommon ? '\u2713' : '\u25CB'} {t('auth.passwordChecks.notCommon')}
                    </p>
                  </div>
                </div>
              )}
            </div>

            <FormField label={t('password.confirmPassword')} htmlFor="confirmPassword" error={errors.confirm_password?.message}>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  placeholder={t('auth.fields.confirmNewPasswordPlaceholder')}
                  {...register('confirm_password', {
                    required: t('auth.validation.confirmNewPasswordRequired'),
                    validate: (value, formValues) =>
                      value === formValues.new_password || t('auth.forceChange.passwordsDoNotMatch'),
                  })}
                  className="pl-9 pr-10 bg-input border-border text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-ring"
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  onClick={toggleConfirmPassword}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              {confirmPassword && newPassword === confirmPassword && newPassword && (
                <p className="text-xs text-status-success-text">✓ {t('auth.passwordChecks.match')}</p>
              )}
            </FormField>

            <Button
              type="submit"
              className="w-full bg-primary hover:bg-primary/90 text-primary-foreground h-11 mt-2"
              disabled={changePasswordMutation.isPending || passwordStrength < 5 || newPassword !== confirmPassword}
            >
              {changePasswordMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  {t('common.processed')}
                </>
              ) : (
                t('auth.forceChange.submit')
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
