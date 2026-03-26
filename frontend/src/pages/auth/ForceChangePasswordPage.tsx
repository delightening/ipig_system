import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useToggle } from '@/hooks/useToggle'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { useAuthStore } from '@/stores/auth'
import { changePasswordSchema, ChangePasswordFormData } from '@/lib/validation'
import { checkPasswordComplexity, getStrengthColor, PASSWORD_MIN_LENGTH } from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Lock, Eye, EyeOff, ShieldAlert } from 'lucide-react'

export function ForceChangePasswordPage() {
  const navigate = useNavigate()
  const { user, checkAuth } = useAuthStore()

  const {
    register,
    handleSubmit,
    watch,
    setError,
    formState: { errors },
  } = useForm<ChangePasswordFormData>({
    resolver: zodResolver(changePasswordSchema),
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
      })
    },
    onSuccess: async () => {
      toast({
        title: '密碼變更成功',
        description: '您的密碼已成功更新',
      })
      // 重新載入用戶資訊
      await checkAuth()
      navigate('/dashboard')
    },
    onError: (error: unknown) => {
      const message = getErrorMessage(error) || '密碼變更失敗'
      toast({
        title: '錯誤',
        description: message,
        variant: 'destructive',
      })
    },
  })

  const onValid = (data: ChangePasswordFormData) => {
    if (data.current_password === data.new_password) {
      setError('new_password', { message: '新密碼不能與目前密碼相同' })
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
            需要變更密碼
          </CardTitle>
          <CardDescription className="text-center text-muted-foreground">
            為了您的帳號安全，請立即變更初始密碼
          </CardDescription>
          {user && (
            <p className="text-center text-sm text-muted-foreground pt-2">
              登入帳號：{user.email}
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
            <FormField label="目前密碼" htmlFor="currentPassword" error={errors.current_password?.message}>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="currentPassword"
                  type={showCurrentPassword ? 'text' : 'password'}
                  placeholder="請輸入目前密碼"
                  {...register('current_password')}
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
              <FormField label="新密碼" htmlFor="newPassword" error={errors.new_password?.message}>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="newPassword"
                    type={showNewPassword ? 'text' : 'password'}
                    placeholder="請輸入新密碼"
                    {...register('new_password')}
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

            <FormField label="確認新密碼" htmlFor="confirmPassword" error={errors.confirm_password?.message}>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  placeholder="再次輸入新密碼"
                  {...register('confirm_password')}
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
                <p className="text-xs text-status-success-text">✓ 密碼一致</p>
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
                  處理中...
                </>
              ) : (
                '確認變更密碼'
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
