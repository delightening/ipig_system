import { useState, useRef, useEffect } from 'react'
import { useToggle } from '@/hooks/useToggle'
import { useNavigate, Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/types/error'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Eye, EyeOff, ShieldCheck, ArrowLeft } from 'lucide-react'

const loginSchema = z.object({
  email: z.string().email('請輸入有效的電子郵件'),
  password: z.string().min(1, '請輸入密碼'),
})

type LoginForm = z.infer<typeof loginSchema>

export function LoginPage() {
  const navigate = useNavigate()
  const { login, verify2FA, isLoading } = useAuthStore()
  const [showPassword, togglePassword] = useToggle()
  const [twoFAState, setTwoFAState] = useState<{ tempToken: string } | null>(null)
  const [totpCode, setTotpCode] = useState('')
  const totpInputRef = useRef<HTMLInputElement>(null)

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginForm>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  })

  useEffect(() => {
    if (twoFAState) totpInputRef.current?.focus()
  }, [twoFAState])

  const onSubmit = async (data: LoginForm) => {
    try {
      await login(data.email, data.password)
      toast({ title: '登入成功', description: '歡迎回來！' })
      navigate('/dashboard')
    } catch (error: unknown) {
      const err = error as { is2FA?: boolean; tempToken?: string }
      if (err?.is2FA) {
        setTwoFAState({ tempToken: err.tempToken! })
        return
      }
      toast({
        title: '登入失敗',
        description: getErrorMessage(error) || '請檢查您的帳號密碼',
        variant: 'destructive',
      })
    }
  }

  const onVerify2FA = async () => {
    if (!twoFAState || totpCode.length < 6) return
    try {
      await verify2FA(twoFAState.tempToken, totpCode)
      toast({ title: '登入成功', description: '歡迎回來！' })
      navigate('/dashboard')
    } catch (error: unknown) {
      toast({
        title: '驗證失敗',
        description: getErrorMessage(error) || '驗證碼錯誤或已過期',
        variant: 'destructive',
      })
      setTotpCode('')
    }
  }

  if (twoFAState) {
    return (
      <Card className="w-full max-w-md animate-fade-in">
        <CardHeader className="space-y-1 text-center">
          <div className="mx-auto mb-2 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <ShieldCheck className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-xl font-bold">兩步驟驗證</CardTitle>
          <CardDescription>請輸入驗證器 App 上的 6 位數驗證碼</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="totp">驗證碼</Label>
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
              也可使用備用碼（8 碼）登入
            </p>
          </div>
          <Button
            className="w-full"
            disabled={isLoading || totpCode.length < 6}
            onClick={onVerify2FA}
          >
            {isLoading ? (
              <><Loader2 className="mr-2 h-4 w-4 animate-spin" />驗證中...</>
            ) : '驗證'}
          </Button>
          <Button
            variant="ghost"
            className="w-full"
            onClick={() => { setTwoFAState(null); setTotpCode('') }}
          >
            <ArrowLeft className="mr-2 h-4 w-4" />返回登入
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
        <CardTitle className="text-2xl font-bold">iPig 統一入口門戶</CardTitle>
        <CardDescription>請輸入您的帳號密碼登入系統</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">電子郵件</Label>
            <Input
              id="email"
              type="email"
              autoComplete="email"
              placeholder="請輸入電子郵件"
              {...register('email')}
              className={errors.email ? 'border-red-500' : ''}
            />
            {errors.email && (
              <p className="text-sm text-red-500">{errors.email.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">密碼</Label>
            <div className="relative">
              <Input
                id="password"
                type={showPassword ? 'text' : 'password'}
                autoComplete="current-password"
                placeholder="••••••••"
                {...register('password')}
                className={errors.password ? 'border-red-500' : ''}
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={togglePassword}
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <Eye className="h-4 w-4 text-muted-foreground" />
                )}
              </Button>
            </div>
            {errors.password && (
              <p className="text-sm text-red-500">{errors.password.message}</p>
            )}
          </div>
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                登入中...
              </>
            ) : (
              '登入'
            )}
          </Button>
        </form>

        <div className="mt-4 text-center">
          <Link
            to="/forgot-password"
            className="text-sm text-blue-600 hover:text-blue-700 hover:underline"
          >
            忘記密碼？
          </Link>
        </div>

        <div className="text-center text-xs text-slate-400 mt-4">
          <Link to="/privacy" className="hover:text-slate-600 hover:underline">隱私權政策</Link>
          {' | '}
          <Link to="/terms" className="hover:text-slate-600 hover:underline">服務條款</Link>
        </div>
      </CardContent>
    </Card>
  )
}
