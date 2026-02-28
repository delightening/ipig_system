/**
 * 通知路由管理元件（改良版）
 * - 分類可收合/展開，減少視覺壓力
 * - Switch 取代 Toggle icon
 * - 角色顯示中文名稱
 * - ConfirmDialog 取代 window.confirm
 * - 整齊的 grid layout
 */
import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
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
import { getApiErrorMessage } from '@/lib/validation'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import {
    Route,
    Plus,
    Trash2,
    Loader2,
    AlertCircle,
    Shield,
    Stethoscope,
    Package,
    Users,
    FileText,
    ChevronDown,
    ChevronRight,
} from 'lucide-react'
import { notificationRoutingApi } from '@/lib/api'
import type {
    NotificationRouting,
    CreateNotificationRoutingRequest,
    UpdateNotificationRoutingRequest,
    EventTypeCategory,
} from '@/types/notification'

const categoryIcons: Record<string, React.ReactNode> = {
    'AUP 計畫審查': <Shield className="h-4 w-4" />,
    '修正案': <FileText className="h-4 w-4" />,
    '動物健康': <Stethoscope className="h-4 w-4" />,
    'ERP 進銷存': <Package className="h-4 w-4" />,
    'HR 人事': <Users className="h-4 w-4" />,
}

interface CategorizedItem {
    category: string
    eventType: string
    eventName: string
    rules: NotificationRouting[]
}

function categorizeRules(
    rules: NotificationRouting[],
    categories: EventTypeCategory[]
): CategorizedItem[] {
    const result: CategorizedItem[] = []
    const eventCategoryMap = new Map<string, { category: string; name: string }>()
    for (const cat of categories) {
        for (const et of cat.event_types) {
            eventCategoryMap.set(et.code, { category: cat.category, name: et.name })
        }
    }

    const grouped = new Map<string, NotificationRouting[]>()
    for (const rule of rules) {
        if (!grouped.has(rule.event_type)) grouped.set(rule.event_type, [])
        grouped.get(rule.event_type)!.push(rule)
    }

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
            eventName: info?.name || eventType,
            rules: eventRules,
        })
    }

    return result
}

export function NotificationRoutingSection() {
    const queryClient = useQueryClient()
    const { dialogState, confirm } = useConfirmDialog()
    const [showAddForm, setShowAddForm] = useState(false)
    const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
    const [newRule, setNewRule] = useState<CreateNotificationRoutingRequest>({
        event_type: '',
        role_code: '',
        channel: 'both',
    })

    const { data: rules, isLoading, error } = useQuery({
        queryKey: ['notification-routing'],
        queryFn: async () => {
            const res = await notificationRoutingApi.list()
            return res.data
        },
    })

    const { data: eventCategories } = useQuery({
        queryKey: ['notification-routing-event-types'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getEventTypes()
            return res.data
        },
    })

    const { data: availableRoles } = useQuery({
        queryKey: ['notification-routing-roles'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getRoles()
            return res.data
        },
    })

    const roleNameMap = useMemo(() => {
        const map = new Map<string, string>()
        for (const role of availableRoles || []) {
            map.set(role.code, role.name)
        }
        return map
    }, [availableRoles])

    const createMutation = useMutation({
        mutationFn: (data: CreateNotificationRoutingRequest) =>
            notificationRoutingApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            setShowAddForm(false)
            setNewRule({ event_type: '', role_code: '', channel: 'both' })
            toast({ title: '成功', description: '已新增通知路由規則' })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '新增失敗'),
                variant: 'destructive',
            })
        },
    })

    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateNotificationRoutingRequest }) =>
            notificationRoutingApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '更新失敗'),
                variant: 'destructive',
            })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: (id: string) => notificationRoutingApi.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            toast({ title: '成功', description: '規則已刪除' })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '刪除失敗'),
                variant: 'destructive',
            })
        },
    })

    const categorizedRules = useMemo(
        () => categorizeRules(rules || [], eventCategories || []),
        [rules, eventCategories]
    )

    const groupedByCategory = useMemo(() => {
        const map = new Map<string, CategorizedItem[]>()
        for (const item of categorizedRules) {
            if (!map.has(item.category)) map.set(item.category, [])
            map.get(item.category)!.push(item)
        }
        return map
    }, [categorizedRules])

    const toggleCategory = (cat: string) => {
        setCollapsedCategories(prev => {
            const next = new Set(prev)
            if (next.has(cat)) next.delete(cat)
            else next.add(cat)
            return next
        })
    }

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

    const handleDelete = async (rule: NotificationRouting) => {
        const roleName = roleNameMap.get(rule.role_code) || rule.role_code
        const ok = await confirm({
            title: '刪除路由規則',
            description: `確定要刪除「${roleName}」的此條規則嗎？此操作無法復原。`,
            variant: 'destructive',
            confirmLabel: '刪除',
        })
        if (ok) deleteMutation.mutate(rule.id)
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

    const totalRules = rules?.length || 0
    const activeRules = rules?.filter(r => r.is_active).length || 0

    return (
        <div className="border-t pt-6">
            <div className="flex items-center justify-between mb-4">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">通知路由管理</h2>
                    <p className="text-muted-foreground">
                        設定各類事件的通知收件角色與管道
                        {totalRules > 0 && (
                            <span className="ml-2 text-xs">
                                （共 {totalRules} 條規則，{activeRules} 條啟用中）
                            </span>
                        )}
                    </p>
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
                        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
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
                                                {role.name}（{role.code}）
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
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
                                <Button size="sm" variant="ghost" onClick={() => setShowAddForm(false)}>
                                    取消
                                </Button>
                            </div>
                        </div>

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

            {isLoading && (
                <div className="flex items-center justify-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            )}

            {error && (
                <Card className="border-red-200 bg-red-50">
                    <CardContent className="flex items-center gap-3 py-6">
                        <AlertCircle className="h-5 w-5 text-red-500" />
                        <span className="text-red-700">無法載入通知路由設定</span>
                    </CardContent>
                </Card>
            )}

            {!isLoading && !error && (
                <div className="space-y-4">
                    {groupedByCategory.size === 0 ? (
                        <Card>
                            <CardContent className="py-8 text-center text-muted-foreground">
                                尚未設定任何通知路由規則
                            </CardContent>
                        </Card>
                    ) : (
                        [...groupedByCategory.entries()].map(([category, items]) => {
                            const isCollapsed = collapsedCategories.has(category)
                            const catRuleCount = items.reduce((sum, i) => sum + i.rules.length, 0)
                            const catActiveCount = items.reduce(
                                (sum, i) => sum + i.rules.filter(r => r.is_active).length, 0
                            )

                            return (
                                <Card key={category}>
                                    {/* 分類標題列（可點擊收合） */}
                                    <button
                                        type="button"
                                        className="flex items-center justify-between w-full px-6 py-4 text-left hover:bg-muted/30 transition-colors rounded-t-lg"
                                        onClick={() => toggleCategory(category)}
                                    >
                                        <div className="flex items-center gap-2.5">
                                            {isCollapsed ? (
                                                <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                            ) : (
                                                <ChevronDown className="h-4 w-4 text-muted-foreground" />
                                            )}
                                            <span className="text-muted-foreground">
                                                {categoryIcons[category] || <Route className="h-4 w-4" />}
                                            </span>
                                            <span className="font-semibold">{category}</span>
                                        </div>
                                        <span className="text-xs text-muted-foreground tabular-nums">
                                            {catActiveCount}/{catRuleCount} 啟用
                                        </span>
                                    </button>

                                    {!isCollapsed && (
                                        <CardContent className="pt-0 pb-4 space-y-4">
                                            {items.map((item) => (
                                                <div key={item.eventType}>
                                                    {/* 事件標題 */}
                                                    <div className="flex items-center gap-2 mb-2">
                                                        <span className="text-sm font-medium">{item.eventName}</span>
                                                        <span className="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
                                                            {item.eventType}
                                                        </span>
                                                    </div>

                                                    {/* 規則列表 */}
                                                    <div className="space-y-1.5 ml-0.5">
                                                        {item.rules.map((rule) => (
                                                            <div
                                                                key={rule.id}
                                                                className={`grid grid-cols-[minmax(100px,1fr)_120px_44px_32px] items-center gap-3 rounded-lg border px-4 py-2.5 transition-colors ${
                                                                    rule.is_active
                                                                        ? 'bg-white'
                                                                        : 'bg-muted/40 opacity-60'
                                                                }`}
                                                            >
                                                                {/* 角色 + 說明 */}
                                                                <div className="flex items-center gap-2 min-w-0">
                                                                    <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10 shrink-0">
                                                                        {roleNameMap.get(rule.role_code) || rule.role_code}
                                                                    </span>
                                                                    {rule.description && (
                                                                        <span className="text-xs text-muted-foreground truncate" title={rule.description}>
                                                                            {rule.description}
                                                                        </span>
                                                                    )}
                                                                </div>

                                                                {/* 通道選擇 */}
                                                                <Select
                                                                    value={rule.channel}
                                                                    onValueChange={(v) => handleChannelChange(rule, v)}
                                                                >
                                                                    <SelectTrigger className="h-8 text-xs">
                                                                        <SelectValue />
                                                                    </SelectTrigger>
                                                                    <SelectContent>
                                                                        <SelectItem value="in_app">站內通知</SelectItem>
                                                                        <SelectItem value="email">Email</SelectItem>
                                                                        <SelectItem value="both">兩者</SelectItem>
                                                                    </SelectContent>
                                                                </Select>

                                                                {/* 啟用 Switch */}
                                                                <Switch
                                                                    checked={rule.is_active}
                                                                    onCheckedChange={() => handleToggleActive(rule)}
                                                                />

                                                                {/* 刪除 */}
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    className="h-8 w-8 text-red-500 hover:text-red-700 hover:bg-red-50"
                                                                    onClick={() => handleDelete(rule)}
                                                                >
                                                                    <Trash2 className="h-4 w-4" />
                                                                </Button>
                                                            </div>
                                                        ))}
                                                    </div>
                                                </div>
                                            ))}
                                        </CardContent>
                                    )}
                                </Card>
                            )
                        })
                    )}
                </div>
            )}

            <ConfirmDialog state={dialogState} />
        </div>
    )
}
