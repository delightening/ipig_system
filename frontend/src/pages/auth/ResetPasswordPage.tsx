import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useToggle } from '@/hooks/useToggle'
import { Link, useSearchParams } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { checkPasswordComplexity, getStrengthColor, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Lock, ArrowLeft, CheckCircle, AlertCircle, Eye, EyeOff, ShieldCheck } from 'lucide-react'

type ResetPasswordFormData = { password: string; confirmPassword: string }

export function ResetPasswordPage() {
  const { t } = useTranslation()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<ResetPasswordFormData>({
    defaultValues: {
      password: '',
      confirmPassword: '',
    },
  })

  const [showPassword, togglePassword] = useToggle()
  const [showConfirmPassword, toggleConfirmPassword] = useToggle()
  const [success, setSuccess] = useState(false)

  const newPassword = watch('password')
  const confirmPassword = watch('confirmPassword')

  // 密碼強度檢查（使用共用模組）
  const passwordChecks = checkPasswordComplexity(newPassword)
  const passwordStrength = Object.values(passwordChecks).filter(Boolean).length

  const resetPasswordMutation = useMutation({
    mutationFn: async (data: ResetPasswordFormData) => {
      return api.post('/auth/reset-password', {
        token,
        new_password: data.password,
      })
    },
    onSuccess: () => {
      setSuccess(true)
      toast({
        title: t('auth.resetPassword.successToastTitle'),
        description: t('auth.resetPassword.successToastDescription'),
      })
    },
    onError: (error: unknown) => {
      const message = getErrorMessage(error) || t('auth.resetPassword.failedFallback')
      if (message.includes('expired') || message.includes('invalid')) {
        toast({
          title: t('auth.resetPassword.linkExpiredTitle'),
          description: t('auth.resetPassword.linkExpiredDescription'),
          variant: 'destructive',
        })
      } else {
        toast({
          title: t('common.error'),
          description: message,
          variant: 'destructive',
        })
      }
    },
  })

  const onValid = (data: ResetPasswordFormData) => {
    resetPasswordMutation.mutate(data)
  }

  // 如果沒有 token，顯示錯誤
  if (!token) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <AlertCircle className="h-12 w-12 text-destructive mx-auto" strokeWidth={1.5} />
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-foreground">{t('auth.resetPassword.invalidLinkTitle')}</h2>
              <p className="text-muted-foreground">
                {t('auth.resetPassword.invalidLinkDescription')}
              </p>
            </div>
            <div className="pt-4">
              <Link to="/forgot-password">
                <Button className="bg-primary hover:bg-primary/90">
                  {t('auth.resetPassword.requestAgain')}
                </Button>
              </Link>
            </div>
            <div className="pt-2">
              <Link to="/login" className="text-primary hover:text-primary/80 text-sm">
                <ArrowLeft className="h-4 w-4 inline mr-1" />
                {t('auth.backToLoginPage')}
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  // 成功頁面
  if (success) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <CheckCircle className="h-12 w-12 text-status-success-text mx-auto" strokeWidth={1.5} />
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-foreground">{t('auth.resetPassword.successTitle')}</h2>
              <p className="text-muted-foreground">
                {t('auth.resetPassword.successDescription')}
              </p>
            </div>
            <div className="pt-4">
              <Link to="/login">
                <Button className="bg-primary hover:bg-primary/90 w-full">
                  {t('auth.resetPassword.goToLogin')}
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

      <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
        <CardHeader className="space-y-1 pb-6">
          <div className="mx-auto w-12 h-12 bg-primary/20 rounded-xl flex items-center justify-center mb-4">
            <ShieldCheck className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl font-bold text-center text-foreground">
            {t('auth.resetPassword.title')}
          </CardTitle>
          <CardDescription className="text-center text-muted-foreground">
            {t('auth.resetPassword.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onValid)} className="space-y-6">
            <div className="space-y-2">
              <FormField label={t('password.newPassword')} htmlFor="newPassword" error={errors.password?.message}>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="newPassword"
                    type={showPassword ? 'text' : 'password'}
                    placeholder={t('auth.fields.newPasswordPlaceholder')}
                    {...register('password', {
                      required: t('auth.validation.newPasswordRequired'),
                      minLength: { value: PASSWORD_MIN_LENGTH, message: t('auth.passwordRules.minLengthShort', { min: PASSWORD_MIN_LENGTH }) },
                    })}
                    className="pl-9 pr-10 bg-input border-border text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-ring"
                    autoComplete="new-password"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={togglePassword}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  >
                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
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

            <FormField label={t('password.confirmPassword')} htmlFor="confirmPassword" error={errors.confirmPassword?.message}>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  placeholder={t('auth.fields.confirmNewPasswordPlaceholder')}
                  {...register('confirmPassword', {
                    required: t('auth.validation.confirmNewPasswordRequired'),
                    validate: (value, formValues) =>
                      value === formValues.password || t('auth.resetPassword.passwordsDoNotMatch'),
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
              className="w-full bg-primary hover:bg-primary/90 text-primary-foreground h-11"
              disabled={resetPasswordMutation.isPending || passwordStrength < 5 || newPassword !== confirmPassword}
            >
              {resetPasswordMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  {t('common.processed')}
                </>
              ) : (
                t('auth.resetPassword.submit')
              )}
            </Button>

            <div className="text-center pt-2">
              <Link to="/login" className="text-primary hover:text-primary/80 text-sm">
                <ArrowLeft className="h-4 w-4 inline mr-1" />
                {t('auth.backToLoginPage')}
              </Link>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
