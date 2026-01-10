import { useState, useEffect } from 'react'
import { Link, useSearchParams, useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Lock, ArrowLeft, CheckCircle, AlertCircle, Eye, EyeOff, ShieldCheck } from 'lucide-react'

export function ResetPasswordPage() {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const token = searchParams.get('token')

  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [success, setSuccess] = useState(false)

  // 密碼強度檢查
  const passwordChecks = {
    length: newPassword.length >= 8,
    uppercase: /[A-Z]/.test(newPassword),
    lowercase: /[a-z]/.test(newPassword),
    number: /[0-9]/.test(newPassword),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(newPassword),
  }
  const passwordStrength = Object.values(passwordChecks).filter(Boolean).length

  const resetPasswordMutation = useMutation({
    mutationFn: async () => {
      return api.post('/auth/reset-password', {
        token,
        new_password: newPassword,
      })
    },
    onSuccess: () => {
      setSuccess(true)
      toast({
        title: '密碼重設成功',
        description: '您可以使用新密碼登入了',
      })
    },
    onError: (error: any) => {
      const message = error?.response?.data?.error?.message || '密碼重設失敗'
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!newPassword || !confirmPassword) {
      toast({
        title: '錯誤',
        description: '請填寫所有欄位',
        variant: 'destructive',
      })
      return
    }

    if (newPassword !== confirmPassword) {
      toast({
        title: '錯誤',
        description: '兩次輸入的密碼不一致',
        variant: 'destructive',
      })
      return
    }

    if (passwordStrength < 3) {
      toast({
        title: '密碼強度不足',
        description: '請使用更強的密碼',
        variant: 'destructive',
      })
      return
    }

    resetPasswordMutation.mutate()
  }

  // 如果沒有 token，顯示錯誤
  if (!token) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-slate-700/50 bg-slate-800/50 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <div className="mx-auto w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center">
              <AlertCircle className="h-8 w-8 text-red-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-white">無效的連結</h2>
              <p className="text-slate-400">
                此密碼重設連結無效或已過期，請重新申請密碼重設。
              </p>
            </div>
            <div className="pt-4">
              <Link to="/forgot-password">
                <Button className="bg-purple-600 hover:bg-purple-700">
                  重新申請密碼重設
                </Button>
              </Link>
            </div>
            <div className="pt-2">
              <Link to="/login" className="text-purple-400 hover:text-purple-300 text-sm">
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
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-slate-700/50 bg-slate-800/50 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <div className="mx-auto w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center">
              <CheckCircle className="h-8 w-8 text-green-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-white">密碼重設成功！</h2>
              <p className="text-slate-400">
                您的密碼已成功重設，請使用新密碼登入。
              </p>
            </div>
            <div className="pt-4">
              <Link to="/login">
                <Button className="bg-purple-600 hover:bg-purple-700 w-full">
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
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

      <Card className="w-full max-w-md relative z-10 border-slate-700/50 bg-slate-800/50 backdrop-blur-xl shadow-2xl">
        <CardHeader className="space-y-1 pb-6">
          <div className="mx-auto w-12 h-12 bg-purple-500/20 rounded-xl flex items-center justify-center mb-4">
            <ShieldCheck className="h-6 w-6 text-purple-400" />
          </div>
          <CardTitle className="text-2xl font-bold text-center text-white">
            設定新密碼
          </CardTitle>
          <CardDescription className="text-center text-slate-400">
            請輸入您的新密碼
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="newPassword" className="text-slate-300">
                新密碼
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <Input
                  id="newPassword"
                  type={showPassword ? 'text' : 'password'}
                  placeholder="請輸入新密碼"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  className="pl-9 pr-10 bg-slate-700/50 border-slate-600 text-white placeholder:text-slate-500 focus:border-purple-500 focus:ring-purple-500/20"
                  autoComplete="new-password"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>

              {/* Password Strength Indicator */}
              {newPassword && (
                <div className="space-y-2 mt-3">
                  <div className="flex gap-1">
                    {[1, 2, 3, 4, 5].map((level) => (
                      <div
                        key={level}
                        className={`h-1 flex-1 rounded-full transition-colors ${
                          level <= passwordStrength
                            ? passwordStrength <= 2
                              ? 'bg-red-500'
                              : passwordStrength <= 3
                              ? 'bg-yellow-500'
                              : 'bg-green-500'
                            : 'bg-slate-600'
                        }`}
                      />
                    ))}
                  </div>
                  <div className="text-xs space-y-1 text-slate-400">
                    <p className={passwordChecks.length ? 'text-green-400' : ''}>
                      {passwordChecks.length ? '✓' : '○'} 至少 8 個字元
                    </p>
                    <p className={passwordChecks.uppercase ? 'text-green-400' : ''}>
                      {passwordChecks.uppercase ? '✓' : '○'} 包含大寫字母
                    </p>
                    <p className={passwordChecks.lowercase ? 'text-green-400' : ''}>
                      {passwordChecks.lowercase ? '✓' : '○'} 包含小寫字母
                    </p>
                    <p className={passwordChecks.number ? 'text-green-400' : ''}>
                      {passwordChecks.number ? '✓' : '○'} 包含數字
                    </p>
                    <p className={passwordChecks.special ? 'text-green-400' : ''}>
                      {passwordChecks.special ? '✓' : '○'} 包含特殊字元
                    </p>
                  </div>
                </div>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="confirmPassword" className="text-slate-300">
                確認新密碼
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  placeholder="再次輸入新密碼"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="pl-9 pr-10 bg-slate-700/50 border-slate-600 text-white placeholder:text-slate-500 focus:border-purple-500 focus:ring-purple-500/20"
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                >
                  {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              {confirmPassword && newPassword !== confirmPassword && (
                <p className="text-xs text-red-400">密碼不一致</p>
              )}
              {confirmPassword && newPassword === confirmPassword && (
                <p className="text-xs text-green-400">✓ 密碼一致</p>
              )}
            </div>

            <Button
              type="submit"
              className="w-full bg-purple-600 hover:bg-purple-700 text-white h-11"
              disabled={resetPasswordMutation.isPending || passwordStrength < 3 || newPassword !== confirmPassword}
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
              <Link to="/login" className="text-purple-400 hover:text-purple-300 text-sm">
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
