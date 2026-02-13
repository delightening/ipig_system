import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
    bloodTestTemplateApi,
    bloodTestPanelApi,
    BloodTestTemplate,
    BloodTestPanel,
    CreateBloodTestTemplateRequest,
    UpdateBloodTestTemplateRequest,
    CreateBloodTestPanelRequest,
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import {
    Plus,
    Search,
    Edit,
    Power,
    PowerOff,
    Loader2,
    Droplets,
    ArrowUpDown,
    ArrowLeft,
    FolderPlus,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { PanelIcon } from '@/components/ui/panel-icon'
import { useNavigate } from 'react-router-dom'

// 排序欄位型別
type SortField = 'code' | 'name' | 'sort_order' | 'default_unit' | 'default_price'
type SortOrder = 'asc' | 'desc'

// 顯示篩選
type ShowFilter = 'all' | 'active' | 'inactive'

export function BloodTestTemplatesPage() {
    const queryClient = useQueryClient()
    const navigate = useNavigate()

    // 狀態
    const [search, setSearch] = useState('')
    const [showFilter, setShowFilter] = useState<ShowFilter>('all')
    const [sortField, setSortField] = useState<SortField>('sort_order')
    const [sortOrder, setSortOrder] = useState<SortOrder>('asc')
    const [dialogOpen, setDialogOpen] = useState(false)
    const [editingTemplate, setEditingTemplate] = useState<BloodTestTemplate | null>(null)
    const [selectedPanel, setSelectedPanel] = useState<string>('all')
    const [formData, setFormData] = useState<CreateBloodTestTemplateRequest>({
        code: '',
        name: '',
        default_unit: '',
        reference_range: '',
        default_price: 0,
        sort_order: 0,
        panel_id: undefined,
    })

    // 新增分類對話框
    const [panelDialogOpen, setPanelDialogOpen] = useState(false)
    const [panelFormData, setPanelFormData] = useState<CreateBloodTestPanelRequest>({
        key: '',
        name: '',
        icon: '',
        sort_order: 0,
    })

    // 查詢所有模板（含停用）
    const { data: templates, isLoading } = useQuery({
        queryKey: ['blood-test-templates-all'],
        queryFn: async () => {
            const response = await bloodTestTemplateApi.listAll()
            return response.data
        },
    })

    // 查詢所有分類（含停用）
    const { data: panels } = useQuery({
        queryKey: ['blood-test-panels-all'],
        queryFn: async () => {
            const response = await bloodTestPanelApi.listAll()
            return response.data
        },
    })

    // 新增
    const createMutation = useMutation({
        mutationFn: (data: CreateBloodTestTemplateRequest) =>
            bloodTestTemplateApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({ title: '成功', description: '檢查項目已建立' })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '建立失敗',
                variant: 'destructive',
            })
        },
    })

    // 更新
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestTemplateRequest }) =>
            bloodTestTemplateApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({ title: '成功', description: '檢查項目已更新' })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '更新失敗',
                variant: 'destructive',
            })
        },
    })

    // 停用/恢復
    const toggleMutation = useMutation({
        mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
            is_active
                ? bloodTestTemplateApi.update(id, { is_active: true })
                : bloodTestTemplateApi.delete(id),
        onSuccess: (_data, variables) => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            toast({
                title: '成功',
                description: variables.is_active ? '項目已恢復啟用' : '項目已停用',
            })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '操作失敗',
                variant: 'destructive',
            })
        },
    })

    // 新增分類
    const createPanelMutation = useMutation({
        mutationFn: (data: CreateBloodTestPanelRequest) =>
            bloodTestPanelApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({ title: '成功', description: '分類已建立' })
            setPanelDialogOpen(false)
            setPanelFormData({ key: '', name: '', icon: '', sort_order: 0 })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '建立分類失敗',
                variant: 'destructive',
            })
        },
    })

    const resetForm = () => {
        setFormData({
            code: '',
            name: '',
            default_unit: '',
            reference_range: '',
            default_price: 0,
            sort_order: 0,
            panel_id: undefined,
        })
        setEditingTemplate(null)
    }

    const handleEdit = (template: BloodTestTemplate) => {
        setEditingTemplate(template)
        // 找出所屬 panel
        const panelKey = templatePanelMap.get(template.id)
        const panel = panels?.find((p) => p.key === panelKey)
        setFormData({
            code: template.code,
            name: template.name,
            default_unit: template.default_unit || '',
            reference_range: template.reference_range || '',
            default_price: template.default_price || 0,
            sort_order: template.sort_order,
            panel_id: panel?.id,
        })
        setDialogOpen(true)
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (editingTemplate) {
            const updateData: UpdateBloodTestTemplateRequest = {
                name: formData.name,
                default_unit: formData.default_unit || undefined,
                reference_range: formData.reference_range || undefined,
                default_price: formData.default_price || undefined,
                sort_order: formData.sort_order,
                panel_id: formData.panel_id || undefined,
            }
            updateMutation.mutate({ id: editingTemplate.id, data: updateData })
        } else {
            createMutation.mutate(formData)
        }
    }

    const handleSort = (field: SortField) => {
        if (sortField === field) {
            setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')
        } else {
            setSortField(field)
            setSortOrder('asc')
        }
    }

    // 建立 template id -> panel key 的對應
    const templatePanelMap = useMemo(() => {
        if (!panels) return new Map<string, string>()
        const map = new Map<string, string>()
        for (const panel of panels) {
            for (const item of panel.items) {
                map.set(item.id, panel.key)
            }
        }
        return map
    }, [panels])

    // 排序函數
    const sortTemplates = (list: BloodTestTemplate[]) => {
        return [...list].sort((a, b) => {
            let cmp = 0
            switch (sortField) {
                case 'code':
                    cmp = a.code.localeCompare(b.code)
                    break
                case 'name':
                    cmp = a.name.localeCompare(b.name)
                    break
                case 'sort_order':
                    cmp = a.sort_order - b.sort_order
                    break
                case 'default_unit':
                    cmp = (a.default_unit || '').localeCompare(b.default_unit || '')
                    break
                case 'default_price':
                    cmp = (a.default_price || 0) - (b.default_price || 0)
                    break
            }
            return sortOrder === 'asc' ? cmp : -cmp
        })
    }

    // 篩選與分組
    const { groupedData, flatFiltered } = useMemo(() => {
        if (!templates) return { groupedData: [], flatFiltered: [] }

        let result = [...templates]

        // 搜尋
        if (search) {
            const q = search.toLowerCase()
            result = result.filter(
                (t) =>
                    t.code.toLowerCase().includes(q) ||
                    t.name.toLowerCase().includes(q) ||
                    (t.default_unit && t.default_unit.toLowerCase().includes(q))
            )
        }

        // 啟用/停用篩選
        if (showFilter === 'active') {
            result = result.filter((t) => t.is_active)
        } else if (showFilter === 'inactive') {
            result = result.filter((t) => !t.is_active)
        }

        // 依分類篩選
        if (selectedPanel !== 'all' && panels) {
            const panel = panels.find((p) => p.key === selectedPanel)
            if (panel) {
                const panelTemplateIds = new Set(panel.items.map((i) => i.id))
                result = result.filter((t) => panelTemplateIds.has(t.id))
            }
        }

        // 排序
        const sorted = sortTemplates(result)

        // 分組（選「全部」時分組顯示，選特定分類時不分組）
        if (selectedPanel === 'all' && panels && !search) {
            const grouped: { panel: BloodTestPanel | null; items: BloodTestTemplate[] }[] = []
            const usedIds = new Set<string>()

            for (const panel of panels) {
                const panelTemplateIds = new Set(panel.items.map((i) => i.id))
                const panelItems = sorted.filter((t) => panelTemplateIds.has(t.id))
                if (panelItems.length > 0) {
                    grouped.push({ panel, items: panelItems })
                    panelItems.forEach((t) => usedIds.add(t.id))
                }
            }

            // 未分類的項目
            const uncategorized = sorted.filter((t) => !usedIds.has(t.id))
            if (uncategorized.length > 0) {
                grouped.push({ panel: null, items: uncategorized })
            }

            return { groupedData: grouped, flatFiltered: sorted }
        }

        return { groupedData: [{ panel: null, items: sorted }], flatFiltered: sorted }
    }, [templates, panels, search, showFilter, sortField, sortOrder, selectedPanel])

    // 排序指示器
    const SortIndicator = ({ field }: { field: SortField }) => (
        <ArrowUpDown
            className={cn(
                'ml-1 h-3 w-3 inline-block cursor-pointer',
                sortField === field ? 'text-blue-600' : 'text-slate-400'
            )}
        />
    )

    // 統計
    const activeCount = templates?.filter((t) => t.is_active).length || 0
    const totalCount = templates?.length || 0

    // 渲染表格列
    const renderTemplateRow = (template: BloodTestTemplate) => (
        <TableRow
            key={template.id}
            className={cn(!template.is_active && 'opacity-50')}
        >
            <TableCell className="font-mono text-sm font-semibold">
                {template.code}
            </TableCell>
            <TableCell className="font-medium">{template.name}</TableCell>
            <TableCell className="text-slate-600">
                {template.default_unit || '—'}
            </TableCell>
            <TableCell className="text-sm text-slate-500">
                {template.reference_range || '—'}
            </TableCell>
            <TableCell className="text-right font-mono text-sm">
                {template.default_price
                    ? `$${Number(template.default_price).toFixed(0)}`
                    : '—'}
            </TableCell>

            <TableCell className="text-center">
                {template.is_active ? (
                    <Badge variant="success">啟用</Badge>
                ) : (
                    <Badge variant="secondary">停用</Badge>
                )}
            </TableCell>
            <TableCell className="text-right">
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleEdit(template)}
                        title="編輯"
                    >
                        <Edit className="h-4 w-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() =>
                            toggleMutation.mutate({
                                id: template.id,
                                is_active: !template.is_active,
                            })
                        }
                        title={template.is_active ? '停用' : '恢復'}
                    >
                        {template.is_active ? (
                            <PowerOff className="h-4 w-4 text-orange-500" />
                        ) : (
                            <Power className="h-4 w-4 text-green-500" />
                        )}
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    )

    return (
        <div className="space-y-6">
            {/* 標題列 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate('/erp?tab=master')}
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">血液檢查項目管理</h1>
                        <p className="text-muted-foreground">
                            管理血液檢查項目模板（共 {totalCount} 個，啟用 {activeCount} 個）
                        </p>
                    </div>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline" onClick={() => setPanelDialogOpen(true)}>
                        <FolderPlus className="mr-2 h-4 w-4" />
                        新增分類
                    </Button>
                    <Button onClick={() => { resetForm(); setDialogOpen(true) }}>
                        <Plus className="mr-2 h-4 w-4" />
                        新增項目
                    </Button>
                </div>
            </div>

            {/* 分類篩選 Tab - 一列自動換行 */}
            <div className="flex flex-wrap gap-1.5">
                <Button
                    variant={selectedPanel === 'all' ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => setSelectedPanel('all')}
                    className="gap-1"
                >
                    全部
                </Button>
                {panels?.map((p) => (
                    <Button
                        key={p.key}
                        variant={selectedPanel === p.key ? 'default' : 'outline'}
                        size="sm"
                        onClick={() => setSelectedPanel(p.key)}
                        className="gap-1"
                    >
                        <PanelIcon icon={p.icon} />
                        {p.name}
                        <Badge variant="secondary" className="ml-1 px-1.5 py-0 text-xs">
                            {p.items.length}
                        </Badge>
                    </Button>
                ))}
            </div>

            {/* 搜尋與篩選列 */}
            <div className="flex gap-4 items-center">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                        placeholder="搜尋代碼或名稱..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="pl-9"
                    />
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
                            <TableHead
                                className="w-[120px] cursor-pointer"
                                onClick={() => handleSort('code')}
                            >
                                代碼 <SortIndicator field="code" />
                            </TableHead>
                            <TableHead className="cursor-pointer" onClick={() => handleSort('name')}>
                                名稱 <SortIndicator field="name" />
                            </TableHead>
                            <TableHead
                                className="w-[100px] cursor-pointer"
                                onClick={() => handleSort('default_unit')}
                            >
                                單位 <SortIndicator field="default_unit" />
                            </TableHead>
                            <TableHead className="w-[140px]">參考範圍</TableHead>
                            <TableHead
                                className="w-[100px] cursor-pointer text-right"
                                onClick={() => handleSort('default_price')}
                            >
                                價格 <SortIndicator field="default_price" />
                            </TableHead>

                            <TableHead className="w-[80px] text-center">狀態</TableHead>
                            <TableHead className="w-[120px] text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8">
                                    <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                                </TableCell>
                            </TableRow>
                        ) : flatFiltered.length > 0 ? (
                            groupedData.map((group, gi) => (
                                <>
                                    {/* 分類標題列（僅在有多個分組時顯示） */}
                                    {groupedData.length > 1 && (
                                        <TableRow key={`group-${gi}`} className="bg-muted/50 hover:bg-muted/50">
                                            <TableCell colSpan={7} className="py-2">
                                                <div className="flex items-center gap-2 font-semibold text-sm">
                                                    {group.panel ? (
                                                        <>
                                                            <PanelIcon icon={group.panel.icon} className="text-base" />
                                                            <span>{group.panel.name}</span>
                                                            <Badge variant="outline" className="ml-1 text-xs">
                                                                {group.items.length} 項
                                                            </Badge>
                                                        </>
                                                    ) : (
                                                        <>
                                                            <span className="text-base">📦</span>
                                                            <span>未分類</span>
                                                            <Badge variant="outline" className="ml-1 text-xs">
                                                                {group.items.length} 項
                                                            </Badge>
                                                        </>
                                                    )}
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    )}
                                    {group.items.map(renderTemplateRow)}
                                </>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell colSpan={7} className="text-center py-8">
                                    <Droplets className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                                    <p className="text-muted-foreground">
                                        {search ? '找不到符合的檢查項目' : '尚無檢查項目資料'}
                                    </p>
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* 新增/編輯對話框 */}
            <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                <DialogContent className="sm:max-w-[500px]">
                    <DialogHeader>
                        <DialogTitle>
                            {editingTemplate ? '編輯檢查項目' : '新增檢查項目'}
                        </DialogTitle>
                        <DialogDescription>
                            {editingTemplate
                                ? `修改 ${editingTemplate.code} 的項目資料`
                                : '建立新的血液檢查項目模板'}
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit}>
                        <div className="grid gap-4 py-4">
                            {/* 代碼 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="code" className="text-right">
                                    代碼 <span className="text-red-500">*</span>
                                </Label>
                                <Input
                                    id="code"
                                    value={formData.code}
                                    onChange={(e) =>
                                        setFormData({ ...formData, code: e.target.value.toUpperCase() })
                                    }
                                    className="col-span-3 font-mono"
                                    placeholder="如: WBC、RBC、AST"
                                    required
                                    disabled={!!editingTemplate}
                                    maxLength={20}
                                />
                            </div>

                            {/* 名稱 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="name" className="text-right">
                                    名稱 <span className="text-red-500">*</span>
                                </Label>
                                <Input
                                    id="name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    placeholder="如: WBC (白血球計數)"
                                    required
                                    maxLength={200}
                                />
                            </div>

                            {/* 單位 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="default_unit" className="text-right">
                                    預設單位
                                </Label>
                                <Input
                                    id="default_unit"
                                    value={formData.default_unit}
                                    onChange={(e) =>
                                        setFormData({ ...formData, default_unit: e.target.value })
                                    }
                                    className="col-span-3"
                                    placeholder="如: 10³/μL、mg/dL、U/L"
                                    maxLength={50}
                                />
                            </div>

                            {/* 參考範圍 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="reference_range" className="text-right">
                                    參考範圍
                                </Label>
                                <Input
                                    id="reference_range"
                                    value={formData.reference_range}
                                    onChange={(e) =>
                                        setFormData({ ...formData, reference_range: e.target.value })
                                    }
                                    className="col-span-3"
                                    placeholder="如: 4.0-10.0"
                                    maxLength={100}
                                />
                            </div>

                            {/* 價格 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="default_price" className="text-right">
                                    預設價格
                                </Label>
                                <Input
                                    id="default_price"
                                    type="number"
                                    min="0"
                                    step="1"
                                    value={formData.default_price || ''}
                                    onChange={(e) =>
                                        setFormData({
                                            ...formData,
                                            default_price: e.target.value ? Number(e.target.value) : 0,
                                        })
                                    }
                                    className="col-span-3"
                                    placeholder="0"
                                />
                            </div>



                            {/* 所屬分類 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel_id" className="text-right">
                                    所屬分類
                                </Label>
                                <Select
                                    value={formData.panel_id || 'none'}
                                    onValueChange={(val) =>
                                        setFormData({
                                            ...formData,
                                            panel_id: val === 'none' ? undefined : val,
                                        })
                                    }
                                >
                                    <SelectTrigger className="col-span-3">
                                        <SelectValue placeholder="選擇分類" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="none">未分類</SelectItem>
                                        {panels?.map((panel) => (
                                            <SelectItem key={panel.id} value={panel.id}>
                                                <PanelIcon icon={panel.icon} /> {panel.name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => setDialogOpen(false)}
                            >
                                取消
                            </Button>
                            <Button
                                type="submit"
                                disabled={createMutation.isPending || updateMutation.isPending}
                            >
                                {(createMutation.isPending || updateMutation.isPending) && (
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                )}
                                {editingTemplate ? '更新' : '建立'}
                            </Button>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>

            {/* 新增分類對話框 */}
            <Dialog open={panelDialogOpen} onOpenChange={setPanelDialogOpen}>
                <DialogContent className="sm:max-w-[450px]">
                    <DialogHeader>
                        <DialogTitle>新增檢查分類</DialogTitle>
                        <DialogDescription>
                            建立新的血液檢查分類（如：CBC、肝臟、腎臟等）
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={(e) => {
                        e.preventDefault()
                        createPanelMutation.mutate(panelFormData)
                    }}>
                        <div className="grid gap-4 py-4">
                            {/* 代碼 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel_key" className="text-right">
                                    代碼 <span className="text-red-500">*</span>
                                </Label>
                                <Input
                                    id="panel_key"
                                    value={panelFormData.key}
                                    onChange={(e) =>
                                        setPanelFormData({ ...panelFormData, key: e.target.value.toUpperCase() })
                                    }
                                    className="col-span-3 font-mono"
                                    placeholder="如: CBC、LIVER、RENAL"
                                    required
                                    maxLength={20}
                                />
                            </div>

                            {/* 名稱 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel_name" className="text-right">
                                    名稱 <span className="text-red-500">*</span>
                                </Label>
                                <Input
                                    id="panel_name"
                                    value={panelFormData.name}
                                    onChange={(e) =>
                                        setPanelFormData({ ...panelFormData, name: e.target.value })
                                    }
                                    className="col-span-3"
                                    placeholder="如: 全血球計數、肝臟功能"
                                    required
                                    maxLength={100}
                                />
                            </div>

                            {/* 圖示 */}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel_icon" className="text-right">
                                    圖示
                                </Label>
                                <div className="col-span-3 flex items-center gap-2">
                                    <Input
                                        id="panel_icon"
                                        value={panelFormData.icon || ''}
                                        onChange={(e) =>
                                            setPanelFormData({ ...panelFormData, icon: e.target.value })
                                        }
                                        placeholder="Emoji 或 SVG 路徑"
                                        maxLength={200}
                                    />
                                    {panelFormData.icon && (
                                        <PanelIcon icon={panelFormData.icon} size={24} />
                                    )}
                                </div>
                            </div>


                        </div>
                        <DialogFooter>
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => setPanelDialogOpen(false)}
                            >
                                取消
                            </Button>
                            <Button
                                type="submit"
                                disabled={createPanelMutation.isPending}
                            >
                                {createPanelMutation.isPending && (
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                )}
                                建立
                            </Button>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>
        </div>
    )
}
