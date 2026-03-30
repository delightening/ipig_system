import React, { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Check, X, Minus, Save, Loader2 } from 'lucide-react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

interface VetReviewItem {
    item_name: string
    compliance: string // 'V' | 'X' | '-'
    comment?: string
    pi_reply?: string
}

interface VetReviewFormProps {
    protocolId: string
    initialData?: {
        items: VetReviewItem[]
        vet_signature?: string
        signed_at?: string
    }
    isEditable?: boolean
}

const DEFAULT_ITEMS = [
    '實驗動物之來源(Sources)',
    '實驗方法與程序概況',
    '動物實驗之必要性(含取代、減量、精緻化之3Rs理由)',
    '動物之品種、品系、數量、性別及體重',
    '動物麻醉藥劑、止痛藥、鎮靜藥之名稱、劑量及給藥路徑',
    '動物保定方式、手術程序、術後照顧及對動物預期造成之痛苦或緊迫情形',
    '不預期發病或傷害之處理與安樂死方式及其評估基準(Humane endpoint)',
    '具危險性實驗之防護措施',
    '實驗期限',
    '參與實驗人員',
    '活體採樣或組織採取',
    '其他'
]

const VetReviewForm: React.FC<VetReviewFormProps> = ({ protocolId, initialData, isEditable = true }) => {
    const { t } = useTranslation()
    const { toast } = useToast()
    const queryClient = useQueryClient()
    const [items, setItems] = useState<VetReviewItem[]>(() => {
        if (initialData?.items && initialData.items.length > 0) {
            return initialData.items
        }
        return DEFAULT_ITEMS.map(name => ({
            item_name: name,
            compliance: '-',
            comment: ''
        }))
    })

    const saveMutation = useMutation({
        mutationFn: (data: Record<string, unknown>) => api.post('/reviews/vet-form', data),
        onSuccess: () => {
            toast({
                title: t('common.success'),
                description: t('protocols.detail.vet_form.save_success')
            })
            queryClient.invalidateQueries({ queryKey: ['protocol', protocolId] })
        },
        onError: (error: unknown) => {
            toast({
                variant: 'destructive',
                title: t('common.error'),
                description: getApiErrorMessage(error, t('common.unknown_error'))
            })
        }
    })

    const handleUpdateItem = (index: number, field: keyof VetReviewItem, value: string) => {
        const newItems = [...items]
        newItems[index] = { ...newItems[index], [field]: value }
        setItems(newItems)
    }

    const handleSave = () => {
        saveMutation.mutate({
            protocol_id: protocolId,
            review_form: { items }
        })
    }

    return (
        <Card className="w-full shadow-lg border-2 border-border">
            <CardHeader className="bg-muted border-b">
                <div className="flex justify-between items-center">
                    <CardTitle className="text-xl font-bold text-foreground">
                        {t('protocols.detail.vet_form.title', '獸醫師審查查檢表')}
                    </CardTitle>
                    {isEditable && (
                        <Button
                            onClick={handleSave}
                            disabled={saveMutation.isPending}
                            className="bg-primary hover:bg-primary/90"
                        >
                            {saveMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Save className="mr-2 h-4 w-4" />}
                            {t('common.save', '儲存審查表')}
                        </Button>
                    )}
                </div>
            </CardHeader>
            <CardContent className="p-0">
                <div className="overflow-x-auto">
                    <table className="w-full border-collapse">
                        <thead>
                            <tr className="bg-muted text-muted-foreground font-semibold text-sm">
                                <th className="p-4 text-left border-b w-12">#</th>
                                <th className="p-4 text-left border-b w-1/3">審查項目</th>
                                <th className="p-4 text-center border-b w-32 text-nowrap">符合性 (V/X/-)</th>
                                <th className="p-4 text-left border-b">審查意見 (與原申請計畫不符處)</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-border">
                            {items.map((item, index) => (
                                <tr key={item.item_name} className="hover:bg-muted/50 transition-colors">
                                    <td className="p-4 text-muted-foreground font-medium">{index + 1}</td>
                                    <td className="p-4 font-medium text-foreground leading-tight">
                                        {item.item_name}
                                    </td>
                                    <td className="p-4">
                                        <Select
                                            disabled={!isEditable}
                                            value={item.compliance}
                                            onValueChange={(val) => handleUpdateItem(index, 'compliance', val)}
                                        >
                                            <SelectTrigger className={`w-28 mx-auto font-bold ${item.compliance === 'V' ? 'text-status-success-text border-status-success-border bg-status-success-bg' :
                                                item.compliance === 'X' ? 'text-status-error-text border-status-error-border bg-status-error-bg' :
                                                    'text-muted-foreground'
                                                }`}>
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="V" className="text-status-success-text">
                                                    <div className="flex items-center"><Check className="mr-2 h-4 w-4" /> 符合 (V)</div>
                                                </SelectItem>
                                                <SelectItem value="X" className="text-status-error-text">
                                                    <div className="flex items-center"><X className="mr-2 h-4 w-4" /> 不符 (X)</div>
                                                </SelectItem>
                                                <SelectItem value="-" className="text-muted-foreground">
                                                    <div className="flex items-center"><Minus className="mr-2 h-4 w-4" /> 不適用 (-)</div>
                                                </SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </td>
                                    <td className="p-4">
                                        <Textarea
                                            disabled={!isEditable}
                                            value={item.comment || ''}
                                            onChange={(e) => handleUpdateItem(index, 'comment', e.target.value)}
                                            placeholder={t('protocols.detail.vet_form.comment_placeholder', '請輸入審查意見...')}
                                            className="min-h-[80px] resize-y bg-card/50 focus:bg-card transition-all border-border"
                                        />
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </CardContent>
        </Card>
    )
}

export default VetReviewForm
