import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import type { User } from '@/lib/api'

interface UserDeleteDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  userToDelete: User | null
  showReauth: boolean
  onReauthOpenChange: (open: boolean) => void
  onConfirmDelete: () => void
  onReauthSubmit: (password: string) => Promise<void>
}

export function UserDeleteDialog({
  open,
  onOpenChange,
  userToDelete,
  showReauth,
  onReauthOpenChange,
  onConfirmDelete,
  onReauthSubmit,
}: UserDeleteDialogProps) {
  const { t } = useTranslation()

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              {t('admin.userDeleteDialog.title')}
            </DialogTitle>
            <DialogDescription>
              {t('admin.userDeleteDialog.descriptionPrefix')}{' '}
              <span className="font-medium">{userToDelete?.display_name}</span>（
              {userToDelete?.email}）
              {t('admin.userDeleteDialog.descriptionSuffix')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <div className="p-3 bg-status-error-bg border border-destructive rounded-md">
              <p className="text-sm text-status-error-text">
                {t('admin.userDeleteDialog.warning')}
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="destructive"
              onClick={onConfirmDelete}
            >
              {t('common.confirmDelete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmPasswordModal
        open={showReauth}
        onOpenChange={onReauthOpenChange}
        title={t('admin.userDeleteDialog.title')}
        description={
          userToDelete
            ? t('admin.userDeleteDialog.reauthDescription', {
                name: userToDelete.display_name,
              })
            : ''
        }
        onSubmit={onReauthSubmit}
      />
    </>
  )
}
