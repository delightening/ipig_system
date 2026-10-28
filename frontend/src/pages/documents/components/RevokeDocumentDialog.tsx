import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertTriangle, Loader2, Undo2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import { MIN_REVOKE_REASON_CHARS } from '../documentActionState'

interface RevokeDocumentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  docNo: string
  isPending: boolean
  /** 傳出的是 trim 後的原因。 */
  onConfirm: (reason: string) => void
}

/**
 * 撤銷已核准、未入庫採購單的對話框：原因必填，空白或過短不可送出。
 *
 * 原因輸入保存在本元件內，關閉時清空——下一次開啟不會帶著上一張單的原因。
 */
export function RevokeDocumentDialog({
  open,
  onOpenChange,
  docNo,
  isPending,
  onConfirm,
}: RevokeDocumentDialogProps) {
  const { t } = useTranslation()
  const [reason, setReason] = useState('')

  useEffect(() => {
    if (!open) setReason('')
  }, [open])

  const trimmed = reason.trim()
  const canSubmit = trimmed.length >= MIN_REVOKE_REASON_CHARS && !isPending

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            {t('erpDocs.documents.detail.revokeDialog.title')}
          </DialogTitle>
          <DialogDescription>
            {t('erpDocs.documents.detail.revokeDialog.description', { docNo })}
          </DialogDescription>
        </DialogHeader>
        <Textarea
          aria-label={t('erpDocs.documents.detail.revokeDialog.reasonLabel')}
          placeholder={t('erpDocs.documents.detail.revokeDialog.placeholder', {
            min: MIN_REVOKE_REASON_CHARS,
          })}
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          rows={3}
        />
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button variant="destructive" onClick={() => onConfirm(trimmed)} disabled={!canSubmit}>
            {isPending ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Undo2 className="mr-2 h-4 w-4" />
            )}
            {t('erpDocs.documents.detail.revokeDialog.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
