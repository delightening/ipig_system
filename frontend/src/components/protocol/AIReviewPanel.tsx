/**
 * R20-6: AI 預審結果面板
 *
 * 顯示 AI 預審的完整結果（Level 1 + Level 2）。
 */
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import {
    AlertCircle,
    AlertTriangle,
    Bot,
    CheckCircle2,
    ChevronDown,
    ChevronRight,
    Clock,
} from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { aiReviewApi } from '@/lib/api'
import { formatDate } from '@/lib/utils'
import type { AiReviewAiResult } from '@/types/aiReview'

interface AIReviewPanelProps {
    protocolId: string
}

export function AIReviewPanel({ protocolId }: AIReviewPanelProps) {
    const [showPassed, setShowPassed] = useState(false)

    const { data: review } = useQuery({
        queryKey: ['ai-review', protocolId],
        queryFn: () => aiReviewApi.getLatestAiReview(protocolId),
    })

    if (!review) return null

    const aiResult = review.ai_result as AiReviewAiResult | undefined

    return (
        <Card className="border-2 border-border">
            <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-lg flex items-center gap-2">
                        <Bot className="h-5 w-5" />
                        AI 預審報告
                    </CardTitle>
                    <div className="flex items-center gap-2">
                        {review.score != null && (
                            <ScoreBadge score={review.score} />
                        )}
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {formatDate(review.created_at)}
                        </span>
                    </div>
                </div>
                {aiResult?.summary && (
                    <p className="text-sm text-muted-foreground mt-1">
                        {aiResult.summary}
                    </p>
                )}
            </CardHeader>
            <CardContent className="space-y-4">
                {/* Rule errors */}
                <IssueSection
                    title="必須修正"
                    count={review.total_errors}
                    icon={<AlertCircle className="h-4 w-4" />}
                    colorClass="text-status-error-text"
                    issues={review.rule_result?.errors ?? []}
                />

                {/* AI errors */}
                {aiResult?.issues
                    ?.filter((i) => i.severity === 'error')
                    .map((issue, idx) => (
                        <div key={`ai-err-${idx}`} className="text-sm ml-5">
                            <div className="font-medium text-status-error-text">
                                [{issue.category}] {issue.message}
                            </div>
                            <div className="text-muted-foreground mt-0.5">
                                {issue.suggestion}
                            </div>
                        </div>
                    ))}

                {/* Rule warnings */}
                <IssueSection
                    title="建議改善"
                    count={review.total_warnings}
                    icon={<AlertTriangle className="h-4 w-4" />}
                    colorClass="text-status-warning-text"
                    issues={review.rule_result?.warnings ?? []}
                />

                {/* AI warnings */}
                {aiResult?.issues
                    ?.filter((i) => i.severity === 'warning')
                    .map((issue, idx) => (
                        <div key={`ai-warn-${idx}`} className="text-sm ml-5">
                            <div className="font-medium text-status-warning-text">
                                [{issue.category}] {issue.message}
                            </div>
                            <div className="text-muted-foreground mt-0.5">
                                {issue.suggestion}
                            </div>
                        </div>
                    ))}

                {/* Passed */}
                {(review.rule_result?.passed?.length ?? 0) > 0 && (
                    <div className="space-y-1">
                        <button
                            onClick={() => setShowPassed(!showPassed)}
                            className="text-sm font-semibold flex items-center gap-1.5 text-status-success-text hover:underline"
                        >
                            {showPassed ? (
                                <ChevronDown className="h-4 w-4" />
                            ) : (
                                <ChevronRight className="h-4 w-4" />
                            )}
                            <CheckCircle2 className="h-4 w-4" />
                            通過檢查（{review.rule_result?.passed?.length ?? 0} 項）
                        </button>
                        {showPassed && (
                            <div className="ml-5 text-sm text-muted-foreground">
                                {review.rule_result?.passed?.join(', ')}
                            </div>
                        )}
                    </div>
                )}

                {/* Meta info */}
                {review.ai_model && (
                    <div className="text-xs text-muted-foreground pt-2 border-t flex gap-4">
                        <span>Model: {review.ai_model}</span>
                        {review.duration_ms && (
                            <span>Duration: {review.duration_ms}ms</span>
                        )}
                    </div>
                )}
            </CardContent>
        </Card>
    )
}

function IssueSection({
    title,
    count,
    icon,
    colorClass,
    issues,
}: {
    title: string
    count: number
    icon: React.ReactNode
    colorClass: string
    issues: Array<{
        code: string
        category: string
        message: string
        suggestion: string
    }>
}) {
    if (count === 0 && issues.length === 0) return null

    return (
        <div className="space-y-2">
            <h4 className={`text-sm font-semibold flex items-center gap-1.5 ${colorClass}`}>
                {icon}
                {title}（{count} 項）
            </h4>
            <ul className="space-y-2 ml-5">
                {issues.map((issue) => (
                    <li key={issue.code} className="text-sm">
                        <div className={`font-medium ${colorClass}`}>
                            [{issue.category}] {issue.message}
                        </div>
                        <div className="text-muted-foreground mt-0.5">
                            {issue.suggestion}
                        </div>
                    </li>
                ))}
            </ul>
        </div>
    )
}

function ScoreBadge({ score }: { score: number }) {
    const variant = score >= 80 ? 'success' : score >= 60 ? 'warning' : 'destructive'
    return (
        <Badge variant={variant}>
            {score} 分
        </Badge>
    )
}
