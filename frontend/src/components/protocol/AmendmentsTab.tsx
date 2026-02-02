import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    AmendmentListItem,
    AmendmentStatus,
    amendmentStatusNames,
    amendmentStatusColors,
    amendmentTypeNames,
    CreateAmendmentRequest,
    AMENDMENT_CHANGE_ITEM_OPTIONS,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import { Loader2, Plus, FileEdit, Send, Eye } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { Link } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'

interface AmendmentsTabProps {
    protocolId: string
    protocolStatus?: string
}

export function AmendmentsTab({ protocolId, protocolStatus }: AmendmentsTabProps) {
    const queryClient = useQueryClient()
    const { user } = useAuthStore()

    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const [amendmentTitle, setAmendmentTitle] = useState('')
    const [amendmentDescription, setAmendmentDescription] = useState('')
    const [selectedChangeItems, setSelectedChangeItems] = useState<string[]>([])

    // 取得變更申請列表
    const { data: amendments, isLoading } = useQuery({
        queryKey: ['protocol-amendments', protocolId],
        queryFn: async () => {
            const response = await api.get<AmendmentListItem[]>(`/protocols/${protocolId}/amendments`)
            return response.data
        },
        enabled: !!protocolId,
    })

    // 建立變更申請
    const createAmendmentMutation = useMutation({
        mutationFn: async (req: CreateAmendmentRequest) => {
            return api.post('/amendments', req)
        },
        onSuccess: () => {
            toast({ title: '成功', description: '變更申請已建立' })
            queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
            resetForm()
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '建立變更申請失敗',
                variant: 'destructive',
            })
        },
    })

    // 提交變更申請
    const submitAmendmentMutation = useMutation({
        mutationFn: async (amendmentId: string) => {
            return api.post(`/amendments/${amendmentId}/submit`)
        },
        onSuccess: () => {
            toast({ title: '成功', description: '變更申請已提交' })
            queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '提交失敗',
                variant: 'destructive',
            })
        },
    })

    const resetForm = () => {
        setShowCreateDialog(false)
        setAmendmentTitle('')
        setAmendmentDescription('')
        setSelectedChangeItems([])
    }

    const handleCreateAmendment = () => {
        if (!amendmentTitle.trim()) {
            toast({ title: '錯誤', description: '請輸入變更標題', variant: 'destructive' })
            return
        }
        if (selectedChangeItems.length === 0) {
            toast({ title: '錯誤', description: '請至少選擇一個變更項目', variant: 'destructive' })
            return
        }

        createAmendmentMutation.mutate({
            protocol_id: protocolId,
            title: amendmentTitle.trim(),
            description: amendmentDescription.trim() || undefined,
            change_items: selectedChangeItems,
        })
    }

    const handleToggleChangeItem = (value: string) => {
        setSelectedChangeItems(prev =>
            prev.includes(value)
                ? prev.filter(item => item !== value)
                : [...prev, value]
        )
    }

    // 檢查是否可以建立變更申請（計畫已核准）
    const canCreateAmendment =
        protocolStatus === 'APPROVED' ||
        protocolStatus === 'APPROVED_WITH_CONDITIONS'

    // 檢查是否為 PI 或相關角色
    const isPIorCoEditor = user?.roles?.some(r =>
        ['PI', 'EXPERIMENT_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
    )

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>變更申請</CardTitle>
                    <CardDescription>計畫書變更申請記錄</CardDescription>
                </div>
                {canCreateAmendment && isPIorCoEditor && (
                    <Button onClick={() => setShowCreateDialog(true)}>
                        <Plus className="mr-2 h-4 w-4" />
                        新增變更申請
                    </Button>
                )}
            </CardHeader>
            <CardContent>
                {isLoading ? (
                    <div className="flex items-center justify-center py-8">
                        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                ) : amendments && amendments.length > 0 ? (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>變更編號</TableHead>
                                <TableHead>標題</TableHead>
                                <TableHead>變更類型</TableHead>
                                <TableHead>狀態</TableHead>
                                <TableHead>提交時間</TableHead>
                                <TableHead>操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {amendments.map((amendment) => (
                                <TableRow key={amendment.id}>
                                    <TableCell className="font-medium">{amendment.amendment_no}</TableCell>
                                    <TableCell>{amendment.title}</TableCell>
                                    <TableCell>
                                        <Badge variant="outline">
                                            {amendmentTypeNames[amendment.amendment_type]}
                                        </Badge>
                                    </TableCell>
                                    <TableCell>
                                        <Badge variant={amendmentStatusColors[amendment.status]}>
                                            {amendmentStatusNames[amendment.status]}
                                        </Badge>
                                    </TableCell>
                                    <TableCell>
                                        {amendment.submitted_at
                                            ? formatDateTime(amendment.submitted_at)
                                            : '-'
                                        }
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-2">
                                            {amendment.status === 'DRAFT' && isPIorCoEditor && (
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    onClick={() => submitAmendmentMutation.mutate(amendment.id)}
                                                    disabled={submitAmendmentMutation.isPending}
                                                >
                                                    {submitAmendmentMutation.isPending ? (
                                                        <Loader2 className="h-4 w-4 animate-spin" />
                                                    ) : (
                                                        <>
                                                            <Send className="mr-1 h-4 w-4" />
                                                            提交
                                                        </>
                                                    )}
                                                </Button>
                                            )}
                                            <Button variant="ghost" size="sm" asChild>
                                                <Link to={`/protocols/${protocolId}?tab=amendments&amendmentId=${amendment.id}`}>
                                                    <Eye className="mr-1 h-4 w-4" />
                                                    查看
                                                </Link>
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                ) : (
                    <div className="text-center py-8 text-muted-foreground">
                        <FileEdit className="h-12 w-12 mx-auto mb-2" />
                        <p>尚無變更申請</p>
                        {canCreateAmendment && isPIorCoEditor && (
                            <p className="text-sm mt-2">點擊「新增變更申請」按鈕來建立您的第一個變更申請</p>
                        )}
                        {!canCreateAmendment && (
                            <p className="text-sm mt-2">計畫書需先核准後才能提出變更申請</p>
                        )}
                    </div>
                )}
            </CardContent>

            {/* 建立變更申請對話框 */}
            <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle>建立變更申請</DialogTitle>
                        <DialogDescription>
                            填寫變更申請資訊，選擇本次變更的項目
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label htmlFor="amendment-title">變更標題 *</Label>
                            <Input
                                id="amendment-title"
                                value={amendmentTitle}
                                onChange={(e) => setAmendmentTitle(e.target.value)}
                                placeholder="例如：第二年動物數量調整"
                            />
                        </div>

                        <div className="space-y-2">
                            <Label>變更項目（可多選）*</Label>
                            <div className="grid grid-cols-2 gap-2 mt-2">
                                {AMENDMENT_CHANGE_ITEM_OPTIONS.map((option) => (
                                    <div key={option.value} className="flex items-center space-x-2">
                                        <Checkbox
                                            id={`change-item-${option.value}`}
                                            checked={selectedChangeItems.includes(option.value)}
                                            onCheckedChange={() => handleToggleChangeItem(option.value)}
                                        />
                                        <label
                                            htmlFor={`change-item-${option.value}`}
                                            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                                        >
                                            {option.label}
                                        </label>
                                    </div>
                                ))}
                            </div>
                            {selectedChangeItems.length > 0 && (
                                <p className="text-xs text-muted-foreground mt-1">
                                    已選擇 {selectedChangeItems.length} 項
                                </p>
                            )}
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="amendment-description">變更說明</Label>
                            <Textarea
                                id="amendment-description"
                                value={amendmentDescription}
                                onChange={(e) => setAmendmentDescription(e.target.value)}
                                placeholder="詳細說明本次變更的原因和內容..."
                                rows={4}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={resetForm}>
                            取消
                        </Button>
                        <Button
                            onClick={handleCreateAmendment}
                            disabled={createAmendmentMutation.isPending}
                        >
                            {createAmendmentMutation.isPending && (
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            )}
                            建立變更申請
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </Card>
    )
}
