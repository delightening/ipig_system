import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Mail, ArrowLeft, CheckCircle } from 'lucide-react'

export function ForgotPasswordPage() {
  const [email, setEmail] = useState('')
  const [submitted, setSubmitted] = useState(false)

  const forgotPasswordMutation = useMutation({
    mutationFn: async (email: string) => {
      return api.post('/auth/forgot-password', { email })
    },
    onSuccess: () => {
      setSubmitted(true)
    },
    onError: (error: any) => {
      // 即使 email 不存在也顯示成功訊息，避免帳號列舉攻擊
      setSubmitted(true)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!email) {
      toast({
        title: '錯誤',
        description: '請輸入電子郵件地址',
        variant: 'destructive',
      })
      return
    }
    forgotPasswordMutation.mutate(email)
  }

  if (submitted) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-4">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

        <Card className="w-full max-w-md relative z-10 border-slate-700/50 bg-slate-800/50 backdrop-blur-xl shadow-2xl">
          <CardContent className="pt-8 pb-8 text-center space-y-6">
            <div className="mx-auto w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center">
              <CheckCircle className="h-8 w-8 text-green-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-white">請檢查您的信箱</h2>
              <p className="text-slate-400">
                如果 <span className="text-white font-medium">{email}</span> 是已註冊的帳號，
                您將收到密碼重設連結。
              </p>
            </div>
            <div className="space-y-3 pt-4">
              <p className="text-sm text-slate-500">
                沒有收到郵件？請檢查垃圾郵件資料夾，或確認您輸入的地址正確。
              </p>
              <Button
                variant="ghost"
                className="text-slate-400 hover:text-white"
                onClick={() => {
                  setSubmitted(false)
                  setEmail('')
                }}
              >
                重新輸入
              </Button>
            </div>
            <div className="pt-4">
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PGNpcmNsZSBjeD0iMzAiIGN5PSIzMCIgcj0iMiIvPjwvZz48L2c+PC9zdmc+')] opacity-50"></div>

      <Card className="w-full max-w-md relative z-10 border-slate-700/50 bg-slate-800/50 backdrop-blur-xl shadow-2xl">
        <CardHeader className="space-y-1 pb-6">
          <div className="mx-auto w-12 h-12 bg-purple-500/20 rounded-xl flex items-center justify-center mb-4">
            <Mail className="h-6 w-6 text-purple-400" />
          </div>
          <CardTitle className="text-2xl font-bold text-center text-white">
            忘記密碼
          </CardTitle>
          <CardDescription className="text-center text-slate-400">
            請輸入您的電子郵件地址，我們將發送密碼重設連結給您
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="email" className="text-slate-300">
                電子郵件
              </Label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <Input
                  id="email"
                  type="email"
                  placeholder="you@example.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="pl-9 bg-slate-700/50 border-slate-600 text-white placeholder:text-slate-500 focus:border-purple-500 focus:ring-purple-500/20"
                  autoComplete="email"
                  autoFocus
                />
              </div>
            </div>

            <Button
              type="submit"
              className="w-full bg-purple-600 hover:bg-purple-700 text-white h-11"
              disabled={forgotPasswordMutation.isPending}
            >
              {forgotPasswordMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  發送中...
                </>
              ) : (
                '發送重設連結'
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
