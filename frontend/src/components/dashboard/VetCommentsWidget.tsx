import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { MessageSquare, Loader2, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'
import api from '@/lib/api'

interface VetComment {
    id: string
    pig_id: number
    ear_tag: string
    pen_location?: string
    content: string
    created_at: string
    created_by_name: string
}

export function VetCommentsWidget() {
    const navigate = useNavigate()

    const { data, isLoading, error } = useQuery({
        queryKey: ['recent-vet-comments'],
        queryFn: async () => {
            // 取得最近的獸醫師評論
            const res = await api.get<{ data: VetComment[] }>('/pigs/vet-comments?per_page=5')
            return res.data.data
        },
    })

    if (isLoading) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <MessageSquare className="h-4 w-4 text-teal-500" />
                        獸醫師建議
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="flex items-center justify-center py-4">
                        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    </div>
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <MessageSquare className="h-4 w-4 text-teal-500" />
                        獸醫師建議
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    // 格式化時間
    const formatTime = (dateStr: string) => {
        const date = new Date(dateStr)
        const now = new Date()
        const diff = now.getTime() - date.getTime()
        const hours = Math.floor(diff / 3600000)
        const days = Math.floor(diff / 86400000)

        if (hours < 1) return '剛剛'
        if (hours < 24) return `${hours} 小時前`
        if (days < 7) return `${days} 天前`
        return date.toLocaleDateString('zh-TW')
    }

    return (
        <Card>
            <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <MessageSquare className="h-4 w-4 text-teal-500" />
                        獸醫師建議
                    </CardTitle>
                </div>
                <CardDescription>最近的獸醫師建議</CardDescription>
            </CardHeader>
            <CardContent>
                {data && data.length > 0 ? (
                    <div className="space-y-3">
                        {data.map((comment) => (
                            <div
                                key={comment.id}
                                className="p-3 border rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
                                onClick={() => navigate(`/pigs/${comment.pig_id}`)}
                            >
                                <div className="flex items-start justify-between mb-1">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-medium">{comment.ear_tag}</span>
                                        {comment.pen_location && (
                                            <span className="text-xs text-muted-foreground">({comment.pen_location})</span>
                                        )}
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs text-muted-foreground">
                                            {formatTime(comment.created_at)}
                                        </span>
                                        <ExternalLink className="h-3 w-3 text-muted-foreground" />
                                    </div>
                                </div>
                                <p className="text-sm text-muted-foreground line-clamp-2">
                                    {comment.content}
                                </p>
                                <p className="text-xs text-muted-foreground mt-1">
                                    — {comment.created_by_name}
                                </p>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
                        <MessageSquare className="h-8 w-8 mb-2" />
                        <p className="text-sm">目前沒有獸醫師建議</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
