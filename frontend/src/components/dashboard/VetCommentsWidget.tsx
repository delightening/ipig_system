import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { MessageSquare, Loader2, User, Calendar } from 'lucide-react'
import api from '@/lib/api'

interface VetComment {
    id: string
    animal_id: string
    ear_tag: string
    author_name: string
    content: string
    created_at: string
}

export function VetCommentsWidget() {
    const { t, i18n } = useTranslation()
    const navigate = useNavigate()

    const { data, isLoading, error } = useQuery({
        queryKey: ['recent-vet-comments'],
        queryFn: async () => {
            const res = await api.get<{ data: VetComment[] }>('/animals/vet-comments?per_page=5')
            return res.data.data
        },
        staleTime: 30_000,
    })

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <MessageSquare className="h-4 w-4 text-emerald-500" />
                        {t('dashboard.widgets.names.vet_comments')}
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
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <MessageSquare className="h-4 w-4 text-emerald-500" />
                        {t('dashboard.widgets.names.vet_comments')}
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card className="h-full flex flex-col">
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <MessageSquare className="h-4 w-4 text-emerald-500" />
                    {t('dashboard.widgets.names.vet_comments')}
                </CardTitle>
                <CardDescription>{t('dashboard.widgets.animals.commentDescription')}</CardDescription>
            </CardHeader>
            <CardContent>
                {data && data.length > 0 ? (
                    <div className="space-y-3">
                        {data.map((comment) => (
                            <div
                                key={comment.id}
                                className="p-3 border rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
                                onClick={() => navigate(`/animals/${comment.animal_id}`)}
                            >
                                <div className="flex items-start justify-between mb-2">
                                    <div className="flex items-center gap-2">
                                        <div className="w-6 h-6 rounded-full bg-emerald-100 flex items-center justify-center">
                                            <User className="h-3 w-3 text-emerald-600" />
                                        </div>
                                        <span className="text-xs font-semibold">{comment.author_name}</span>
                                    </div>
                                    <span className="text-[10px] text-muted-foreground flex items-center gap-1">
                                        <Calendar className="h-3 w-3" />
                                        {new Date(comment.created_at).toLocaleDateString(i18n.language)}
                                    </span>
                                </div>
                                <p className="text-xs text-muted-foreground line-clamp-2 mb-2 italic">
                                    "{comment.content}"
                                </p>
                                <div className="flex justify-end">
                                    <span className="text-[10px] bg-slate-100 px-2 py-0.5 rounded-full font-medium">
                                        {t('dashboard.widgets.animals.earTag')}: {comment.ear_tag}
                                    </span>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
                        <MessageSquare className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-sm">{t('dashboard.widgets.animals.noComments')}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
