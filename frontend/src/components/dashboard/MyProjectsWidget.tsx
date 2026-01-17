import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { FolderOpen, Loader2, FileSearch, PlayCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import api, { ProtocolListItem } from '@/lib/api'

// 狀態類別
const REVIEW_STATUSES = ['SUBMITTED', 'PRE_REVIEW', 'UNDER_REVIEW', 'PENDING_REVISION']
const ACTIVE_STATUSES = ['APPROVED', 'APPROVED_WITH_CONDITIONS']

export function MyProjectsWidget() {
    const navigate = useNavigate()

    const { data: projects, isLoading, error } = useQuery({
        queryKey: ['my-projects-widget'],
        queryFn: async () => {
            // 直接調用 protocols API（不使用 my-projects）
            const res = await api.get<ProtocolListItem[]>('/my-projects')
            return res.data
        },
    })

    if (isLoading) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <FolderOpen className="h-4 w-4 text-purple-500" />
                        我的計畫
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
                        <FolderOpen className="h-4 w-4 text-purple-500" />
                        我的計畫
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    // 計算各類計畫數量
    const totalCount = projects?.length || 0
    const reviewingProjects = projects?.filter(p => REVIEW_STATUSES.includes(p.status)) || []
    const activeProjects = projects?.filter(p => ACTIVE_STATUSES.includes(p.status)) || []

    return (
        <Card>
            <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <FolderOpen className="h-4 w-4 text-purple-500" />
                        我的計畫
                    </CardTitle>
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => navigate('/my-projects')}
                        className="text-xs"
                    >
                        查看全部
                    </Button>
                </div>
                <CardDescription>您參與或主持的 IACUC 計畫</CardDescription>
            </CardHeader>
            <CardContent>
                {/* 統計數字 */}
                <div className="grid grid-cols-3 gap-2 mb-4">
                    <div className="text-center p-2 bg-slate-50 rounded-lg">
                        <div className="text-2xl font-bold text-slate-700">{totalCount}</div>
                        <div className="text-xs text-muted-foreground">全部計畫</div>
                    </div>
                    <div className="text-center p-2 bg-yellow-50 rounded-lg">
                        <div className="text-2xl font-bold text-yellow-600">{reviewingProjects.length}</div>
                        <div className="text-xs text-muted-foreground">審查中</div>
                    </div>
                    <div className="text-center p-2 bg-green-50 rounded-lg">
                        <div className="text-2xl font-bold text-green-600">{activeProjects.length}</div>
                        <div className="text-xs text-muted-foreground">執行中</div>
                    </div>
                </div>

                {/* 審查中的計畫 */}
                {reviewingProjects.length > 0 && (
                    <div className="mb-3">
                        <div className="flex items-center gap-1 text-xs font-medium text-yellow-700 mb-1">
                            <FileSearch className="h-3 w-3" />
                            審查中
                        </div>
                        <div className="space-y-1">
                            {reviewingProjects.slice(0, 3).map((project) => (
                                <div
                                    key={project.id}
                                    className="text-xs p-2 bg-yellow-50/50 rounded border border-yellow-100 hover:bg-yellow-50 cursor-pointer transition-colors"
                                    onClick={() => navigate(`/protocols/${project.id}`)}
                                >
                                    <span className="font-medium">{project.title}</span>
                                    {project.iacuc_no && (
                                        <span className="text-muted-foreground ml-2">({project.iacuc_no})</span>
                                    )}
                                </div>
                            ))}
                            {reviewingProjects.length > 3 && (
                                <div className="text-xs text-muted-foreground text-center">
                                    +{reviewingProjects.length - 3} 更多...
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* 執行中的計畫 */}
                {activeProjects.length > 0 && (
                    <div>
                        <div className="flex items-center gap-1 text-xs font-medium text-green-700 mb-1">
                            <PlayCircle className="h-3 w-3" />
                            執行中
                        </div>
                        <div className="space-y-1">
                            {activeProjects.slice(0, 3).map((project) => (
                                <div
                                    key={project.id}
                                    className="text-xs p-2 bg-green-50/50 rounded border border-green-100 hover:bg-green-50 cursor-pointer transition-colors"
                                    onClick={() => navigate(`/protocols/${project.id}`)}
                                >
                                    <span className="font-medium">{project.title}</span>
                                    {project.iacuc_no && (
                                        <span className="text-muted-foreground ml-2">({project.iacuc_no})</span>
                                    )}
                                </div>
                            ))}
                            {activeProjects.length > 3 && (
                                <div className="text-xs text-muted-foreground text-center">
                                    +{activeProjects.length - 3} 更多...
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* 無計畫時顯示 */}
                {totalCount === 0 && (
                    <div className="flex flex-col items-center justify-center py-4 text-muted-foreground">
                        <FolderOpen className="h-8 w-8 mb-2" />
                        <p className="text-sm">尚無參與的計畫</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
