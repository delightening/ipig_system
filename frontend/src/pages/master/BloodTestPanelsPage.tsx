import { useState, useMemo } from 'react'
import { STALE_TIME } from '@/lib/query'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
    bloodTestPanelApi,
    bloodTestTemplateApi,
    BloodTestPanel,
    CreateBloodTestPanelRequest,
    UpdateBloodTestPanelRequest,
    UpdateBloodTestPanelItemsRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
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
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import {
    Plus,
    Search,
    Edit,
    Power,
    PowerOff,
    Loader2,
    ArrowLeft,
    Settings,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { PanelIcon } from '@/components/ui/panel-icon'
import { useNavigate } from 'react-router-dom'

// 顯示篩選
type ShowFilter = 'all' | 'active' | 'inactive'

export function BloodTestPanelsPage() {
    const queryClient = useQueryClient()
    const navigate = useNavigate()

    // 狀態
    const [search, setSearch] = useState('')
    const [showFilter, setShowFilter] = useState<ShowFilter>('all')
    const [dialogOpen, setDialogOpen] = useState(false)
    const [editingPanel, setEditingPanel] = useState<BloodTestPanel | null>(null)
    const [formData, setFormData] = useState<CreateBloodTestPanelRequest>({
        key: '',
        name: '',
        icon: '',
        sort_order: 0,
    })

    // 管理項目對話框
    const [itemsDialogOpen, setItemsDialogOpen] = useState(false)
    const [managingPanel, setManagingPanel] = useState<BloodTestPanel | null>(null)
    const [selectedTemplateIds, setSelectedTemplateIds] = useState<Set<string>>(new Set())
    const [itemSearch, setItemSearch] = useState('')

    // 查詢所有 Panel（含停用）
    const { data: panels, isLoading } = useQuery({
        queryKey: ['blood-test-panels-all'],
        staleTime: STALE_TIME.REFERENCE,
        queryFn: async () => {
            const response = await bloodTestPanelApi.listAll()
            return response.data
        },
    })

    // 查詢所有模板（供管理項目用）
    const { data: allTemplates } = useQuery({
        queryKey: ['blood-test-templates-all'],
        staleTime: STALE_TIME.REFERENCE,
        queryFn: async () => {
            const response = await bloodTestTemplateApi.listAll()
            return response.data
        },
    })

    // 新增 Panel
    const createMutation = useMutation({
        mutationFn: (data: CreateBloodTestPanelRequest) =>
            bloodTestPanelApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({ title: '成功', description: '分類已建立' })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, '建立失敗')
            toast({ title: '錯誤', description: msg, variant: 'destructive' })
        },
    })

    // 更新 Panel
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestPanelRequest }) =>
            bloodTestPanelApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            toast({ title: '成功', description: '分類已更新' })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, '更新失敗')
            toast({ title: '錯誤', description: msg, variant: 'destructive' })
        },
    })

    // 停用/啟用 Panel
    const toggleMutation = useMutation({
        mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
            bloodTestPanelApi.update(id, { is_active }),
        onSuccess: (_data, variables) => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({
                title: '成功',
                description: variables.is_active ? '分類已恢復啟用' : '分類已停用',
            })
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, '操作失敗')
            toast({ title: '錯誤', description: msg, variant: 'destructive' })
        },
    })

    // 更新 Panel 項目
    const updateItemsMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestPanelItemsRequest }) =>
            bloodTestPanelApi.updateItems(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            toast({ title: '成功', description: '分類項目已更新' })
            setItemsDialogOpen(false)
            setManagingPanel(null)
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, '更新項目失敗')
            toast({ title: '錯誤', description: msg, variant: 'destructive' })
        },
    })

    // 重置表單
    const resetForm = () => {
        setEditingPanel(null)
        setFormData({ key: '', name: '', icon: '', sort_order: 0 })
    }

    // 開啟編輯
    const handleEdit = (panel: BloodTestPanel) => {
        setEditingPanel(panel)
        setFormData({
            key: panel.key,
            name: panel.name,
            icon: panel.icon || '',
            sort_order: panel.sort_order,
        })
        setDialogOpen(true)
    }

    // 開啟管理項目
    const handleManageItems = (panel: BloodTestPanel) => {
        setManagingPanel(panel)
        setSelectedTemplateIds(new Set(panel.items.map((t) => t.id)))
        setItemSearch('')
        setItemsDialogOpen(true)
    }

    // 提交表單
    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (editingPanel) {
            updateMutation.mutate({
                id: editingPanel.id,
                data: {
                    name: formData.name,
                    icon: formData.icon || undefined,
                    sort_order: formData.sort_order,
                },
            })
        } else {
            createMutation.mutate(formData)
        }
    }

    // 儲存項目
    const handleSaveItems = () => {
        if (!managingPanel) return
        updateItemsMutation.mutate({
            id: managingPanel.id,
            data: { template_ids: Array.from(selectedTemplateIds) },
        })
    }

    // 切換項目選取
    const toggleTemplate = (id: string) => {
        setSelectedTemplateIds((prev) => {
            const next = new Set(prev)
            if (next.has(id)) {
                next.delete(id)
            } else {
                next.add(id)
            }
            return next
        })
    }

    // 統計
    const totalCount = panels?.length ?? 0
    const activeCount = panels?.filter((p) => p.is_active).length ?? 0

    // 篩選
    const filteredPanels = useMemo(() => {
        if (!panels) return []
        let result = [...panels]

        // 搜尋
        if (search) {
            const q = search.toLowerCase()
            result = result.filter(
                (p) =>
                    p.key.toLowerCase().includes(q) ||
                    p.name.toLowerCase().includes(q)
            )
        }

        // 啟用/停用篩選
        if (showFilter === 'active') {
            result = result.filter((p) => p.is_active)
        } else if (showFilter === 'inactive') {
            result = result.filter((p) => !p.is_active)
        }

        // 排序
        result.sort((a, b) => a.sort_order - b.sort_order)

        return result
    }, [panels, search, showFilter])

    // 篩選可選模板
    const filteredTemplates = useMemo(() => {
        if (!allTemplates) return []
        let result = allTemplates.filter((t) => t.is_active)
        if (itemSearch) {
            const q = itemSearch.toLowerCase()
            result = result.filter(
                (t) =>
                    t.code.toLowerCase().includes(q) ||
                    t.name.toLowerCase().includes(q)
            )
        }
        result.sort((a, b) => a.sort_order - b.sort_order)
        return result
    }, [allTemplates, itemSearch])

    const isSaving = createMutation.isPending || updateMutation.isPending

    return (
        <div className="space-y-6">
            {/* 標題列 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate('/blood-test-templates')}
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">血液檢查分類管理</h1>
                        <p className="text-muted-foreground">
                            管理血液檢查組合分類（共 {totalCount} 個，啟用 {activeCount} 個）
                        </p>
                    </div>
                </div>
                <Button onClick={() => { resetForm(); setDialogOpen(true) }}>
                    <Plus className="mr-2 h-4 w-4" />
                    新增分類
                </Button>
            </div>

            {/* 搜尋與篩選列 */}
            <div className="flex gap-4 items-center">
                <div className="flex gap-2 flex-1 max-w-md">
                    <div className="relative flex-1">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            placeholder="搜尋代碼或名稱..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] }) } }}
                            className="pl-9"
                        />
                    </div>
                    <Button type="button" variant="secondary" onClick={() => queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })} aria-label="搜尋" className="cursor-pointer transition-colors hover:bg-secondary/70 hover:ring-2 hover:ring-primary/20 hover:ring-offset-2">
                        <Search className="h-4 w-4 md:mr-1.5" />
                        <span className="hidden md:inline">搜尋</span>
                    </Button>
                </div>
                <div className="flex gap-1">
                    {(['all', 'active', 'inactive'] as ShowFilter[]).map((f) => (
                        <Button
                            key={f}
                            variant={showFilter === f ? 'default' : 'outline'}
                            size="sm"
                            onClick={() => setShowFilter(f)}
                        >
                            {f === 'all' ? '全部' : f === 'active' ? '啟用中' : '已停用'}
                        </Button>
                    ))}
                </div>
            </div>

            {/* 表格 */}
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[60px]">圖示</TableHead>
                            <TableHead className="w-[120px]">代碼</TableHead>
                            <TableHead>名稱</TableHead>
                            <TableHead className="w-[80px] text-center">排序</TableHead>
                            <TableHead className="w-[100px] text-center">包含項目</TableHead>
                            <TableHead className="w-[80px] text-center">狀態</TableHead>
                            <TableHead className="w-[180px] text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-12">
                                    <Loader2 className="h-6 w-6 animate-spin mx-auto" />
                                </TableCell>
                            </TableRow>
                        ) : filteredPanels.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-12 text-muted-foreground">
                                    沒有符合條件的分類
                                </TableCell>
                            </TableRow>
                        ) : (
                            filteredPanels.map((panel) => (
                                <TableRow
                                    key={panel.id}
                                    className={cn(!panel.is_active && 'opacity-50')}
                                >
                                    <TableCell>
                                        <PanelIcon icon={panel.icon} size={24} />
                                    </TableCell>
                                    <TableCell className="font-mono text-sm">{panel.key}</TableCell>
                                    <TableCell className="font-medium">{panel.name}</TableCell>
                                    <TableCell className="text-center">{panel.sort_order}</TableCell>
                                    <TableCell className="text-center">
                                        <Badge variant="secondary">{panel.items.length}</Badge>
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <Badge variant={panel.is_active ? 'default' : 'outline'}>
                                            {panel.is_active ? '啟用' : '停用'}
                                        </Badge>
                                    </TableCell>
                                    <TableCell className="text-right">
                                        <div className="flex justify-end gap-1">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => handleManageItems(panel)}
                                                title="管理項目"
                                            >
                                                <Settings className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => handleEdit(panel)}
                                                title="編輯"
                                            >
                                                <Edit className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() =>
                                                    toggleMutation.mutate({
                                                        id: panel.id,
                                                        is_active: !panel.is_active,
                                                    })
                                                }
                                                title={panel.is_active ? '停用' : '啟用'}
                                            >
                                                {panel.is_active ? (
                                                    <PowerOff className="h-4 w-4 text-destructive" />
                                                ) : (
                                                    <Power className="h-4 w-4 text-green-500" />
                                                )}
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* 新增/編輯對話框 */}
            <Dialog open={dialogOpen} onOpenChange={(open) => { if (!open) { resetForm(); setDialogOpen(false) } }}>
                <DialogContent className="sm:max-w-[425px]">
                    <DialogHeader>
                        <DialogTitle>{editingPanel ? '編輯分類' : '新增分類'}</DialogTitle>
                        <DialogDescription>
                            {editingPanel ? '修改分類的名稱、圖示和排序' : '請輸入新分類的資訊'}
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-key" className="text-right">代碼</Label>
                                <Input
                                    id="panel-key"
                                    value={formData.key}
                                    onChange={(e) => setFormData({ ...formData, key: e.target.value })}
                                    className="col-span-3"
                                    placeholder="例：CBC"
                                    disabled={!!editingPanel}
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-name" className="text-right">名稱</Label>
                                <Input
                                    id="panel-name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    placeholder="例：血液常規"
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-icon" className="text-right">圖示</Label>
                                <div className="col-span-3 flex items-center gap-2">
                                    <Input
                                        id="panel-icon"
                                        value={formData.icon}
                                        onChange={(e) => setFormData({ ...formData, icon: e.target.value })}
                                        placeholder="輸入 emoji（例：🩸）"
                                        className="flex-1"
                                    />
                                    {formData.icon && (
                                        <span className="text-2xl"><PanelIcon icon={formData.icon} size={28} /></span>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-sort" className="text-right">排序</Label>
                                <Input
                                    id="panel-sort"
                                    type="number"
                                    value={formData.sort_order}
                                    onChange={(e) =>
                                        setFormData({ ...formData, sort_order: parseInt(e.target.value) || 0 })
                                    }
                                    className="col-span-3"
                                />
                            </div>
                        </div>
                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => { resetForm(); setDialogOpen(false) }}>
                                取消
                            </Button>
                            <Button type="submit" disabled={isSaving}>
                                {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                {editingPanel ? '儲存' : '建立'}
                            </Button>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>

            {/* 管理項目對話框 */}
            <Dialog open={itemsDialogOpen} onOpenChange={(open) => { if (!open) { setItemsDialogOpen(false); setManagingPanel(null) } }}>
                <DialogContent className="sm:max-w-[600px] max-h-[80vh]">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            {managingPanel && <PanelIcon icon={managingPanel.icon} size={22} />}
                            管理「{managingPanel?.name}」包含項目
                        </DialogTitle>
                        <DialogDescription>
                            勾選要包含在此分類中的檢查項目（已選 {selectedTemplateIds.size} 項）
                        </DialogDescription>
                    </DialogHeader>

                    {/* 項目搜尋 */}
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            placeholder="搜尋代碼或名稱..."
                            value={itemSearch}
                            onChange={(e) => setItemSearch(e.target.value)}
                            className="pl-9"
                        />
                    </div>

                    {/* 項目列表（可捲動） */}
                    <div className="overflow-y-auto max-h-[400px] border rounded-md">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="w-[40px]"></TableHead>
                                    <TableHead className="w-[100px]">代碼</TableHead>
                                    <TableHead>名稱</TableHead>
                                    <TableHead className="w-[80px]">單位</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {filteredTemplates.map((t) => (
                                    <TableRow
                                        key={t.id}
                                        className="cursor-pointer"
                                        onClick={() => toggleTemplate(t.id)}
                                    >
                                        <TableCell>
                                            <Checkbox
                                                checked={selectedTemplateIds.has(t.id)}
                                                onCheckedChange={() => toggleTemplate(t.id)}
                                            />
                                        </TableCell>
                                        <TableCell className="font-mono text-sm">{t.code}</TableCell>
                                        <TableCell>{t.name}</TableCell>
                                        <TableCell className="text-muted-foreground">{t.default_unit || '-'}</TableCell>
                                    </TableRow>
                                ))}
                                {filteredTemplates.length === 0 && (
                                    <TableRow>
                                        <TableCell colSpan={4} className="text-center py-8 text-muted-foreground">
                                            沒有符合條件的項目
                                        </TableCell>
                                    </TableRow>
                                )}
                            </TableBody>
                        </Table>
                    </div>

                    <DialogFooter>
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => { setItemsDialogOpen(false); setManagingPanel(null) }}
                        >
                            取消
                        </Button>
                        <Button
                            onClick={handleSaveItems}
                            disabled={updateItemsMutation.isPending}
                        >
                            {updateItemsMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            儲存 ({selectedTemplateIds.size} 項)
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}
