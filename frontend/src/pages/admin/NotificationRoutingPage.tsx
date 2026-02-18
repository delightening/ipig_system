import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import {
    Loader2,
    Bell,
    Plus,
    Pencil,
    Trash2,
    Mail,
    MessageSquare,
    Radio,
} from 'lucide-react'

// ============================================
// 型別定義
// ============================================

interface NotificationRouting {
    id: string
    event_type: string
    role_code: string
    channel: string
    is_active: boolean
    description: string | null
    created_at: string
    updated_at: string
}

interface EventTypeInfo {
    code: string
    name: string
}

interface EventTypeCategory {
    category: string
    event_types: EventTypeInfo[]
}

interface RoleInfo {
    code: string
    name: string
}

interface CreateRoutingData {
    event_type: string
    role_code: string
    channel: string
    description: string
}

interface UpdateRoutingData {
    channel?: string
    is_active?: boolean
    description?: string
}

// ============================================
// 通道顯示輔助
// ============================================

const channelOptions = [
    { value: 'in_app', label: '站內通知', icon: MessageSquare },
    { value: 'email', label: 'Email', icon: Mail },
    { value: 'both', label: '站內 + Email', icon: Radio },
]

function ChannelBadge({ channel }: { channel: string }) {
    const opt = channelOptions.find((o) => o.value === channel)
    if (!opt) return <Badge variant="outline">{channel}</Badge>
    const Icon = opt.icon
    return (
        <Badge variant="secondary" className="gap-1">
            <Icon className="h-3 w-3" />
            {opt.label}
        </Badge>
    )
}

// ============================================
// 主頁面元件
// ============================================

export function NotificationRoutingPage() {
    const queryClient = useQueryClient()
    const { toast } = useToast()
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const [showEditDialog, setShowEditDialog] = useState(false)
    const [selectedRule, setSelectedRule] = useState<NotificationRouting | null>(null)
    const [createForm, setCreateForm] = useState<CreateRoutingData>({
        event_type: '',
        role_code: '',
        channel: 'both',
        description: '',
    })
    const [editForm, setEditForm] = useState<UpdateRoutingData>({})

    // ---- 查詢 ----

    const { data: rules, isLoading } = useQuery({
        queryKey: ['notification-routing'],
        queryFn: async () => {
            const res = await api.get<NotificationRouting[]>('/admin/notification-routing')
            return res.data
        },
    })

    const { data: eventCategories } = useQuery({
        queryKey: ['notification-routing-event-types'],
        queryFn: async () => {
            const res = await api.get<EventTypeCategory[]>('/admin/notification-routing/event-types')
            return res.data
        },
    })

    const { data: roles } = useQuery({
        queryKey: ['notification-routing-roles'],
        queryFn: async () => {
            const res = await api.get<RoleInfo[]>('/admin/notification-routing/roles')
            return res.data
        },
    })

    // 建立事件類型 code → 中文名的 map
    const eventNameMap = useMemo(() => {
        const map: Record<string, string> = {}
        eventCategories?.forEach((cat) => {
            cat.event_types.forEach((et) => {
                map[et.code] = et.name
            })
        })
        return map
    }, [eventCategories])

    // 建立角色 code → 名稱的 map
    const roleNameMap = useMemo(() => {
        const map: Record<string, string> = {}
        roles?.forEach((r) => {
            map[r.code] = r.name
        })
        return map
    }, [roles])

    // ---- Mutation ----

    const createMutation = useMutation({
        mutationFn: async (data: CreateRoutingData) => {
            const res = await api.post('/admin/notification-routing', data)
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            setShowCreateDialog(false)
            resetCreateForm()
            toast({ title: '成功', description: '通知路由規則已建立' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error.response?.data?.error?.message || '建立失敗',
                variant: 'destructive',
            })
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: UpdateRoutingData }) => {
            const res = await api.put(`/admin/notification-routing/${id}`, data)
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            setShowEditDialog(false)
            setSelectedRule(null)
            toast({ title: '成功', description: '通知路由規則已更新' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error.response?.data?.error?.message || '更新失敗',
                variant: 'destructive',
            })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            await api.delete(`/admin/notification-routing/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            toast({ title: '成功', description: '通知路由規則已刪除' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error.response?.data?.error?.message || '刪除失敗',
                variant: 'destructive',
            })
        },
    })

    // 快速切換啟用狀態
    const toggleActiveMutation = useMutation({
        mutationFn: async ({ id, is_active }: { id: string; is_active: boolean }) => {
            const res = await api.put(`/admin/notification-routing/${id}`, { is_active })
            return res.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error.response?.data?.error?.message || '切換失敗',
                variant: 'destructive',
            })
        },
    })

    // ---- Handlers ----

    const resetCreateForm = () => {
        setCreateForm({ event_type: '', role_code: '', channel: 'both', description: '' })
    }

    const handleCreate = () => {
        if (!createForm.event_type || !createForm.role_code) {
            toast({ title: '錯誤', description: '請選擇事件類型與角色', variant: 'destructive' })
            return
        }
        createMutation.mutate(createForm)
    }

    const handleEdit = (rule: NotificationRouting) => {
        setSelectedRule(rule)
        setEditForm({
            channel: rule.channel,
            is_active: rule.is_active,
            description: rule.description || '',
        })
        setShowEditDialog(true)
    }

    const handleUpdate = () => {
        if (!selectedRule) return
        updateMutation.mutate({ id: selectedRule.id, data: editForm })
    }

    const handleDelete = (rule: NotificationRouting) => {
        const eventName = eventNameMap[rule.event_type] || rule.event_type
        const roleName = roleNameMap[rule.role_code] || rule.role_code
        if (confirm(`確定要刪除「${eventName} → ${roleName}」的路由規則嗎？`)) {
            deleteMutation.mutate(rule.id)
        }
    }

    // ---- Loading State ----

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        )
    }

    return (
        <div className="space-y-6">
            {/* 頁面標題 */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">通知路由設定</h1>
                    <p className="text-muted-foreground">
                        管理事件觸發時的通知對象與通知方式
                    </p>
                </div>
                <Button onClick={() => setShowCreateDialog(true)}>
                    <Plus className="h-4 w-4 mr-2" />
                    新增規則
                </Button>
            </div>

            {/* 規則列表 Table */}
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[200px]">事件類型</TableHead>
                            <TableHead className="w-[140px]">通知角色</TableHead>
                            <TableHead className="w-[160px]">通知管道</TableHead>
                            <TableHead className="w-[80px] text-center">啟用</TableHead>
                            <TableHead>描述</TableHead>
                            <TableHead className="w-[100px] text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {rules && rules.length > 0 ? (
                            rules.map((rule) => (
                                <TableRow key={rule.id} className={!rule.is_active ? 'opacity-50' : ''}>
                                    <TableCell>
                                        <div className="flex items-center gap-2">
                                            <Bell className="h-4 w-4 text-blue-500 shrink-0" />
                                            <div>
                                                <div className="font-medium">
                                                    {eventNameMap[rule.event_type] || rule.event_type}
                                                </div>
                                                <div className="text-xs text-muted-foreground font-mono">
                                                    {rule.event_type}
                                                </div>
                                            </div>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <Badge variant="outline">
                                            {roleNameMap[rule.role_code] || rule.role_code}
                                        </Badge>
                                    </TableCell>
                                    <TableCell>
                                        <ChannelBadge channel={rule.channel} />
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <Switch
                                            checked={rule.is_active}
                                            onCheckedChange={(checked) =>
                                                toggleActiveMutation.mutate({ id: rule.id, is_active: checked })
                                            }
                                        />
                                    </TableCell>
                                    <TableCell className="text-sm text-muted-foreground">
                                        {rule.description || '—'}
                                    </TableCell>
                                    <TableCell className="text-right">
                                        <div className="flex justify-end gap-1">
                                            <Button variant="ghost" size="icon" onClick={() => handleEdit(rule)}>
                                                <Pencil className="h-4 w-4" />
                                            </Button>
                                            <Button variant="ghost" size="icon" onClick={() => handleDelete(rule)}>
                                                <Trash2 className="h-4 w-4 text-red-500" />
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                                    尚無通知路由規則
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* 新增規則 Dialog */}
            <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle>新增通知路由規則</DialogTitle>
                        <DialogDescription>
                            設定當特定事件發生時，通知哪個角色以及使用何種通知方式
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        {/* 事件類型（分類 grouped select） */}
                        <div className="space-y-2">
                            <Label>事件類型 *</Label>
                            <Select
                                value={createForm.event_type}
                                onValueChange={(v) => setCreateForm({ ...createForm, event_type: v })}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇事件類型" />
                                </SelectTrigger>
                                <SelectContent>
                                    {eventCategories?.map((cat) => (
                                        <SelectGroup key={cat.category}>
                                            <SelectLabel>{cat.category}</SelectLabel>
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

                        {/* 通知角色 */}
                        <div className="space-y-2">
                            <Label>通知角色 *</Label>
                            <Select
                                value={createForm.role_code}
                                onValueChange={(v) => setCreateForm({ ...createForm, role_code: v })}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇角色" />
                                </SelectTrigger>
                                <SelectContent>
                                    {roles?.map((r) => (
                                        <SelectItem key={r.code} value={r.code}>
                                            {r.name}（{r.code}）
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 通知管道 */}
                        <div className="space-y-2">
                            <Label>通知管道</Label>
                            <Select
                                value={createForm.channel}
                                onValueChange={(v) => setCreateForm({ ...createForm, channel: v })}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {channelOptions.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {opt.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 描述 */}
                        <div className="space-y-2">
                            <Label>描述</Label>
                            <Input
                                value={createForm.description}
                                onChange={(e) => setCreateForm({ ...createForm, description: e.target.value })}
                                placeholder="例如：計畫提交後通知 IACUC 執行秘書"
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                            取消
                        </Button>
                        <Button onClick={handleCreate} disabled={createMutation.isPending}>
                            {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                            建立
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 編輯規則 Dialog */}
            <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle>編輯通知路由規則</DialogTitle>
                        <DialogDescription>
                            {selectedRule && (
                                <>
                                    {eventNameMap[selectedRule.event_type] || selectedRule.event_type}
                                    {' → '}
                                    {roleNameMap[selectedRule.role_code] || selectedRule.role_code}
                                </>
                            )}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        {/* 通知管道 */}
                        <div className="space-y-2">
                            <Label>通知管道</Label>
                            <Select
                                value={editForm.channel}
                                onValueChange={(v) => setEditForm({ ...editForm, channel: v })}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {channelOptions.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {opt.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 啟用 / 停用 */}
                        <div className="flex items-center justify-between">
                            <Label>啟用狀態</Label>
                            <Switch
                                checked={editForm.is_active ?? true}
                                onCheckedChange={(checked) => setEditForm({ ...editForm, is_active: checked })}
                            />
                        </div>

                        {/* 描述 */}
                        <div className="space-y-2">
                            <Label>描述</Label>
                            <Input
                                value={editForm.description ?? ''}
                                onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                                placeholder="規則描述"
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowEditDialog(false)}>
                            取消
                        </Button>
                        <Button onClick={handleUpdate} disabled={updateMutation.isPending}>
                            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                            儲存
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}
