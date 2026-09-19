/**
 * R20-7: 執行秘書 AI 標註面板
 *
 * 三類標註：needs_attention / concern / suggestion
 * 支援勾選後批次退回補件（batch_return_to_pi）。
 * 僅 IACUC_STAFF / IACUC_CHAIR 可見。
 */
import { useState } from 'react'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import {
    AlertCircle,
    AlertTriangle,
    Bot,
    Info,
    Loader2,
    RefreshCw,
    Send,
} from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { toast } from '@/components/ui/use-toast'
import { aiReviewApi } from '@/lib/api'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/apiError'
import type { BatchReturnFlag, StaffAiResult, StaffReviewFlag } from '@/types/aiReview'

interface StaffReviewAssistPanelProps {
    protocolId: string
}

const FLAG_CONFIG: Record<
    string,
    // labelKey 為 i18n 鍵；渲染時才 t(labelKey)（模組頂層不可存翻譯後字串）
    { icon: typeof AlertCircle; labelKey: string; colorClass: string }
> = {
    needs_attention: {
        icon: AlertCircle,
        labelKey: 'protocolComponents.staffReviewAssist.flags.needsAttention',
        colorClass: 'text-status-error-text',
    },
    concern: {
        icon: AlertTriangle,
        labelKey: 'protocolComponents.staffReviewAssist.flags.concern',
        colorClass: 'text-status-warning-text',
    },
    suggestion: {
        icon: Info,
        labelKey: 'protocolComponents.staffReviewAssist.flags.suggestion',
        colorClass: 'text-primary',
    },
}

export function StaffReviewAssistPanel({
    protocolId,
}: StaffReviewAssistPanelProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set())
    const [showDialog, setShowDialog] = useState(false)
    const [additionalNote, setAdditionalNote] = useState('')

    const { data: review, isLoading } = useQuery({
        queryKey: ['staff-review', protocolId],
        queryFn: () => aiReviewApi.getLatestStaffReview(protocolId),
    })

    const refreshMutation = useMutation({
        mutationFn: () => aiReviewApi.requestStaffReview(protocolId),
        onSuccess: () => {
            toast({ title: t('protocolComponents.staffReviewAssist.reanalyzed') })
            setSelectedKeys(new Set())
            queryClient.invalidateQueries({
                queryKey: ['staff-review', protocolId],
            })
        },
        onError: (error: unknown) => {
            toast({
                title: t('protocolComponents.staffReviewAssist.analyzeFailed'),
                description: getApiErrorMessage(error, t('protocolComponents.staffReviewAssist.tryAgainLater')),
                variant: 'destructive',
            })
        },
    })

    const batchReturnMutation = useMutation({
        mutationFn: (flags: BatchReturnFlag[]) =>
            aiReviewApi.batchReturn(protocolId, {
                flags,
                additional_note: additionalNote.trim() || undefined,
            }),
        onSuccess: (data) => {
            toast({
                title: t('protocolComponents.staffReviewAssist.returnSuccessTitle'),
                description: t('protocolComponents.staffReviewAssist.returnSuccessDescription', { count: data.created_comments }),
            })
            setShowDialog(false)
            setSelectedKeys(new Set())
            setAdditionalNote('')
            queryClient.invalidateQueries({ queryKey: ['staff-review', protocolId] })
            queryClient.invalidateQueries({ queryKey: ['protocol', protocolId] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('protocolComponents.staffReviewAssist.returnFailed'),
                description: getApiErrorMessage(error, t('protocolComponents.staffReviewAssist.tryAgainLater')),
                variant: 'destructive',
            })
        },
    })

    const handleToggleFlag = (key: string) => {
        setSelectedKeys((prev) => {
            const next = new Set(prev)
            if (next.has(key)) next.delete(key)
            else next.add(key)
            return next
        })
    }

    const handleConfirmReturn = () => {
        const selected = allFlags
            .filter((_, idx) => selectedKeys.has(String(idx)))
            .map((f) => ({
                section: f.section,
                message: f.message,
                suggestion: f.suggestion,
            }))
        batchReturnMutation.mutate(selected)
    }

    if (isLoading) {
        return (
            <Card>
                <CardContent className="flex items-center justify-center py-6">
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    <span className="ml-2 text-sm text-muted-foreground">
                        {t('protocolComponents.staffReviewAssist.loading')}
                    </span>
                </CardContent>
            </Card>
        )
    }

    const aiResult = review?.ai_result as StaffAiResult | undefined
    const ruleResult = review?.rule_result

    const flags: StaffReviewFlag[] = aiResult?.flags ?? []

    const ruleFlags: StaffReviewFlag[] = [
        ...(ruleResult?.errors ?? []).map((e) => ({
            flag_type: 'needs_attention' as const,
            section: e.section,
            message: e.message,
            suggestion: e.suggestion,
        })),
        ...(ruleResult?.warnings ?? []).map((w) => ({
            flag_type: 'concern' as const,
            section: w.section,
            message: w.message,
            suggestion: w.suggestion,
        })),
    ]

    const allFlags = [...ruleFlags, ...flags]

    const grouped = {
        needs_attention: allFlags
            .map((f, idx) => ({ flag: f, idx }))
            .filter(({ flag }) => flag.flag_type === 'needs_attention'),
        concern: allFlags
            .map((f, idx) => ({ flag: f, idx }))
            .filter(({ flag }) => flag.flag_type === 'concern'),
        suggestion: allFlags
            .map((f, idx) => ({ flag: f, idx }))
            .filter(({ flag }) => flag.flag_type === 'suggestion'),
    }

    const hasAnyFlags = allFlags.length > 0
    const selectedCount = selectedKeys.size

    return (
        <>
            <Card className="border-2 border-border">
                <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-lg flex items-center gap-2">
                            <Bot className="h-5 w-5" />
                            {t('protocolComponents.staffReviewAssist.title')}
                        </CardTitle>
                        <div className="flex items-center gap-2">
                            {review && (
                                <span className="text-xs text-muted-foreground">
                                    {formatDate(review.created_at)}
                                </span>
                            )}
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => refreshMutation.mutate()}
                                disabled={refreshMutation.isPending}
                            >
                                {refreshMutation.isPending ? (
                                    <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                                ) : (
                                    <RefreshCw className="mr-1 h-3 w-3" />
                                )}
                                {t('protocolComponents.staffReviewAssist.reanalyze')}
                            </Button>
                        </div>
                    </div>
                    {aiResult?.summary && (
                        <p className="text-sm text-muted-foreground mt-1">
                            {aiResult.summary}
                        </p>
                    )}
                </CardHeader>
                <CardContent className="space-y-4">
                    {!hasAnyFlags && !review && (
                        <p className="text-sm text-muted-foreground text-center py-4">
                            {t('protocolComponents.staffReviewAssist.noResult')}
                        </p>
                    )}

                    {!hasAnyFlags && review && (
                        <p className="text-sm text-muted-foreground text-center py-4">
                            {t('protocolComponents.staffReviewAssist.noFlags')}
                        </p>
                    )}

                    {(['needs_attention', 'concern', 'suggestion'] as const).map(
                        (type) => {
                            const items = grouped[type]
                            if (items.length === 0) return null
                            const config = FLAG_CONFIG[type]
                            const Icon = config.icon
                            return (
                                <div key={type} className="space-y-2">
                                    <h4
                                        className={`text-sm font-semibold flex items-center gap-1.5 ${config.colorClass}`}
                                    >
                                        <Icon className="h-4 w-4" />
                                        {t('protocolComponents.staffReviewAssist.flagGroupTitle', { label: t(config.labelKey), count: items.length })}
                                    </h4>
                                    <ul className="space-y-2 ml-2">
                                        {items.map(({ flag, idx }) => {
                                            const key = String(idx)
                                            return (
                                                <li
                                                    key={idx}
                                                    className="flex items-start gap-2"
                                                >
                                                    <Checkbox
                                                        id={`flag-${key}`}
                                                        checked={selectedKeys.has(key)}
                                                        onCheckedChange={() =>
                                                            handleToggleFlag(key)
                                                        }
                                                        className="mt-0.5 shrink-0"
                                                    />
                                                    <Label
                                                        htmlFor={`flag-${key}`}
                                                        className="text-sm cursor-pointer"
                                                    >
                                                        <span
                                                            className={`font-medium ${config.colorClass}`}
                                                        >
                                                            [{flag.section}]{' '}
                                                            {flag.message}
                                                        </span>
                                                        <span className="block text-muted-foreground mt-0.5">
                                                            {flag.suggestion}
                                                        </span>
                                                    </Label>
                                                </li>
                                            )
                                        })}
                                    </ul>
                                </div>
                            )
                        }
                    )}

                    {hasAnyFlags && (
                        <div className="flex items-center justify-between pt-2 border-t">
                            <span className="text-xs text-muted-foreground">
                                {selectedCount > 0
                                    ? t('protocolComponents.staffReviewAssist.selectedCount', { count: selectedCount })
                                    : t('protocolComponents.staffReviewAssist.selectHint')}
                            </span>
                            <Button
                                size="sm"
                                variant="destructive"
                                disabled={selectedCount === 0}
                                onClick={() => setShowDialog(true)}
                            >
                                <Send className="mr-1.5 h-3.5 w-3.5" />
                                {t('protocolComponents.staffReviewAssist.returnButton', { count: selectedCount })}
                            </Button>
                        </div>
                    )}

                    {/* 免責聲明 */}
                    <p className="text-xs text-muted-foreground pt-1 italic">
                        {t('protocolComponents.staffReviewAssist.disclaimer')}
                    </p>
                </CardContent>
            </Card>

            <Dialog open={showDialog} onOpenChange={setShowDialog}>
                <DialogContent size="sm">
                    <DialogHeader>
                        <DialogTitle>{t('protocolComponents.staffReviewAssist.dialogTitle')}</DialogTitle>
                        <DialogDescription>
                            {t('protocolComponents.staffReviewAssist.dialogDescription', { count: selectedCount })}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="space-y-3">
                        <ul className="text-sm space-y-1 max-h-40 overflow-y-auto rounded border p-3 bg-muted/30">
                            {allFlags
                                .filter((_, idx) =>
                                    selectedKeys.has(String(idx))
                                )
                                .map((f, i) => (
                                    <li key={i} className="text-muted-foreground">
                                        <span className="font-medium text-foreground">
                                            [{f.section}]
                                        </span>{' '}
                                        {f.message}
                                    </li>
                                ))}
                        </ul>

                        <div className="space-y-1.5">
                            <Label htmlFor="additional-note" className="text-sm">
                                {t('protocolComponents.staffReviewAssist.additionalNote')}
                            </Label>
                            <Textarea
                                id="additional-note"
                                placeholder={t('protocolComponents.staffReviewAssist.additionalNotePlaceholder')}
                                value={additionalNote}
                                onChange={(e) => setAdditionalNote(e.target.value)}
                                rows={3}
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button
                            variant="outline"
                            onClick={() => setShowDialog(false)}
                            disabled={batchReturnMutation.isPending}
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            variant="destructive"
                            onClick={handleConfirmReturn}
                            disabled={batchReturnMutation.isPending}
                        >
                            {batchReturnMutation.isPending ? (
                                <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                            ) : (
                                <Send className="mr-1.5 h-4 w-4" />
                            )}
                            {t('protocolComponents.staffReviewAssist.confirmReturn')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
