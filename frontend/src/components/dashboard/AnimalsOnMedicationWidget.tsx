import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Pill, Loader2, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'
import api, { PigListItem } from '@/lib/api'

export function AnimalsOnMedicationWidget() {
    const navigate = useNavigate()

    const { data, isLoading, error } = useQuery({
        queryKey: ['animals-on-medication'],
        queryFn: async () => {
            // 取得正在用藥的動物列表
            const res = await api.get<PigListItem[]>('/pigs?is_on_medication=true&per_page=10')
            return res.data
        },
    })

    if (isLoading) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Pill className="h-4 w-4 text-red-500" />
                        正在用藥動物
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
                        <Pill className="h-4 w-4 text-red-500" />
                        正在用藥動物
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card>
            <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Pill className="h-4 w-4 text-red-500" />
                        正在用藥動物
                    </CardTitle>
                    {data && data.length > 0 && (
                        <Badge variant="destructive" className="text-xs">
                            {data.length}
                        </Badge>
                    )}
                </div>
                <CardDescription>需要持續照護的動物</CardDescription>
            </CardHeader>
            <CardContent>
                {data && data.length > 0 ? (
                    <div className="space-y-2">
                        {data.slice(0, 5).map((pig) => (
                            <div
                                key={pig.id}
                                className="flex items-center justify-between p-2 border rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
                                onClick={() => navigate(`/pigs/${pig.id}`)}
                            >
                                <div className="flex items-center gap-3">
                                    <div className="w-8 h-8 rounded-full bg-red-100 flex items-center justify-center">
                                        <Pill className="h-4 w-4 text-red-600" />
                                    </div>
                                    <div>
                                        <p className="text-sm font-medium">{pig.ear_tag}</p>
                                        <p className="text-xs text-muted-foreground">{pig.pen_location || '未指定欄位'}</p>
                                    </div>
                                </div>
                                <div className="flex items-center gap-2">
                                    {pig.last_medication_date && (
                                        <span className="text-xs text-muted-foreground">
                                            {new Date(pig.last_medication_date).toLocaleDateString('zh-TW')}
                                        </span>
                                    )}
                                    <ExternalLink className="h-4 w-4 text-muted-foreground" />
                                </div>
                            </div>
                        ))}
                        {data.length > 5 && (
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => navigate('/pigs?is_on_medication=true')}
                                className="w-full text-xs"
                            >
                                查看全部 {data.length} 隻
                            </Button>
                        )}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
                        <Pill className="h-8 w-8 mb-2" />
                        <p className="text-sm">目前沒有正在用藥的動物</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
