/**
 * R20-6: AI 預審觸發按鈕
 *
 * 顯示剩餘次數，觸發 AI 預審。
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Bot, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { aiReviewApi } from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'

interface AIReviewButtonProps {
    protocolId: string
    onResult?: () => void
}

export function AIReviewButton({ protocolId, onResult }: AIReviewButtonProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const { data: remainingData } = useQuery({
        queryKey: ['ai-review-remaining'],
        queryFn: aiReviewApi.getRemainingCount,
    })

    const mutation = useMutation({
        mutationFn: () => aiReviewApi.requestAiReview(protocolId),
        onSuccess: () => {
            toast({
                title: t('protocolComponents.aiReview.button.successTitle'),
                description: t('protocolComponents.aiReview.button.successDescription'),
            })
            queryClient.invalidateQueries({
                queryKey: ['ai-review', protocolId],
            })
            queryClient.invalidateQueries({
                queryKey: ['ai-review-remaining'],
            })
            onResult?.()
        },
        onError: (error: unknown) => {
            toast({
                title: t('protocolComponents.aiReview.button.failedTitle'),
                description: getApiErrorMessage(error, t('protocolComponents.aiReview.button.requestFailed')),
                variant: 'destructive',
            })
        },
    })

    const remaining = remainingData?.remaining ?? 10

    return (
        <Button
            variant="outline"
            size="sm"
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending || remaining <= 0}
        >
            {mutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
                <Bot className="mr-2 h-4 w-4" />
            )}
            {t('protocolComponents.aiReview.button.label')}
            {remaining < 10 && (
                <span className="ml-1 text-xs text-muted-foreground">
                    ({remaining})
                </span>
            )}
        </Button>
    )
}
