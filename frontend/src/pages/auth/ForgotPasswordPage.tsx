import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { useMutation } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Loader2, Mail, ArrowLeft, CheckCircle } from 'lucide-react'

type ForgotPasswordFormData = { email: string }
const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

export function ForgotPasswordPage() {
  const { t } = useTranslation()
  const [submitted, setSubmitted] = useState(false)
  const { register, handleSubmit, watch, reset, formState: { errors } } = useForm<ForgotPasswordFormData>({
    defaultValues: { email: '' },
  })
  const email = watch('email')

  const forgotPasswordMutation = useMutation({
    mutationFn: async (data: ForgotPasswordFormData) => {
      return api.post('/auth/forgot-password', { email: data.email })
    },
    onSuccess: () => {
      setSubmitted(true)
    },
    onError: (_error: unknown) => {
      // 即使 email 不存在也顯示成功訊息，避免帳號列舉攻擊
      setSubmitted(true)
    },
  })

  const onValid = (data: ForgotPasswordFormData) => {
    forgotPasswordMutation.mutate(data)
  }

  if (submitted) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <CheckCircle className="h-12 w-12 text-status-success-text mx-auto" strokeWidth={1.5} />
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-foreground">{t('auth.forgotPassword.checkInboxTitle')}</h2>
              <p className="text-muted-foreground">
                <Trans
                  i18nKey="auth.forgotPassword.sentNotice"
                  values={{ email }}
                  components={{ emphasis: <span className="text-foreground font-medium" /> }}
                />
              </p>
            </div>
            <div className="space-y-3 pt-4">
              <p className="text-sm text-muted-foreground">
                {t('auth.forgotPassword.noEmailHint')}
              </p>
              <Button
                variant="ghost"
                className="text-muted-foreground hover:text-foreground"
                onClick={() => {
                  setSubmitted(false)
                  reset()
                }}
              >
                {t('auth.forgotPassword.retry')}
              </Button>
            </div>
            <div className="pt-4">
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

      <Card className="w-full max-w-md relative z-10 border-border bg-card/80 backdrop-blur-xl shadow-2xl">
        <CardHeader className="space-y-1 pb-6">
          <div className="mx-auto mb-4">
            <Mail className="h-10 w-10 text-primary" strokeWidth={1.5} />
          </div>
          <CardTitle className="text-2xl font-bold text-center text-foreground">
            {t('auth.forgotPassword.title')}
          </CardTitle>
          <CardDescription className="text-center text-muted-foreground">
            {t('auth.forgotPassword.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onValid)} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="email" className="text-muted-foreground">
                {t('auth.fields.email')}
              </Label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="email"
                  type="email"
                  placeholder="you@example.com"
                  {...register('email', {
                    required: t('auth.validation.emailRequired'),
                    pattern: { value: EMAIL_PATTERN, message: t('auth.validation.emailInvalid') },
                  })}
                  className="pl-9 bg-input border-border text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-ring"
                  autoComplete="email"
                  autoFocus
                />
              </div>
              {errors.email && <p className="text-sm text-destructive">{errors.email.message}</p>}
            </div>

            <Button
              type="submit"
              className="w-full bg-primary hover:bg-primary/90 text-primary-foreground h-11"
              disabled={forgotPasswordMutation.isPending}
            >
              {forgotPasswordMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  {t('auth.forgotPassword.sending')}
                </>
              ) : (
                t('auth.forgotPassword.submit')
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
