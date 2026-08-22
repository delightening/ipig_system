import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
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
import { FormField } from '@/components/ui/form-field'
import { Loader2, Key, Eye, EyeOff } from 'lucide-react'
import { useToggle } from '@/hooks/useToggle'
import type { User } from '@/lib/api'

export interface AdminResetPasswordFormData {
  reauth_password: string
  new_password: string
  confirm_password: string
}

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
  const { t } = useTranslation()
  const [showReauthPassword, toggleReauthPassword] = useToggle()
  const [showNewPassword, toggleNewPassword] = useToggle()
  const [showConfirmPassword, toggleConfirmPassword] = useToggle()

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<AdminResetPasswordFormData>({
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
            {t('admin.userResetPasswordDialog.title')}
          </DialogTitle>
          <DialogDescription>
            {t('admin.userResetPasswordDialog.descriptionPrefix')}{' '}
            <span className="font-medium">{userToResetPassword?.display_name}</span>（
            {userToResetPassword?.email}）{t('admin.userResetPasswordDialog.descriptionSuffix')}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
          <div className="p-3 bg-status-warning-bg border border-status-warning-border rounded-md">
            <p className="text-sm text-status-warning-text">
              {t('admin.userResetPasswordDialog.warning')}
            </p>
          </div>
          <FormField label={t('admin.userResetPasswordDialog.reauthLabel')} error={errors.reauth_password?.message} htmlFor="reset-reauth-password">
            <div className="relative">
              <Input
                id="reset-reauth-password"
                type={showReauthPassword ? 'text' : 'password'}
                {...register('reauth_password', { required: t('admin.userResetPasswordDialog.reauthRequired') })}
                placeholder={t('admin.userResetPasswordDialog.reauthPlaceholder')}
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleReauthPassword}
                aria-label={showReauthPassword ? t('admin.userResetPasswordDialog.hidePassword') : t('admin.userResetPasswordDialog.showPassword')}
              >
                {showReauthPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
          </FormField>
          <FormField label={t('admin.userResetPasswordDialog.newPasswordLabel')} error={errors.new_password?.message} htmlFor="reset-new-password">
            <div className="relative">
              <Input
                id="reset-new-password"
                type={showNewPassword ? 'text' : 'password'}
                {...register('new_password', {
                  required: t('admin.userResetPasswordDialog.passwordMinLength'),
                  minLength: { value: 10, message: t('admin.userResetPasswordDialog.passwordMinLength') },
                })}
                placeholder={t('admin.userResetPasswordDialog.newPasswordPlaceholder')}
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleNewPassword}
                aria-label={showNewPassword ? t('admin.userResetPasswordDialog.hidePassword') : t('admin.userResetPasswordDialog.showPassword')}
              >
                {showNewPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
          </FormField>
          <FormField label={t('admin.userResetPasswordDialog.confirmPasswordLabel')} error={errors.confirm_password?.message} htmlFor="reset-confirm-password">
            <div className="relative">
              <Input
                id="reset-confirm-password"
                type={showConfirmPassword ? 'text' : 'password'}
                {...register('confirm_password', {
                  required: t('admin.userResetPasswordDialog.confirmPasswordRequired'),
                  validate: (value, formValues) => value === formValues.new_password || t('admin.userResetPasswordDialog.passwordMismatch'),
                })}
                placeholder={t('admin.userResetPasswordDialog.confirmPasswordPlaceholder')}
                className="pr-10"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                onClick={toggleConfirmPassword}
                aria-label={showConfirmPassword ? t('admin.userResetPasswordDialog.hidePassword') : t('admin.userResetPasswordDialog.showPassword')}
              >
                {showConfirmPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
              </Button>
            </div>
          </FormField>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose} disabled={isPending}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('admin.userResetPasswordDialog.confirmReset')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
