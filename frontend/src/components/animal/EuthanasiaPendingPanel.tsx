import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { signatureApi } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2, AlertOctagon, CheckCircle2, Hand, Clock, PenLine } from 'lucide-react'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

interface EuthanasiaOrder {
    id: string
    animal_id: string
    vet_user_id: string
    pi_user_id: string
    reason: string
    status: string
    deadline_at: string
    created_at: string
    animal_ear_tag?: string
    animal_iacuc_no?: string
    vet_name?: string
    pi_name?: string
}

function formatCountdown(deadline: string, t: TFunction): string {
    const now = new Date()
    const deadlineDate = new Date(deadline)
    const diff = deadlineDate.getTime() - now.getTime()

    if (diff <= 0) {
        return t('animalActions.euthanasia.countdown.expired')
    }

    const hours = Math.floor(diff / (1000 * 60 * 60))
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))

    if (hours > 0) {
        return t('animalActions.euthanasia.countdown.hoursMinutes', { hours, minutes })
    }
    return t('animalActions.euthanasia.countdown.minutes', { minutes })
}

export function EuthanasiaPendingPanel() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [selectedOrder, setSelectedOrder] = useState<EuthanasiaOrder | null>(null)
    const [showAppealDialog, setShowAppealDialog] = useState(false)
    const [appealReason, setAppealReason] = useState('')
    // 簽名相關狀態
    const [signingOrderId, setSigningOrderId] = useState<string | null>(null)
    const [signatureData, setSignatureData] = useState<SignatureData | null>(null)

    const { data: orders, isLoading } = useQuery<EuthanasiaOrder[]>({
        queryKey: ['euthanasia-pending'],
        queryFn: async () => {
            const res = await api.get('/euthanasia/orders/pending')
            return res.data
        },
    })

    // 同意 + 簽章 mutation
    const approveMutation = useMutation({
        mutationFn: async ({ orderId, sigData }: { orderId: string; sigData: SignatureData }) => {
            // 先建立簽章記錄
            await signatureApi.signEuthanasia(orderId, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
                signature_type: 'APPROVE',
            })
            // 再執行同意操作
            return api.post(`/euthanasia/orders/${orderId}/approve`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['euthanasia-pending'] })
            toast({
                title: t('animalActions.euthanasia.pending.approvedTitle'),
                description: t('animalActions.euthanasia.pending.approvedDescription'),
            })
            setSigningOrderId(null)
            setSignatureData(null)
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('animalActions.common.operationFailed')),
                variant: 'destructive',
            })
        },
    })

    const appealMutation = useMutation({
        mutationFn: async ({ orderId, reason }: { orderId: string; reason: string }) => {
            return api.post(`/euthanasia/orders/${orderId}/appeal`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['euthanasia-pending'] })
            toast({
                title: t('animalActions.euthanasia.pending.appealSubmittedTitle'),
                description: t('animalActions.euthanasia.pending.appealSubmittedDescription'),
            })
            setShowAppealDialog(false)
            setAppealReason('')
            setSelectedOrder(null)
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('animalActions.common.operationFailed')),
                variant: 'destructive',
            })
        },
    })

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
        )
    }

    if (!orders || orders.length === 0) {
        return null // No pending orders
    }

    return (
        <>
            {/* Pending Orders Alert Card */}
            <Card className="border-status-error-border bg-status-error-bg mb-6">
                <CardHeader className="pb-3">
                    <CardTitle className="text-status-error-text flex items-center gap-2">
                        <AlertOctagon className="h-5 w-5" />
                        {t('animalActions.euthanasia.pending.title')}
                    </CardTitle>
                    <CardDescription className="text-status-error-text">
                        {t('animalActions.euthanasia.pending.description', { count: orders.length })}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        {orders.map((order) => (
                            <div
                                key={order.id}
                                className="bg-white rounded-lg p-4 border border-status-error-border"
                            >
                                <div className="flex items-center justify-between mb-3">
                                    <div className="space-y-1">
                                        <div className="flex items-center gap-3">
                                            <span className="font-bold text-lg text-status-warning-text">
                                                #{order.animal_ear_tag}
                                            </span>
                                            {order.animal_iacuc_no && (
                                                <Badge variant="outline">{order.animal_iacuc_no}</Badge>
                                            )}
                                        </div>
                                        <p className="text-sm text-muted-foreground">
                                            {t('animalActions.euthanasia.pending.orderingVet', { name: order.vet_name })}
                                        </p>
                                        <p className="text-sm text-foreground line-clamp-2">
                                            {t('animalActions.euthanasia.pending.reason', { reason: order.reason })}
                                        </p>
                                        <div className="flex items-center gap-2 text-sm text-status-error-text">
                                            <Clock className="h-4 w-4" />
                                            {t('animalActions.euthanasia.pending.timeRemaining', { time: formatCountdown(order.deadline_at, t) })}
                                        </div>
                                    </div>
                                    {signingOrderId !== order.id && (
                                        <div className="flex flex-col gap-2">
                                            <Button
                                                size="sm"
                                                className="bg-status-success-solid hover:bg-status-success-solid/90"
                                                onClick={() => {
                                                    setSigningOrderId(order.id)
                                                    setSignatureData(null)
                                                }}
                                                disabled={approveMutation.isPending}
                                            >
                                                <PenLine className="h-4 w-4 mr-1" />
                                                {t('animalActions.euthanasia.pending.approveExecute')}
                                            </Button>
                                            <Button
                                                size="sm"
                                                variant="outline"
                                                className="border-status-warning-solid text-status-warning-text hover:bg-status-warning-bg"
                                                onClick={() => {
                                                    setSelectedOrder(order)
                                                    setShowAppealDialog(true)
                                                }}
                                            >
                                                <Hand className="h-4 w-4 mr-1" />
                                                {t('animalActions.euthanasia.pending.requestDeferral')}
                                            </Button>
                                        </div>
                                    )}
                                </div>

                                {/* 手寫簽名區塊 */}
                                {signingOrderId === order.id && (
                                    <div className="space-y-3 pt-3 border-t border-red-100">
                                        <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                                            <PenLine className="h-4 w-4" />
                                            {t('animalActions.euthanasia.pending.signatureConfirmApprove', { signature: t('signature.handwriting') })}
                                        </div>
                                        <HandwrittenSignaturePad
                                            onSignatureChange={setSignatureData}
                                            height={160}
                                        />
                                        <div className="flex gap-2 justify-end">
                                            <Button
                                                size="sm"
                                                variant="outline"
                                                onClick={() => {
                                                    setSigningOrderId(null)
                                                    setSignatureData(null)
                                                }}
                                            >
                                                {t('common.cancel')}
                                            </Button>
                                            <Button
                                                size="sm"
                                                className="bg-status-success-solid hover:bg-status-success-solid/90"
                                                onClick={() => {
                                                    if (signatureData) {
                                                        approveMutation.mutate({ orderId: order.id, sigData: signatureData })
                                                    }
                                                }}
                                                disabled={!signatureData || approveMutation.isPending}
                                            >
                                                {approveMutation.isPending ? (
                                                    <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                                ) : (
                                                    <CheckCircle2 className="h-4 w-4 mr-1" />
                                                )}
                                                {t('signature.confirmSign')}
                                            </Button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>

            {/* Appeal Dialog */}
            <Dialog open={showAppealDialog} onOpenChange={setShowAppealDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2 text-status-warning-text">
                            <Hand className="h-5 w-5" />
                            {t('animalActions.euthanasia.pending.appealTitle')}
                        </DialogTitle>
                        <DialogDescription>
                            {selectedOrder && (
                                <>
                                    {t('animalActions.common.earTagLabel', { earTag: selectedOrder.animal_ear_tag })}
                                    {selectedOrder.animal_iacuc_no && ` | IACUC No.: ${selectedOrder.animal_iacuc_no}`}
                                </>
                            )}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="space-y-4">
                        <div className="bg-status-warning-bg border border-status-warning-border rounded-lg p-4 text-sm text-status-warning-text">
                            <p className="font-medium mb-2">{t('animalActions.euthanasia.pending.appealNotesTitle')}</p>
                            <ul className="list-disc pl-4 space-y-1">
                                <li>{t('animalActions.euthanasia.pending.appealNote1')}</li>
                                <li>{t('animalActions.euthanasia.pending.appealNote2')}</li>
                                <li>{t('animalActions.euthanasia.pending.appealNote3')}</li>
                            </ul>
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="appeal_reason">{t('animalActions.euthanasia.pending.appealReasonRequired')}</Label>
                            <Textarea
                                id="appeal_reason"
                                value={appealReason}
                                onChange={(e) => setAppealReason(e.target.value)}
                                placeholder={t('animalActions.euthanasia.pending.appealReasonHint')}
                                className="min-h-[120px]"
                                required
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => {
                                setShowAppealDialog(false)
                                setAppealReason('')
                            }}
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            type="button"
                            className="bg-status-warning-solid text-white hover:bg-status-warning-solid/90"
                            onClick={() => {
                                if (selectedOrder && appealReason.trim()) {
                                    appealMutation.mutate({ orderId: selectedOrder.id, reason: appealReason })
                                }
                            }}
                            disabled={appealMutation.isPending || !appealReason.trim()}
                        >
                            {appealMutation.isPending ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                                <Hand className="h-4 w-4 mr-2" />
                            )}
                            {t('animalActions.euthanasia.pending.submitAppeal')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
