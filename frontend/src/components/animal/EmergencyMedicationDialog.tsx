import { useState, useEffect, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
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
import { Loader2, AlertTriangle } from 'lucide-react'
import { DrugCombobox } from '@/components/animal/DrugCombobox'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    animalId: string
    earTag: string
}

export function EmergencyMedicationDialog({ open, onOpenChange, animalId, earTag }: Props) {
    const queryClient = useQueryClient()

    // Form state
    const [formData, setFormData] = useState({
        event_date: new Date().toISOString().split('T')[0],
        emergency_reason: '',
        drug: '',
        dosage: '',
        content: '',
        drug_option_id: undefined as string | undefined,
        dosage_unit: '',
    })

    // Countdown state for confirmation
    const [countdown, setCountdown] = useState(3)
    const [isConfirming, setIsConfirming] = useState(false)

    // Reset when dialog opens
    useEffect(() => {
        if (open) {
            setFormData({
                event_date: new Date().toISOString().split('T')[0],
                emergency_reason: '',
                drug: '',
                dosage: '',
                content: '',
                drug_option_id: undefined,
                dosage_unit: '',
            })
            setCountdown(3)
            setIsConfirming(false)
        }
    }, [open])

    // Countdown logic
    useEffect(() => {
        if (isConfirming && countdown > 0) {
            const timer = setTimeout(() => setCountdown(countdown - 1), 1000)
            return () => clearTimeout(timer)
        }
    }, [isConfirming, countdown])

    const mutation = useMutation({
        mutationFn: async () => {
            const payload = {
                event_date: formData.event_date,
                record_type: 'abnormal',
                content: formData.content,
                is_emergency_medication: true,
                emergency_reason: formData.emergency_reason,
                treatments: [
                    {
                        drug: formData.drug,
                        dosage: formData.dosage,
                    },
                ],
            }
            return api.post(`/animals/${animalId}/observations`, payload)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
            toast({
                title: '緊急處置已記錄',
                description: '系統已通知獸醫師和計畫主持人，請等待追認。',
            })
            onOpenChange(false)
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '儲存失敗',
                variant: 'destructive',
            })
        },
    })

    const handleConfirmClick = useCallback(() => {
        if (!isConfirming) {
            // Start countdown
            setIsConfirming(true)
            setCountdown(3)
        } else if (countdown === 0) {
            // Submit
            if (!formData.emergency_reason.trim() || !formData.drug.trim() || !formData.content.trim()) {
                toast({ title: '錯誤', description: '請填寫必要欄位', variant: 'destructive' })
                return
            }
            mutation.mutate()
        }
    }, [isConfirming, countdown, formData, mutation])

    const handleCancel = () => {
        if (isConfirming) {
            setIsConfirming(false)
            setCountdown(3)
        } else {
            onOpenChange(false)
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-red-600">
                        <AlertTriangle className="h-5 w-5" />
                        緊急處置
                    </DialogTitle>
                    <DialogDescription>
                        耳號：{earTag}
                        <br />
                        <span className="text-red-500">
                            此功能用於獸醫不在場時的緊急處置，將通知獸醫師和 PI 進行追認。
                        </span>
                    </DialogDescription>
                </DialogHeader>

                {!isConfirming ? (
                    <form className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="event_date">處置日期 *</Label>
                            <Input
                                id="event_date"
                                type="date"
                                value={formData.event_date}
                                onChange={(e) => setFormData({ ...formData, event_date: e.target.value })}
                                required
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="emergency_reason">緊急處置原因 *</Label>
                            <Textarea
                                id="emergency_reason"
                                value={formData.emergency_reason}
                                onChange={(e) => setFormData({ ...formData, emergency_reason: e.target.value })}
                                placeholder="說明為何需要緊急處置（例如：動物出現急性症狀、獸醫暫時離場...）"
                                className="min-h-[80px]"
                                required
                            />
                        </div>

                        <div className="space-y-2">
                            <Label>使用藥品 *</Label>
                            <DrugCombobox
                                value={{
                                    drug_option_id: formData.drug_option_id,
                                    drug_name: formData.drug,
                                    dosage_value: formData.dosage,
                                    dosage_unit: formData.dosage_unit,
                                }}
                                onChange={(sel) => setFormData({
                                    ...formData,
                                    drug: sel.drug_name,
                                    dosage: sel.dosage_value,
                                    drug_option_id: sel.drug_option_id,
                                    dosage_unit: sel.dosage_unit,
                                })}
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="content">處置描述 *</Label>
                            <Textarea
                                id="content"
                                value={formData.content}
                                onChange={(e) => setFormData({ ...formData, content: e.target.value })}
                                placeholder="詳細描述執行的緊急處置內容..."
                                className="min-h-[100px]"
                                required
                            />
                        </div>

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={handleCancel}>
                                取消
                            </Button>
                            <Button
                                type="button"
                                onClick={handleConfirmClick}
                                className="bg-red-600 hover:bg-red-700"
                                disabled={!formData.emergency_reason.trim() || !formData.drug.trim() || !formData.content.trim()}
                            >
                                <AlertTriangle className="h-4 w-4 mr-2" />
                                確認緊急處置
                            </Button>
                        </DialogFooter>
                    </form>
                ) : (
                    <div className="space-y-6 py-4">
                        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-center">
                            <AlertTriangle className="h-12 w-12 text-red-500 mx-auto mb-3" />
                            <h3 className="text-lg font-semibold text-red-700 mb-2">
                                確認執行緊急處置？
                            </h3>
                            <p className="text-sm text-red-600 mb-4">
                                此操作將記錄緊急給藥並通知獸醫師追認
                            </p>

                            <div className="bg-red-100 rounded-full w-20 h-20 mx-auto flex items-center justify-center mb-4">
                                <span className="text-3xl font-bold text-red-700">
                                    {countdown > 0 ? countdown : '✓'}
                                </span>
                            </div>

                            <p className="text-xs text-red-500">
                                {countdown > 0 ? `請等待 ${countdown} 秒後確認...` : '請點擊下方按鈕確認'}
                            </p>
                        </div>

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={handleCancel}>
                                取消
                            </Button>
                            <Button
                                type="button"
                                onClick={handleConfirmClick}
                                className="bg-red-600 hover:bg-red-700"
                                disabled={countdown > 0 || mutation.isPending}
                            >
                                {mutation.isPending ? (
                                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                ) : (
                                    <AlertTriangle className="h-4 w-4 mr-2" />
                                )}
                                {countdown > 0 ? `等待 ${countdown} 秒` : '確認執行'}
                            </Button>
                        </DialogFooter>
                    </div>
                )}
            </DialogContent>
        </Dialog>
    )
}
