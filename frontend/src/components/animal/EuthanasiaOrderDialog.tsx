import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { Loader2, AlertOctagon, Clock } from 'lucide-react'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    animalId: string
    earTag: string
    iacucNo?: string | null
}

export function EuthanasiaOrderDialog({ open, onOpenChange, animalId, earTag, iacucNo }: Props) {
    const queryClient = useQueryClient()

    const [reason, setReason] = useState('')
    const [confirmed, setConfirmed] = useState(false)

    // Reset when dialog opens
    useEffect(() => {
        if (open) {
            setReason('')
            setConfirmed(false)
        }
    }, [open])

    const mutation = useMutation({
        mutationFn: async () => {
            const payload = {
                animal_id: animalId,
                reason: reason,
            }
            return api.post('/euthanasia/orders', payload)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
            queryClient.invalidateQueries({ queryKey: ['euthanasia-orders'] })
            toast({
                title: '安樂死單已開立',
                description: '系統已通知計畫主持人，請等待回應或 24 小時後自動解鎖。',
            })
            onOpenChange(false)
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '開立失敗',
                variant: 'destructive',
            })
        },
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!reason.trim()) {
            toast({ title: '錯誤', description: '請填寫安樂死原因', variant: 'destructive' })
            return
        }
        if (!confirmed) {
            toast({ title: '錯誤', description: '請確認已閱讀注意事項', variant: 'destructive' })
            return
        }
        mutation.mutate()
    }

    // Calculate deadline (24 hours from now)
    const deadline = new Date()
    deadline.setHours(deadline.getHours() + 24)
    const deadlineStr = deadline.toLocaleString('zh-TW', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    })

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-red-600">
                        <AlertOctagon className="h-5 w-5" />
                        開立安樂死單
                    </DialogTitle>
                    <DialogDescription>
                        <span className="font-medium">耳號：{earTag}</span>
                        {iacucNo && <span className="ml-4">IACUC NO.: {iacucNo}</span>}
                    </DialogDescription>
                </DialogHeader>

                <form onSubmit={handleSubmit} className="space-y-4">
                    {/* Warning Box */}
                    <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                        <h4 className="font-medium text-red-800 mb-2 flex items-center gap-2">
                            <Clock className="h-4 w-4" />
                            注意事項
                        </h4>
                        <ul className="text-sm text-red-700 space-y-1 list-disc pl-5">
                            <li>開立後將通知計畫主持人（PI）</li>
                            <li>PI 需在 <strong>24 小時內</strong> 回應「同意」或「申請暫緩」</li>
                            <li>若 PI 未於時限內回應，系統將自動解鎖執行權限</li>
                            <li>
                                <strong>執行期限：{deadlineStr}</strong>
                            </li>
                        </ul>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="reason">安樂死原因 *</Label>
                        <Textarea
                            id="reason"
                            value={reason}
                            onChange={(e) => setReason(e.target.value)}
                            placeholder="詳細說明執行安樂死的原因，包括動物狀況、臨床症狀等..."
                            className="min-h-[120px]"
                            required
                        />
                    </div>

                    <div className="flex items-start gap-2">
                        <input
                            type="checkbox"
                            id="confirm"
                            checked={confirmed}
                            onChange={(e) => setConfirmed(e.target.checked)}
                            className="h-4 w-4 mt-1 text-red-600 rounded"
                        />
                        <Label htmlFor="confirm" className="text-sm text-gray-600">
                            我已閱讀並理解上述注意事項，確認開立安樂死單據
                        </Label>
                    </div>

                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                            取消
                        </Button>
                        <Button
                            type="submit"
                            className="bg-red-600 hover:bg-red-700"
                            disabled={mutation.isPending || !reason.trim() || !confirmed}
                        >
                            {mutation.isPending ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                                <AlertOctagon className="h-4 w-4 mr-2" />
                            )}
                            確認開立
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
