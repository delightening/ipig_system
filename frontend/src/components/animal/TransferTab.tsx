import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { transferApi, AnimalTransfer, TransferVetEvaluation, AnimalTransferStatus, transferStatusNames, signatureApi } from '@/lib/api'
import type { ProtocolListItem } from '@/lib/api'
import api from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import {
    ArrowRightLeft, CheckCircle2, XCircle, Clock, Stethoscope,
    FileCheck, UserCheck, Loader2, Plus, ChevronDown, ChevronUp,
    AlertTriangle, PenLine,
} from 'lucide-react'
import { useAuthStore } from '@/stores/auth'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'
import { useTranslation } from 'react-i18next'

// ============================================
// Stepper 步驟定義
// ============================================

const TRANSFER_STEPS: { status: AnimalTransferStatus; label: string; icon: typeof Clock }[] = [
    { status: 'pending', label: '發起', icon: Clock },
    { status: 'vet_evaluated', label: '獸醫評估', icon: Stethoscope },
    { status: 'plan_assigned', label: '指定新計劃', icon: FileCheck },
    { status: 'pi_approved', label: 'PI 同意', icon: UserCheck },
    { status: 'completed', label: '完成', icon: CheckCircle2 },
]

function getStepIndex(status: AnimalTransferStatus): number {
    if (status === 'rejected') return -1
    return TRANSFER_STEPS.findIndex(s => s.status === status)
}

// ============================================
// Props
// ============================================

interface Props {
    animalId: string
    animalStatus: string
    earTag: string
}

// ============================================
// 轉讓 Stepper 元件
// ============================================

function TransferStepper({ transfer }: { transfer: AnimalTransfer }) {
    const currentIdx = getStepIndex(transfer.status)
    const isRejected = transfer.status === 'rejected'

    return (
        <div className="flex items-center gap-1 w-full overflow-x-auto py-2">
            {TRANSFER_STEPS.map((step, idx) => {
                const Icon = step.icon
                const isDone = !isRejected && currentIdx >= idx
                const isCurrent = !isRejected && currentIdx === idx

                return (
                    <div key={step.status} className="flex items-center flex-1 min-w-0">
                        <div className={`flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium whitespace-nowrap
              ${isDone ? 'bg-indigo-100 text-indigo-700' : 'bg-slate-100 text-slate-400'}
              ${isCurrent ? 'ring-2 ring-indigo-400' : ''}
            `}>
                            <Icon className="h-3.5 w-3.5 flex-shrink-0" />
                            <span className="hidden sm:inline">{step.label}</span>
                        </div>
                        {idx < TRANSFER_STEPS.length - 1 && (
                            <div className={`h-0.5 flex-1 mx-1 min-w-[12px] ${isDone && idx < currentIdx ? 'bg-indigo-400' : 'bg-slate-200'}`} />
                        )}
                    </div>
                )
            })}
            {isRejected && (
                <div className="flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-700 ring-2 ring-red-400 ml-2">
                    <XCircle className="h-3.5 w-3.5" />
                    <span>已拒絕</span>
                </div>
            )}
        </div>
    )
}

// ============================================
// 主元件
// ============================================

export function TransferTab({ animalId, animalStatus, earTag }: Props) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const user = useAuthStore(s => s.user)
    const [showInitiateForm, setShowInitiateForm] = useState(false)
    const [showHistory, setShowHistory] = useState(false)

    // 表單狀態
    const [initiateReason, setInitiateReason] = useState('')
    const [initiateRemark, setInitiateRemark] = useState('')
    const [vetHealthStatus, setVetHealthStatus] = useState('')
    const [vetFit, setVetFit] = useState(true)
    const [vetConditions, setVetConditions] = useState('')
    const [targetIacuc, setTargetIacuc] = useState('')
    const [rejectReason, setRejectReason] = useState('')
    // 簽名相關狀態
    const [showApproveSignature, setShowApproveSignature] = useState(false)
    const [approveSignatureData, setApproveSignatureData] = useState<SignatureData | null>(null)
    const [showCompleteSignature, setShowCompleteSignature] = useState(false)
    const [completeSignatureData, setCompleteSignatureData] = useState<SignatureData | null>(null)

    // 查詢轉讓記錄
    const { data: transfers = [], isLoading } = useQuery({
        queryKey: ['animal-transfers', animalId],
        queryFn: async () => {
            const res = await transferApi.list(animalId)
            return res.data
        },
    })

    // 查詢核准計劃（用於指定新計劃下拉）
    const { data: approvedProtocols } = useQuery({
        queryKey: ['approved-protocols'],
        queryFn: async () => {
            const res = await api.get<ProtocolListItem[]>('/protocols')
            return res.data.filter(p =>
                (p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && p.iacuc_no
            )
        },
    })

    // 進行中的轉讓
    const activeTransfer = transfers.find(t =>
        !['completed', 'rejected'].includes(t.status)
    )

    // 歷史轉讓
    const historyTransfers = transfers.filter(t =>
        ['completed', 'rejected'].includes(t.status)
    )

    // 角色判斷
    const isVet = user?.roles?.includes('VET') ?? false
    const isPI = user?.roles?.includes('PI') ?? false
    const isAdmin = user?.roles?.includes('ADMIN') ?? false
    const canInitiate = (animalStatus === 'completed') && !activeTransfer
    const canVetEvaluate = (isVet || isAdmin) && activeTransfer?.status === 'pending'
    const canAssignPlan = (isAdmin || isPI) && activeTransfer?.status === 'vet_evaluated'
    const canApprove = (isAdmin || isPI) && activeTransfer?.status === 'plan_assigned'
    const canComplete = isAdmin && activeTransfer?.status === 'pi_approved'
    const canReject = (isAdmin || isPI || isVet) && activeTransfer && !['completed', 'rejected'].includes(activeTransfer.status)

    // Mutations
    const invalidate = () => {
        queryClient.invalidateQueries({ queryKey: ['animal-transfers', animalId] })
        queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
        queryClient.invalidateQueries({ queryKey: ['animals'] })
    }

    const initiateMutation = useMutation({
        mutationFn: () => transferApi.initiate(animalId, { reason: initiateReason, remark: initiateRemark || undefined }),
        onSuccess: () => {
            toast({ title: '成功', description: '已發起轉讓申請' })
            setShowInitiateForm(false)
            setInitiateReason('')
            setInitiateRemark('')
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '發起失敗', variant: 'destructive' }),
    })

    const vetEvaluateMutation = useMutation({
        mutationFn: () => transferApi.vetEvaluate(activeTransfer!.id, {
            health_status: vetHealthStatus,
            is_fit_for_transfer: vetFit,
            conditions: vetConditions || undefined,
        }),
        onSuccess: () => {
            toast({ title: '成功', description: '獸醫評估已提交' })
            setVetHealthStatus('')
            setVetConditions('')
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '評估失敗', variant: 'destructive' }),
    })

    const assignPlanMutation = useMutation({
        mutationFn: () => transferApi.assignPlan(activeTransfer!.id, { to_iacuc_no: targetIacuc }),
        onSuccess: () => {
            toast({ title: '成功', description: '已指定新計劃' })
            setTargetIacuc('')
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '指定失敗', variant: 'destructive' }),
    })

    const approveMutation = useMutation({
        mutationFn: async (sigData: SignatureData) => {
            // 先簽章
            await signatureApi.signTransfer(activeTransfer!.id, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
                signature_type: 'APPROVE',
            })
            return transferApi.approve(activeTransfer!.id)
        },
        onSuccess: () => {
            toast({ title: '成功', description: 'PI 已同意轉讓，簽章已記錄' })
            setShowApproveSignature(false)
            setApproveSignatureData(null)
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '同意失敗', variant: 'destructive' }),
    })

    const completeMutation = useMutation({
        mutationFn: async (sigData: SignatureData) => {
            // 先簽章
            await signatureApi.signTransfer(activeTransfer!.id, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
                signature_type: 'CONFIRM',
            })
            return transferApi.complete(activeTransfer!.id)
        },
        onSuccess: () => {
            toast({ title: '成功', description: '轉讓已完成，動物已分配到新計劃' })
            setShowCompleteSignature(false)
            setCompleteSignatureData(null)
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '完成失敗', variant: 'destructive' }),
    })

    const rejectMutation = useMutation({
        mutationFn: () => transferApi.reject(activeTransfer!.id, { reason: rejectReason }),
        onSuccess: () => {
            toast({ title: '已拒絕', description: '轉讓申請已拒絕' })
            setRejectReason('')
            invalidate()
        },
        onError: (e: any) => toast({ title: '錯誤', description: e?.response?.data?.error?.message || '拒絕失敗', variant: 'destructive' }),
    })

    if (isLoading) {
        return <div className="flex items-center justify-center py-12"><Loader2 className="h-6 w-6 animate-spin text-slate-400" /></div>
    }

    return (
        <div className="space-y-6">
            {/* 發起轉讓按鈕 */}
            {canInitiate && !showInitiateForm && (
                <Button
                    className="bg-indigo-600 hover:bg-indigo-700"
                    onClick={() => setShowInitiateForm(true)}
                >
                    <Plus className="h-4 w-4 mr-2" />
                    發起轉讓
                </Button>
            )}

            {/* 發起表單 */}
            {showInitiateForm && (
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
                                value={initiateReason}
                                onChange={e => setInitiateReason(e.target.value)}
                                placeholder="說明轉讓原因..."
                                className="min-h-[80px]"
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="transfer-remark">備註</Label>
                            <Input
                                id="transfer-remark"
                                value={initiateRemark}
                                onChange={e => setInitiateRemark(e.target.value)}
                                placeholder="可選備註"
                            />
                        </div>
                        <div className="flex gap-2">
                            <Button
                                onClick={() => initiateMutation.mutate()}
                                disabled={!initiateReason.trim() || initiateMutation.isPending}
                                className="bg-indigo-600 hover:bg-indigo-700"
                            >
                                {initiateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                                確認發起
                            </Button>
                            <Button variant="outline" onClick={() => setShowInitiateForm(false)}>取消</Button>
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* 進行中的轉讓 */}
            {activeTransfer && (
                <Card className="border-indigo-200 bg-indigo-50/30">
                    <CardHeader>
                        <div className="flex items-center justify-between">
                            <CardTitle className="text-lg flex items-center gap-2">
                                <ArrowRightLeft className="h-5 w-5 text-indigo-600" />
                                進行中的轉讓
                            </CardTitle>
                            <Badge className="bg-indigo-100 text-indigo-700">{transferStatusNames[activeTransfer.status]}</Badge>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        {/* Stepper */}
                        <TransferStepper transfer={activeTransfer} />

                        {/* 轉讓資訊 */}
                        <div className="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-slate-500">原計劃</span>
                                <p className="font-medium">{activeTransfer.from_iacuc_no}</p>
                            </div>
                            {activeTransfer.to_iacuc_no && (
                                <div>
                                    <span className="text-slate-500">新計劃</span>
                                    <p className="font-medium">{activeTransfer.to_iacuc_no}</p>
                                </div>
                            )}
                            <div className="col-span-2">
                                <span className="text-slate-500">原因</span>
                                <p className="font-medium">{activeTransfer.reason}</p>
                            </div>
                            {activeTransfer.remark && (
                                <div className="col-span-2">
                                    <span className="text-slate-500">備註</span>
                                    <p>{activeTransfer.remark}</p>
                                </div>
                            )}
                            <div>
                                <span className="text-slate-500">發起時間</span>
                                <p>{new Date(activeTransfer.created_at).toLocaleString('zh-TW')}</p>
                            </div>
                        </div>

                        {/* 步驟 2：獸醫評估表單 */}
                        {canVetEvaluate && (
                            <Card className="border-emerald-200">
                                <CardHeader>
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Stethoscope className="h-4 w-4 text-emerald-600" />
                                        獸醫評估
                                    </CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-3">
                                    <div className="space-y-2">
                                        <Label>健康狀態 *</Label>
                                        <Textarea
                                            value={vetHealthStatus}
                                            onChange={e => setVetHealthStatus(e.target.value)}
                                            placeholder="描述動物當前健康狀態..."
                                            className="min-h-[60px]"
                                        />
                                    </div>
                                    <div className="flex items-center gap-3">
                                        <Label>是否適合轉讓</Label>
                                        <Select value={vetFit ? 'yes' : 'no'} onValueChange={v => setVetFit(v === 'yes')}>
                                            <SelectTrigger className="w-[140px]">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="yes">✅ 適合</SelectItem>
                                                <SelectItem value="no">❌ 不適合</SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </div>
                                    <div className="space-y-2">
                                        <Label>附帶條件</Label>
                                        <Input
                                            value={vetConditions}
                                            onChange={e => setVetConditions(e.target.value)}
                                            placeholder="如有附帶條件請說明"
                                        />
                                    </div>
                                    <Button
                                        onClick={() => vetEvaluateMutation.mutate()}
                                        disabled={!vetHealthStatus.trim() || vetEvaluateMutation.isPending}
                                        className="bg-emerald-600 hover:bg-emerald-700"
                                    >
                                        {vetEvaluateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                                        提交評估
                                    </Button>
                                </CardContent>
                            </Card>
                        )}

                        {/* 步驟 3：指定新計劃 */}
                        {canAssignPlan && (
                            <Card className="border-blue-200">
                                <CardHeader>
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <FileCheck className="h-4 w-4 text-blue-600" />
                                        指定新計劃
                                    </CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-3">
                                    <div className="space-y-2">
                                        <Label>目標 IACUC No. *</Label>
                                        <Select value={targetIacuc} onValueChange={setTargetIacuc}>
                                            <SelectTrigger>
                                                <SelectValue placeholder="選擇目標計劃..." />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {approvedProtocols?.filter(p => p.iacuc_no !== activeTransfer.from_iacuc_no).map(p => (
                                                    <SelectItem key={p.id} value={p.iacuc_no!}>
                                                        {p.iacuc_no} — {p.title}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                    <Button
                                        onClick={() => assignPlanMutation.mutate()}
                                        disabled={!targetIacuc || assignPlanMutation.isPending}
                                        className="bg-blue-600 hover:bg-blue-700"
                                    >
                                        {assignPlanMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                                        確認指定
                                    </Button>
                                </CardContent>
                            </Card>
                        )}

                        {/* 步驟 4：PI 同意 + 手寫簽名 */}
                        {canApprove && (
                            <Card className="border-green-200">
                                <CardContent className="pt-4 space-y-3">
                                    {!showApproveSignature ? (
                                        <Button
                                            onClick={() => setShowApproveSignature(true)}
                                            disabled={approveMutation.isPending}
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
                                            <HandwrittenSignaturePad
                                                onSignatureChange={setApproveSignatureData}
                                                height={140}
                                            />
                                            <div className="flex gap-2 justify-end">
                                                <Button size="sm" variant="outline" onClick={() => { setShowApproveSignature(false); setApproveSignatureData(null) }}>
                                                    取消
                                                </Button>
                                                <Button
                                                    size="sm"
                                                    className="bg-green-600 hover:bg-green-700"
                                                    onClick={() => approveSignatureData && approveMutation.mutate(approveSignatureData)}
                                                    disabled={!approveSignatureData || approveMutation.isPending}
                                                >
                                                    {approveMutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <UserCheck className="h-4 w-4 mr-1" />}
                                                    {t('signature.confirmSign', '確認簽署')}
                                                </Button>
                                            </div>
                                        </div>
                                    )}
                                </CardContent>
                            </Card>
                        )}

                        {/* 步驟 5：完成轉讓 + 手寫簽名 */}
                        {canComplete && (
                            <Card className="border-indigo-200">
                                <CardContent className="pt-4 space-y-3">
                                    {!showCompleteSignature ? (
                                        <Button
                                            onClick={() => setShowCompleteSignature(true)}
                                            disabled={completeMutation.isPending}
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
                                            <HandwrittenSignaturePad
                                                onSignatureChange={setCompleteSignatureData}
                                                height={140}
                                            />
                                            <div className="flex gap-2 justify-end">
                                                <Button size="sm" variant="outline" onClick={() => { setShowCompleteSignature(false); setCompleteSignatureData(null) }}>
                                                    取消
                                                </Button>
                                                <Button
                                                    size="sm"
                                                    className="bg-indigo-600 hover:bg-indigo-700"
                                                    onClick={() => completeSignatureData && completeMutation.mutate(completeSignatureData)}
                                                    disabled={!completeSignatureData || completeMutation.isPending}
                                                >
                                                    {completeMutation.isPending ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <CheckCircle2 className="h-4 w-4 mr-1" />}
                                                    {t('signature.confirmSign', '確認簽署')}
                                                </Button>
                                            </div>
                                        </div>
                                    )}
                                </CardContent>
                            </Card>
                        )}

                        {/* 拒絕 */}
                        {canReject && (
                            <Card className="border-red-200">
                                <CardContent className="pt-4 space-y-3">
                                    <div className="flex items-center gap-2 text-red-600 text-sm font-medium">
                                        <AlertTriangle className="h-4 w-4" />
                                        拒絕轉讓
                                    </div>
                                    <div className="flex gap-2">
                                        <Input
                                            value={rejectReason}
                                            onChange={e => setRejectReason(e.target.value)}
                                            placeholder="拒絕原因（必填）"
                                            className="flex-1"
                                        />
                                        <Button
                                            variant="destructive"
                                            onClick={() => rejectMutation.mutate()}
                                            disabled={!rejectReason.trim() || rejectMutation.isPending}
                                        >
                                            {rejectMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                                            拒絕
                                        </Button>
                                    </div>
                                </CardContent>
                            </Card>
                        )}
                    </CardContent>
                </Card>
            )}

            {/* 無資料提示 */}
            {!activeTransfer && !showInitiateForm && historyTransfers.length === 0 && (
                <Card className="bg-slate-50">
                    <CardContent className="py-8 text-center text-slate-500">
                        <ArrowRightLeft className="h-8 w-8 mx-auto mb-3 text-slate-300" />
                        <p>此動物尚無轉讓記錄</p>
                        {canInitiate && <p className="text-xs mt-1">點擊上方按鈕發起轉讓</p>}
                    </CardContent>
                </Card>
            )}

            {/* 歷史紀錄 */}
            {historyTransfers.length > 0 && (
                <div>
                    <button
                        className="flex items-center gap-2 text-sm font-medium text-slate-600 hover:text-slate-800 mb-3"
                        onClick={() => setShowHistory(!showHistory)}
                    >
                        {showHistory ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                        歷史轉讓紀錄 ({historyTransfers.length})
                    </button>
                    {showHistory && (
                        <div className="space-y-3">
                            {historyTransfers.map(t => (
                                <Card key={t.id} className={`border ${t.status === 'completed' ? 'border-green-200' : 'border-red-200'}`}>
                                    <CardContent className="pt-4">
                                        <div className="flex items-center justify-between mb-2">
                                            <div className="flex items-center gap-2">
                                                {t.status === 'completed' ? (
                                                    <CheckCircle2 className="h-4 w-4 text-green-600" />
                                                ) : (
                                                    <XCircle className="h-4 w-4 text-red-600" />
                                                )}
                                                <span className="text-sm font-medium">
                                                    {t.from_iacuc_no} → {t.to_iacuc_no || '—'}
                                                </span>
                                            </div>
                                            <Badge className={t.status === 'completed' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}>
                                                {transferStatusNames[t.status]}
                                            </Badge>
                                        </div>
                                        <p className="text-sm text-slate-600">{t.reason}</p>
                                        {t.rejected_reason && (
                                            <p className="text-sm text-red-600 mt-1">拒絕原因：{t.rejected_reason}</p>
                                        )}
                                        <p className="text-xs text-slate-400 mt-2">
                                            {new Date(t.created_at).toLocaleString('zh-TW')}
                                            {t.completed_at && ` → ${new Date(t.completed_at).toLocaleString('zh-TW')}`}
                                        </p>
                                    </CardContent>
                                </Card>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    )
}
