import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
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
import { Loader2, AlertOctagon, CheckCircle2, Hand, Clock } from 'lucide-react'

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

function formatCountdown(deadline: string): string {
    const now = new Date()
    const deadlineDate = new Date(deadline)
    const diff = deadlineDate.getTime() - now.getTime()

    if (diff <= 0) {
        return '已到期'
    }

    const hours = Math.floor(diff / (1000 * 60 * 60))
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))

    if (hours > 0) {
        return `${hours} 小時 ${minutes} 分`
    }
    return `${minutes} 分`
}

export function EuthanasiaPendingPanel() {
    const queryClient = useQueryClient()
    const [selectedOrder, setSelectedOrder] = useState<EuthanasiaOrder | null>(null)
    const [showAppealDialog, setShowAppealDialog] = useState(false)
    const [appealReason, setAppealReason] = useState('')

    const { data: orders, isLoading } = useQuery<EuthanasiaOrder[]>({
        queryKey: ['euthanasia-pending'],
        queryFn: async () => {
            const res = await api.get('/euthanasia/orders/pending')
            return res.data
        },
    })

    const approveMutation = useMutation({
        mutationFn: async (orderId: string) => {
            return api.post(`/euthanasia/orders/${orderId}/approve`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['euthanasia-pending'] })
            toast({
                title: '已同意執行安樂死',
                description: '獸醫師將收到通知並可執行操作。',
            })
            setSelectedOrder(null)
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '操作失敗',
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
                title: '暫緩申請已送出',
                description: 'CHAIR 將於 24 小時內進行仲裁。',
            })
            setShowAppealDialog(false)
            setAppealReason('')
            setSelectedOrder(null)
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '操作失敗',
                variant: 'destructive',
            })
        },
    })

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-slate-400" />
            </div>
        )
    }

    if (!orders || orders.length === 0) {
        return null // No pending orders
    }

    return (
        <>
            {/* Pending Orders Alert Card */}
            <Card className="border-red-200 bg-red-50 mb-6">
                <CardHeader className="pb-3">
                    <CardTitle className="text-red-700 flex items-center gap-2">
                        <AlertOctagon className="h-5 w-5" />
                        待處理安樂死單
                    </CardTitle>
                    <CardDescription className="text-red-600">
                        您有 {orders.length} 筆安樂死通知待處理
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        {orders.map((order) => (
                            <div
                                key={order.id}
                                className="bg-white rounded-lg p-4 border border-red-200 flex items-center justify-between"
                            >
                                <div className="space-y-1">
                                    <div className="flex items-center gap-3">
                                        <span className="font-bold text-lg text-orange-600">
                                            #{order.animal_ear_tag}
                                        </span>
                                        {order.animal_iacuc_no && (
                                            <Badge variant="outline">{order.animal_iacuc_no}</Badge>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-600">
                                        開單獸醫：{order.vet_name}
                                    </p>
                                    <p className="text-sm text-gray-700 line-clamp-2">
                                        原因：{order.reason}
                                    </p>
                                    <div className="flex items-center gap-2 text-sm text-red-600">
                                        <Clock className="h-4 w-4" />
                                        剩餘時間：{formatCountdown(order.deadline_at)}
                                    </div>
                                </div>
                                <div className="flex flex-col gap-2">
                                    <Button
                                        size="sm"
                                        className="bg-green-600 hover:bg-green-700"
                                        onClick={() => {
                                            if (confirm('確定同意執行安樂死？此操作不可逆。')) {
                                                approveMutation.mutate(order.id)
                                            }
                                        }}
                                        disabled={approveMutation.isPending}
                                    >
                                        <CheckCircle2 className="h-4 w-4 mr-1" />
                                        同意執行
                                    </Button>
                                    <Button
                                        size="sm"
                                        variant="outline"
                                        className="border-amber-500 text-amber-600 hover:bg-amber-50"
                                        onClick={() => {
                                            setSelectedOrder(order)
                                            setShowAppealDialog(true)
                                        }}
                                    >
                                        <Hand className="h-4 w-4 mr-1" />
                                        申請暫緩
                                    </Button>
                                </div>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>

            {/* Appeal Dialog */}
            <Dialog open={showAppealDialog} onOpenChange={setShowAppealDialog}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2 text-amber-600">
                            <Hand className="h-5 w-5" />
                            申請暫緩安樂死
                        </DialogTitle>
                        <DialogDescription>
                            {selectedOrder && (
                                <>
                                    耳號：{selectedOrder.animal_ear_tag}
                                    {selectedOrder.animal_iacuc_no && ` | IACUC NO.: ${selectedOrder.animal_iacuc_no}`}
                                </>
                            )}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="space-y-4">
                        <div className="bg-amber-50 border border-amber-200 rounded-lg p-4 text-sm text-amber-800">
                            <p className="font-medium mb-2">暫緩申請說明：</p>
                            <ul className="list-disc pl-4 space-y-1">
                                <li>提交暫緩申請後，CHAIR 將於 24 小時內進行仲裁</li>
                                <li>若 CHAIR 未於時限內回應，系統將自動核准執行安樂死</li>
                                <li>請詳細說明希望暫緩的理由</li>
                            </ul>
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="appeal_reason">暫緩理由 *</Label>
                            <Textarea
                                id="appeal_reason"
                                value={appealReason}
                                onChange={(e) => setAppealReason(e.target.value)}
                                placeholder="請說明希望暫緩安樂死的原因，例如：動物狀況好轉、需要更多觀察時間、計畫需調整等..."
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
                            取消
                        </Button>
                        <Button
                            type="button"
                            className="bg-amber-500 hover:bg-amber-600"
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
                            送出暫緩申請
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
