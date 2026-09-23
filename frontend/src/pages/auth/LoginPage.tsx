import { useState, useRef, useEffect } from 'react'
import { useToggle } from '@/hooks/useToggle'
import { useNavigate, Link, useSearchParams } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { getGoogleLoginStatus, googleLoginStartUrl } from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/types/error'
import { GoogleGIcon } from './components/GoogleGIcon'
import { googleErrorMessageKey, isNeutralGoogleError, type GoogleErrorCode } from './googleLogin'
import { useGoogleLoginRedirect } from './hooks/useGoogleLoginRedirect'

const GUEST_EMAIL = 'guest@guest.com'
const GUEST_PASSWORD = 'guest'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FormField } from '@/components/ui/form-field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Eye, EyeOff, ShieldCheck, ArrowLeft } from 'lucide-react'

// R57-2: 移除 Zod，改 React Hook Form 原生 validation rules（避免 Zod 4 的
// Function() probe 撞 CSP 'unsafe-eval' 限制）。RHF built-in `required` /
// `pattern` 直接用 native string error messages，UX 等價。
type LoginForm = {
  email: string
  password: string
}
const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

function isTwoFAError(err: unknown): err is { is2FA: true; tempToken: string } {
  return (
    typeof err === 'object' &&
    err !== null &&
    'is2FA' in err &&
    (err as Record<string, unknown>).is2FA === true &&
    'tempToken' in err &&
    typeof (err as Record<string, unknown>).tempToken === 'string'
  )
}

export function LoginPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const { login, verify2FA, checkAuth, isLoading, enterGuestMode } = useAuthStore()
  const [showPassword, togglePassword] = useToggle()
  const [twoFAState, setTwoFAState] = useState<{ tempToken: string } | null>(null)
  const [totpCode, setTotpCode] = useState('')
  const [googleFinishing, setGoogleFinishing] = useState(false)
  const totpInputRef = useRef<HTMLInputElement>(null)

  // R121：查不到（未設定或請求失敗）一律不顯示 Google 按鈕——登入頁仍可用帳密登入，
  // 按鈕消失就是這個查詢失敗時唯一需要的降級行為。
  const { data: googleStatus } = useQuery({
    queryKey: queryKeys.users.googleLoginStatus,
    queryFn: getGoogleLoginStatus,
    staleTime: 5 * 60 * 1000,
  })

  const showGoogleError = (code: GoogleErrorCode) => {
    toast({
      title: t('auth.login.googleFailedTitle'),
      description: t(googleErrorMessageKey(code)),
      ...(isNeutralGoogleError(code) ? {} : { variant: 'destructive' as const }),
    })
  }

  // 後端已設好 cookie；以 /me 讀回使用者，與重新整理頁面時的還原路徑相同
  const finishGoogleLogin = async () => {
    setGoogleFinishing(true)
    await checkAuth()
    setGoogleFinishing(false)
    if (!useAuthStore.getState().isAuthenticated) {
      showGoogleError('session_missing')
      return
    }
    useAuthStore.setState({ isImpersonating: false })
    toast({ title: t('auth.login.successTitle'), description: t('auth.login.successDescription') })
    navigate('/dashboard')
  }

  useGoogleLoginRedirect(async (result) => {
    switch (result.kind) {
      case 'twoFactor':
        setTwoFAState({ tempToken: result.tempToken })
        return
      case 'error':
        showGoogleError(result.code)
        return
      case 'success':
        await finishGoogleLogin()
        return
    }
  })

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginForm>({
    defaultValues: {
      email: '',
      password: '',
    },
  })

  useEffect(() => {
    if (twoFAState) totpInputRef.current?.focus()
  }, [twoFAState])

  // 2026-05-18: 從 ?reason=session_expired 顯示中性灰色 toast（非紅色警告）。
  // 由 api/client.ts 401 interceptor 在 refresh 失敗時導向此 URL。
  useEffect(() => {
    if (searchParams.get('reason') === 'session_expired') {
      toast({
        title: t('auth.login.sessionExpiredTitle'),
        description: t('auth.login.sessionExpiredDescription'),
        // 不指定 variant → 預設 default (中性灰色)，避免被誤認為錯誤警告
      })
      // 清掉 URL param，避免重整再次彈出
      searchParams.delete('reason')
      setSearchParams(searchParams, { replace: true })
    }
  }, [searchParams, setSearchParams, t])

  const onSubmit = async (data: LoginForm) => {
    // 訪客試用模式：純前端，不呼叫後端
    if (data.email.trim().toLowerCase() === GUEST_EMAIL && data.password === GUEST_PASSWORD) {
      enterGuestMode()
      navigate('/demo')
      return
    }

    try {
      await login(data.email, data.password)
      toast({ title: t('auth.login.successTitle'), description: t('auth.login.successDescription') })
      navigate('/dashboard')
    } catch (error: unknown) {
      if (isTwoFAError(error)) {
        setTwoFAState({ tempToken: error.tempToken })
        return
      }
      toast({
        title: t('auth.login.failedTitle'),
        description: getErrorMessage(error) || t('auth.login.failedFallback'),
        variant: 'destructive',
      })
    }
  }

  const onVerify2FA = async () => {
    if (!twoFAState || totpCode.length < 6) return
    try {
      await verify2FA(twoFAState.tempToken, totpCode)
      toast({ title: t('auth.login.successTitle'), description: t('auth.login.successDescription') })
      navigate('/dashboard')
    } catch (error: unknown) {
      toast({
        title: t('auth.twoFactor.verifyFailed'),
        description: getErrorMessage(error) || t('auth.twoFactor.codeInvalidOrExpired'),
        variant: 'destructive',
      })
      setTotpCode('')
    }
  }

  if (googleFinishing) {
    return (
      <Card className="w-full max-w-md animate-fade-in">
        <CardContent className="flex items-center justify-center gap-2 py-10 text-muted-foreground">
          <Loader2 className="h-5 w-5 animate-spin" />
          <span role="status">{t('auth.login.googleCheckingSession')}</span>
        </CardContent>
      </Card>
    )
  }

  if (twoFAState) {
    return (
      <Card className="w-full max-w-md animate-fade-in">
        <CardHeader className="space-y-1 text-center">
          <div className="mx-auto mb-2 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <ShieldCheck className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-xl font-bold">{t('auth.twoFactor.challengeTitle')}</CardTitle>
          <CardDescription>{t('auth.twoFactor.challengeDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="totp">{t('auth.twoFactor.codeLabel')}</Label>
            <Input
              ref={totpInputRef}
              id="totp"
              type="text"
              inputMode="numeric"
              autoComplete="one-time-code"
              placeholder="000000"
              maxLength={8}
              value={totpCode}
              onChange={(e) => setTotpCode(e.target.value.replace(/\D/g, ''))}
              onKeyDown={(e) => { if (e.key === 'Enter') onVerify2FA() }}
              className="text-center text-2xl tracking-[0.5em] font-mono"
            />
            <p className="text-xs text-muted-foreground text-center">
              {t('auth.twoFactor.backupHint')}
            </p>
          </div>
          <Button
            className="w-full"
            disabled={isLoading || totpCode.length < 6}
            onClick={onVerify2FA}
          >
            {isLoading ? (
              <><Loader2 className="mr-2 h-4 w-4 animate-spin" />{t('auth.twoFactor.verifying')}</>
            ) : t('auth.twoFactor.verify')}
          </Button>
          <Button
            variant="ghost"
            className="w-full"
            onClick={() => { setTwoFAState(null); setTotpCode('') }}
          >
            <ArrowLeft className="mr-2 h-4 w-4" />{t('auth.twoFactor.backToLogin')}
          </Button>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full max-w-md animate-fade-in">
      <CardHeader className="space-y-1 text-center">
        <div className="mx-auto mb-4 flex items-center justify-center">
          <img src="/pigmodel-logo.png" alt="Logo" className="h-20 w-auto" />
        </div>
        <CardTitle className="text-2xl font-bold">{t('auth.login.portalTitle')}</CardTitle>
        <CardDescription>{t('auth.login.portalDescription')}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <FormField label={t('auth.fields.email')} htmlFor="email" error={errors.email?.message}>
            <Input
              id="email"
              type="email"
              autoComplete="email"
              placeholder={t('auth.login.emailPlaceholder')}
              {...register('email', {
                required: t('auth.validation.emailRequired'),
                pattern: { value: EMAIL_PATTERN, message: t('auth.validation.emailInvalid') },
              })}
              className={errors.email ? 'border-destructive' : ''}
            />
          </FormField>
          <FormField label={t('auth.fields.password')} htmlFor="password" error={errors.password?.message}>
            <div className="relative">
              <Input
                id="password"
                type={showPassword ? 'text' : 'password'}
                autoComplete="current-password"
                placeholder="••••••••"
                {...register('password', { required: t('auth.validation.passwordRequired') })}
                className={errors.password ? 'border-destructive' : ''}
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={togglePassword}
                aria-label={showPassword ? t('password.hidePassword') : t('password.showPassword')}
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <Eye className="h-4 w-4 text-muted-foreground" />
                )}
              </Button>
            </div>
          </FormField>
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                {t('auth.login.loggingIn')}
              </>
            ) : (
              t('auth.login.submit')
            )}
          </Button>
        </form>

        {googleStatus?.enabled && (
          <>
            <div className="relative my-4">
              <div className="absolute inset-0 flex items-center" aria-hidden="true">
                <span className="w-full border-t border-border" />
              </div>
              <div className="relative flex justify-center text-xs">
                <span className="bg-card px-2 text-muted-foreground">{t('auth.login.orDivider')}</span>
              </div>
            </div>
            {/* 頂層導覽（不是 fetch）：後端設 flow cookie 後 302 到 Google */}
            <Button asChild variant="outline" className="w-full">
              <a href={googleLoginStartUrl()}>
                <GoogleGIcon className="mr-2 h-4 w-4" />
                {t('auth.login.googleButton')}
              </a>
            </Button>
          </>
        )}

        <div className="mt-4 text-center">
          <Link
            to="/forgot-password"
            className="text-sm text-primary hover:text-primary/80 hover:underline"
          >
            {t('auth.login.forgotPassword')}
          </Link>
        </div>

        <div className="text-center text-xs text-muted-foreground mt-4">
          <Link to="/privacy" className="hover:text-foreground hover:underline">{t('auth.login.privacyPolicy')}</Link>
          {' | '}
          <Link to="/terms" className="hover:text-foreground hover:underline">{t('auth.login.termsOfService')}</Link>
        </div>
      </CardContent>
    </Card>
  )
}
