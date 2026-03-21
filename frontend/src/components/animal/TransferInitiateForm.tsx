import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { transferApi, transferTypeNames } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { ArrowRightLeft, Loader2 } from 'lucide-react'

import { useTransferInvalidate } from './useTransferMutations'

interface TransferInitiateFormProps {
    animalId: string
    earTag: string
    onClose: () => void
}

export function TransferInitiateForm({ animalId, earTag, onClose }: TransferInitiateFormProps) {
    const queryClient = useQueryClient()
    const invalidate = useTransferInvalidate(animalId, queryClient)

    const [reason, setReason] = useState('')
    const [remark, setRemark] = useState('')
    const [transferType, setTransferType] = useState<'external' | 'internal'>('internal')

    const initiateMutation = useMutation({
        mutationFn: () => transferApi.initiate(animalId, {
            reason,
            remark: remark || undefined,
            transfer_type: transferType,
        }),
        onSuccess: () => {
            toast({ title: '成功', description: '已發起轉讓申請' })
            onClose()
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '發起失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-indigo-200">
            <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                    <ArrowRightLeft className="h-5 w-5 text-indigo-600" />
                    發起轉讓 — {earTag}
                </CardTitle>
                <CardDescription>
                    請說明轉讓原因。發起後動物狀態將變為「已轉讓」，等待獸醫評估。
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="space-y-2">
                    <Label htmlFor="transfer-reason">轉讓原因 *</Label>
                    <Textarea
                        id="transfer-reason"
                        value={reason}
                        onChange={e => setReason(e.target.value)}
                        placeholder="說明轉讓原因..."
                        className="min-h-[80px]"
                    />
                </div>
                <div className="space-y-2">
                    <Label>轉讓類型</Label>
                    <Select value={transferType} onValueChange={(v: 'external' | 'internal') => setTransferType(v)}>
                        <SelectTrigger>
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="internal">{transferTypeNames.internal}</SelectItem>
                            <SelectItem value="external">{transferTypeNames.external}</SelectItem>
                        </SelectContent>
                    </Select>
                    <p className="text-xs text-slate-500">
                        {transferType === 'external' ? '轉給其他機構：完成轉讓後將清空欄位。' : '仍在機構內：完成轉讓後保留欄位，僅移出原計劃。'}
                    </p>
                </div>
                <div className="space-y-2">
                    <Label htmlFor="transfer-remark">備註</Label>
                    <Input
                        id="transfer-remark"
                        value={remark}
                        onChange={e => setRemark(e.target.value)}
                        placeholder="可選備註"
                    />
                </div>
                <div className="flex gap-2">
                    <Button
                        onClick={() => initiateMutation.mutate()}
                        disabled={!reason.trim() || initiateMutation.isPending}
                        className="bg-indigo-600 hover:bg-indigo-700"
                    >
                        {initiateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        確認發起
                    </Button>
                    <Button variant="outline" onClick={onClose}>取消</Button>
                </div>
            </CardContent>
        </Card>
    )
}
