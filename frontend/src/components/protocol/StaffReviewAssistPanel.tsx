/**
 * R20-7: 執行秘書 AI 標註面板
 *
 * 三類標註：needs_attention / concern / suggestion
 * 僅 IACUC_STAFF / IACUC_CHAIR 可見。
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { AlertCircle, AlertTriangle, Bot, Info, Loader2, RefreshCw } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { aiReviewApi } from '@/lib/api'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import type { StaffAiResult, StaffReviewFlag } from '@/types/aiReview'

interface StaffReviewAssistPanelProps {
    protocolId: string
}

const FLAG_CONFIG: Record<
    string,
    { icon: typeof AlertCircle; label: string; colorClass: string }
> = {
    needs_attention: {
        icon: AlertCircle,
        label: '需要注意',
        colorClass: 'text-status-error-text',
    },
    concern: {
        icon: AlertTriangle,
        label: '留意事項',
        colorClass: 'text-status-warning-text',
    },
    suggestion: {
        icon: Info,
        label: '審查建議',
        colorClass: 'text-primary',
    },
}

export function StaffReviewAssistPanel({
    protocolId,
}: StaffReviewAssistPanelProps) {
    const queryClient = useQueryClient()

    const { data: review, isLoading } = useQuery({
        queryKey: ['staff-review', protocolId],
        queryFn: () => aiReviewApi.getLatestStaffReview(protocolId),
    })

    const refreshMutation = useMutation({
        mutationFn: () => aiReviewApi.requestStaffReview(protocolId),
        onSuccess: () => {
            toast({ title: '重新分析完成' })
            queryClient.invalidateQueries({
                queryKey: ['staff-review', protocolId],
            })
        },
        onError: (error: unknown) => {
            toast({
                title: 'AI 分析失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    if (isLoading) {
        return (
            <Card>
                <CardContent className="flex items-center justify-center py-6">
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    <span className="ml-2 text-sm text-muted-foreground">
                        載入 AI 標註...
                    </span>
                </CardContent>
            </Card>
        )
    }

    // 解析 AI 結果
    const aiResult = review?.ai_result as StaffAiResult | undefined
    const ruleResult = review?.rule_result

    // 合併規則結果與 AI flags
    const flags: StaffReviewFlag[] = aiResult?.flags ?? []

    // 將規則 errors 轉為 needs_attention flags
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
        needs_attention: allFlags.filter(
            (f) => f.flag_type === 'needs_attention'
        ),
        concern: allFlags.filter((f) => f.flag_type === 'concern'),
        suggestion: allFlags.filter((f) => f.flag_type === 'suggestion'),
    }

    const hasAnyFlags = allFlags.length > 0

    return (
        <Card className="border-2 border-border">
            <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-lg flex items-center gap-2">
                        <Bot className="h-5 w-5" />
                        Pre-Review 審查輔助
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
                            重新分析
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
                        尚無 AI 標註結果。點擊「重新分析」開始。
                    </p>
                )}

                {!hasAnyFlags && review && (
                    <p className="text-sm text-muted-foreground text-center py-4">
                        未發現需特別注意的事項。
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
                                    {config.label}（{items.length} 項）
                                </h4>
                                <ul className="space-y-2 ml-5">
                                    {items.map((flag, idx) => (
                                        <li key={idx} className="text-sm">
                                            <div
                                                className={`font-medium ${config.colorClass}`}
                                            >
                                                [{flag.section}]{' '}
                                                {flag.message}
                                            </div>
                                            <div className="text-muted-foreground mt-0.5">
                                                {flag.suggestion}
                                            </div>
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )
                    }
                )}

                {/* 免責聲明 */}
                <p className="text-xs text-muted-foreground pt-2 border-t italic">
                    AI 標註僅供參考，請依專業判斷審查。
                </p>
            </CardContent>
        </Card>
    )
}
