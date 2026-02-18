/**
 * 通知路由管理元件（進階版）
 * 使用下拉選單取代文字輸入，並以分類卡片分組顯示規則
 */
import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
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
    Shield,
    Stethoscope,
    Package,
    Users,
    FileText,
} from 'lucide-react'
import { notificationRoutingApi, eventTypeNames, channelNames } from '@/lib/api'
import type {
    NotificationRouting,
    CreateNotificationRoutingRequest,
    UpdateNotificationRoutingRequest,
    EventTypeCategory,
    RoleInfo,
} from '@/types/notification'

/** 分類圖示對照 */
const categoryIcons: Record<string, React.ReactNode> = {
    'AUP 計畫審查': <Shield className="h-4 w-4" />,
    '修正案': <FileText className="h-4 w-4" />,
    '動物健康': <Stethoscope className="h-4 w-4" />,
    'ERP 進銷存': <Package className="h-4 w-4" />,
    'HR 人事': <Users className="h-4 w-4" />,
}

/** 將事件類型依分類歸類 */
function categorizeRules(
    rules: NotificationRouting[],
    categories: EventTypeCategory[]
): { category: string; eventType: string; eventName: string; rules: NotificationRouting[] }[] {
    const result: { category: string; eventType: string; eventName: string; rules: NotificationRouting[] }[] = []

    // 建立 eventType → category 對照
    const eventCategoryMap = new Map<string, { category: string; name: string }>()
    for (const cat of categories) {
        for (const et of cat.event_types) {
            eventCategoryMap.set(et.code, { category: cat.category, name: et.name })
        }
    }

    // 按事件類型分組
    const grouped = new Map<string, NotificationRouting[]>()
    for (const rule of rules) {
        if (!grouped.has(rule.event_type)) grouped.set(rule.event_type, [])
        grouped.get(rule.event_type)!.push(rule)
    }

    // 按分類排序
    const categoryOrder = categories.map(c => c.category)
    const sortedEntries = [...grouped.entries()].sort((a, b) => {
        const catA = eventCategoryMap.get(a[0])?.category || '其他'
        const catB = eventCategoryMap.get(b[0])?.category || '其他'
        const idxA = categoryOrder.indexOf(catA)
        const idxB = categoryOrder.indexOf(catB)
        return (idxA === -1 ? 99 : idxA) - (idxB === -1 ? 99 : idxB)
    })

    for (const [eventType, eventRules] of sortedEntries) {
        const info = eventCategoryMap.get(eventType)
        result.push({
            category: info?.category || '其他',
            eventType,
            eventName: info?.name || eventTypeNames[eventType] || eventType,
            rules: eventRules,
        })
    }

    return result
}

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

    // 取得事件類型分類
    const { data: eventCategories } = useQuery({
        queryKey: ['notification-routing-event-types'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getEventTypes()
            return res.data
        },
    })

    // 取得角色列表
    const { data: availableRoles } = useQuery({
        queryKey: ['notification-routing-roles'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getRoles()
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

    // 按分類整理規則
    const categorizedRules = useMemo(
        () => categorizeRules(rules || [], eventCategories || []),
        [rules, eventCategories]
    )

    // 按分類分組顯示
    const groupedByCategory = useMemo(() => {
        const map = new Map<string, typeof categorizedRules>()
        for (const item of categorizedRules) {
            if (!map.has(item.category)) map.set(item.category, [])
            map.get(item.category)!.push(item)
        }
        return map
    }, [categorizedRules])

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
                description: '請選擇事件類型和角色',
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

            {/* 新增表單（下拉選單版） */}
            {showAddForm && (
                <Card className="mb-6 border-blue-200 bg-blue-50/30">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-base">新增路由規則</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="grid gap-4 sm:grid-cols-4">
                            {/* 事件類型下拉選單 */}
                            <div className="space-y-1.5">
                                <Label>事件類型</Label>
                                <Select
                                    value={newRule.event_type}
                                    onValueChange={(v) => setNewRule({ ...newRule, event_type: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇事件類型" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {(eventCategories || []).map((cat) => (
                                            <SelectGroup key={cat.category}>
                                                <SelectLabel className="flex items-center gap-1.5">
                                                    {categoryIcons[cat.category] || <Route className="h-3 w-3" />}
                                                    {cat.category}
                                                </SelectLabel>
                                                {cat.event_types.map((et) => (
                                                    <SelectItem key={et.code} value={et.code}>
                                                        {et.name}
                                                    </SelectItem>
                                                ))}
                                            </SelectGroup>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>

                            {/* 角色下拉選單 */}
                            <div className="space-y-1.5">
                                <Label>角色</Label>
                                <Select
                                    value={newRule.role_code}
                                    onValueChange={(v) => setNewRule({ ...newRule, role_code: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇角色" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {(availableRoles || []).map((role) => (
                                            <SelectItem key={role.code} value={role.code}>
                                                {role.name} ({role.code})
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>

                            {/* 通道選擇 */}
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

                            {/* 操作按鈕 */}
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

                        {/* 描述欄位 */}
                        <div className="mt-3">
                            <Label>說明（選填）</Label>
                            <Input
                                value={newRule.description || ''}
                                onChange={(e) => setNewRule({ ...newRule, description: e.target.value })}
                                placeholder="管理者備註"
                                className="mt-1"
                            />
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

            {/* 規則列表（按分類卡片分組） */}
            {!isLoading && !error && (
                <div className="space-y-6">
                    {groupedByCategory.size === 0 ? (
                        <Card>
                            <CardContent className="py-8 text-center text-muted-foreground">
                                尚未設定任何通知路由規則
                            </CardContent>
                        </Card>
                    ) : (
                        [...groupedByCategory.entries()].map(([category, items]) => (
                            <div key={category}>
                                {/* 分類標題 */}
                                <div className="flex items-center gap-2 mb-3">
                                    <span className="text-muted-foreground">
                                        {categoryIcons[category] || <Route className="h-4 w-4" />}
                                    </span>
                                    <h3 className="text-lg font-semibold">{category}</h3>
                                    <span className="text-xs text-muted-foreground">
                                        ({items.reduce((sum, i) => sum + i.rules.length, 0)} 條規則)
                                    </span>
                                </div>

                                {/* 事件卡片 */}
                                <div className="space-y-3 ml-1">
                                    {items.map((item) => (
                                        <Card key={item.eventType}>
                                            <CardHeader className="pb-3">
                                                <CardTitle className="flex items-center gap-2 text-base">
                                                    <Route className="h-4 w-4" />
                                                    {item.eventName}
                                                </CardTitle>
                                                <CardDescription className="text-xs font-mono">
                                                    {item.eventType}
                                                </CardDescription>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="space-y-2">
                                                    {item.rules.map((rule) => (
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
                                                                {/* 說明 */}
                                                                {rule.description && (
                                                                    <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                                                                        {rule.description}
                                                                    </span>
                                                                )}
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
                                    ))}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            )}
        </div>
    )
}
