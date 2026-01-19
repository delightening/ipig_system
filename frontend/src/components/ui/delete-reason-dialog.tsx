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

interface DeleteReasonDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    title?: string
    description?: string
    onConfirm: (reason: string) => void
    isPending?: boolean
}

/**
 * GLP 合規刪除對話框
 * 刪除操作必須提供刪除原因
 */
export function DeleteReasonDialog({
    open,
    onOpenChange,
    title = '確認刪除',
    description = '此操作無法復原。請提供刪除原因以符合 GLP 規範。',
    onConfirm,
    isPending = false,
}: DeleteReasonDialogProps) {
    const [reason, setReason] = useState('')
    const [error, setError] = useState('')

    const handleConfirm = () => {
        if (!reason.trim()) {
            setError('請輸入刪除原因')
            return
        }
        if (reason.trim().length < 5) {
            setError('刪除原因至少需要 5 個字元')
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
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-red-600">
                        <AlertTriangle className="h-5 w-5" />
                        {title}
                    </DialogTitle>
                    <DialogDescription>{description}</DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="delete-reason" className="text-sm font-medium">
                            刪除原因 <span className="text-red-500">*</span>
                        </Label>
                        <Textarea
                            id="delete-reason"
                            placeholder="請說明刪除此紀錄的原因..."
                            value={reason}
                            onChange={(e) => {
                                setReason(e.target.value)
                                if (error) setError('')
                            }}
                            className={error ? 'border-red-500' : ''}
                            rows={3}
                            disabled={isPending}
                        />
                        {error && <p className="text-sm text-red-500">{error}</p>}
                    </div>

                    <div className="bg-amber-50 border border-amber-200 rounded-lg p-3">
                        <p className="text-sm text-amber-800">
                            <strong>GLP 合規提醒：</strong>
                            刪除操作將被記錄於審計日誌中，包含刪除原因及操作者資訊。
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
                        確認刪除
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
