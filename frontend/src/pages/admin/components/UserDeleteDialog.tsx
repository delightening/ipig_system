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
  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              確認刪除使用者
            </DialogTitle>
            <DialogDescription>
              此操作無法復原。確定要刪除使用者{' '}
              <span className="font-medium">{userToDelete?.display_name}</span>（
              {userToDelete?.email}）嗎？
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <div className="p-3 bg-status-error-bg border border-destructive rounded-md">
              <p className="text-sm text-status-error-text">
                刪除後，該使用者將無法再登入系統。如果只是暫時停用，建議使用「停用」功能。
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={onConfirmDelete}
            >
              確認刪除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmPasswordModal
        open={showReauth}
        onOpenChange={onReauthOpenChange}
        title="確認刪除使用者"
        description={
          userToDelete
            ? `確定要刪除使用者「${userToDelete.display_name}」？此操作無法復原，請輸入您的登入密碼以確認。`
            : ''
        }
        onSubmit={onReauthSubmit}
      />
    </>
  )
}
