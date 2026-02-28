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
import { getApiErrorMessage } from '@/lib/validation'
import { Loader2, Gavel, CheckCircle2, XCircle, Clock, FileText } from 'lucide-react'

interface EuthanasiaAppeal {
    id: string
    order_id: string
    pi_user_id: string
    reason: string
    attachment_path?: string
    chair_user_id?: string
    chair_decision?: string
    chair_decided_at?: string
    chair_deadline_at: string
    created_at: string
    // Joined fields
    animal_ear_tag?: string
    animal_iacuc_no?: string
    vet_name?: string
    pi_name?: string
    vet_reason?: string
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

export function EuthanasiaChairArbitrationPanel() {
    const queryClient = useQueryClient()
    const [selectedAppeal, setSelectedAppeal] = useState<EuthanasiaAppeal | null>(null)
    const [showDecisionDialog, setShowDecisionDialog] = useState(false)
    const [decision, setDecision] = useState<'approve_appeal' | 'reject_appeal' | ''>('')
    const [decisionReason, setDecisionReason] = useState('')

    const { data: appeals, isLoading } = useQuery<EuthanasiaAppeal[]>({
        queryKey: ['euthanasia-appeals-pending'],
        queryFn: async () => {
            const res = await api.get('/euthanasia/appeals/pending')
            return res.data
        },
    })

    const decideMutation = useMutation({
        mutationFn: async ({ appealId, decision, comment }: { appealId: string; decision: string; comment?: string }) => {
            return api.post(`/euthanasia/appeals/${appealId}/decide`, { decision, comment })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['euthanasia-appeals-pending'] })
            toast({
                title: '裁決已送出',
                description: decision === 'approve_appeal' ? '已核准暫緩，安樂死取消。' : '已駁回暫緩，將執行安樂死。',
            })
            setShowDecisionDialog(false)
            setDecision('')
            setDecisionReason('')
            setSelectedAppeal(null)
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '操作失敗'),
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

    if (!appeals || appeals.length === 0) {
        return null // No pending appeals
    }

    return (
        <>
            {/* Pending Appeals Card */}
            <Card className="border-amber-200 bg-amber-50 mb-6">
                <CardHeader className="pb-3">
                    <CardTitle className="text-amber-700 flex items-center gap-2">
                        <Gavel className="h-5 w-5" />
                        待仲裁安樂死暫緩申請
                    </CardTitle>
                    <CardDescription className="text-amber-600">
                        您有 {appeals.length} 筆暫緩申請待裁決
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        {appeals.map((appeal) => (
                            <div
                                key={appeal.id}
                                className="bg-white rounded-lg p-4 border border-amber-200"
                            >
                                <div className="flex items-start justify-between">
                                    <div className="space-y-2 flex-1">
                                        <div className="flex items-center gap-3">
                                            <span className="font-bold text-lg text-orange-600">
                                                #{appeal.animal_ear_tag}
                                            </span>
                                            {appeal.animal_iacuc_no && (
                                                <Badge variant="outline">{appeal.animal_iacuc_no}</Badge>
                                            )}
                                        </div>

                                        <div className="grid grid-cols-2 gap-4 text-sm">
                                            <div>
                                                <span className="text-gray-500">獸醫：</span>
                                                <span className="ml-1">{appeal.vet_name}</span>
                                            </div>
                                            <div>
                                                <span className="text-gray-500">PI：</span>
                                                <span className="ml-1">{appeal.pi_name}</span>
                                            </div>
                                        </div>

                                        <div className="bg-red-50 rounded p-3 text-sm">
                                            <p className="text-gray-600 mb-1">
                                                <strong>獸醫安樂死原因：</strong>
                                            </p>
                                            <p className="text-gray-800">{appeal.vet_reason}</p>
                                        </div>

                                        <div className="bg-amber-50 rounded p-3 text-sm">
                                            <p className="text-gray-600 mb-1">
                                                <strong>PI 暫緩理由：</strong>
                                            </p>
                                            <p className="text-gray-800">{appeal.reason}</p>
                                            {appeal.attachment_path && (
                                                <a
                                                    href={appeal.attachment_path}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="inline-flex items-center gap-1 text-blue-600 hover:underline mt-2"
                                                >
                                                    <FileText className="h-4 w-4" />
                                                    查看附件
                                                </a>
                                            )}
                                        </div>

                                        <div className="flex items-center gap-2 text-sm text-amber-600">
                                            <Clock className="h-4 w-4" />
                                            裁決期限：{formatCountdown(appeal.chair_deadline_at)}
                                        </div>
                                    </div>

                                    <div className="flex flex-col gap-2 ml-4">
                                        <Button
                                            size="sm"
                                            className="bg-green-600 hover:bg-green-700"
                                            onClick={() => {
                                                setSelectedAppeal(appeal)
                                                setDecision('approve_appeal')
                                                setShowDecisionDialog(true)
                                            }}
                                        >
                                            <CheckCircle2 className="h-4 w-4 mr-1" />
                                            核准暫緩
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="destructive"
                                            onClick={() => {
                                                setSelectedAppeal(appeal)
                                                setDecision('reject_appeal')
                                                setShowDecisionDialog(true)
                                            }}
                                        >
                                            <XCircle className="h-4 w-4 mr-1" />
                                            駁回申請
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>

            {/* Decision Dialog */}
            <Dialog open={showDecisionDialog} onOpenChange={setShowDecisionDialog}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <Gavel className="h-5 w-5" />
                            {decision === 'approve_appeal' ? (
                                <span className="text-green-600">核准暫緩</span>
                            ) : (
                                <span className="text-red-600">駁回申請</span>
                            )}
                        </DialogTitle>
                        <DialogDescription>
                            {selectedAppeal && (
                                <>
                                    耳號：{selectedAppeal.animal_ear_tag}
                                    {selectedAppeal.animal_iacuc_no && ` | IACUC No.: ${selectedAppeal.animal_iacuc_no}`}
                                </>
                            )}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="space-y-4">
                        {decision === 'approve_appeal' ? (
                            <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-sm text-green-800">
                                <p className="font-medium mb-2">核准暫緩</p>
                                <p>核准暫緩後，安樂死單將被取消，動物可繼續留在計畫中。</p>
                            </div>
                        ) : (
                            <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-800">
                                <p className="font-medium mb-2">駁回暫緩申請</p>
                                <p>駁回後，獸醫師將可執行安樂死操作。</p>
                            </div>
                        )}

                        <div className="space-y-2">
                            <Label htmlFor="decision_reason">裁決說明（選填）</Label>
                            <Textarea
                                id="decision_reason"
                                value={decisionReason}
                                onChange={(e) => setDecisionReason(e.target.value)}
                                placeholder="可填寫裁決的理由或說明..."
                                className="min-h-[100px]"
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => {
                                setShowDecisionDialog(false)
                                setDecision('')
                                setDecisionReason('')
                            }}
                        >
                            取消
                        </Button>
                        <Button
                            type="button"
                            className={decision === 'approve_appeal' ? 'bg-green-600 hover:bg-green-700' : 'bg-red-600 hover:bg-red-700'}
                            onClick={() => {
                                if (selectedAppeal && decision) {
                                    decideMutation.mutate({
                                        appealId: selectedAppeal.id,
                                        decision,
                                        comment: decisionReason || undefined,
                                    })
                                }
                            }}
                            disabled={decideMutation.isPending}
                        >
                            {decideMutation.isPending ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                                <Gavel className="h-4 w-4 mr-2" />
                            )}
                            確認裁決
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
