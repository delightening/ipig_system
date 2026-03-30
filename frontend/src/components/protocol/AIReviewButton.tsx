/**
 * R20-6: AI 預審觸發按鈕
 *
 * 顯示剩餘次數，觸發 AI 預審。
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Bot, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { aiReviewApi } from '@/lib/api'
import { getApiErrorMessage } from '@/lib/validation'

interface AIReviewButtonProps {
    protocolId: string
    onResult?: () => void
}

export function AIReviewButton({ protocolId, onResult }: AIReviewButtonProps) {
    const queryClient = useQueryClient()

    const { data: remainingData } = useQuery({
        queryKey: ['ai-review-remaining'],
        queryFn: aiReviewApi.getRemainingCount,
    })

    const mutation = useMutation({
        mutationFn: () => aiReviewApi.requestAiReview(protocolId),
        onSuccess: () => {
            toast({
                title: 'AI 預審完成',
                description: '請查看預審報告。',
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
                title: 'AI 預審失敗',
                description: getApiErrorMessage(error, 'AI 預審請求失敗'),
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
            AI 預審
            {remaining < 10 && (
                <span className="ml-1 text-xs text-muted-foreground">
                    ({remaining})
                </span>
            )}
        </Button>
    )
}
