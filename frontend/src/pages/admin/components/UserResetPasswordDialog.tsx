import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2, Key, Eye, EyeOff } from 'lucide-react'
import { useToggle } from '@/hooks/useToggle'
import {
  adminResetPasswordSchema,
  type AdminResetPasswordFormData,
} from '@/lib/validation'
import type { User } from '@/lib/api'

interface UserResetPasswordDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  userToResetPassword: User | null
  isPending: boolean
  onSubmit: (data: AdminResetPasswordFormData) => void
  onClose: () => void
}

export function UserResetPasswordDialog({
  open,
  onOpenChange,
  userToResetPassword,
  isPending,
  onSubmit,
  onClose,
}: UserResetPasswordDialogProps) {
  const [showReauthPassword, toggleReauthPassword] = useToggle()
  const [showNewPassword, toggleNewPassword] = useToggle()
  const [showConfirmPassword, toggleConfirmPassword] = useToggle()

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<AdminResetPasswordFormData>({
    resolver: zodResolver(adminResetPasswordSchema),
    defaultValues: {
      reauth_password: '',
      new_password: '',
      confirm_password: '',
    },
  })

  useEffect(() => {
    if (open) {
      reset({ reauth_password: '', new_password: '', confirm_password: '' })
    }
  }, [open, reset])

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        onOpenChange(o)
        if (!o) onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Key className="h-5 w-5 text-status-warning-text" />
            重設使用者密碼
          </DialogTitle>
          <DialogDescription>
            為使用者{' '}
            <span className="font-medium">{userToResetPassword?.display_name}</span>（
            {userToResetPassword?.email}）設定新密碼。
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
          <div className="p-3 bg-status-warning-bg border border-status-warning-border rounded-md">
            <p className="text-sm text-status-warning-text">
              重設密碼後，該使用者需要使用新密碼重新登入。建議通知該使用者密碼已變更。
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-reauth-password">您的登入密碼（確認身份）</Label>
            <div className="relative">
              <Input
                id="reset-reauth-password"
                type={showReauthPassword ? 'text' : 'password'}
                {...register('reauth_password')}
                placeholder="請輸入您的密碼以確認此操作"
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleReauthPassword}
                aria-label={showReauthPassword ? '隱藏密碼' : '顯示密碼'}
              >
                {showReauthPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
            {errors.reauth_password && (
              <p className="text-sm text-destructive">{errors.reauth_password.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-new-password">新密碼</Label>
            <div className="relative">
              <Input
                id="reset-new-password"
                type={showNewPassword ? 'text' : 'password'}
                {...register('new_password')}
                placeholder="至少 10 個字元"
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleNewPassword}
                aria-label={showNewPassword ? '隱藏密碼' : '顯示密碼'}
              >
                {showNewPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
            {errors.new_password && (
              <p className="text-sm text-destructive">{errors.new_password.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-confirm-password">確認新密碼</Label>
            <div className="relative">
              <Input
                id="reset-confirm-password"
                type={showConfirmPassword ? 'text' : 'password'}
                {...register('confirm_password')}
                placeholder="再次輸入新密碼"
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleConfirmPassword}
                aria-label={showConfirmPassword ? '隱藏密碼' : '顯示密碼'}
              >
                {showConfirmPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
            {errors.confirm_password && (
              <p className="text-sm text-destructive">{errors.confirm_password.message}</p>
            )}
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose} disabled={isPending}>
              取消
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認重設
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
