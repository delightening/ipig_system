import { useState, useEffect } from 'react'
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
import { Loader2 } from 'lucide-react'

/** SEC-33：敏感操作二級認證 — 請使用者重新輸入密碼以確認 */
export interface ConfirmPasswordModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  /** 回傳後由呼叫端負責呼叫 confirmPassword 並執行實際操作；失敗時可 toast 並保持 modal 開啟 */
  onSubmit: (password: string) => Promise<void>
}

export function ConfirmPasswordModal({
  open,
  onOpenChange,
  title,
  description,
  onSubmit,
}: ConfirmPasswordModalProps) {
  const [password, setPassword] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      setPassword('')
      setError(null)
    }
  }, [open])

  const handleSubmit = async () => {
    if (!password.trim()) {
      setError('請輸入密碼')
      return
    }
    setError(null)
    setIsSubmitting(true)
    try {
      await onSubmit(password)
      onOpenChange(false)
    } catch {
      setError('密碼錯誤或已過期，請重新輸入')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="reauth-password">您的登入密碼</Label>
            <Input
              id="reauth-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="請輸入密碼以確認身份"
              disabled={isSubmitting}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
              autoFocus
            />
            {error && (
              <p className="text-sm text-destructive">{error}</p>
            )}
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSubmitting}
          >
            取消
          </Button>
          <Button onClick={handleSubmit} disabled={isSubmitting}>
            {isSubmitting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
