import { useState } from 'react'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { AlertTriangle, Loader2 } from 'lucide-react'

/** 對話框文案。整組收在一個物件裡，讓 props 維持在 6 個以內（CLAUDE.md 門檻）。 */
export interface ReasonDialogCopy {
    title?: string
    description?: string
    /** 理由欄位標籤、placeholder 與確認鈕的主詞，例如「作廢」。預設「刪除」 */
    actionNoun?: string
}

interface DeleteReasonDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    copy?: ReasonDialogCopy
    onConfirm: (reason: string) => void
    isPending?: boolean
}

/**
 * GLP 合規「填理由才能執行」對話框。
 * 原為刪除專用；R86-2 加班作廢沿用同一元件（`copy.actionNoun` 換字）。
 */
export function DeleteReasonDialog({
    open,
    onOpenChange,
    copy,
    onConfirm,
    isPending = false,
}: DeleteReasonDialogProps) {
    const actionNoun = copy?.actionNoun ?? '刪除'
    const title = copy?.title ?? `確認${actionNoun}`
    const description =
        copy?.description ?? `此操作無法復原。請提供${actionNoun}原因以符合 GLP 規範。`
    const confirmLabel = `確認${actionNoun}`
    const [reason, setReason] = useState('')
    const [error, setError] = useState('')

    const handleConfirm = () => {
        if (!reason.trim()) {
            setError(`請輸入${actionNoun}原因`)
            return
        }
        if (reason.trim().length < 5) {
            setError(`${actionNoun}原因至少需要 5 個字元`)
            return
        }
        setError('')
        onConfirm(reason.trim())
    }

    const handleClose = (newOpen: boolean) => {
        if (!newOpen) {
            setReason('')
            setError('')
        }
        onOpenChange(newOpen)
    }

    return (
        <Dialog open={open} onOpenChange={handleClose}>
            <DialogContent size="sm">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-error-text">
                        <AlertTriangle className="h-5 w-5" />
                        {title}
                    </DialogTitle>
                    <DialogDescription>{description}</DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="delete-reason" className="text-sm font-medium">
                            {actionNoun}原因 <span className="text-status-error-solid">*</span>
                        </Label>
                        <Textarea
                            id="delete-reason"
                            placeholder={`請說明${actionNoun}此紀錄的原因...`}
                            value={reason}
                            onChange={(e) => {
                                setReason(e.target.value)
                                if (error) setError('')
                            }}
                            className={error ? 'border-status-error-solid' : ''}
                            rows={3}
                            disabled={isPending}
                        />
                        {error && <p className="text-sm text-status-error-solid">{error}</p>}
                    </div>

                    <div className="bg-status-warning-bg border border-status-warning-border rounded-lg p-3">
                        <p className="text-sm text-status-warning-text">
                            <strong>GLP 合規提醒：</strong>
                            {actionNoun}操作將被記錄於操作日誌中，包含{actionNoun}原因及操作者資訊。
                        </p>
                    </div>
                </div>

                <DialogFooter className="gap-2">
                    <Button variant="outline" onClick={() => handleClose(false)} disabled={isPending}>
                        取消
                    </Button>
                    <Button
                        variant="destructive"
                        onClick={handleConfirm}
                        disabled={isPending || !reason.trim()}
                    >
                        {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {confirmLabel}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
