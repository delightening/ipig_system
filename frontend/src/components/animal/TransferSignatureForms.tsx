import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { transferApi, signatureApi } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
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
            toast({ title: t('common.success'), description: t('animalActions.transfer.approve.approved') })
            setShowSignature(false)
            setSignatureData(null)
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.approve.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-success-border">
            <CardContent className="pt-4 space-y-3">
                {!showSignature ? (
                    <Button
                        onClick={() => setShowSignature(true)}
                        disabled={mutation.isPending}
                        className="bg-status-success-solid hover:bg-status-success-solid/90"
                    >
                        <PenLine className="h-4 w-4 mr-2" />
                        {t('animalActions.transfer.approve.button')}
                    </Button>
                ) : (
                    <div className="space-y-3">
                        <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                            <PenLine className="h-4 w-4" />
                            {t('animalActions.transfer.approve.signatureConfirm', { signature: t('signature.handwriting') })}
                        </div>
                        <HandwrittenSignaturePad onSignatureChange={setSignatureData} height={140} />
                        <div className="flex gap-2 justify-end">
                            <Button size="sm" variant="outline" onClick={() => { setShowSignature(false); setSignatureData(null) }}>
                                {t('common.cancel')}
                            </Button>
                            <Button
                                size="sm"
                                className="bg-status-success-solid hover:bg-status-success-solid/90"
                                onClick={() => signatureData && mutation.mutate(signatureData)}
                                disabled={!signatureData || mutation.isPending}
                            >
                                {mutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <UserCheck className="h-4 w-4 mr-1" />}
                                {t('signature.confirmSign')}
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
            toast({ title: t('common.success'), description: t('animalActions.transfer.complete.completed') })
            setShowSignature(false)
            setSignatureData(null)
            invalidate()
        },
        onError: (e: unknown) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('animalActions.transfer.complete.failed')), variant: 'destructive' }),
    })

    return (
        <Card className="border-status-info-border">
            <CardContent className="pt-4 space-y-3">
                {!showSignature ? (
                    <Button
                        onClick={() => setShowSignature(true)}
                        disabled={mutation.isPending}
                        className="bg-primary hover:bg-primary/90"
                    >
                        <PenLine className="h-4 w-4 mr-2" />
                        {t('animalActions.transfer.complete.button')}
                    </Button>
                ) : (
                    <div className="space-y-3">
                        <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                            <PenLine className="h-4 w-4" />
                            {t('animalActions.transfer.complete.signatureConfirm', { signature: t('signature.handwriting') })}
                        </div>
                        <HandwrittenSignaturePad onSignatureChange={setSignatureData} height={140} />
                        <div className="flex gap-2 justify-end">
                            <Button size="sm" variant="outline" onClick={() => { setShowSignature(false); setSignatureData(null) }}>
                                {t('common.cancel')}
                            </Button>
                            <Button
                                size="sm"
                                className="bg-primary hover:bg-primary/90"
                                onClick={() => signatureData && mutation.mutate(signatureData)}
                                disabled={!signatureData || mutation.isPending}
                            >
                                {mutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <CheckCircle2 className="h-4 w-4 mr-1" />}
                                {t('signature.confirmSign')}
                            </Button>
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
