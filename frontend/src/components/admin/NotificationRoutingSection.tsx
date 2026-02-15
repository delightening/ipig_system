/**
 * 通知路由管理元件
 * 允許管理員設定「哪些事件」由「哪些角色」透過「哪種管道」接收通知
 */
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import {
    Route,
    Plus,
    Trash2,
    Loader2,
    AlertCircle,
    ToggleLeft,
    ToggleRight,
} from 'lucide-react'
import { notificationRoutingApi, eventTypeNames, channelNames } from '@/lib/api'
import type { NotificationRouting, CreateNotificationRoutingRequest, UpdateNotificationRoutingRequest } from '@/types/notification'

export function NotificationRoutingSection() {
    const queryClient = useQueryClient()
    const [showAddForm, setShowAddForm] = useState(false)
    const [newRule, setNewRule] = useState<CreateNotificationRoutingRequest>({
        event_type: '',
        role_code: '',
        channel: 'both',
    })

    // 取得路由規則列表
    const { data: rules, isLoading, error } = useQuery({
        queryKey: ['notification-routing'],
        queryFn: async () => {
            const res = await notificationRoutingApi.list()
            return res.data
        },
    })

    // 新增規則
    const createMutation = useMutation({
        mutationFn: (data: CreateNotificationRoutingRequest) =>
            notificationRoutingApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            setShowAddForm(false)
            setNewRule({ event_type: '', role_code: '', channel: 'both' })
            toast({ title: '成功', description: '已新增通知路由規則' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '新增失敗',
                variant: 'destructive',
            })
        },
    })

    // 更新規則
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateNotificationRoutingRequest }) =>
            notificationRoutingApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            toast({ title: '成功', description: '規則已更新' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '更新失敗',
                variant: 'destructive',
            })
        },
    })

    // 刪除規則
    const deleteMutation = useMutation({
        mutationFn: (id: string) => notificationRoutingApi.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            toast({ title: '成功', description: '規則已刪除' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '刪除失敗',
                variant: 'destructive',
            })
        },
    })

    // 依事件類型分組
    const groupedRules = (rules || []).reduce<Record<string, NotificationRouting[]>>((acc, rule) => {
        if (!acc[rule.event_type]) acc[rule.event_type] = []
        acc[rule.event_type].push(rule)
        return acc
    }, {})

    const handleToggleActive = (rule: NotificationRouting) => {
        updateMutation.mutate({
            id: rule.id,
            data: { is_active: !rule.is_active },
        })
    }

    const handleChannelChange = (rule: NotificationRouting, channel: string) => {
        updateMutation.mutate({
            id: rule.id,
            data: { channel },
        })
    }

    const handleCreate = () => {
        if (!newRule.event_type || !newRule.role_code) {
            toast({
                title: '提示',
                description: '請填寫事件類型和角色代碼',
                variant: 'destructive',
            })
            return
        }
        createMutation.mutate(newRule)
    }

    return (
        <div className="border-t pt-6">
            <div className="flex items-center justify-between mb-4">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">通知路由管理</h2>
                    <p className="text-muted-foreground">設定各類事件的通知收件角色與管道</p>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAddForm(!showAddForm)}
                >
                    <Plus className="mr-1 h-4 w-4" />
                    新增規則
                </Button>
            </div>

            {/* 新增表單 */}
            {showAddForm && (
                <Card className="mb-6 border-blue-200 bg-blue-50/30">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-base">新增路由規則</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="grid gap-4 sm:grid-cols-4">
                            <div className="space-y-1.5">
                                <Label>事件類型</Label>
                                <Input
                                    value={newRule.event_type}
                                    onChange={(e) => setNewRule({ ...newRule, event_type: e.target.value })}
                                    placeholder="例: leave_submitted"
                                />
                            </div>
                            <div className="space-y-1.5">
                                <Label>角色代碼</Label>
                                <Input
                                    value={newRule.role_code}
                                    onChange={(e) => setNewRule({ ...newRule, role_code: e.target.value })}
                                    placeholder="例: ADMIN_STAFF"
                                />
                            </div>
                            <div className="space-y-1.5">
                                <Label>通道</Label>
                                <Select
                                    value={newRule.channel || 'both'}
                                    onValueChange={(v) => setNewRule({ ...newRule, channel: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="in_app">站內通知</SelectItem>
                                        <SelectItem value="email">Email</SelectItem>
                                        <SelectItem value="both">兩者</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="flex items-end gap-2">
                                <Button
                                    size="sm"
                                    onClick={handleCreate}
                                    disabled={createMutation.isPending}
                                >
                                    {createMutation.isPending ? (
                                        <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                                    ) : (
                                        <Plus className="mr-1 h-4 w-4" />
                                    )}
                                    新增
                                </Button>
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => setShowAddForm(false)}
                                >
                                    取消
                                </Button>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* 載入中 */}
            {isLoading && (
                <div className="flex items-center justify-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            )}

            {/* 錯誤 */}
            {error && (
                <Card className="border-red-200 bg-red-50">
                    <CardContent className="flex items-center gap-3 py-6">
                        <AlertCircle className="h-5 w-5 text-red-500" />
                        <span className="text-red-700">無法載入通知路由設定</span>
                    </CardContent>
                </Card>
            )}

            {/* 規則列表（按事件類型分組） */}
            {!isLoading && !error && (
                <div className="space-y-4">
                    {Object.keys(groupedRules).length === 0 ? (
                        <Card>
                            <CardContent className="py-8 text-center text-muted-foreground">
                                尚未設定任何通知路由規則
                            </CardContent>
                        </Card>
                    ) : (
                        Object.entries(groupedRules).map(([eventType, eventRules]) => (
                            <Card key={eventType}>
                                <CardHeader className="pb-3">
                                    <CardTitle className="flex items-center gap-2 text-base">
                                        <Route className="h-4 w-4" />
                                        {eventTypeNames[eventType] || eventType}
                                    </CardTitle>
                                    <CardDescription className="text-xs font-mono">
                                        {eventType}
                                    </CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <div className="space-y-2">
                                        {eventRules.map((rule) => (
                                            <div
                                                key={rule.id}
                                                className={`flex items-center justify-between rounded-lg border px-4 py-2.5 transition-colors ${rule.is_active
                                                        ? 'bg-white'
                                                        : 'bg-muted/50 opacity-60'
                                                    }`}
                                            >
                                                <div className="flex items-center gap-3">
                                                    {/* 角色 badge */}
                                                    <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10">
                                                        {rule.role_code}
                                                    </span>
                                                    {/* 通道選擇 */}
                                                    <Select
                                                        value={rule.channel}
                                                        onValueChange={(v) => handleChannelChange(rule, v)}
                                                    >
                                                        <SelectTrigger className="w-[120px] h-8 text-xs">
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="in_app">站內通知</SelectItem>
                                                            <SelectItem value="email">Email</SelectItem>
                                                            <SelectItem value="both">兩者</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                </div>

                                                <div className="flex items-center gap-2">
                                                    {/* 啟/停用 */}
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => handleToggleActive(rule)}
                                                        className="h-8 px-2"
                                                        title={rule.is_active ? '停用' : '啟用'}
                                                    >
                                                        {rule.is_active ? (
                                                            <ToggleRight className="h-5 w-5 text-green-600" />
                                                        ) : (
                                                            <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                                                        )}
                                                    </Button>
                                                    {/* 刪除 */}
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        className="h-8 px-2 text-red-500 hover:text-red-700 hover:bg-red-50"
                                                        onClick={() => {
                                                            if (confirm('確定要刪除此規則？')) {
                                                                deleteMutation.mutate(rule.id)
                                                            }
                                                        }}
                                                    >
                                                        <Trash2 className="h-4 w-4" />
                                                    </Button>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </CardContent>
                            </Card>
                        ))
                    )}
                </div>
            )}
        </div>
    )
}
