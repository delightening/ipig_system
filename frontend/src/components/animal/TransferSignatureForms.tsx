import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { transferApi, signatureApi } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { CheckCircle2, UserCheck, Loader2, PenLine } from 'lucide-react'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'

// --- Approve Form (PI signature) ---

export function ApproveForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const { t } = useTranslation()
    const [showSignature, setShowSignature] = useState(false)
    const [signatureData, setSignatureData] = useState<SignatureData | null>(null)

    const mutation = useMutation({
        mutationFn: async (sigData: SignatureData) => {
            await signatureApi.signTransfer(transferId, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
                signature_type: 'APPROVE',
            })
            return transferApi.approve(transferId)
        },
        onSuccess: () => {
            toast({ title: '成功', description: 'PI 已同意轉讓，簽章已記錄' })
            setShowSignature(false)
            setSignatureData(null)
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '同意失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-green-200">
            <CardContent className="pt-4 space-y-3">
                {!showSignature ? (
                    <Button
                        onClick={() => setShowSignature(true)}
                        disabled={mutation.isPending}
                        className="bg-green-600 hover:bg-green-700"
                    >
                        <PenLine className="h-4 w-4 mr-2" />
                        PI 同意轉讓（需簽名）
                    </Button>
                ) : (
                    <div className="space-y-3">
                        <div className="flex items-center gap-2 text-sm font-medium text-gray-700">
                            <PenLine className="h-4 w-4" />
                            {t('signature.handwriting', '手寫簽名')} — 確認同意轉讓
                        </div>
                        <HandwrittenSignaturePad onSignatureChange={setSignatureData} height={140} />
                        <div className="flex gap-2 justify-end">
                            <Button size="sm" variant="outline" onClick={() => { setShowSignature(false); setSignatureData(null) }}>
                                取消
                            </Button>
                            <Button
                                size="sm"
                                className="bg-green-600 hover:bg-green-700"
                                onClick={() => signatureData && mutation.mutate(signatureData)}
                                disabled={!signatureData || mutation.isPending}
                            >
                                {mutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <UserCheck className="h-4 w-4 mr-1" />}
                                {t('signature.confirmSign', '確認簽署')}
                            </Button>
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}

// --- Complete Form (Admin signature) ---

export function CompleteForm({ transferId, invalidate }: { transferId: string; invalidate: () => void }) {
    const { t } = useTranslation()
    const [showSignature, setShowSignature] = useState(false)
    const [signatureData, setSignatureData] = useState<SignatureData | null>(null)

    const mutation = useMutation({
        mutationFn: async (sigData: SignatureData) => {
            await signatureApi.signTransfer(transferId, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
                signature_type: 'CONFIRM',
            })
            return transferApi.complete(transferId)
        },
        onSuccess: () => {
            toast({ title: '成功', description: '轉讓已完成，動物已分配到新計劃' })
            setShowSignature(false)
            setSignatureData(null)
            invalidate()
        },
        onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '完成失敗'), variant: 'destructive' }),
    })

    return (
        <Card className="border-indigo-200">
            <CardContent className="pt-4 space-y-3">
                {!showSignature ? (
                    <Button
                        onClick={() => setShowSignature(true)}
                        disabled={mutation.isPending}
                        className="bg-indigo-600 hover:bg-indigo-700"
                    >
                        <PenLine className="h-4 w-4 mr-2" />
                        完成轉讓（需簽名）
                    </Button>
                ) : (
                    <div className="space-y-3">
                        <div className="flex items-center gap-2 text-sm font-medium text-gray-700">
                            <PenLine className="h-4 w-4" />
                            {t('signature.handwriting', '手寫簽名')} — 確認完成轉讓
                        </div>
                        <HandwrittenSignaturePad onSignatureChange={setSignatureData} height={140} />
                        <div className="flex gap-2 justify-end">
                            <Button size="sm" variant="outline" onClick={() => { setShowSignature(false); setSignatureData(null) }}>
                                取消
                            </Button>
                            <Button
                                size="sm"
                                className="bg-indigo-600 hover:bg-indigo-700"
                                onClick={() => signatureData && mutation.mutate(signatureData)}
                                disabled={!signatureData || mutation.isPending}
                            >
                                {mutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <CheckCircle2 className="h-4 w-4 mr-1" />}
                                {t('signature.confirmSign', '確認簽署')}
                            </Button>
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
