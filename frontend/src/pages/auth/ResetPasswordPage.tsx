import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useToggle } from '@/hooks/useToggle'
import { Link, useSearchParams } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { resetPasswordSchema, ResetPasswordFormData } from '@/lib/validation'
import { checkPasswordComplexity, getStrengthColor, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Lock, ArrowLeft, CheckCircle, AlertCircle, Eye, EyeOff, ShieldCheck } from 'lucide-react'

export function ResetPasswordPage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<ResetPasswordFormData>({
    resolver: zodResolver(resetPasswordSchema),
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
        title: '密碼重設成功',
        description: '您可以使用新密碼登入了',
      })
    },
    onError: (error: unknown) => {
      const message = getErrorMessage(error) || '密碼重設失敗'
      if (message.includes('expired') || message.includes('invalid')) {
        toast({
          title: '連結已失效',
          description: '此密碼重設連結已失效或過期，請重新申請',
          variant: 'destructive',
        })
      } else {
        toast({
          title: '錯誤',
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
              <h2 className="text-xl font-semibold text-foreground">無效的連結</h2>
              <p className="text-muted-foreground">
                此密碼重設連結無效或已過期，請重新申請密碼重設。
              </p>
            </div>
            <div className="pt-4">
              <Link to="/forgot-password">
                <Button className="bg-primary hover:bg-primary/90">
                  重新申請密碼重設
                </Button>
              </Link>
            </div>
            <div className="pt-2">
              <Link to="/login" className="text-primary hover:text-primary/80 text-sm">
                <ArrowLeft className="h-4 w-4 inline mr-1" />
                返回登入頁面
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
              <h2 className="text-xl font-semibold text-foreground">密碼重設成功！</h2>
              <p className="text-muted-foreground">
                您的密碼已成功重設，請使用新密碼登入。
              </p>
            </div>
            <div className="pt-4">
              <Link to="/login">
                <Button className="bg-primary hover:bg-primary/90 w-full">
                  前往登入
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
            設定新密碼
          </CardTitle>
          <CardDescription className="text-center text-muted-foreground">
            請輸入您的新密碼
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onValid)} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="newPassword" className="text-muted-foreground">
                新密碼
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="newPassword"
                  type={showPassword ? 'text' : 'password'}
                  placeholder="請輸入新密碼"
                  {...register('password')}
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
              {errors.password && <p className="text-sm text-destructive">{errors.password.message}</p>}

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
                      {passwordChecks.length ? '\u2713' : '\u25CB'} {`\u81F3\u5C11 ${PASSWORD_MIN_LENGTH} \u500B\u5B57\u5143`}
                    </p>
                    <p className={passwordChecks.uppercase ? 'text-status-success-text' : ''}>
                      {passwordChecks.uppercase ? '\u2713' : '\u25CB'} {'\u5305\u542B\u5927\u5BEB\u5B57\u6BCD'}
                    </p>
                    <p className={passwordChecks.lowercase ? 'text-status-success-text' : ''}>
                      {passwordChecks.lowercase ? '\u2713' : '\u25CB'} {'\u5305\u542B\u5C0F\u5BEB\u5B57\u6BCD'}
                    </p>
                    <p className={passwordChecks.number ? 'text-status-success-text' : ''}>
                      {passwordChecks.number ? '\u2713' : '\u25CB'} {'\u5305\u542B\u6578\u5B57'}
                    </p>
                    <p className={passwordChecks.notCommon ? 'text-status-success-text' : ''}>
                      {passwordChecks.notCommon ? '\u2713' : '\u25CB'} {'\u975E\u5E38\u898B\u5F31\u5BC6\u78BC'}
                    </p>
                  </div>
                </div>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="confirmPassword" className="text-muted-foreground">
                確認新密碼
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  placeholder="再次輸入新密碼"
                  {...register('confirmPassword')}
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
              {errors.confirmPassword && <p className="text-sm text-destructive">{errors.confirmPassword.message}</p>}
              {confirmPassword && newPassword === confirmPassword && newPassword && (
                <p className="text-xs text-status-success-text">✓ 密碼一致</p>
              )}
            </div>

            <Button
              type="submit"
              className="w-full bg-primary hover:bg-primary/90 text-primary-foreground h-11"
              disabled={resetPasswordMutation.isPending || passwordStrength < 5 || newPassword !== confirmPassword}
            >
              {resetPasswordMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  處理中...
                </>
              ) : (
                '確認重設密碼'
              )}
            </Button>

            <div className="text-center pt-2">
              <Link to="/login" className="text-primary hover:text-primary/80 text-sm">
                <ArrowLeft className="h-4 w-4 inline mr-1" />
                返回登入頁面
              </Link>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
